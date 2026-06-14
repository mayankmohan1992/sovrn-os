//! JSON-RPC 2.0 server for message queue service

use std::sync::Arc;
use anyhow::{anyhow, Result};
use serde_json::Value;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;
use crate::MqService;

pub async fn serve(service: Arc<MqService>) -> Result<()> {
    let socket_path = std::path::PathBuf::from(&service.config.sockets_dir).join("mq.sock");
    if socket_path.exists() { std::fs::remove_file(&socket_path)?; }
    if let Some(parent) = socket_path.parent() { std::fs::create_dir_all(parent)?; }
    let listener = UnixListener::bind(&socket_path)?;
    info!("Message Queue RPC server listening on {:?}", socket_path);

    loop {
        let (stream, _) = listener.accept().await?;
        let svc = service.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(stream, svc).await {
                tracing::warn!("MQ connection error: {}", e);
            }
        });
    }
}

async fn handle_conn(stream: tokio::net::UnixStream, service: Arc<MqService>) -> Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = vec![0u8; 65536];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 { break; }
        let request: Value = match serde_json::from_slice(&buf[..n]) {
            Ok(r) => r,
            Err(e) => {
                let resp = jerr(-32700, &format!("Parse error: {}", e), Value::Null);
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
            Ok(v) => jok(v, id),
            Err(e) => jerr(-32603, &e.to_string(), id),
        };
        writer.write_all(&serde_json::to_vec(&response)?).await?;
        writer.write_all(b"\n").await?;
    }
    Ok(())
}

fn dispatch(service: &MqService, method: &str, params: Value) -> Result<Value> {
    let mut store = service.store.lock().unwrap();
    match method {
        "mq.get_conversations" => {
            let user_id = params["user_id"].as_str().ok_or_else(|| anyhow!("missing user_id"))?;
            let limit = params["limit"].as_u64().unwrap_or(20) as usize;
            let convs = store.get_conversations(user_id, limit)?;
            Ok(serde_json::json!({"conversations": convs}))
        }
        "mq.get_messages" => {
            let peer_id = params["peer_id"].as_str().ok_or_else(|| anyhow!("missing peer_id"))?;
            let user_id = params["user_id"].as_str().ok_or_else(|| anyhow!("missing user_id"))?;
            let cursor = params["cursor"].as_i64();
            let limit = params["limit"].as_u64().unwrap_or(50) as usize;
            let messages = store.get_messages(peer_id, user_id, cursor, limit)?;
            let has_more = messages.len() == limit;
            let next_cursor = messages.last().map(|m| m.created_at);
            Ok(serde_json::json!({"messages": messages, "cursor": next_cursor, "has_more": has_more}))
        }
        "mq.send_dm" => {
            let from = params["from_id"].as_str().ok_or_else(|| anyhow!("missing from_id"))?;
            let to = params["to_id"].as_str().ok_or_else(|| anyhow!("missing to_id"))?;
            let content = params["content"].as_str().ok_or_else(|| anyhow!("missing content"))?;
            let sig = params["sig"].as_str().unwrap_or("");
            let msg_id = store.send_dm(from, to, content, sig)?;
            Ok(serde_json::json!({"message_id": msg_id}))
        }
        "mq.mark_read" => {
            let peer_id = params["peer_id"].as_str().ok_or_else(|| anyhow!("missing peer_id"))?;
            let user_id = params["user_id"].as_str().ok_or_else(|| anyhow!("missing user_id"))?;
            store.mark_read(peer_id, user_id)?;
            Ok(serde_json::json!({"updated": true}))
        }
        "mq.receive_dm" => {
            let from = params["from_id"].as_str().ok_or_else(|| anyhow!("missing from_id"))?;
            let to = params["to_id"].as_str().ok_or_else(|| anyhow!("missing to_id"))?;
            let content = params["content"].as_str().ok_or_else(|| anyhow!("missing content"))?;
            let sig = params["sig"].as_str().unwrap_or("");
            store.receive_dm(from, to, content, sig)?;
            Ok(serde_json::json!({"received": true}))
        }
        "mq.get_delivery_status" => {
            let msg_id = params["message_id"].as_str().ok_or_else(|| anyhow!("missing message_id"))?;
            let status = store.get_delivery_status(msg_id)?;
            Ok(serde_json::json!({"status": status}))
        }
        "mq.health" => {
            let depth = store.get_queue_depth()?;
            Ok(serde_json::json!({"status": "ok", "queue_depth": depth, "uptime": 0}))
        }
        _ => Err(anyhow!("Unknown method: {}", method)),
    }
}

fn jok(result: Value, id: Value) -> Value {
    serde_json::json!({"jsonrpc": "2.0", "result": result, "id": id})
}
fn jerr(code: i64, msg: &str, id: Value) -> Value {
    serde_json::json!({"jsonrpc": "2.0", "error": {"code": code, "message": msg}, "id": id})
}
