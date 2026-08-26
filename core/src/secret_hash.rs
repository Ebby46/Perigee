//! One-way Argon2 hashing for stored secrets (e.g. webhook signing secrets).
//!
//! Any secret persisted through this module can never be recovered in
//! plaintext — only verified against a value presented back by the caller.
//! Do not use this for values that must later be read back in plaintext
//! (e.g. an HMAC signing key); those require encryption, not hashing.

use argon2::password_hash::{
    rand_core::OsRng, Error as PasswordHashError, PasswordHash, PasswordHasher,
    PasswordVerifier, SaltString,
};
use argon2::Argon2;

/// Hashes a plaintext secret with Argon2id, returning a self-describing
/// encoded hash (algorithm, params, salt, and digest) safe to store at rest.
pub fn hash_secret(secret: &str) -> Result<String, PasswordHashError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(secret.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verifies a plaintext secret against a previously stored Argon2 hash.
/// Returns `false` (rather than erroring) on any malformed hash or mismatch.
pub fn verify_secret(secret: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_succeeds() {
        let hash = hash_secret("my-webhook-secret").expect("hashing should succeed");
        assert!(verify_secret("my-webhook-secret", &hash));
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let hash = hash_secret("my-webhook-secret").expect("hashing should succeed");
        assert!(!verify_secret("not-the-secret", &hash));
    }

    #[test]
    fn verify_rejects_malformed_hash() {
        assert!(!verify_secret("anything", "not-a-real-hash"));
    }

    #[test]
    fn same_secret_hashes_differently_each_time() {
        let first = hash_secret("repeat-secret").expect("hashing should succeed");
        let second = hash_secret("repeat-secret").expect("hashing should succeed");
        assert_ne!(first, second, "salts must be randomized per hash");
        assert!(verify_secret("repeat-secret", &first));
        assert!(verify_secret("repeat-secret", &second));
    }
}
