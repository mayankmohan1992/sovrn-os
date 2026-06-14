//! JSON-RPC 2.0 server for feed service

use std::sync::Arc;
use anyhow::{anyhow, Result};
use serde_json::Value;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;
use crate::FeedService;

pub async fn serve(service: Arc<FeedService>) -> Result<()> {
    let socket_path = std::path::PathBuf::from(&service.config.sockets_dir).join("feed.sock");
    if socket_path.exists() { std::fs::remove_file(&socket_path)?; }
    if let Some(parent) = socket_path.parent() { std::fs::create_dir_all(parent)?; }
    let listener = UnixListener::bind(&socket_path)?;
    info!("Feed RPC server listening on {:?}", socket_path);

    loop {
        let (stream, _) = listener.accept().await?;
        let svc = service.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(stream, svc).await {
                tracing::warn!("Feed connection error: {}", e);
            }
        });
    }
}

async fn handle_conn(stream: tokio::net::UnixStream, service: Arc<FeedService>) -> Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = vec![0u8; 131072]; // Larger buffer for feed events
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 { break; }
        let request: Value = match serde_json::from_slice(&buf[..n]) {
            Ok(r) => r,
            Err(e) => {
                let resp = json_err(-32700, &format!("Parse error: {}", e), Value::Null);
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
            Ok(v) => json_ok(v, id),
            Err(e) => json_err(-32603, &e.to_string(), id),
        };
        writer.write_all(&serde_json::to_vec(&response)?).await?;
        writer.write_all(b"\n").await?;
    }
    Ok(())
}

fn dispatch(service: &FeedService, method: &str, params: Value) -> Result<Value> {
    let mut store = service.store.lock().unwrap();
    match method {
        "feed.get_timeline" => {
            let user_id = params["user_id"].as_str().unwrap_or("default");
            let limit = params["limit"].as_u64().unwrap_or(50) as usize;
            let cursor = params["cursor"].as_str();
            let (events, next_cursor, has_more) = store.get_timeline(user_id, cursor, limit)?;
            Ok(serde_json::json!({"events": events, "cursor": next_cursor, "has_more": has_more}))
        }
        "feed.get_event" => {
            let event_id = params["event_id"].as_str().ok_or_else(|| anyhow!("missing event_id"))?;
            let event = store.get_event(event_id)?;
            Ok(serde_json::to_value(event)?)
        }
        "feed.create_event" => {
            let event = crate::event::Event {
                id: params["id"].as_str().unwrap_or("").into(),
                kind: params["kind"].as_i64().unwrap_or(1) as i32,
                author: params["author"].as_str().unwrap_or("").into(),
                content: params["content"].as_str().unwrap_or("").into(),
                created_at: params["created_at"].as_i64().unwrap_or(0),
                sig: params["sig"].as_str().unwrap_or("").into(),
                raw_json: params["raw_json"].as_str().map(|s| s.into()),
                tags: vec![],
            };
            let id = crate::validation::compute_event_id(event.kind, &event.author, &event.content, event.created_at, &event.tags);
            let mut event = event;
            event.id = id;
            store.create_event(&event)?;
            Ok(serde_json::to_value(&event)?)
        }
        "feed.delete_event" => {
            let event_id = params["event_id"].as_str().ok_or_else(|| anyhow!("missing event_id"))?;
            store.delete_event(event_id)?;
            Ok(serde_json::json!({"deleted": true}))
        }
        "feed.push_event" => {
            let event: crate::event::Event = serde_json::from_value(params)?;
            store.create_event(&event)?;
            Ok(serde_json::json!({"received": true}))
        }
        "feed.get_reactions" => {
            let event_id = params["event_id"].as_str().ok_or_else(|| anyhow!("missing event_id"))?;
            let reactions = store.get_reactions(event_id)?;
            Ok(serde_json::json!({"like": reactions.like, "boost": reactions.boost}))
        }
        "feed.add_reaction" => {
            let id = params["id"].as_str().ok_or_else(|| anyhow!("missing id"))?;
            let target = params["event_id"].as_str().ok_or_else(|| anyhow!("missing event_id"))?;
            let kind = params["kind"].as_i64().unwrap_or(7) as i32;
            let author = params["author"].as_str().ok_or_else(|| anyhow!("missing author"))?;
            store.add_reaction(id, target, kind, author)?;
            Ok(serde_json::json!({"added": true}))
        }
        "feed.search" => {
            let query = params["query"].as_str().ok_or_else(|| anyhow!("missing query"))?;
            let kind = params["kind"].as_i64().map(|k| k as i32);
            let limit = params["limit"].as_u64().unwrap_or(20) as usize;
            let events = store.search(query, kind, limit)?;
            Ok(serde_json::json!({"events": events}))
        }
        "feed.health" => {
            Ok(serde_json::json!({"status": "ok", "events_count": 0, "uptime": 0}))
        }
        _ => Err(anyhow!("Unknown method: {}", method)),
    }
}

fn json_ok(result: Value, id: Value) -> Value {
    serde_json::json!({"jsonrpc": "2.0", "result": result, "id": id})
}
fn json_err(code: i64, msg: &str, id: Value) -> Value {
    serde_json::json!({"jsonrpc": "2.0", "error": {"code": code, "message": msg}, "id": id})
}
