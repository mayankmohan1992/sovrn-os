//! Feed event types (Nostr-compatible)

use serde::{Deserialize, Serialize};

/// Event kind constants
pub const KIND_PROFILE: i32 = 0;
pub const KIND_POST: i32 = 1;
pub const KIND_SHORT_NOTE: i32 = 2;
pub const KIND_REPOST: i32 = 6;
pub const KIND_REACTION: i32 = 7;
pub const KIND_DM: i32 = 14;
pub const KIND_CHANNEL_CREATE: i32 = 40;
pub const KIND_CHANNEL_MSG: i32 = 41;
pub const KIND_FOLLOW_LIST: i32 = 3;

/// Feed event — core data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub kind: i32,
    pub author: String,
    pub content: String,
    pub created_at: i64,
    pub tags: Vec<Vec<String>>,
    pub sig: String,
    pub raw_json: Option<String>,
}

/// Reaction summary for an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionSummary {
    pub like: i32,
    pub boost: i32,
}

/// Timeline response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResponse {
    pub events: Vec<Event>,
    pub cursor: Option<String>,
    pub has_more: bool,
}
