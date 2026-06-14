//! Heartbeat protocol — 2-minute intervals with explicit offline signal

use crate::PresenceService;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub version: u8,
    pub public_key: String,
    pub domain: String,
    pub status: PeerStatus,
    pub timestamp: i64,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PeerStatus {
    Online,
    Away,
    Offline,
}

impl PresenceService {
    /// Create a heartbeat message for this node
    pub fn create_heartbeat(&self, status: PeerStatus) -> Heartbeat {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Heartbeat {
            version: 1,
            public_key: self.node_id.clone(),
            domain: String::new(), // Filled by sovrnd
            status,
            timestamp: now,
            sig: String::new(), // Signed by identity service
        }
    }

    /// Process an incoming heartbeat from a peer
    pub fn process_heartbeat(&self, hb: &Heartbeat) -> anyhow::Result<()> {
        let mut ps = self.peer_store.lock().unwrap();
        ps.update_peer(
            &hb.public_key,
            &hb.domain,
            &format!("{:?}", hb.status).to_lowercase(),
            hb.timestamp,
        )
    }
}
