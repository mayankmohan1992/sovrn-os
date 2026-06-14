//! JSON-RPC 2.0 server for presence service

use std::sync::Arc;
use anyhow::{anyhow, Result};
use serde_json::Value;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;

use crate::PresenceService;

pub async fn serve(service: Arc<PresenceService>) -> Result<()> {
    let socket_path = std::path::PathBuf::from(&service.config.sockets_dir).join("presence.sock");
    if socket_path.exists() { std::fs::remove_file(&socket_path)?; }
    if let Some(parent) = socket_path.parent() { std::fs::create_dir_all(parent)?; }

    let listener = UnixListener::bind(&socket_path)?;
    info!("Presence RPC server listening on {:?}", socket_path);

    loop {
        let (stream, _) = listener.accept().await?;
        let service = service.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, service).await {
                tracing::warn!("Presence connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: tokio::net::UnixStream, service: Arc<PresenceService>) -> Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = vec![0u8; 65536];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 { break; }
        let request: Value = match serde_json::from_slice(&buf[..n]) {
            Ok(r) => r,
            Err(e) => {
                let resp = json_error(-32700, &format!("Parse error: {}", e), Value::Null);
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
            Ok(v) => json_result(v, id),
            Err(e) => json_error(-32603, &e.to_string(), id),
        };
        writer.write_all(&serde_json::to_vec(&response)?).await?;
        writer.write_all(b"\n").await?;
    }
    Ok(())
}

fn dispatch(service: &PresenceService, method: &str, params: Value) -> Result<Value> {
    let ps = service.peer_store.lock().unwrap();
    match method {
        "presence.get_online" => {
            let limit = params["limit"].as_u64().unwrap_or(50) as usize;
            let peers = ps.get_online_peers(limit)?;
            Ok(serde_json::json!({"peers": peers}))
        }
        "presence.set_status" => {
            let status = params["status"].as_str().ok_or_else(|| anyhow!("missing status"))?;
            drop(ps);
            let mut ps = service.peer_store.lock().unwrap();
            ps.set_status(&service.node_id, status)?;
            Ok(serde_json::json!({"status": status}))
        }
        "presence.heartbeat" => {
            Ok(serde_json::json!({"acknowledged": true}))
        }
        "presence.get_status" => {
            let pubkey = params["public_key"].as_str().ok_or_else(|| anyhow!("missing public_key"))?;
            let peer = ps.get_peer_status(pubkey)?;
            Ok(serde_json::to_value(peer)?)
        }
        _ => Err(anyhow!("Unknown method: {}", method)),
    }
}

fn json_result(result: Value, id: Value) -> Value {
    serde_json::json!({"jsonrpc": "2.0", "result": result, "id": id})
}
fn json_error(code: i64, message: &str, id: Value) -> Value {
    serde_json::json!({"jsonrpc": "2.0", "error": {"code": code, "message": message}, "id": id})
}
