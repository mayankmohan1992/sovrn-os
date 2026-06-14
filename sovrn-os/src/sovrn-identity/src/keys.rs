//! Ed25519 + X25519 keypair generation and management

use ed25519_dalek::SigningKey;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// A complete Sovrn keypair: Ed25519 for signing, X25519 for encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovrnKeyPair {
    /// Ed25519 signing key (master key)
    pub signing_key_hex: String,
    /// Ed25519 verifying key (public)
    pub verifying_key_hex: String,
    /// X25519 public key (for DM encryption)
    pub x25519_public_hex: String,
    /// Derived from master key — human-readable identity
    pub pubkey_hash: String,
    /// BIP-39 seed phrase (generated once, stored encrypted)
    pub seed_phrase: Option<String>,
}

impl SovrnKeyPair {
/// Generate a new random keypair
pub fn generate() -> Self {
    let mut csprng = OsRng;
    let bytes: [u8; 32] = {
        let mut b = [0u8; 32];
        use rand::RngCore;
        csprng.fill_bytes(&mut b);
        b
    };
    let signing_key = SigningKey::from_bytes(&bytes);
    let verifying_key = signing_key.verifying_key();

        // Derive X25519 from Ed25519 key
        let x25519_secret = X25519StaticSecret::from(signing_key.to_bytes());
        let x25519_public = X25519PublicKey::from(&x25519_secret);

        // Create public key hash for identity
        let pubkey_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(verifying_key.as_bytes());
            let hash = hasher.finalize();
            hex::encode(&hash[..8]) // First 8 bytes = 16 hex chars
        };

        Self {
            signing_key_hex: hex::encode(signing_key.to_bytes()),
            verifying_key_hex: hex::encode(verifying_key.as_bytes()),
            x25519_public_hex: hex::encode(x25519_public.as_bytes()),
            pubkey_hash,
            seed_phrase: None,
        }
    }

    /// Restore keypair from seed phrase using BIP-39
    pub fn from_seed_phrase(phrase: &str) -> anyhow::Result<Self> {
        let seed = crate::seed::seed_phrase_to_seed(phrase)?;
        Self::from_seed(&seed)
    }

    /// Derive keypair from raw seed bytes
    pub fn from_seed(seed: &[u8]) -> anyhow::Result<Self> {
        let signing_key = SigningKey::from_bytes(
            &seed[..32].try_into().map_err(|_| anyhow::anyhow!("invalid seed length"))?
        );
        let verifying_key = signing_key.verifying_key();
        let x25519_secret = X25519StaticSecret::from(signing_key.to_bytes());
        let x25519_public = X25519PublicKey::from(&x25519_secret);

        let pubkey_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(verifying_key.as_bytes());
            let hash = hasher.finalize();
            hex::encode(&hash[..8])
        };

        Ok(Self {
            signing_key_hex: hex::encode(signing_key.to_bytes()),
            verifying_key_hex: hex::encode(verifying_key.as_bytes()),
            x25519_public_hex: hex::encode(x25519_public.as_bytes()),
            pubkey_hash,
            seed_phrase: None,
        })
    }

    /// Get Ed25519 signing key from hex
    pub fn signing_key(&self) -> anyhow::Result<SigningKey> {
        let bytes = hex::decode(&self.signing_key_hex)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| anyhow::anyhow!("invalid key"))?;
        Ok(SigningKey::from_bytes(&arr))
    }
}
