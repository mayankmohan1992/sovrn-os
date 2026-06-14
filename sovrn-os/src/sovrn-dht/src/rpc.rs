
//! JSON-RPC 2.0 server for sovrnd ↔ DHT communication

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn};

use crate::DhtService;

/// JSON-RPC 2.0 request
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// Start the JSON-RPC server on Unix socket
pub async fn serve(service: Arc<DhtService>) -> Result<()> {
    let socket_path = PathBuf::from(&service.config.sockets_dir).join("dht.sock");

    // Remove stale socket
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    info!("DHT RPC server listening on {:?}", socket_path);

    loop {
        let (stream, _) = listener.accept().await?;
        let service = service.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, service).await {
                warn!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    service: Arc<DhtService>,
) -> Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = vec![0u8; 65536];

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        let request: JsonRpcRequest = match serde_json::from_slice(&buf[..n]) {
            Ok(req) => req,
            Err(e) => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                    id: Value::Null,
                };
                let resp_bytes = serde_json::to_vec(&resp)?;
                writer.write_all(&resp_bytes).await?;
                writer.write_all(b"\n").await?;
                continue;
            }
        };

        let result = dispatch(&service, &request.method, request.params);
        let id = request.id.unwrap_or(Value::Null);

        let response = match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                result: Some(value),
                error: None,
                id,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: e.to_string(),
                }),
                id,
            },
        };

        let resp_bytes = serde_json::to_vec(&response)?;
        writer.write_all(&resp_bytes).await?;
        writer.write_all(b"\n").await?;
    }

    Ok(())
}

fn dispatch(service: &DhtService, method: &str, params: Value) -> Result<Value> {
    match method {
        "dht.lookup" => {
            let domain: String = params["domain"].as_str()
                .ok_or_else(|| anyhow!("missing domain param"))?
                .to_string();
            let mut store = service.store.lock().unwrap();
            let record = store.lookup(&domain)?;
            Ok(serde_json::to_value(record)?)
        }
        "dht.register" => {
            let name = params["name"].as_str()
                .ok_or_else(|| anyhow!("missing name param"))?
                .to_string();
            let public_key = params["public_key"].as_str()
                .ok_or_else(|| anyhow!("missing public_key param"))?
                .to_string();
            let signature = params["signature"].as_str()
                .ok_or_else(|| anyhow!("missing signature param"))?
                .to_string();
            let ygg_address = params["ygg_address"].as_str()
                .unwrap_or("")
                .to_string();

            let mut store = service.store.lock().unwrap();
            let domain = store.register(&name, &public_key, &signature, &ygg_address)?;
            Ok(serde_json::json!({
                "domain": domain,
                "registered": true
            }))
        }
        "dht.check_available" => {
            let name = params["name"].as_str()
                .ok_or_else(|| anyhow!("missing name param"))?
                .to_string();
            let mut store = service.store.lock().unwrap();
            let available = store.check_available(&name)?;
            let suggestions = store.suggest_alternatives(&name, 3)?;
            Ok(serde_json::json!({
                "available": available,
                "suggestions": suggestions
            }))
        }
        "dht.get_peers" => {
            let limit = params["limit"].as_u64().unwrap_or(20) as usize;
            let rt = service.routing_table.lock().unwrap();
            let peers = rt.get_closest_peers(&service.node_id, limit);
            Ok(serde_json::to_value(peers)?)
        }
        "dht.put" => {
            let key = params["key"].as_str()
                .ok_or_else(|| anyhow!("missing key param"))?
                .to_string();
            let value = params["value"].as_str()
                .ok_or_else(|| anyhow!("missing value param"))?
                .to_string();
            let ttl = params["ttl"].as_u64().unwrap_or(86400);
            let mut store = service.store.lock().unwrap();
            store.put(&key, &value, ttl as i64)?;
            Ok(serde_json::json!({"stored": true}))
        }
        "dht.get" => {
            let key = params["key"].as_str()
                .ok_or_else(|| anyhow!("missing key param"))?
                .to_string();
            let mut store = service.store.lock().unwrap();
            let value = store.get(&key)?;
            Ok(serde_json::json!({
                "value": value,
                "found": value.is_some()
            }))
        }
        "dht.health" => {
            let node_count = service.routing_table.lock().unwrap().bucket_count();
            Ok(serde_json::json!({
                "status": "ok",
                "nodes_count": node_count,
                "uptime_seconds": 0
            }))
        }
        _ => Err(anyhow!("Unknown method: {}", method)),
    }
}
