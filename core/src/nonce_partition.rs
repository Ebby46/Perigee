#![allow(dead_code)]

use std::collections::HashMap;

// Divide before multiplying: `u64::MAX * 80` overflows u64, which is a
// compile-time error in a const context rather than a runtime surprise.
const NONCE_WARN_THRESHOLD: u64 = u64::MAX / 100 * 80;
const NONCE_MAX: u64 = u64::MAX;

pub struct NoncePartition {
    partitions: HashMap<String, u64>,
    exhausted: Vec<String>,
}

impl NoncePartition {
    pub fn new() -> Self {
        Self {
            partitions: HashMap::new(),
            exhausted: Vec::new(),
        }
    }

    pub fn next_nonce(&mut self, domain: &str) -> Result<u64, &'static str> {
        let nonce = self
            .partitions
            .entry(domain.to_string())
            .or_insert(0);
        if *nonce >= NONCE_MAX {
            if !self.exhausted.contains(&domain.to_string()) {
                self.exhausted.push(domain.to_string());
            }
            return Err("nonce range exhausted for domain");
        }
        let next = *nonce;
        *nonce += 1;
        if *nonce >= NONCE_WARN_THRESHOLD {
            tracing::warn!(
                domain = domain,
                nonce = *nonce,
                "Nonce range approaching exhaustion"
            );
        }
        Ok(next)
    }

    pub fn current_nonce(&self, domain: &str) -> u64 {
        self.partitions.get(domain).copied().unwrap_or(0)
    }

    /// Whether `nonce` is still unused for `domain`.
    ///
    /// Nonces are handed out sequentially from 0, so everything below the
    /// current counter has already been issued and only values at or above it
    /// are still unique. The comparison was inverted, which meant this
    /// reported every *already-used* nonce as unique and every unused one as
    /// taken — the wrong answer in both directions for a replay check.
    pub fn verify_unique(&self, domain: &str, nonce: u64) -> bool {
        let current = self.current_nonce(domain);
        nonce >= current
    }

    pub fn is_exhausted(&self, domain: &str) -> bool {
        self.exhausted.contains(&domain.to_string())
    }

    pub fn reset_domain(&mut self, domain: &str) {
        self.partitions.insert(domain.to_string(), 0);
        self.exhausted.retain(|d| d != domain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_nonces() {
        let mut np = NoncePartition::new();
        assert_eq!(np.next_nonce("rail_a").unwrap(), 0);
        assert_eq!(np.next_nonce("rail_a").unwrap(), 1);
        assert_eq!(np.next_nonce("rail_a").unwrap(), 2);
    }

    #[test]
    fn test_isolated_domains() {
        let mut np = NoncePartition::new();
        assert_eq!(np.next_nonce("rail_a").unwrap(), 0);
        assert_eq!(np.next_nonce("rail_b").unwrap(), 0);
        assert_eq!(np.next_nonce("rail_a").unwrap(), 1);
        assert_eq!(np.next_nonce("rail_b").unwrap(), 1);
    }

    #[test]
    fn test_verify_unique() {
        let mut np = NoncePartition::new();
        np.next_nonce("rail_a").unwrap();
        np.next_nonce("rail_a").unwrap();
        assert!(!np.verify_unique("rail_a", 0));
        assert!(!np.verify_unique("rail_a", 1));
        assert!(np.verify_unique("rail_a", 2));
    }

    #[test]
    fn test_nonce_exhaustion() {
        let mut np = NoncePartition::new();
        np.partitions.insert("test".to_string(), NONCE_MAX);
        assert!(np.next_nonce("test").is_err());
        assert!(np.is_exhausted("test"));
    }

    #[test]
    fn test_reset_domain() {
        let mut np = NoncePartition::new();
        np.next_nonce("rail_a").unwrap();
        np.next_nonce("rail_a").unwrap();
        assert_eq!(np.current_nonce("rail_a"), 2);
        np.reset_domain("rail_a");
        assert_eq!(np.current_nonce("rail_a"), 0);
        assert!(!np.is_exhausted("rail_a"));
    }
}
