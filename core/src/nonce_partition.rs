#![allow(dead_code)]

use std::collections::HashMap;

pub struct NoncePartition {
    partitions: HashMap<String, u64>,
}

impl NoncePartition {
    pub fn new() -> Self {
        Self {
            partitions: HashMap::new(),
        }
    }

    pub fn next_nonce(&mut self, domain: &str) -> u64 {
        let nonce = self
            .partitions
            .entry(domain.to_string())
            .or_insert(0);
        let next = *nonce;
        *nonce += 1;
        next
    }

    pub fn current_nonce(&self, domain: &str) -> u64 {
        self.partitions.get(domain).copied().unwrap_or(0)
    }

    pub fn verify_unique(&self, domain: &str, nonce: u64) -> bool {
        let current = self.current_nonce(domain);
        nonce < current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_nonces() {
        let mut np = NoncePartition::new();
        assert_eq!(np.next_nonce("rail_a"), 0);
        assert_eq!(np.next_nonce("rail_a"), 1);
        assert_eq!(np.next_nonce("rail_a"), 2);
    }

    #[test]
    fn test_isolated_domains() {
        let mut np = NoncePartition::new();
        assert_eq!(np.next_nonce("rail_a"), 0);
        assert_eq!(np.next_nonce("rail_b"), 0);
        assert_eq!(np.next_nonce("rail_a"), 1);
        assert_eq!(np.next_nonce("rail_b"), 1);
    }

    #[test]
    fn test_verify_unique() {
        let mut np = NoncePartition::new();
        np.next_nonce("rail_a");
        np.next_nonce("rail_a");
        assert!(!np.verify_unique("rail_a", 0));
        assert!(!np.verify_unique("rail_a", 1));
        assert!(np.verify_unique("rail_a", 2));
    }
}
