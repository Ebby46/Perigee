use std::collections::HashMap;
use std::time::Instant;

pub struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    pub fn available(&self) -> f64 {
        self.tokens
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

pub struct AgentRateLimiter {
    buckets: HashMap<String, TokenBucket>,
    default_max: f64,
    default_refill_rate: f64,
}

impl AgentRateLimiter {
    pub fn new(default_max: f64, default_refill_rate: f64) -> Self {
        Self {
            buckets: HashMap::new(),
            default_max,
            default_refill_rate,
        }
    }

    pub fn register_agent(&mut self, agent_id: &str) {
        self.buckets
            .entry(agent_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.default_max, self.default_refill_rate));
    }

    pub fn try_acquire(&mut self, agent_id: &str, tokens: f64) -> bool {
        let default_max = self.default_max;
        let default_refill_rate = self.default_refill_rate;
        let bucket = self
            .buckets
            .entry(agent_id.to_string())
            .or_insert_with(|| TokenBucket::new(default_max, default_refill_rate));
        bucket.try_consume(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(3.0, 1.0);
        assert!(bucket.try_consume(1.0));
        assert!(bucket.try_consume(1.0));
        assert!(bucket.try_consume(1.0));
        assert!(!bucket.try_consume(1.0));
    }

    #[test]
    fn test_token_bucket_available() {
        let mut bucket = TokenBucket::new(5.0, 1.0);
        assert_eq!(bucket.available(), 5.0);
        bucket.try_consume(2.0);
        assert_eq!(bucket.available(), 3.0);
    }

    #[test]
    fn test_agent_rate_limiter() {
        let mut limiter = AgentRateLimiter::new(2.0, 0.0);
        assert!(limiter.try_acquire("a", 1.0));
        assert!(limiter.try_acquire("a", 1.0));
        assert!(!limiter.try_acquire("a", 1.0));
    }

    #[test]
    fn test_agent_rate_limiter_auto_register() {
        let mut limiter = AgentRateLimiter::new(1.0, 0.0);
        assert!(limiter.try_acquire("new-agent", 1.0));
        assert!(!limiter.try_acquire("new-agent", 1.0));
    }
}
