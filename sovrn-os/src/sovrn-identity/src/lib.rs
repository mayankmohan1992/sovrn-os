//! Sovrn Identity Service — Key management, seed phrases, signing
//!
//! Manages Ed25519 signing keys, X25519 encryption keys, BIP-39 seed phrases,
//! HKDF key derivation, DID generation, and key storage. Communicates with
//! sovrnd via JSON-RPC 2.0 over Unix sockets.

pub mod keys;
pub mod seed;
pub mod derive;
pub mod sign;
pub mod did;
pub mod store;
pub mod rpc;

use serde::{Deserialize, Serialize};

/// Identity service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    pub sockets_dir: String,
    pub data_dir: String,
    pub key_derivation_iterations: u32,
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            sockets_dir: "/var/lib/sovrn/sockets".into(),
            data_dir: "/var/lib/sovrn/identity".into(),
            key_derivation_iterations: 600_000,
        }
    }
}

/// Core identity service
pub struct IdentityService {
    pub config: IdentityConfig,
    pub key_store: std::sync::Mutex<store::KeyStore>,
}

impl IdentityService {
    pub fn new(config: IdentityConfig) -> anyhow::Result<Self> {
        let db_path = std::path::PathBuf::from(&config.data_dir).join("identity.db");
        let key_store = store::KeyStore::new(&db_path)?;
        Ok(Self { config, key_store: std::sync::Mutex::new(key_store) })
    }

    pub fn init_db(&self) -> anyhow::Result<()> {
        let mut ks = self.key_store.lock().unwrap();
        ks.init_tables()
    }
}
