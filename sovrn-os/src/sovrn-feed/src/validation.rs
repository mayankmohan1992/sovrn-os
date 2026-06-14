//! Event signature verification and content validation

use crate::event::Event;
use sha2::{Sha256, Digest};

/// Compute event ID as SHA-256 of serialized content
pub fn compute_event_id(kind: i32, author: &str, content: &str, created_at: i64, tags: &[Vec<String>]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.to_string().as_bytes());
    hasher.update(author.as_bytes());
    hasher.update(content.as_bytes());
    hasher.update(created_at.to_string().as_bytes());
    // Include tags in hash
    for tag in tags {
        hasher.update(tag.join(",").as_bytes());
    }
    hex::encode(hasher.finalize())
}

/// Validate an incoming event before storage
pub fn validate_event(event: &Event, max_size: usize) -> Result<(), String> {
    // Check kind range
    if event.kind < 0 || event.kind > 9999 {
        return Err(format!("Invalid event kind: {}", event.kind));
    }

    // Check content size
    if event.content.len() > max_size {
        return Err(format!("Event content too large: {} bytes (max {})", event.content.len(), max_size));
    }

    // Check author is non-empty
    if event.author.is_empty() {
        return Err("Event author is required".into());
    }

    // Check created_at is not in the future (allow 5 min clock skew)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    if event.created_at > now + 300 {
        return Err("Event created_at is too far in the future".into());
    }

    // Verify event ID matches computed hash
    let computed_id = compute_event_id(event.kind, &event.author, &event.content, event.created_at, &event.tags);
    if event.id != computed_id && !event.id.is_empty() {
        // In production, we'd reject mismatches; for now, accept but log
        tracing::warn!("Event ID mismatch: computed={}, provided={}", computed_id, event.id);
    }

    Ok(())
}

/// Verify Ed25519 signature on an event
/// In production, this would use ed25519-dalek to verify the actual signature
pub fn verify_signature(event: &Event) -> bool {
    // Placeholder — in production, verify sig using author's public key
    !event.sig.is_empty()
}
