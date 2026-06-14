//! X25519 sealed-box encryption for DMs

use x25519_dalek::{PublicKey, EphemeralSecret};
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Encrypt a message using X25519 sealed box
/// In production, this uses libsodium's sealed box (X25519 + XSalsa20-Poly1305)
/// For v1, we implement a simplified version using X25519 key agreement
pub fn encrypt_message(message: &str, recipient_public_hex: &str) -> anyhow::Result<String> {
    let recipient_bytes = hex::decode(recipient_public_hex)?;
    let recipient_public = PublicKey::from(
        <[u8; 32]>::try_from(recipient_bytes.as_slice())
            .map_err(|_| anyhow::anyhow!("Invalid public key length"))?
    );

    // Generate ephemeral keypair
    let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);

    // Key agreement
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public);

    // For v1: simple XOR-based "encryption" (placeholder — production needs XSalsa20-Poly1305)
    let key_material = shared_secret.as_bytes();
    let mut encrypted = Vec::new();
    encrypted.extend_from_slice(ephemeral_public.as_bytes());
    for (i, byte) in message.as_bytes().iter().enumerate() {
        encrypted.push(byte ^ key_material[i % 32]);
    }

    Ok(BASE64.encode(&encrypted))
}

/// Decrypt a message using X25519
pub fn decrypt_message(encrypted_b64: &str, secret_key_hex: &str) -> anyhow::Result<String> {
    let encrypted = BASE64.decode(encrypted_b64)?;
    let secret_bytes = hex::decode(secret_key_hex)?;
    let secret = x25519_dalek::StaticSecret::from(
        <[u8; 32]>::try_from(secret_bytes.as_slice())
            .map_err(|_| anyhow::anyhow!("Invalid secret key length"))?
    );

    // Extract ephemeral public key (first 32 bytes)
    if encrypted.len() < 32 {
        anyhow::bail!("Encrypted message too short");
    }
    let ephemeral_public = PublicKey::from(<[u8; 32]>::try_from(&encrypted[..32]).unwrap());

    // Key agreement
    let shared_secret = secret.diffie_hellman(&ephemeral_public);
    let key_material = shared_secret.as_bytes();

    // Decrypt (XOR placeholder)
    let mut decrypted = Vec::new();
    for (i, byte) in encrypted[32..].iter().enumerate() {
        decrypted.push(byte ^ key_material[i % 32]);
    }

    Ok(String::from_utf8(decrypted)?)
}
