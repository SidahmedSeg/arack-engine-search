use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed - requests pass through normally
    Closed,
    /// Circuit is open - requests are rejected immediately
    Open,
    /// Circuit is half-open - limited requests are allowed to test recovery
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Duration to keep circuit open before transitioning to half-open
    pub open_timeout: Duration,
    /// Number of consecutive successes in half-open state to close circuit
    pub success_threshold: u32,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_timeout: Duration::from_secs(60),
            success_threshold: 2,
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker for a single domain
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    /// Current state of the circuit
    state: CircuitState,
    /// Number of consecutive failures
    failure_count: u32,
    /// Number of consecutive successes in half-open state
    success_count: u32,
    /// Time when the circuit was opened
    opened_at: Option<Instant>,
    /// Number of requests in half-open state
    half_open_requests: u32,
    /// Total failures tracked
    total_failures: u64,
    /// Total successes tracked
    total_successes: u64,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            opened_at: None,
            half_open_requests: 0,
            total_failures: 0,
            total_successes: 0,
        }
    }
}

impl CircuitBreakerState {
    /// Check if circuit should transition to half-open
    fn should_attempt_reset(&self, config: &CircuitBreakerConfig) -> bool {
        if self.state != CircuitState::Open {
            return false;
        }

        if let Some(opened_at) = self.opened_at {
            Instant::now().duration_since(opened_at) >= config.open_timeout
        } else {
            false
        }
    }

    /// Transition to half-open state
    fn transition_to_half_open(&mut self) {
        self.state = CircuitState::HalfOpen;
        self.half_open_requests = 0;
        self.success_count = 0;
        debug!("Circuit transitioned to HalfOpen state");
    }

    /// Transition to open state
    fn transition_to_open(&mut self) {
        self.state = CircuitState::Open;
        self.opened_at = Some(Instant::now());
        self.failure_count = 0;
        self.success_count = 0;
        warn!("Circuit opened due to repeated failures");
    }

    /// Transition to closed state
    fn transition_to_closed(&mut self) {
        self.state = CircuitState::Closed;
        self.opened_at = None;
        self.failure_count = 0;
        self.success_count = 0;
        self.half_open_requests = 0;
        info!("Circuit closed - normal operation resumed");
    }

    /// Record a successful request
    fn record_success(&mut self, config: &CircuitBreakerConfig) {
        self.total_successes += 1;

        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= config.success_threshold {
                    self.transition_to_closed();
                }
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                debug!("Received success in Open state, this shouldn't happen");
            }
        }
    }

    /// Record a failed request
    fn record_failure(&mut self, config: &CircuitBreakerConfig) {
        self.total_failures += 1;

        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= config.failure_threshold {
                    self.transition_to_open();
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open immediately reopens circuit
                self.transition_to_open();
            }
            CircuitState::Open => {
                // Already open, just track the failure
            }
        }
    }

    /// Check if a request can proceed
    fn can_proceed(&mut self, config: &CircuitBreakerConfig) -> bool {
        // Check if we should attempt reset
        if self.should_attempt_reset(config) {
            self.transition_to_half_open();
        }

        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => {
                if self.half_open_requests < config.half_open_max_requests {
                    self.half_open_requests += 1;
                    true
                } else {
                    false
                }
            }
        }
    }
}

/// Circuit breaker manager for multiple domains
#[derive(Clone)]
pub struct CircuitBreakerManager {
    /// Circuit breaker state per domain
    circuits: Arc<DashMap<String, CircuitBreakerState>>,
    /// Configuration
    config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// Create a new circuit breaker manager
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            circuits: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Check if a request to a domain can proceed
    pub fn can_proceed(&self, domain: &str) -> bool {
        let mut entry = self.circuits.entry(domain.to_string()).or_default();
        let can_proceed = entry.can_proceed(&self.config);

        if !can_proceed {
            debug!("Circuit breaker blocked request to {}", domain);
        }

        can_proceed
    }

    /// Record a successful request for a domain
    pub fn record_success(&self, domain: &str) {
        let mut entry = self.circuits.entry(domain.to_string()).or_default();
        entry.record_success(&self.config);
    }

    /// Record a failed request for a domain
    pub fn record_failure(&self, domain: &str) {
        let mut entry = self.circuits.entry(domain.to_string()).or_default();
        entry.record_failure(&self.config);
    }

    /// Get the current state of a domain's circuit
    pub fn get_state(&self, domain: &str) -> CircuitState {
        self.circuits
            .get(domain)
            .map(|entry| entry.state)
            .unwrap_or(CircuitState::Closed)
    }

    /// Reset a circuit breaker for a domain
    pub fn reset(&self, domain: &str) {
        if let Some(mut entry) = self.circuits.get_mut(domain) {
            entry.transition_to_closed();
            info!("Manually reset circuit breaker for {}", domain);
        }
    }

    /// Get statistics for a domain
    pub fn get_domain_stats(&self, domain: &str) -> Option<DomainCircuitStats> {
        self.circuits.get(domain).map(|entry| DomainCircuitStats {
            state: entry.state,
            failure_count: entry.failure_count,
            success_count: entry.success_count,
            total_failures: entry.total_failures,
            total_successes: entry.total_successes,
        })
    }

    /// Get overall statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        let mut open_count = 0;
        let mut half_open_count = 0;
        let mut closed_count = 0;

        for entry in self.circuits.iter() {
            match entry.state {
                CircuitState::Open => open_count += 1,
                CircuitState::HalfOpen => half_open_count += 1,
                CircuitState::Closed => closed_count += 1,
            }
        }

        CircuitBreakerStats {
            total_circuits: self.circuits.len(),
            open_circuits: open_count,
            half_open_circuits: half_open_count,
            closed_circuits: closed_count,
        }
    }

    /// Clear all circuit breakers
    pub fn clear_all(&self) {
        self.circuits.clear();
        info!("Cleared all circuit breakers");
    }

    /// Get all domains and their circuit states (Phase 6.10)
    pub fn get_all_domains(&self) -> Vec<(String, DomainCircuitStats)> {
        self.circuits
            .iter()
            .map(|entry| {
                let domain = entry.key().clone();
                let state = entry.value();
                let stats = DomainCircuitStats {
                    state: state.state,
                    failure_count: state.failure_count,
                    success_count: state.success_count,
                    total_failures: state.total_failures,
                    total_successes: state.total_successes,
                };
                (domain, stats)
            })
            .collect()
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Serialize)]
pub struct CircuitBreakerStats {
    pub total_circuits: usize,
    pub open_circuits: usize,
    pub half_open_circuits: usize,
    pub closed_circuits: usize,
}

/// Statistics for a specific domain's circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCircuitStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub total_failures: u64,
    pub total_successes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let manager = CircuitBreakerManager::default();
        assert_eq!(manager.get_state("example.com"), CircuitState::Closed);
        assert!(manager.can_proceed("example.com"));
    }

    #[test]
    fn test_circuit_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        let domain = "example.com";

        // First 2 failures - should stay closed
        manager.record_failure(domain);
        manager.record_failure(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Closed);
        assert!(manager.can_proceed(domain));

        // Third failure - should open
        manager.record_failure(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Open);
        assert!(!manager.can_proceed(domain));
    }

    #[test]
    fn test_circuit_resets_on_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        let domain = "example.com";

        // Record 2 failures
        manager.record_failure(domain);
        manager.record_failure(domain);

        // Success should reset failure count
        manager.record_success(domain);

        // Now 2 more failures shouldn't open circuit
        manager.record_failure(domain);
        manager.record_failure(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            open_timeout: Duration::from_millis(100),
            success_threshold: 2,
            half_open_max_requests: 3,
        };
        let manager = CircuitBreakerManager::new(config);

        let domain = "example.com";

        // Open the circuit
        manager.record_failure(domain);
        manager.record_failure(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Open);

        // Should not allow requests
        assert!(!manager.can_proceed(domain));

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should transition to half-open on next check
        assert!(manager.can_proceed(domain));
        assert_eq!(manager.get_state(domain), CircuitState::HalfOpen);

        // Record successes to close circuit
        manager.record_success(domain);
        manager.record_success(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_reopens_on_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            open_timeout: Duration::from_millis(100),
            success_threshold: 2,
            half_open_max_requests: 3,
        };
        let manager = CircuitBreakerManager::new(config);

        let domain = "example.com";

        // Open the circuit
        manager.record_failure(domain);
        manager.record_failure(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Open);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Transition to half-open
        assert!(manager.can_proceed(domain));
        assert_eq!(manager.get_state(domain), CircuitState::HalfOpen);

        // Any failure in half-open should reopen circuit
        manager.record_failure(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Open);
        assert!(!manager.can_proceed(domain));
    }

    #[test]
    fn test_half_open_request_limit() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            open_timeout: Duration::from_millis(100),
            success_threshold: 2,
            half_open_max_requests: 2,
        };
        let manager = CircuitBreakerManager::new(config);

        let domain = "example.com";

        // Open the circuit
        manager.record_failure(domain);
        manager.record_failure(domain);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should allow up to half_open_max_requests
        assert!(manager.can_proceed(domain)); // Request 1
        assert!(manager.can_proceed(domain)); // Request 2
        assert!(!manager.can_proceed(domain)); // Request 3 - blocked
    }

    #[test]
    fn test_manual_reset() {
        let manager = CircuitBreakerManager::default();
        let domain = "example.com";

        // Open the circuit
        for _ in 0..5 {
            manager.record_failure(domain);
        }
        assert_eq!(manager.get_state(domain), CircuitState::Open);

        // Manually reset
        manager.reset(domain);
        assert_eq!(manager.get_state(domain), CircuitState::Closed);
        assert!(manager.can_proceed(domain));
    }

    #[test]
    fn test_stats() {
        let manager = CircuitBreakerManager::default();

        // Open circuit for domain1
        for _ in 0..5 {
            manager.record_failure("domain1.com");
        }

        // Keep domain2 closed
        manager.record_success("domain2.com");

        let stats = manager.stats();
        assert_eq!(stats.total_circuits, 2);
        assert_eq!(stats.open_circuits, 1);
        assert_eq!(stats.closed_circuits, 1);
    }

    #[test]
    fn test_domain_stats() {
        let manager = CircuitBreakerManager::default();
        let domain = "example.com";

        manager.record_failure(domain);
        manager.record_failure(domain);
        manager.record_success(domain);

        let stats = manager.get_domain_stats(domain).unwrap();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.total_failures, 2);
        assert_eq!(stats.total_successes, 1);
    }

    #[test]
    fn test_multiple_domains_independent() {
        let manager = CircuitBreakerManager::default();

        // Open circuit for domain1
        for _ in 0..5 {
            manager.record_failure("domain1.com");
        }

        // domain2 should still work
        assert_eq!(manager.get_state("domain1.com"), CircuitState::Open);
        assert_eq!(manager.get_state("domain2.com"), CircuitState::Closed);
        assert!(!manager.can_proceed("domain1.com"));
        assert!(manager.can_proceed("domain2.com"));
    }
}
