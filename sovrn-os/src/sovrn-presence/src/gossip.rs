//! Gossip protocol for presence propagation
//!
//! Heartbeats propagate with TTL=3 (max 3 hops). Each node
//! forwards heartbeats to known peers, decrementing TTL.

use crate::heartbeat::Heartbeat;
use crate::PresenceService;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GossipMessage {
    pub heartbeat: Heartbeat,
    pub ttl: u32,
    pub origin: String,
}

impl PresenceService {
    /// Create a gossip message wrapping a heartbeat
    pub fn create_gossip(&self, hb: Heartbeat) -> GossipMessage {
        GossipMessage {
            heartbeat: hb,
            ttl: self.config.gossip_ttl,
            origin: self.node_id.clone(),
        }
    }

    /// Process an incoming gossip message
    /// Returns true if this message should be forwarded
    pub fn process_gossip(&self, msg: &GossipMessage) -> anyhow::Result<bool> {
        // Process the heartbeat
        self.process_heartbeat(&msg.heartbeat)?;

        // Forward only if TTL > 0
        Ok(msg.ttl > 0)
    }
}
