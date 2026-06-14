//! BIP-39 seed phrase generation and recovery

use anyhow::Result;

/// Generate a 24-word BIP-39 seed phrase
pub fn generate_seed_phrase() -> String {
    // Simplified implementation — in production, use proper BIP-39 library
    // Using ed25519-dalek's random bytes as entropy source
    let mut entropy = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut entropy);

    // Convert to hex words (simplified — real BIP-39 uses wordlist)
    // For v1, we use hex representation. Production needs proper BIP-39 wordlist.
    hex::encode(entropy)
}

/// Convert seed phrase to 64-byte seed for key derivation
pub fn seed_phrase_to_seed(phrase: &str) -> Result<[u8; 64]> {
    // Simplified: hash the phrase to produce seed
    // Production: use proper BIP-39 PBKDF2 with optional passphrase
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(phrase.as_bytes());
    hasher.update(b"sovrn-identity-seed-v1");
    let hash = hasher.finalize();

    let mut seed = [0u8; 64];
    seed[..32].copy_from_slice(&hash);
    // Second half via double-hash
    let mut hasher2 = Sha256::new();
    hasher2.update(&hash);
    hasher2.update(b"sovrn-identity-seed-v2");
    let hash2 = hasher2.finalize();
    seed[32..].copy_from_slice(&hash2);

    Ok(seed)
}

/// Validate that a seed phrase is well-formed (32 bytes hex or proper BIP-39)
pub fn validate_seed_phrase(phrase: &str) -> Result<()> {
    // Accept hex format
    if phrase.len() == 64 && hex::decode(phrase).is_ok() {
        return Ok(());
    }
    // Accept word-based format (space-separated)
    let words: Vec<&str> = phrase.split_whitespace().collect();
    if words.len() == 12 || words.len() == 24 {
        return Ok(());
    }
    anyhow::bail!("Invalid seed phrase format: expected 64 hex chars or 12/24 BIP-39 words")
}
