//! DID generation: did:mesh:{pubkey_hash}

use sha2::{Sha256, Digest};

/// Generate a Sovrn DID from a public key
/// Format: did:mesh:{first 16 hex chars of SHA-256 of public key}
pub fn generate_did(public_key: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();
    let pubkey_hash = hex::encode(&hash[..8]); // First 8 bytes = 16 hex chars
    format!("did:mesh:{}", pubkey_hash)
}

/// Parse a DID and extract the pubkey hash
pub fn parse_did(did: &str) -> Option<&str> {
    did.strip_prefix("did:mesh:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_did() {
        let key = [0u8; 32];
        let did = generate_did(&key);
        assert!(did.starts_with("did:mesh:"));
        assert_eq!(did.len(), 24); // "did:mesh:" (9) + 16 hex chars
    }

    #[test]
    fn test_parse_did() {
        let did = "did:mesh:a7x3k9m2p8f1q4b5";
        assert_eq!(parse_did(did), Some("a7x3k9m2p8f1q4b5"));
        assert_eq!(parse_did("invalid"), None);
    }
}
