//! Feed timeline construction from follows and filters

use crate::storage::FeedStore;
use crate::event::TimelineResponse;
use anyhow::Result;

/// Build a timeline for a user based on their follows
pub fn build_timeline(
    store: &FeedStore,
    user_id: &str,
    cursor: Option<&str>,
    limit: usize,
) -> Result<TimelineResponse> {
    let (events, next_cursor, has_more) = store.get_timeline(user_id, cursor, limit)?;

    // In production, this would filter by follows and apply moderation rules
    Ok(TimelineResponse {
        events,
        cursor: next_cursor,
        has_more,
    })
}
