//! Yggdrasil multicast for peer discovery

use crate::PresenceService;
use anyhow::Result;
use tracing::info;

impl PresenceService {
    pub async fn listen_yggdrasil(&self) -> Result<()> {
        let listen_addr = format!("[::]:{}", self.config.listen_port);
        info!("Presence listening on Yggdrasil at {}", listen_addr);

        let listener = tokio::net::UdpSocket::bind(&listen_addr).await?;

        let mut buf = vec![0u8; 65535];
        loop {
            let (n, _addr) = listener.recv_from(&mut buf).await?;
            let data = &buf[..n];
            if let Ok(msg) = serde_json::from_slice::<crate::heartbeat::Heartbeat>(data) {
                info!("Presence heartbeat from {}: status={:?}", msg.public_key, msg.status);
                self.process_heartbeat(&msg)?;
            }
        }
    }
}
