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
        "identity.get_profile" => {
            let pubkey = params["public_key"].as_str().ok_or_else(|| anyhow!("missing public_key"))?;
            let profile = ks.get_profile(pubkey)?;
            Ok(serde_json::to_value(profile)?)
        }
        "identity.update_profile" => {
            let pubkey = params["public_key"].as_str().ok_or_else(|| anyhow!("missing public_key"))?;
            let name = params["name"].as_str();
            let about = params["about"].as_str();
            let domain = params["domain"].as_str();
            ks.store_profile(pubkey, name, about, domain)?;
            Ok(serde_json::json!({"updated": true}))
        }
        "identity.list_aliases" => {
            let key_id = params["key_id"].as_str().ok_or_else(|| anyhow!("missing key_id"))?;
            let aliases = ks.list_aliases(key_id)?;
            Ok(serde_json::json!({"aliases": aliases}))
        }
        "identity.create_alias" => {
            let key_id = params["key_id"].as_str().ok_or_else(|| anyhow!("missing key_id"))?;
            let name = params["name"].as_str().ok_or_else(|| anyhow!("missing name"))?;
            let domain = format!("{}.sovrn", name);
            ks.add_alias(key_id, name, &domain)?;
            Ok(serde_json::json!({"name": name, "domain": domain, "registered": true}))
        }
        "identity.delete_alias" => {
            let domain = params["domain"].as_str().ok_or_else(|| anyhow!("missing domain"))?;
            ks.delete_alias(domain)?;
            Ok(serde_json::json!({"deleted": true}))
        }
        "identity.sign_event" => {
            // In production, this would use the stored signing key
            // For now, return a placeholder
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
            Ok(serde_json::json!({"id": "placeholder", "name": "new_user", "domain": "new_user.sovrn"}))
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
