//! HKDF key derivation for app passwords and sub-keys

use hkdf::Hkdf;
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Derive app-specific credentials from master key
/// Returns (username, password) for a given app_id
pub fn derive_app_key(master_key: &[u8], app_id: &str) -> (String, String) {
    let hkdf = Hkdf::<Sha256>::new(
        Some(b"sovrn-app-key-v1"),
        master_key,
    );

    // Derive username (16 bytes, hex-encoded, truncated to 12 chars)
    let mut username_bytes = [0u8; 16];
    hkdf.expand(format!("sovrn:username:{}", app_id).as_bytes(), &mut username_bytes)
        .expect("HKDF expansion failed");
    let username = hex::encode(username_bytes)[..12].to_string();

    // Derive password (32 bytes, base64-encoded)
    let mut password_bytes = [0u8; 32];
    hkdf.expand(format!("sovrn:password:{}", app_id).as_bytes(), &mut password_bytes)
        .expect("HKDF expansion failed");
    let password = BASE64.encode(&password_bytes);

    (username, password)
}

/// Derive a sub-key for a specific purpose
pub fn derive_sub_key(master_key: &[u8], purpose: &str, context: &str) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(
        Some(format!("sovrn:{}", purpose).as_bytes()),
        master_key,
    );

    let mut key = [0u8; 32];
    hkdf.expand(context.as_bytes(), &mut key)
        .expect("HKDF expansion failed");
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_app_key_deterministic() {
        let master = [0u8; 32];
        let (u1, p1) = derive_app_key(&master, "nextcloud");
        let (u2, p2) = derive_app_key(&master, "nextcloud");
        assert_eq!(u1, u2);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_different_apps_different_keys() {
        let master = [0u8; 32];
        let (u1, p1) = derive_app_key(&master, "nextcloud");
        let (u2, p2) = derive_app_key(&master, "vaultwarden");
        assert_ne!(u1, u2);
        assert_ne!(p1, p2);
    }
}
