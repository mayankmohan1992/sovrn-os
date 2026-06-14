//! Sovrn Feed Service — Social feed event processing
//!
//! Append-only event storage with Ed25519 signature verification,
//! timeline construction from follows, and node-to-node push protocol.
//! Communicates with sovrnd via JSON-RPC 2.0 over Unix sockets.

pub mod event;
pub mod validation;
pub mod storage;
pub mod timeline;
pub mod push;
pub mod rpc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    pub sockets_dir: String,
    pub data_dir: String,
    pub max_event_size: usize,
    pub timeline_page_size: usize,
}

impl Default for FeedConfig {
    fn default() -> Self {
        Self {
            sockets_dir: "/var/lib/sovrn/sockets".into(),
            data_dir: "/var/lib/sovrn/feed".into(),
            max_event_size: 65536,
            timeline_page_size: 50,
        }
    }
}

pub struct FeedService {
    pub config: FeedConfig,
    pub store: std::sync::Mutex<storage::FeedStore>,
}

impl FeedService {
    pub fn new(config: FeedConfig) -> anyhow::Result<Self> {
        let db_path = std::path::PathBuf::from(&config.data_dir).join("feed.db");
        let store = storage::FeedStore::new(&db_path)?;
        Ok(Self { config, store: std::sync::Mutex::new(store) })
    }

    pub fn init_db(&self) -> anyhow::Result<()> {
        let mut store = self.store.lock().unwrap();
        store.init_tables()
    }
}
