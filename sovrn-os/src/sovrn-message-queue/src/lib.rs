//! Sovrn Message Queue — Encrypted DM delivery queue
//!
//! Manages encrypted direct messages with X25519 sealed-box encryption,
//! persistent SQLite storage (WAL mode), delivery receipts, and
//! node-to-node delivery. Communicates with sovrnd via JSON-RPC 2.0
//! over Unix sockets.

pub mod queue;
pub mod encryption;
pub mod delivery;
pub mod conversation;
pub mod rpc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqConfig {
    pub sockets_dir: String,
    pub data_dir: String,
    pub max_message_size: usize,
    pub delivery_timeout_secs: u64,
}

impl Default for MqConfig {
    fn default() -> Self {
        Self {
            sockets_dir: "/var/lib/sovrn/sockets".into(),
            data_dir: "/var/lib/sovrn/messages".into(),
            max_message_size: 65536,
            delivery_timeout_secs: 300,
        }
    }
}

pub struct MqService {
    pub config: MqConfig,
    pub store: std::sync::Mutex<queue::MessageStore>,
}

impl MqService {
    pub fn new(config: MqConfig) -> anyhow::Result<Self> {
        let db_path = std::path::PathBuf::from(&config.data_dir).join("messages.db");
        let store = queue::MessageStore::new(&db_path)?;
        Ok(Self { config, store: std::sync::Mutex::new(store) })
    }

    pub fn init_db(&self) -> anyhow::Result<()> {
        let mut store = self.store.lock().unwrap();
        store.init_tables()
    }
}
