use anyhow::Result;
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tracing::{debug, info, warn};
use url::Url;

/// Politeness manager for respectful crawling
#[derive(Clone)]
pub struct PolitenessManager {
    /// Map of domain -> last request time
    last_request: Arc<DashMap<String, Instant>>,
    /// Map of domain -> custom crawl delay (from robots.txt)
    crawl_delays: Arc<DashMap<String, Duration>>,
    /// Default crawl delay
    default_delay: Duration,
    /// Maximum retry attempts
    max_retries: u32,
    /// Base backoff duration
    base_backoff: Duration,
    /// Maximum backoff duration
    max_backoff: Duration,
}

impl PolitenessManager {
    /// Create a new politeness manager
    pub fn new(default_delay_ms: u64, max_retries: u32) -> Self {
        info!(
            "Initializing politeness manager: default delay {}ms, max retries {}",
            default_delay_ms, max_retries
        );

        Self {
            last_request: Arc::new(DashMap::new()),
            crawl_delays: Arc::new(DashMap::new()),
            default_delay: Duration::from_millis(default_delay_ms),
            max_retries,
            base_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }

    /// Wait appropriate time before making request to URL
    pub async fn wait_before_request(&self, url: &str) -> Result<()> {
        let domain = self.extract_domain(url)?;

        // Get crawl delay for this domain
        let delay = self
            .crawl_delays
            .get(&domain)
            .map(|d| *d)
            .unwrap_or(self.default_delay);

        // Check if we need to wait based on last request
        if let Some(last_time) = self.last_request.get(&domain) {
            let elapsed = last_time.elapsed();

            if elapsed < delay {
                let wait_time = delay - elapsed;
                debug!(
                    "Politeness delay for {}: waiting {:?}",
                    domain, wait_time
                );
                sleep(wait_time).await;
            }
        }

        // Update last request time
        self.last_request.insert(domain.clone(), Instant::now());

        Ok(())
    }

    /// Set custom crawl delay for a domain (typically from robots.txt)
    pub fn set_crawl_delay(&self, domain: &str, delay_secs: f64) {
        let delay = Duration::from_secs_f64(delay_secs);
        info!("Setting crawl delay for {}: {:?}", domain, delay);
        self.crawl_delays.insert(domain.to_string(), delay);
    }

    /// Calculate backoff duration for retry attempt
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        // Exponential backoff: base * 2^attempt
        let backoff_secs = self.base_backoff.as_secs() * 2_u64.pow(attempt);
        let backoff = Duration::from_secs(backoff_secs.min(self.max_backoff.as_secs()));

        debug!("Calculated backoff for attempt {}: {:?}", attempt, backoff);
        backoff
    }

    /// Execute a request with retry logic and exponential backoff
    pub async fn execute_with_retry<F, T, E>(&self, url: &str, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display,
    {
        let domain = self.extract_domain(url)?;
        let mut attempt = 0;

        loop {
            // Wait before request
            self.wait_before_request(url).await?;

            // Try the operation
            match operation() {
                Ok(result) => {
                    if attempt > 0 {
                        info!("Request to {} succeeded after {} retries", domain, attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    attempt += 1;

                    if attempt >= self.max_retries {
                        warn!(
                            "Request to {} failed after {} attempts: {}",
                            domain, attempt, e
                        );
                        return Err(anyhow::anyhow!(
                            "Max retries ({}) exceeded for {}: {}",
                            self.max_retries,
                            domain,
                            e
                        ));
                    }

                    let backoff = self.calculate_backoff(attempt);
                    warn!(
                        "Request to {} failed (attempt {}): {}. Retrying in {:?}",
                        domain, attempt, e, backoff
                    );
                    sleep(backoff).await;
                }
            }
        }
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

    /// Get statistics
    pub fn stats(&self) -> PolitenessStats {
        PolitenessStats {
            tracked_domains: self.last_request.len(),
            domains_with_custom_delays: self.crawl_delays.len(),
            default_delay_ms: self.default_delay.as_millis() as u64,
            max_retries: self.max_retries,
        }
    }

    /// Clear all tracking data (useful for testing)
    pub fn clear(&self) {
        self.last_request.clear();
        self.crawl_delays.clear();
        info!("Cleared politeness tracking data");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PolitenessStats {
    pub tracked_domains: usize,
    pub domains_with_custom_delays: usize,
    pub default_delay_ms: u64,
    pub max_retries: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        let manager = PolitenessManager::new(1000, 3);

        assert_eq!(
            manager.extract_domain("https://example.com/path").unwrap(),
            "example.com"
        );
    }

    #[test]
    fn test_backoff_calculation() {
        let manager = PolitenessManager::new(1000, 3);

        assert_eq!(manager.calculate_backoff(0), Duration::from_secs(1));
        assert_eq!(manager.calculate_backoff(1), Duration::from_secs(2));
        assert_eq!(manager.calculate_backoff(2), Duration::from_secs(4));
        assert_eq!(manager.calculate_backoff(3), Duration::from_secs(8));

        // Should cap at max_backoff (60 seconds)
        assert_eq!(manager.calculate_backoff(10), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_wait_before_request() {
        let manager = PolitenessManager::new(100, 3); // 100ms delay

        let url = "https://example.com/page";

        let start = Instant::now();
        manager.wait_before_request(url).await.unwrap();
        manager.wait_before_request(url).await.unwrap();
        let elapsed = start.elapsed();

        // Second request should have waited ~100ms
        assert!(elapsed >= Duration::from_millis(90)); // Allow some tolerance
    }

    #[tokio::test]
    async fn test_custom_crawl_delay() {
        let manager = PolitenessManager::new(100, 3);

        // Set custom delay for domain
        manager.set_crawl_delay("example.com", 0.5); // 500ms

        let url = "https://example.com/page";

        let start = Instant::now();
        manager.wait_before_request(url).await.unwrap();
        manager.wait_before_request(url).await.unwrap();
        let elapsed = start.elapsed();

        // Should use custom 500ms delay, not default 100ms
        assert!(elapsed >= Duration::from_millis(450)); // Allow some tolerance
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let manager = PolitenessManager::new(10, 3); // 10ms delay, 3 retries

        let url = "https://example.com/page";
        let mut attempt_count = 0;

        // Simulate operation that fails twice then succeeds
        let result = manager
            .execute_with_retry(url, || {
                attempt_count += 1;
                if attempt_count < 3 {
                    Err("Simulated error")
                } else {
                    Ok("Success")
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count, 3);
    }
}
