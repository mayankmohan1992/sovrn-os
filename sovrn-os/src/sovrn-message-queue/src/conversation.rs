//! Conversation threading and management

use crate::queue::Message;
use serde::{Deserialize, Serialize};

/// Conversation summary for the PWA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub peer_id: String,
    pub peer_name: Option<String>,
    pub last_message: Option<String>,
    pub last_message_at: i64,
    pub unread_count: i32,
}

/// Message with decryption status for the PWA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedMessage {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub content: String,
    pub created_at: i64,
    pub status: String,
}

impl From<&Message> for DecryptedMessage {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.id.clone(),
            from_id: msg.from_id.clone(),
            to_id: msg.to_id.clone(),
            content: msg.content.clone(),
            created_at: msg.created_at,
            status: msg.delivery_status.clone(),
        }
    }
}
