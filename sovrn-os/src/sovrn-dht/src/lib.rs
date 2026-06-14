
//! Sovrn DHT Service — Distributed Hash Table for .sovrn domain resolution
//!
//! Implements a Kademlia-based DHT for resolving .sovrn domains to Yggdrasil
//! addresses. Communicates with sovrnd via JSON-RPC 2.0 over Unix sockets.

pub mod node;
pub mod protocol;
pub mod routing;
pub mod rpc;
pub mod store;
pub mod ygg;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// DHT service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtConfig {
    /// Port for Yggdrasil peer communication
    pub listen_port: u16,
    /// Yggdrasil local address (auto-detected if empty)
    pub ygg_address: Option<String>,
    /// Seed nodes for bootstrapping
    pub bootstrap_nodes: Vec<String>,
    /// K-bucket size for routing table
    pub k_bucket_size: usize,
    /// Default TTL for DHT records (seconds)
    pub default_ttl: u64,
    /// Directory for Unix domain sockets
    pub sockets_dir: String,
    /// Directory for persistent data
    pub data_dir: String,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            listen_port: 54774,
            ygg_address: None,
            bootstrap_nodes: vec![],
            k_bucket_size: 20,
            default_ttl: 86400,
            sockets_dir: "/var/lib/sovrn/sockets".into(),
            data_dir: "/var/lib/sovrn/dht".into(),
        }
    }
}

/// Core DHT service
pub struct DhtService {
    pub config: DhtConfig,
    pub node_id: String,
    pub routing_table: std::sync::Mutex<routing::RoutingTable>,
    pub store: std::sync::Mutex<store::DhtStore>,
}

impl DhtService {
    pub fn new(config: DhtConfig) -> anyhow::Result<Self> {
        let db_path = PathBuf::from(&config.data_dir).join("dht.db");
        let store = store::DhtStore::new(&db_path)?;
        let node_id = Self::generate_node_id();
        let routing_table = routing::RoutingTable::new(node_id.clone(), config.k_bucket_size);

        Ok(Self {
            config,
            node_id,
            routing_table: std::sync::Mutex::new(routing_table),
            store: std::sync::Mutex::new(store),
        })
    }

    fn generate_node_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.gen();
        hex::encode(bytes)
    }

    pub fn init_db(&self) -> anyhow::Result<()> {
        let mut store = self.store.lock().unwrap();
        store.init_tables()
    }
}
