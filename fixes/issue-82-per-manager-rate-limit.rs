//! Fix for #82: apply a rate limit per manager instead of one shared
//! across all partners, so a single abusive manager can't degrade others.

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PerManagerRateLimiter {
    pub max_requests: u32,
    pub window: Duration,
    buckets: HashMap<String, (u32, Instant)>,
}

impl PerManagerRateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self { max_requests, window, buckets: HashMap::new() }
    }

    pub fn allow(&mut self, manager_id: &str) -> bool {
        let now = Instant::now();
        let entry = self
            .buckets
            .entry(manager_id.to_string())
            .or_insert((0, now));

        if now.duration_since(entry.1) > self.window {
            *entry = (0, now);
        }

        if entry.0 < self.max_requests {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_manager_exceeding_limit_does_not_block_another() {
        let mut limiter = PerManagerRateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.allow("mgr-a"));
        assert!(limiter.allow("mgr-a"));
        assert!(!limiter.allow("mgr-a"));
        assert!(limiter.allow("mgr-b"));
    }
}
