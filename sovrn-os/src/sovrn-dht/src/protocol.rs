
//! Wire protocol for DHT node-to-node communication
//! Length-prefixed JSON frames over Yggdrasil TCP connections

use serde::{Deserialize, Serialize};

/// DHT protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DhtMessage {
    /// Find a node responsible for a key
    #[serde(rename = "dht_find_node")]
    FindNode {
        version: u8,
        query_id: String,
        target_key: String,
        sender: String,
        sender_addr: String,
    },
    /// Response to FindNode
    #[serde(rename = "dht_find_node_response")]
    FindNodeResponse {
        query_id: String,
        nodes: Vec<NodeInfo>,
    },
    /// Store a value in the DHT
    #[serde(rename = "dht_store")]
    Store {
        key: String,
        value: String,
        ttl: u64,
        signature: String,
    },
    /// Response to Store
    #[serde(rename = "dht_store_response")]
    StoreResponse {
        stored: bool,
    },
    /// Ping to check if a node is alive
    #[serde(rename = "dht_ping")]
    Ping {
        sender: String,
        sender_addr: String,
        timestamp: i64,
    },
    /// Pong response
    #[serde(rename = "dht_pong")]
    Pong {
        sender: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub public_key: String,
    pub addr: String,
    pub distance: usize,
}

/// Encode a message as a length-prefixed JSON frame
pub fn encode(msg: &DhtMessage) -> Vec<u8> {
    let json = serde_json::to_vec(msg).unwrap_or_default();
    let len = json.len() as u32;
    let mut frame = len.to_be_bytes().to_vec();
    frame.extend_from_slice(&json);
    frame
}

/// Decode a length-prefixed JSON frame
pub fn decode(frame: &[u8]) -> Result<DhtMessage, serde_json::Error> {
    serde_json::from_slice(frame)
}
