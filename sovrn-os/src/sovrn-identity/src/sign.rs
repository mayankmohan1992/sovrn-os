//! Event signing (Nostr-compatible Ed25519)

use ed25519_dalek::{SigningKey, Signer, Verifier, Signature};
use serde_json::Value;

/// Sign a JSON event with Ed25519
/// Follows Nostr convention: sign the serialized event (sans sig field)
pub fn sign_event(signing_key: &SigningKey, event: &mut Value) -> anyhow::Result<String> {
    // Remove any existing sig field
    if let Some(obj) = event.as_object_mut() {
        obj.remove("sig");
    }

    // Serialize the event for signing (canonical JSON)
    let serialized = serde_json::to_string(event)?;

    // Sign with Ed25519
    let signature = signing_key.sign(serialized.as_bytes());
    Ok(hex::encode(signature.to_bytes()))
}

/// Verify a signed event
pub fn verify_event(verifying_key_bytes: &[u8], event: &Value, sig_hex: &str) -> bool {
    let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(
        &<[u8; 32]>::try_from(verifying_key_bytes).unwrap_or([0u8; 32])
    ) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let sig_bytes = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let signature = match Signature::from_slice(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Recreate the event without sig for verification
    let mut event_copy = event.clone();
    if let Some(obj) = event_copy.as_object_mut() {
        obj.remove("sig");
    }

    let serialized = match serde_json::to_string(&event_copy) {
        Ok(s) => s,
        Err(_) => return false,
    };

    verifying_key.verify(serialized.as_bytes(), &signature).is_ok()
}
