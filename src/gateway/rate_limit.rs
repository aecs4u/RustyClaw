//! Per-IP token bucket rate limiting for gateway message processing.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
        }
    }
}

/// Shared per-IP token bucket limiter.
#[derive(Clone)]
pub struct IpRateLimiter {
    capacity: f64,
    refill_per_sec: f64,
    buckets: Arc<Mutex<HashMap<IpAddr, TokenBucket>>>,
}

impl IpRateLimiter {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            capacity: capacity.max(1.0),
            refill_per_sec: refill_per_sec.max(0.1),
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Try to consume `cost` tokens for `ip`.
    ///
    /// Returns:
    /// - `None` when the request is allowed.
    /// - `Some(retry_after_secs)` when rate-limited.
    pub async fn consume(&self, ip: IpAddr, cost: f64) -> Option<u64> {
        let mut map = self.buckets.lock().await;
        let now = Instant::now();
        let cost = cost.max(1.0);

        let bucket = map.entry(ip).or_insert_with(|| TokenBucket::new(self.capacity));

        // Refill tokens based on elapsed time.
        let elapsed = now.saturating_duration_since(bucket.last_refill).as_secs_f64();
        if elapsed > 0.0 {
            bucket.tokens = (bucket.tokens + elapsed * self.refill_per_sec).min(self.capacity);
            bucket.last_refill = now;
        }

        if bucket.tokens >= cost {
            bucket.tokens -= cost;
            None
        } else {
            let missing = cost - bucket.tokens;
            let retry_after = (missing / self.refill_per_sec).ceil().max(1.0) as u64;
            Some(retry_after)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_rate_limiter_allows_then_limits() {
        let limiter = IpRateLimiter::new(2.0, 0.5);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert!(limiter.consume(ip, 1.0).await.is_none());
        assert!(limiter.consume(ip, 1.0).await.is_none());
        assert!(limiter.consume(ip, 1.0).await.is_some());
    }
}
