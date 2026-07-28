#![allow(dead_code)]

use sha2::{Digest, Sha256};

pub struct LogRedactor {
    salt: String,
}

impl LogRedactor {
    pub fn new(salt: &str) -> Self {
        Self {
            salt: salt.to_string(),
        }
    }

    pub fn redact_vault_id(&self, vault_id: &str) -> String {
        let hash = self.hash_for_audit(vault_id);
        format!("vault:{}", &hash[..16])
    }

    pub fn redact_address(&self, address: &str) -> String {
        let hash = self.hash_for_audit(address);
        format!("addr:{}", &hash[..16])
    }

    pub fn redact_log_line(&self, log_line: &str) -> String {
        let mut result = log_line.to_string();

        let vault_pattern = regex_like_pattern("vault", &result);
        for matched in vault_pattern {
            let redacted = self.redact_vault_id(&matched);
            result = result.replace(&matched, &redacted);
        }

        let addr_pattern = regex_like_pattern("addr", &result);
        for matched in addr_pattern {
            let redacted = self.redact_address(&matched);
            result = result.replace(&matched, &redacted);
        }

        result
    }

    pub fn hash_for_audit(&self, identifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.salt.as_bytes());
        hasher.update(identifier.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

fn regex_like_pattern(_prefix: &str, _input: &str) -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_vault_id() {
        let redactor = LogRedactor::new("test_salt");
        let redacted = redactor.redact_vault_id("vault_abc123");
        assert!(redacted.starts_with("vault:"));
        assert_ne!(redacted, "vault_abc123");
    }

    #[test]
    fn test_redact_address() {
        let redactor = LogRedactor::new("test_salt");
        let redacted = redactor.redact_address("GAEXAMPLEADDRESS123");
        assert!(redacted.starts_with("addr:"));
    }

    #[test]
    fn test_hash_consistency() {
        let redactor = LogRedactor::new("salt");
        let h1 = redactor.hash_for_audit("test");
        let h2 = redactor.hash_for_audit("test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_different_salt_different_hash() {
        let r1 = LogRedactor::new("salt1");
        let r2 = LogRedactor::new("salt2");
        assert_ne!(r1.hash_for_audit("test"), r2.hash_for_audit("test"));
    }
}
