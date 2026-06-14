
//! Yggdrasil integration for DHT mesh communication

use crate::DhtService;
use anyhow::Result;
use tracing::{info, debug, warn};

use std::sync::Arc;

/// Listen for incoming DHT messages on Yggdrasil
pub async fn listen_yggdrasil(service: Arc<DhtService>) -> Result<()> {
    let listen_addr = format!("[::]:{}", service.config.listen_port);
    info!("DHT listening on Yggdrasil at {}", listen_addr);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        debug!("DHT connection from {}", addr);
        let svc = Arc::clone(&service);

        tokio::spawn(async move {
            if let Err(e) = handle_ygg_connection(stream, svc).await {
                warn!("Ygg connection error from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_ygg_connection(
    mut stream: tokio::net::TcpStream,
    _service: Arc<DhtService>,
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut len_buf = [0u8; 4];
    loop {
        // Read length prefix
        let n = stream.read(&mut len_buf).await?;
        if n == 0 {
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        // Read payload
        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload).await?;

        // Parse message
        let msg = match crate::protocol::decode(&payload) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to decode DHT message: {}", e);
                continue;
            }
        };

        debug!("Received DHT message: {:?}", msg);

        // Process message (route to appropriate handler)
        let response = crate::protocol::encode(&crate::protocol::DhtMessage::Pong {
            sender: _service.node_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64,
        });

        let resp_len = response.len() as u32;
        stream.write_all(&resp_len.to_be_bytes()).await?;
        stream.write_all(&response).await?;
    }

    Ok(())
}
