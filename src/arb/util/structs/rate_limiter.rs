use parking_lot::RwLock;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct RateLimiterConfig {
    pub max_requests: u32,
    pub window_duration: Duration,
    pub burst_capacity: u32,
    pub name: String,
}

pub struct RateLimiter {
    inner: Arc<RwLock<RateLimiterInner>>,
    config: RateLimiterConfig,
}

struct RateLimiterInner {
    available_tokens: f64,
    last_refill: Instant,
    accepted_count: u64,
    rejected_count: u64,
}

impl RateLimiter {
    pub fn new(
        max_requests: u32,
        window_duration: Duration,
        burst_capacity: u32,
        name: String,
    ) -> Self {
        let config = RateLimiterConfig {
            max_requests,
            window_duration,
            burst_capacity,
            name,
        };
        Self::from_config(config)
    }

    pub fn from_config(config: RateLimiterConfig) -> Self {
        let inner = Arc::new(RwLock::new(RateLimiterInner {
            available_tokens: config.burst_capacity as f64,
            last_refill: Instant::now(),
            accepted_count: 0,
            rejected_count: 0,
        }));

        Self { inner, config }
    }

    pub fn try_acquire(&self) -> bool {
        self.try_acquire_n(1)
    }

    pub fn try_acquire_err(&self) -> Result<(), RateLimitError> {
        if self.try_acquire() {
            Ok(())
        } else {
            let inner = self.inner.read();
            Err(RateLimitError::ExceededLimit {
                name: self.config.name.clone(),
                available_tokens: inner.available_tokens,
                requested: 1,
                rejected_count: inner.rejected_count,
            })
        }
    }

    pub fn try_acquire_n(&self, n: u32) -> bool {
        let mut inner = self.inner.write();
        self.refill_tokens(&mut inner);

        if inner.available_tokens >= n as f64 {
            inner.available_tokens -= n as f64;
            inner.accepted_count += n as u64;
            true
        } else {
            inner.rejected_count += n as u64;
            false
        }
    }

    pub fn try_acquire_n_err(&self, n: u32) -> Result<(), RateLimitError> {
        if self.try_acquire_n(n) {
            Ok(())
        } else {
            let inner = self.inner.read();
            Err(RateLimitError::ExceededLimit {
                name: self.config.name.clone(),
                available_tokens: inner.available_tokens,
                requested: n,
                rejected_count: inner.rejected_count,
            })
        }
    }

    pub async fn acquire(&self, timeout: Duration) -> bool {
        self.acquire_n(1, timeout).await
    }

    pub async fn acquire_n(&self, n: u32, timeout: Duration) -> bool {
        let start = Instant::now();

        loop {
            if self.try_acquire_n(n) {
                return true;
            }

            if start.elapsed() >= timeout {
                return false;
            }

            let wait_time = self.calculate_wait_time(n);
            let remaining = timeout.saturating_sub(start.elapsed());
            tokio::time::sleep(wait_time.min(remaining)).await;
        }
    }

    pub fn reset(&self) {
        let mut inner = self.inner.write();
        inner.available_tokens = self.config.burst_capacity as f64;
        inner.last_refill = Instant::now();
        inner.accepted_count = 0;
        inner.rejected_count = 0;
    }

    pub fn metrics(&self) -> RateLimiterMetrics {
        let inner = self.inner.read();
        RateLimiterMetrics {
            available_tokens: inner.available_tokens as u32,
            accepted_count: inner.accepted_count,
            rejected_count: inner.rejected_count,
            name: self.config.name.clone(),
        }
    }

    fn refill_tokens(&self, inner: &mut RateLimiterInner) {
        let now = Instant::now();
        let elapsed = now.duration_since(inner.last_refill);

        let refill_rate =
            self.config.max_requests as f64 / self.config.window_duration.as_secs_f64();
        let tokens_to_add = elapsed.as_secs_f64() * refill_rate;

        inner.available_tokens =
            (inner.available_tokens + tokens_to_add).min(self.config.burst_capacity as f64);
        inner.last_refill = now;
    }

    fn calculate_wait_time(&self, n: u32) -> Duration {
        let inner = self.inner.read();
        let tokens_needed = n as f64 - inner.available_tokens;

        if tokens_needed <= 0.0 {
            return Duration::ZERO;
        }

        let refill_rate =
            self.config.max_requests as f64 / self.config.window_duration.as_secs_f64();
        let seconds_to_wait = tokens_needed / refill_rate;

        Duration::from_secs_f64(seconds_to_wait + 0.001)
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitError {
    ExceededLimit {
        name: String,
        available_tokens: f64,
        requested: u32,
        rejected_count: u64,
    },
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimitError::ExceededLimit {
                name,
                available_tokens,
                requested,
                rejected_count,
            } => write!(
                f,
                "Rate limit exceeded for {}: available_tokens={:.2}, requested={}, total_rejected={}",
                name, available_tokens, requested, rejected_count
            ),
        }
    }
}

impl Error for RateLimitError {}

#[derive(Debug, Clone)]
pub struct RateLimiterMetrics {
    pub available_tokens: u32,
    pub accepted_count: u64,
    pub rejected_count: u64,
    pub name: String,
}

unsafe impl Send for RateLimiter {}
unsafe impl Sync for RateLimiter {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_rate_limiting() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1), 5, "test".to_string());

        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }
        assert!(!limiter.try_acquire());

        sleep(Duration::from_secs(1)).await;
        assert!(limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_burst_capacity() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1), 10, "test".to_string());

        for _ in 0..10 {
            assert!(limiter.try_acquire());
        }
        assert!(!limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_acquire_with_timeout() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1), 2, "test".to_string());

        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());

        let start = Instant::now();
        let acquired = limiter.acquire(Duration::from_millis(600)).await;
        let elapsed = start.elapsed();

        assert!(acquired);
        assert!(elapsed >= Duration::from_millis(500));
        assert!(elapsed < Duration::from_millis(700));
    }

    #[tokio::test]
    async fn test_acquire_timeout_exceeded() {
        let limiter = RateLimiter::new(1, Duration::from_secs(2), 1, "test".to_string());

        assert!(limiter.try_acquire());

        let acquired = limiter.acquire(Duration::from_millis(100)).await;
        assert!(!acquired);
    }

    #[tokio::test]
    async fn test_batch_acquisition() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1), 15, "test".to_string());

        assert!(limiter.try_acquire_n(5));
        assert!(limiter.try_acquire_n(5));
        assert!(limiter.try_acquire_n(5));
        assert!(!limiter.try_acquire_n(5));
        assert!(!limiter.try_acquire_n(1));
    }

    #[tokio::test]
    async fn test_metrics() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1), 10, "metrics_test".to_string());

        for _ in 0..10 {
            assert!(limiter.try_acquire());
        }
        for _ in 0..5 {
            assert!(!limiter.try_acquire());
        }

        let metrics = limiter.metrics();
        assert_eq!(metrics.accepted_count, 10);
        assert_eq!(metrics.rejected_count, 5);
        assert_eq!(metrics.available_tokens, 0);
        assert_eq!(metrics.name, "metrics_test");
    }

    #[tokio::test]
    async fn test_reset() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1), 5, "test".to_string());

        for _ in 0..5 {
            limiter.try_acquire();
        }
        assert!(!limiter.try_acquire());

        limiter.reset();
        assert!(limiter.try_acquire());

        let metrics = limiter.metrics();
        assert_eq!(metrics.accepted_count, 1);
        assert_eq!(metrics.rejected_count, 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let limiter = Arc::new(RateLimiter::new(
            100,
            Duration::from_secs(1),
            100,
            "concurrent".to_string(),
        ));
        let success_count = Arc::new(AtomicU32::new(0));

        let mut handles = vec![];
        for _ in 0..200 {
            let limiter_clone = limiter.clone();
            let count_clone = success_count.clone();
            handles.push(tokio::spawn(async move {
                if limiter_clone.try_acquire() {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(success_count.load(Ordering::SeqCst), 100);
    }

    #[tokio::test]
    async fn test_from_config() {
        let limiter = RateLimiter::from_config(RateLimiterConfig {
            max_requests: 20,
            window_duration: Duration::from_secs(2),
            burst_capacity: 25,
            name: "config_test".to_string(),
        });

        assert!(limiter.try_acquire());
        let metrics = limiter.metrics();
        assert_eq!(metrics.name, "config_test");
    }

    #[tokio::test]
    async fn test_refill_calculation() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1), 10, "refill_test".to_string());

        for _ in 0..10 {
            limiter.try_acquire();
        }
        assert!(!limiter.try_acquire());

        sleep(Duration::from_millis(500)).await;

        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }
        assert!(!limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_try_acquire_err() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1), 2, "error_test".to_string());

        assert!(limiter.try_acquire_err().is_ok());
        assert!(limiter.try_acquire_err().is_ok());
        
        let result = limiter.try_acquire_err();
        assert!(result.is_err());
        
        if let Err(RateLimitError::ExceededLimit { name, available_tokens, requested, rejected_count }) = result {
            assert_eq!(name, "error_test");
            assert!(available_tokens < 1.0);
            assert_eq!(requested, 1);
            assert_eq!(rejected_count, 1);
        } else {
            panic!("Expected RateLimitError::ExceededLimit");
        }
    }

    #[tokio::test]
    async fn test_try_acquire_n_err() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1), 10, "batch_error_test".to_string());

        assert!(limiter.try_acquire_n_err(5).is_ok());
        assert!(limiter.try_acquire_n_err(5).is_ok());
        
        let result = limiter.try_acquire_n_err(3);
        assert!(result.is_err());
        
        if let Err(RateLimitError::ExceededLimit { name, available_tokens, requested, rejected_count }) = result {
            assert_eq!(name, "batch_error_test");
            assert!(available_tokens < 3.0);
            assert_eq!(requested, 3);
            assert_eq!(rejected_count, 3);
        } else {
            panic!("Expected RateLimitError::ExceededLimit");
        }
    }
}
