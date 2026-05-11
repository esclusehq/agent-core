//! Rate limiting for tasks

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

const DEFAULT_LIMITS: &[(&str, u32, u64)] = &[
    ("server.start", 10, 60),
    ("server.stop", 10, 60),
    ("server.restart", 10, 60),
    ("server.create", 5, 60),
    ("server.delete", 5, 60),
    ("backup.create", 3, 3600),
    ("backup.restore", 2, 3600),
    ("ssh.execute", 30, 60),
    ("metrics.report", 60, 60),
];

pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    limits: HashMap<String, (u32, u64)>,
}

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: u32, per_seconds: u64) -> Self {
        Self {
            tokens: max_tokens as f64,
            max_tokens: max_tokens as f64,
            refill_rate: max_tokens as f64 / per_seconds as f64,
            last_refill: Instant::now(),
        }
    }

    fn try_acquire(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        let mut limits = HashMap::new();
        for (task_type, count, period) in DEFAULT_LIMITS {
            limits.insert(task_type.to_string(), (*count, *period));
        }

        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            limits,
        }
    }

    pub fn with_limits(mut self, limits: HashMap<String, (u32, u64)>) -> Self {
        self.limits = limits;
        self
    }

    pub async fn check(&self, task_type: &str) -> Result<(), RateLimitError> {
        if let Some((max_tokens, per_seconds)) = self.limits.get(task_type) {
            let mut buckets = self.buckets.lock().await;

            let bucket = buckets
                .entry(task_type.to_string())
                .or_insert_with(|| TokenBucket::new(*max_tokens, *per_seconds));

            if !bucket.try_acquire() {
                return Err(RateLimitError {
                    task_type: task_type.to_string(),
                });
            }
        }

        Ok(())
    }

    pub async fn reset(&self, task_type: &str) {
        let mut buckets = self.buckets.lock().await;
        buckets.remove(task_type);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RateLimitError {
    pub task_type: String,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rate limit exceeded for task type: {}", self.task_type)
    }
}

impl std::error::Error for RateLimitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows() {
        let limiter = RateLimiter::new();
        
        let result = limiter.check("server.start").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_respects_limit() {
        let limiter = RateLimiter::new();
        
        for _ in 0..10 {
            limiter.check("server.start").await.unwrap();
        }
        
        let result = limiter.check("server.start").await;
        assert!(result.is_err());
    }
}
