//! JSON-RPC 2.0 server for identity service

use std::sync::Arc;
use anyhow::{anyhow, Result};
use serde_json::Value;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn};

use crate::IdentityService;

pub async fn serve(service: Arc<IdentityService>) -> Result<()> {
    let socket_path = std::path::PathBuf::from(&service.config.sockets_dir).join("identity.sock");

    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    info!("Identity RPC server listening on {:?}", socket_path);

    loop {
        let (stream, _) = listener.accept().await?;
        let service = service.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, service).await {
                warn!("Identity connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: tokio::net::UnixStream, service: Arc<IdentityService>) -> Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = vec![0u8; 65536];

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 { break; }

        let request: serde_json::Value = match serde_json::from_slice(&buf[..n]) {
            Ok(req) => req,
            Err(e) => {
                let resp = make_error(-32700, &format!("Parse error: {}", e), Value::Null);
                writer.write_all(&serde_json::to_vec(&resp)?).await?;
                writer.write_all(b"\n").await?;
                continue;
            }
        };

        let method = request["method"].as_str().unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(Value::Null);
        let id = request.get("id").cloned().unwrap_or(Value::Null);

        let result = dispatch(&service, method, params);
        let response = match result {
            Ok(value) => make_result(value, id),
            Err(e) => make_error(-32603, &e.to_string(), id),
        };

        writer.write_all(&serde_json::to_vec(&response)?).await?;
        writer.write_all(b"\n").await?;
    }
    Ok(())
}

fn dispatch(service: &IdentityService, method: &str, params: Value) -> Result<Value> {
    let ks = service.key_store.lock().unwrap();
    match method {
        "identity.health" => {
            Ok(serde_json::json!({"status": "ok", "uptime_seconds": 0}))
        }
        "identity.get_profile" => {
            let pubkey = params["public_key"].as_str().ok_or_else(|| anyhow!("missing public_key"))?;
            match ks.get_profile(pubkey)? {
                Some((id, name, about, domain, avatar_cid)) => {
                    Ok(serde_json::json!({
                        "id": id,
                        "public_key": pubkey,
                        "name": name.unwrap_or_default(),
                        "about": about.unwrap_or_default(),
                        "domain": domain.unwrap_or_default(),
                        "avatar_cid": avatar_cid.unwrap_or_default(),
                    }))
                }
                None => {
                    match ks.get_key_by_hash(pubkey)? {
                        Some((id, _, _, _)) => {
                            Ok(serde_json::json!({
                                "id": id,
                                "public_key": pubkey,
                                "name": "Anonymous",
                                "about": "",
                                "domain": "",
                                "avatar_cid": "",
                            }))
                        }
                        None => {
                            anyhow::bail!("Profile not found for public key: {}", pubkey)
                        }
                    }
                }
            }
        }
        "identity.update_profile" => {
            let pubkey = params["public_key"].as_str().ok_or_else(|| anyhow!("missing public_key"))?;
            let name = params["name"].as_str();
            let about = params["about"].as_str();
            let domain = params["domain"].as_str();
            let id = match ks.get_profile(pubkey)? {
                Some((id, _, _, _, _)) => id,
                None => {
                    match ks.get_key_by_hash(pubkey)? {
                        Some((id, _, _, _)) => id,
                        None => anyhow::bail!("Key not found for pubkey hash to update profile")
                    }
                }
            };
            ks.store_profile(&id, name, about, domain)?;
            Ok(serde_json::json!({"updated": true}))
        }
        "identity.list_aliases" => {
            let key_id = params["key_id"].as_str().ok_or_else(|| anyhow!("missing key_id"))?;
            let aliases = ks.list_aliases(key_id)?;
            Ok(serde_json::json!({"aliases": aliases.into_iter().map(|(n, d)| serde_json::json!({"name": n, "domain": d})).collect::<Vec<_>>()}))
        }
        "identity.create_alias" => {
            let key_id = params["key_id"].as_str().ok_or_else(|| anyhow!("missing key_id"))?;
            let name = params["name"].as_str().ok_or_else(|| anyhow!("missing name"))?;
            let domain = format!("{}.sovrn", name);
            ks.add_alias(key_id, name, &domain)?;

            if let Some(pubkey_hash) = crate::did::parse_did(key_id) {
                if let Some((_, _, _, current_domain, _)) = ks.get_profile(pubkey_hash)? {
                    if current_domain.is_none() || current_domain.unwrap().is_empty() {
                        ks.store_profile(key_id, None, None, Some(&domain))?;
                    }
                } else {
                    ks.store_profile(key_id, Some("Anonymous"), Some(""), Some(&domain))?;
                }
            }
            Ok(serde_json::json!({"name": name, "domain": domain, "registered": true}))
        }
        "identity.list_all_identities" => {
            let identities = ks.list_all_identities()?;
            let list = identities.into_iter().map(|(pk, name, domain)| {
                serde_json::json!({
                    "public_key": pk,
                    "name": name,
                    "domain": domain
                })
            }).collect::<Vec<_>>();
            Ok(serde_json::json!({"identities": list}))
        }
        "identity.list_all_domains" => {
            let domains = ks.list_all_domains()?;
            let list = domains.into_iter().map(|(name, domain)| {
                serde_json::json!({
                    "name": name,
                    "domain": domain
                })
            }).collect::<Vec<_>>();
            Ok(serde_json::json!({"domains": list}))
        }
        "identity.delete_alias" => {
            let domain = params["domain"].as_str().ok_or_else(|| anyhow!("missing domain"))?;
            ks.delete_alias(domain)?;
            Ok(serde_json::json!({"deleted": true}))
        }
        "identity.sign_event" => {
            Ok(serde_json::json!({"signature": "placeholder_sig"}))
        }
        "identity.verify_event" => {
            Ok(serde_json::json!({"valid": true}))
        }
        "identity.derive_app_key" => {
            let master_key = hex::decode(params["master_key"].as_str().unwrap_or(""))?;
            let app_id = params["app_id"].as_str().ok_or_else(|| anyhow!("missing app_id"))?;
            let (username, password) = crate::derive::derive_app_key(&master_key, app_id);
            Ok(serde_json::json!({"username": username, "password": password}))
        }
        "identity.export_identity" => {
            Ok(serde_json::json!({"seed_phrase": "placeholder", "master_keypair": {}, "aliases": []}))
        }
        "identity.import_identity" => {
            let seed_phrase = params["seed_phrase"].as_str().unwrap_or("").trim();
            let phrase = if seed_phrase.is_empty() {
                crate::seed::generate_seed_phrase()
            } else {
                seed_phrase.to_string()
            };
            crate::seed::validate_seed_phrase(&phrase)?;
            let keypair = crate::keys::SovrnKeyPair::from_seed_phrase(&phrase)?;
            let verifying_key_bytes = hex::decode(&keypair.verifying_key_hex)?;
            let id = crate::did::generate_did(&verifying_key_bytes);
            
            ks.store_key(
                &id,
                &keypair.signing_key_hex,
                &keypair.verifying_key_hex,
                &keypair.x25519_public_hex,
                &keypair.pubkey_hash,
            )?;
            
            if ks.get_profile(&keypair.pubkey_hash)?.is_none() {
                ks.store_profile(&id, Some("Anonymous"), Some(""), Some(""))?;
            }

            Ok(serde_json::json!({
                "id": id,
                "public_key": keypair.pubkey_hash,
                "name": "Anonymous",
                "domain": "",
                "seed_phrase": phrase,
            }))
        }
        _ => Err(anyhow!("Unknown method: {}", method)),
    }
}

fn make_result(result: Value, id: Value) -> serde_json::Value {
    serde_json::json!({"jsonrpc": "2.0", "result": result, "id": id})
}

fn make_error(code: i64, message: &str, id: Value) -> serde_json::Value {
    serde_json::json!({"jsonrpc": "2.0", "error": {"code": code, "message": message}, "id": id})
}
