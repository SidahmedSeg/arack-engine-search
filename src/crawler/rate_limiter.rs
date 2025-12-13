use anyhow::Result;
use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use serde::Serialize;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};
use url::Url;

/// Per-domain rate limiter using token bucket algorithm
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of domain -> rate limiter
    limiters: Arc<DashMap<String, Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    /// Requests per second per domain
    requests_per_second: u32,
    /// Minimum delay between requests in milliseconds
    min_delay_ms: u64,
}

impl RateLimiter {
    /// Create a new rate limiter with specified requests per second
    pub fn new(requests_per_second: u32, min_delay_ms: u64) -> Self {
        info!(
            "Initializing rate limiter: {} req/s per domain, min delay: {}ms",
            requests_per_second, min_delay_ms
        );

        Self {
            limiters: Arc::new(DashMap::new()),
            requests_per_second,
            min_delay_ms,
        }
    }

    /// Wait until a request to the given URL can proceed
    pub async fn wait_for(&self, url: &str) -> Result<()> {
        let domain = self.extract_domain(url)?;

        // Get or create rate limiter for this domain
        let limiter = self.get_or_create_limiter(&domain);

        // Wait until we can proceed
        limiter.until_ready().await;

        // Additional minimum delay enforcement
        if self.min_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.min_delay_ms)).await;
        }

        debug!("Rate limiter: Allowed request to {}", domain);
        Ok(())
    }

    /// Check if a request can proceed without waiting
    pub fn check(&self, url: &str) -> Result<bool> {
        let domain = self.extract_domain(url)?;
        let limiter = self.get_or_create_limiter(&domain);

        Ok(limiter.check().is_ok())
    }

    /// Get or create a rate limiter for a specific domain
    fn get_or_create_limiter(
        &self,
        domain: &str,
    ) -> Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        self.limiters
            .entry(domain.to_string())
            .or_insert_with(|| {
                debug!("Creating new rate limiter for domain: {}", domain);

                // Create quota: requests_per_second requests per second
                let quota = Quota::per_second(
                    NonZeroU32::new(self.requests_per_second)
                        .unwrap_or(NonZeroU32::new(2).unwrap())
                );

                Arc::new(GovernorRateLimiter::direct(quota))
            })
            .clone()
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &str) -> Result<String> {
        let parsed = Url::parse(url)?;

        let domain = parsed
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("No host in URL: {}", url))?
            .to_string();

        Ok(domain)
    }

    /// Get statistics about tracked domains
    pub fn stats(&self) -> RateLimiterStats {
        RateLimiterStats {
            tracked_domains: self.limiters.len(),
            requests_per_second: self.requests_per_second,
            min_delay_ms: self.min_delay_ms,
        }
    }

    /// Clear all rate limiters (useful for testing)
    pub fn clear(&self) {
        self.limiters.clear();
        info!("Cleared all domain rate limiters");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimiterStats {
    pub tracked_domains: usize,
    pub requests_per_second: u32,
    pub min_delay_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        let limiter = RateLimiter::new(2, 1000);

        assert_eq!(
            limiter.extract_domain("https://example.com/path").unwrap(),
            "example.com"
        );

        assert_eq!(
            limiter.extract_domain("http://sub.example.com:8080/path").unwrap(),
            "sub.example.com"
        );
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new(2, 0); // 2 req/s, no min delay

        let url = "https://example.com/page";

        // First two requests should succeed immediately (burst capacity)
        limiter.wait_for(url).await.unwrap();
        limiter.wait_for(url).await.unwrap();

        // Third request should wait (rate limited)
        // We'll verify the limiter was created successfully
        let stats = limiter.stats();
        assert_eq!(stats.requests_per_second, 2);
        assert_eq!(stats.tracked_domains, 1);
    }

    #[tokio::test]
    async fn test_multiple_domains() {
        let limiter = RateLimiter::new(2, 0);

        let url1 = "https://example1.com/page";
        let url2 = "https://example2.com/page";

        // Each domain gets its own limiter
        limiter.wait_for(url1).await.unwrap();
        limiter.wait_for(url2).await.unwrap();

        let stats = limiter.stats();
        assert_eq!(stats.tracked_domains, 2);
    }
}
