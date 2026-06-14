//! Sovrn Presence Service — Heartbeat & peer tracking
//!
//! Manages online/offline status, 2-minute heartbeats, and gossip-based
//! presence propagation over Yggdrasil mesh. Communicates with sovrnd
//! via JSON-RPC 2.0 over Unix sockets.

pub mod heartbeat;
pub mod gossip;
pub mod peer_store;
pub mod rpc;
pub mod ygg;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceConfig {
    pub listen_port: u16,
    pub ygg_address: Option<String>,
    pub heartbeat_interval_secs: u64,
    pub gossip_ttl: u32,
    pub cache_ttl_secs: u64,
    pub sockets_dir: String,
    pub data_dir: String,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            listen_port: 54775,
            ygg_address: None,
            heartbeat_interval_secs: 120,
            gossip_ttl: 3,
            cache_ttl_secs: 180,
            sockets_dir: "/var/lib/sovrn/sockets".into(),
            data_dir: "/var/lib/sovrn/presence".into(),
        }
    }
}

pub struct PresenceService {
    pub config: PresenceConfig,
    pub node_id: String,
    pub peer_store: std::sync::Mutex<peer_store::PeerStore>,
}

impl PresenceService {
    pub fn new(config: PresenceConfig) -> anyhow::Result<Self> {
        let db_path = std::path::PathBuf::from(&config.data_dir).join("presence.db");
        let peer_store = peer_store::PeerStore::new(&db_path)?;
        let node_id = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let bytes: [u8; 16] = rng.gen();
            hex::encode(bytes)
        };
        Ok(Self {
            config,
            node_id,
            peer_store: std::sync::Mutex::new(peer_store),
        })
    }

    pub fn init_db(&self) -> anyhow::Result<()> {
        let mut ps = self.peer_store.lock().unwrap();
        ps.init_tables()
    }
}
