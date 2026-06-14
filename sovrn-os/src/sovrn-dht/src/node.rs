
//! DHT node: join, routing, bucket management

use crate::routing::RoutingEntry;

/// DHT node state
pub struct DhtNode {
    pub node_id: String,
    pub public_key: String,
    pub ygg_address: String,
    pub is_online: bool,
}

impl DhtNode {
    pub fn new(node_id: &str, public_key: &str, ygg_address: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            public_key: public_key.to_string(),
            ygg_address: ygg_address.to_string(),
            is_online: true,
        }
    }

    pub fn to_routing_entry(&self) -> RoutingEntry {
        RoutingEntry {
            node_id: self.node_id.clone(),
            public_key: self.public_key.clone(),
            addr: self.ygg_address.clone(),
            distance: 0,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        }
    }
}
