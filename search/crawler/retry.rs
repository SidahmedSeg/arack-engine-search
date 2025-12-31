use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay in seconds (will be multiplied exponentially)
    pub base_delay_secs: f64,
    /// Maximum delay in seconds
    pub max_delay_secs: f64,
    /// HTTP status codes that should trigger a retry
    pub retryable_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_secs: 1.0,
            max_delay_secs: 60.0,
            retryable_status_codes: vec![
                408, // Request Timeout
                429, // Too Many Requests
                500, // Internal Server Error
                502, // Bad Gateway
                503, // Service Unavailable
                504, // Gateway Timeout
            ],
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_retries: u32, base_delay_secs: f64, max_delay_secs: f64) -> Self {
        Self {
            max_retries,
            base_delay_secs,
            max_delay_secs,
            ..Default::default()
        }
    }

    /// Check if a status code is retryable
    pub fn is_retryable(&self, status_code: u16) -> bool {
        self.retryable_status_codes.contains(&status_code)
    }

    /// Calculate exponential backoff delay for a given attempt number
    /// Formula: min(base * 2^attempt, max_delay)
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_secs = self.base_delay_secs * 2_f64.powi(attempt as i32);
        let capped_delay = delay_secs.min(self.max_delay_secs);
        Duration::from_secs_f64(capped_delay)
    }
}

/// Retry policy for HTTP requests
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    config: RetryConfig,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            config: RetryConfig::default(),
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a retry policy with custom configuration
    pub fn with_config(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("Operation succeeded after {} attempts", attempt + 1);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if attempt >= self.config.max_retries {
                        warn!(
                            "Operation failed after {} attempts: {}",
                            attempt + 1,
                            e
                        );
                        return Err(anyhow::anyhow!("Max retries exceeded: {}", e));
                    }

                    let delay = self.config.calculate_delay(attempt);
                    warn!(
                        "Operation failed (attempt {}), retrying in {:?}: {}",
                        attempt + 1,
                        delay,
                        e
                    );

                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Execute HTTP request with retry logic
    pub async fn execute_http<F, Fut>(&self, url: &str, mut request_fn: F) -> Result<reqwest::Response>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let mut attempt = 0;

        loop {
            match request_fn().await {
                Ok(response) => {
                    let status = response.status();

                    // If status is retryable and we haven't exceeded max retries
                    if self.config.is_retryable(status.as_u16()) && attempt < self.config.max_retries {
                        let delay = self.config.calculate_delay(attempt);
                        warn!(
                            "HTTP request to {} returned retryable status {} (attempt {}), retrying in {:?}",
                            url,
                            status,
                            attempt + 1,
                            delay
                        );
                        sleep(delay).await;
                        attempt += 1;
                        continue;
                    }

                    if attempt > 0 {
                        debug!("HTTP request to {} succeeded after {} attempts", url, attempt + 1);
                    }

                    return Ok(response);
                }
                Err(e) => {
                    if attempt >= self.config.max_retries {
                        warn!(
                            "HTTP request to {} failed after {} attempts: {}",
                            url,
                            attempt + 1,
                            e
                        );
                        return Err(anyhow::anyhow!("Max retries exceeded: {}", e));
                    }

                    let delay = self.config.calculate_delay(attempt);
                    warn!(
                        "HTTP request to {} failed (attempt {}), retrying in {:?}: {}",
                        url,
                        attempt + 1,
                        delay,
                        e
                    );

                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }

    /// Get the retry configuration
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
}

/// Retry statistics
#[derive(Debug, Clone, Default)]
pub struct RetryStats {
    /// Total number of operations attempted
    pub total_operations: u64,
    /// Number of operations that succeeded on first try
    pub first_try_success: u64,
    /// Number of operations that required retries
    pub retried_operations: u64,
    /// Number of operations that failed after all retries
    pub failed_operations: u64,
    /// Total number of retry attempts across all operations
    pub total_retry_attempts: u64,
}

impl RetryStats {
    /// Create new retry statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful operation
    pub fn record_success(&mut self, attempts: u32) {
        self.total_operations += 1;
        if attempts == 0 {
            self.first_try_success += 1;
        } else {
            self.retried_operations += 1;
            self.total_retry_attempts += attempts as u64;
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self, attempts: u32) {
        self.total_operations += 1;
        self.failed_operations += 1;
        self.total_retry_attempts += attempts as u64;
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        let successful = self.first_try_success + self.retried_operations;
        successful as f64 / self.total_operations as f64
    }

    /// Calculate average retry attempts per operation
    pub fn avg_retry_attempts(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        self.total_retry_attempts as f64 / self.total_operations as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_secs, 1.0);
        assert_eq!(config.max_delay_secs, 60.0);
        assert!(config.is_retryable(500));
        assert!(config.is_retryable(503));
        assert!(!config.is_retryable(200));
        assert!(!config.is_retryable(404));
    }

    #[test]
    fn test_exponential_backoff() {
        let config = RetryConfig::default();

        // Attempt 0: 1 * 2^0 = 1 second
        assert_eq!(config.calculate_delay(0), Duration::from_secs(1));

        // Attempt 1: 1 * 2^1 = 2 seconds
        assert_eq!(config.calculate_delay(1), Duration::from_secs(2));

        // Attempt 2: 1 * 2^2 = 4 seconds
        assert_eq!(config.calculate_delay(2), Duration::from_secs(4));

        // Attempt 3: 1 * 2^3 = 8 seconds
        assert_eq!(config.calculate_delay(3), Duration::from_secs(8));

        // Attempt 10: should be capped at max_delay (60 seconds)
        assert_eq!(config.calculate_delay(10), Duration::from_secs(60));
    }

    #[test]
    fn test_custom_retry_config() {
        let config = RetryConfig::new(5, 2.0, 30.0);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay_secs, 2.0);
        assert_eq!(config.max_delay_secs, 30.0);

        // Attempt 0: 2 * 2^0 = 2 seconds
        assert_eq!(config.calculate_delay(0), Duration::from_secs(2));

        // Attempt 1: 2 * 2^1 = 4 seconds
        assert_eq!(config.calculate_delay(1), Duration::from_secs(4));

        // Attempt 5: should be capped at 30 seconds
        assert_eq!(config.calculate_delay(5), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_retry_policy_success_first_try() {
        use std::sync::{Arc, Mutex};

        let policy = RetryPolicy::new();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = policy
            .execute(|| {
                let count = call_count_clone.clone();
                async move {
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    Ok::<i32, String>(42)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_retry_policy_success_after_retries() {
        use std::sync::{Arc, Mutex};

        let policy = RetryPolicy::new();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = policy
            .execute(|| {
                let count = call_count_clone.clone();
                async move {
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    if *count < 3 {
                        Err("Temporary error".to_string())
                    } else {
                        Ok::<i32, String>(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_retry_policy_max_retries_exceeded() {
        use std::sync::{Arc, Mutex};

        let policy = RetryPolicy::new();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = policy
            .execute(|| {
                let count = call_count_clone.clone();
                async move {
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    Err::<i32, String>("Persistent error".to_string())
                }
            })
            .await;

        assert!(result.is_err());
        // Should try once + 3 retries = 4 total
        assert_eq!(*call_count.lock().unwrap(), 4);
    }

    #[test]
    fn test_retry_stats() {
        let mut stats = RetryStats::new();

        // Record some successes
        stats.record_success(0); // First try success
        stats.record_success(0); // First try success
        stats.record_success(2); // Success after 2 retries

        // Record a failure
        stats.record_failure(3); // Failed after 3 retries

        assert_eq!(stats.total_operations, 4);
        assert_eq!(stats.first_try_success, 2);
        assert_eq!(stats.retried_operations, 1);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.total_retry_attempts, 5); // 2 + 3

        // Success rate: 3/4 = 0.75
        assert_eq!(stats.success_rate(), 0.75);

        // Average retry attempts: 5/4 = 1.25
        assert_eq!(stats.avg_retry_attempts(), 1.25);
    }

    #[test]
    fn test_is_retryable() {
        let config = RetryConfig::default();

        // Retryable status codes
        assert!(config.is_retryable(408));
        assert!(config.is_retryable(429));
        assert!(config.is_retryable(500));
        assert!(config.is_retryable(502));
        assert!(config.is_retryable(503));
        assert!(config.is_retryable(504));

        // Non-retryable status codes
        assert!(!config.is_retryable(200));
        assert!(!config.is_retryable(404));
        assert!(!config.is_retryable(400));
        assert!(!config.is_retryable(403));
        assert!(!config.is_retryable(301));
    }
}
