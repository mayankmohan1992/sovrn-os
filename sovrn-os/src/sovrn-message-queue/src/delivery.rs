//! Node-to-node message delivery protocol

use crate::queue::Message;
use serde::{Deserialize, Serialize};

/// DM delivery envelope (node-to-node)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmDelivery {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub version: u8,
    pub message_id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub created_at: i64,
    pub sig: String,
}

/// DM delivery acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmAck {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub message_id: String,
    pub status: String,
}

impl DmDelivery {
    pub fn from_message(msg: &Message) -> Self {
        Self {
            msg_type: "dm_delivery".into(),
            version: 1,
            message_id: msg.id.clone(),
            from: msg.from_id.clone(),
            to: msg.to_id.clone(),
            content: msg.content.clone(),
            created_at: msg.created_at,
            sig: msg.sig.clone(),
        }
    }

    pub fn ack(&self, status: &str) -> DmAck {
        DmAck {
            msg_type: "dm_ack".into(),
            message_id: self.message_id.clone(),
            status: status.into(),
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
}
