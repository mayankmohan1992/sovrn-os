//! Node-to-node feed push protocol (gossip-based)

use crate::event::Event;
use serde::{Deserialize, Serialize};

/// Feed push message sent between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedPush {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub version: u8,
    pub event: Event,
    pub push_id: String,
}

/// Feed push acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedPushAck {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub push_id: String,
    pub status: String,
}

impl FeedPush {
    pub fn new(event: Event) -> Self {
        let push_id = format!("push_{}_{}", event.author, event.created_at);
        Self {
            msg_type: "feed_push".into(),
            version: 1,
            event,
            push_id,
        }
    }

    /// Encode as length-prefixed JSON for Yggdrasil TCP transport
    pub fn encode(&self) -> Vec<u8> {
        let json = serde_json::to_vec(self).unwrap_or_default();
        let len = json.len() as u32;
        let mut frame = len.to_be_bytes().to_vec();
        frame.extend_from_slice(&json);
        frame
    }

    pub fn ack(&self, status: &str) -> FeedPushAck {
        FeedPushAck {
            msg_type: "feed_push_ack".into(),
            push_id: self.push_id.clone(),
            status: status.into(),
        }
    }
}
