use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};
use crate::config::RateLimiterConfig;

pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    window_size: Duration,
    retry_after: Duration,
}

impl RateLimiter {
    pub fn new(config: &RateLimiterConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_requests as usize)),
            window_size: config.window_size(),
            retry_after: config.retry_after(),
        }
    }

    /// Acquires a permit to make a request. If no permits are available,
    /// waits for the retry_after duration and tries again.
    pub async fn acquire(&self) {
        loop {
            if let Some(permit) = self.try_acquire() {
                // Schedule permit release after window_size
                let semaphore = self.semaphore.clone();
                let window_size = self.window_size;
                tokio::spawn(async move {
                    sleep(window_size).await;
                    drop(permit);
                });
                return;
            }
            
            // No permits available, wait and retry
            sleep(self.retry_after).await;
        }
    }

    pub fn try_acquire(&self) -> Option<tokio::sync::SemaphorePermit<'_>> {
        self.semaphore.try_acquire().ok()
    }
}

/// A guard that ensures rate limiting for a block of code
pub struct RateLimitGuard<'a> {
    limiter: &'a RateLimiter,
    start: Instant,
}

impl<'a> RateLimitGuard<'a> {
    pub async fn new(limiter: &'a RateLimiter) -> Self {
        limiter.acquire().await;
        Self {
            limiter,
            start: Instant::now(),
        }
    }

    /// Get the duration since this guard was created
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RateLimiterConfig;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimiterConfig {
            max_requests: 2,
            window_size_seconds: 1,
            retry_after_seconds: 1,
        };

        let limiter = RateLimiter::new(&config);
        
        // First two requests should be immediate
        let start = Instant::now();
        let _guard1 = RateLimitGuard::new(&limiter).await;
        let _guard2 = RateLimitGuard::new(&limiter).await;
        assert!(start.elapsed() < Duration::from_millis(100));

        // Third request should be delayed
        let _guard3 = RateLimitGuard::new(&limiter).await;
        assert!(start.elapsed() >= Duration::from_secs(1));
    }
} 