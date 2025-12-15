use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Crawl frequency for a page
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CrawlFrequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Never,
}

impl CrawlFrequency {
    /// Convert frequency to a Duration
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            CrawlFrequency::Hourly => Some(Duration::hours(1)),
            CrawlFrequency::Daily => Some(Duration::days(1)),
            CrawlFrequency::Weekly => Some(Duration::weeks(1)),
            CrawlFrequency::Monthly => Some(Duration::days(30)),
            CrawlFrequency::Never => None,
        }
    }

    /// Suggest frequency based on change frequency
    pub fn from_change_frequency(changes_per_day: f64) -> Self {
        if changes_per_day >= 24.0 {
            CrawlFrequency::Hourly
        } else if changes_per_day >= 1.0 {
            CrawlFrequency::Daily
        } else if changes_per_day >= 0.14 {
            // ~1 per week
            CrawlFrequency::Weekly
        } else if changes_per_day > 0.0 {
            CrawlFrequency::Monthly
        } else {
            CrawlFrequency::Never
        }
    }
}

/// Scheduled crawl task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledCrawl {
    /// URL to crawl
    pub url: String,
    /// Priority score (0-100, higher = more important)
    pub priority: u8,
    /// When this page should be crawled next
    pub next_crawl_at: DateTime<Utc>,
    /// Last time this page was crawled
    pub last_crawled_at: Option<DateTime<Utc>>,
    /// How often to crawl this page
    pub frequency: CrawlFrequency,
    /// Freshness score (0.0-1.0, higher = fresher)
    pub freshness_score: f64,
}

impl ScheduledCrawl {
    /// Create a new scheduled crawl
    pub fn new(url: String, frequency: CrawlFrequency, priority: u8) -> Self {
        let next_crawl_at = Utc::now();

        Self {
            url,
            priority,
            next_crawl_at,
            last_crawled_at: None,
            frequency,
            freshness_score: 0.0,
        }
    }

    /// Check if this crawl is due
    pub fn is_due(&self) -> bool {
        Utc::now() >= self.next_crawl_at
    }

    /// Update after successful crawl
    pub fn mark_crawled(&mut self) {
        self.last_crawled_at = Some(Utc::now());

        // Calculate next crawl time
        if let Some(duration) = self.frequency.to_duration() {
            self.next_crawl_at = Utc::now() + duration;
        } else {
            // Never recrawl
            self.next_crawl_at = DateTime::<Utc>::MAX_UTC;
        }

        // Reset freshness score after crawl
        self.freshness_score = 1.0;
    }

    /// Calculate priority score for scheduling
    /// Combines priority, freshness, and due time
    pub fn scheduling_score(&self) -> i64 {
        let mut score = self.priority as i64 * 100;

        // Add freshness penalty (older = higher priority)
        let freshness_penalty = ((1.0 - self.freshness_score) * 50.0) as i64;
        score += freshness_penalty;

        // Add overdue penalty
        let now = Utc::now();
        if now > self.next_crawl_at {
            let overdue_hours = (now - self.next_crawl_at).num_hours();
            score += overdue_hours.min(1000); // Cap at 1000
        }

        score
    }

    /// Update freshness score based on time since last crawl
    pub fn update_freshness(&mut self) {
        if let Some(last_crawled) = self.last_crawled_at {
            if let Some(frequency_duration) = self.frequency.to_duration() {
                let elapsed = Utc::now() - last_crawled;
                let decay_factor = elapsed.num_seconds() as f64 / frequency_duration.num_seconds() as f64;

                // Exponential decay: freshness = e^(-decay_factor)
                self.freshness_score = (-decay_factor).exp().max(0.0).min(1.0);
            }
        }
    }
}

// Implement ordering for priority queue (max-heap based on scheduling score)
impl Ord for ScheduledCrawl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scheduling_score().cmp(&other.scheduling_score())
    }
}

impl PartialOrd for ScheduledCrawl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ScheduledCrawl {}

impl PartialEq for ScheduledCrawl {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

/// Crawl scheduler for managing recrawls and priorities
#[derive(Clone)]
pub struct CrawlScheduler {
    /// Priority queue of scheduled crawls
    queue: Arc<Mutex<BinaryHeap<ScheduledCrawl>>>,
}

impl CrawlScheduler {
    /// Create a new crawl scheduler
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    /// Add a URL to the schedule
    pub fn schedule(&self, url: String, frequency: CrawlFrequency, priority: u8) -> Result<()> {
        let crawl = ScheduledCrawl::new(url.clone(), frequency, priority);

        let mut queue = self.queue.lock().unwrap();
        queue.push(crawl);

        debug!("Scheduled {} with frequency {:?} and priority {}", url, frequency, priority);
        Ok(())
    }

    /// Schedule a crawl task
    pub fn schedule_task(&self, task: ScheduledCrawl) -> Result<()> {
        let url = task.url.clone();
        let priority = task.priority;

        let mut queue = self.queue.lock().unwrap();
        queue.push(task);

        debug!("Scheduled task for {} (priority: {})", url, priority);
        Ok(())
    }

    /// Get the next URL to crawl (if any are due)
    pub fn pop_due(&self) -> Option<ScheduledCrawl> {
        let mut queue = self.queue.lock().unwrap();

        // Update freshness scores for all tasks
        let mut tasks: Vec<ScheduledCrawl> = queue.drain().collect();
        for task in &mut tasks {
            task.update_freshness();
        }

        // Rebuild the heap
        for task in tasks {
            queue.push(task);
        }

        // Check if the highest priority task is due
        if let Some(task) = queue.peek() {
            if task.is_due() {
                return queue.pop();
            }
        }

        None
    }

    /// Get the next N URLs to crawl
    pub fn pop_due_batch(&self, count: usize) -> Vec<ScheduledCrawl> {
        let mut results = Vec::new();

        for _ in 0..count {
            if let Some(task) = self.pop_due() {
                results.push(task);
            } else {
                break;
            }
        }

        results
    }

    /// Reschedule a task after crawling
    pub fn reschedule(&self, mut task: ScheduledCrawl) -> Result<()> {
        task.mark_crawled();

        // Only reschedule if frequency is not Never
        if task.frequency != CrawlFrequency::Never {
            let mut queue = self.queue.lock().unwrap();
            queue.push(task.clone());
            info!("Rescheduled {} for {}", task.url, task.next_crawl_at);
        }

        Ok(())
    }

    /// Get the number of scheduled tasks
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// Check if the scheduler is empty
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// Get all scheduled tasks (for debugging/monitoring)
    pub fn get_all(&self) -> Vec<ScheduledCrawl> {
        let queue = self.queue.lock().unwrap();
        queue.clone().into_sorted_vec()
    }

    /// Clear all scheduled tasks
    pub fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
        info!("Cleared all scheduled tasks");
    }
}

impl Default for CrawlScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for the scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub due_tasks: usize,
    pub overdue_tasks: usize,
    pub average_freshness: f64,
}

impl CrawlScheduler {
    /// Get scheduler statistics
    pub fn stats(&self) -> SchedulerStats {
        let queue = self.queue.lock().unwrap();
        let tasks: Vec<_> = queue.clone().into_vec();

        let now = Utc::now();
        let due_tasks = tasks.iter().filter(|t| t.is_due()).count();
        let overdue_tasks = tasks.iter().filter(|t| now > t.next_crawl_at).count();

        let average_freshness = if !tasks.is_empty() {
            tasks.iter().map(|t| t.freshness_score).sum::<f64>() / tasks.len() as f64
        } else {
            0.0
        };

        SchedulerStats {
            total_tasks: tasks.len(),
            due_tasks,
            overdue_tasks,
            average_freshness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawl_frequency_to_duration() {
        assert_eq!(CrawlFrequency::Hourly.to_duration(), Some(Duration::hours(1)));
        assert_eq!(CrawlFrequency::Daily.to_duration(), Some(Duration::days(1)));
        assert_eq!(CrawlFrequency::Weekly.to_duration(), Some(Duration::weeks(1)));
        assert_eq!(CrawlFrequency::Monthly.to_duration(), Some(Duration::days(30)));
        assert_eq!(CrawlFrequency::Never.to_duration(), None);
    }

    #[test]
    fn test_crawl_frequency_from_change_frequency() {
        assert_eq!(CrawlFrequency::from_change_frequency(30.0), CrawlFrequency::Hourly);
        assert_eq!(CrawlFrequency::from_change_frequency(2.0), CrawlFrequency::Daily);
        assert_eq!(CrawlFrequency::from_change_frequency(0.5), CrawlFrequency::Weekly);
        assert_eq!(CrawlFrequency::from_change_frequency(0.01), CrawlFrequency::Monthly);
        assert_eq!(CrawlFrequency::from_change_frequency(0.0), CrawlFrequency::Never);
    }

    #[test]
    fn test_scheduled_crawl_is_due() {
        let crawl = ScheduledCrawl::new(
            "https://example.com".to_string(),
            CrawlFrequency::Hourly,
            50,
        );

        // Should be due immediately after creation
        assert!(crawl.is_due());
    }

    #[test]
    fn test_scheduled_crawl_mark_crawled() {
        let mut crawl = ScheduledCrawl::new(
            "https://example.com".to_string(),
            CrawlFrequency::Hourly,
            50,
        );

        crawl.mark_crawled();

        assert!(crawl.last_crawled_at.is_some());
        assert_eq!(crawl.freshness_score, 1.0);
        assert!(crawl.next_crawl_at > Utc::now());
    }

    #[test]
    fn test_scheduled_crawl_scheduling_score() {
        let crawl1 = ScheduledCrawl::new(
            "https://example.com".to_string(),
            CrawlFrequency::Daily,
            100, // High priority
        );

        let crawl2 = ScheduledCrawl::new(
            "https://example.org".to_string(),
            CrawlFrequency::Daily,
            10, // Low priority
        );

        // Higher priority should have higher score
        assert!(crawl1.scheduling_score() > crawl2.scheduling_score());
    }

    #[test]
    fn test_scheduler_basic() {
        let scheduler = CrawlScheduler::new();

        scheduler.schedule(
            "https://example.com".to_string(),
            CrawlFrequency::Daily,
            50,
        ).unwrap();

        assert_eq!(scheduler.len(), 1);
        assert!(!scheduler.is_empty());
    }

    #[test]
    fn test_scheduler_pop_due() {
        let scheduler = CrawlScheduler::new();

        scheduler.schedule(
            "https://example.com".to_string(),
            CrawlFrequency::Daily,
            50,
        ).unwrap();

        let task = scheduler.pop_due();
        assert!(task.is_some());
        assert_eq!(task.unwrap().url, "https://example.com");
        assert_eq!(scheduler.len(), 0);
    }

    #[test]
    fn test_scheduler_priority_order() {
        let scheduler = CrawlScheduler::new();

        // Add tasks with different priorities
        scheduler.schedule(
            "https://low.com".to_string(),
            CrawlFrequency::Daily,
            10,
        ).unwrap();

        scheduler.schedule(
            "https://high.com".to_string(),
            CrawlFrequency::Daily,
            100,
        ).unwrap();

        scheduler.schedule(
            "https://medium.com".to_string(),
            CrawlFrequency::Daily,
            50,
        ).unwrap();

        // Should pop in priority order (high -> medium -> low)
        assert_eq!(scheduler.pop_due().unwrap().url, "https://high.com");
        assert_eq!(scheduler.pop_due().unwrap().url, "https://medium.com");
        assert_eq!(scheduler.pop_due().unwrap().url, "https://low.com");
    }

    #[test]
    fn test_scheduler_reschedule() {
        let scheduler = CrawlScheduler::new();

        scheduler.schedule(
            "https://example.com".to_string(),
            CrawlFrequency::Daily,
            50,
        ).unwrap();

        let task = scheduler.pop_due().unwrap();
        scheduler.reschedule(task).unwrap();

        // Should be rescheduled
        assert_eq!(scheduler.len(), 1);

        // Should not be due immediately
        assert!(scheduler.pop_due().is_none());
    }

    #[test]
    fn test_scheduler_batch() {
        let scheduler = CrawlScheduler::new();

        for i in 0..10 {
            scheduler.schedule(
                format!("https://example{}.com", i),
                CrawlFrequency::Daily,
                50,
            ).unwrap();
        }

        let batch = scheduler.pop_due_batch(5);
        assert_eq!(batch.len(), 5);
        assert_eq!(scheduler.len(), 5);
    }

    #[test]
    fn test_scheduler_stats() {
        let scheduler = CrawlScheduler::new();

        scheduler.schedule(
            "https://example.com".to_string(),
            CrawlFrequency::Daily,
            50,
        ).unwrap();

        let stats = scheduler.stats();
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.due_tasks, 1);
    }

    #[test]
    fn test_freshness_decay() {
        let mut crawl = ScheduledCrawl::new(
            "https://example.com".to_string(),
            CrawlFrequency::Hourly,
            50,
        );

        // Simulate a crawl 30 minutes ago
        crawl.last_crawled_at = Some(Utc::now() - Duration::minutes(30));
        crawl.freshness_score = 1.0;

        crawl.update_freshness();

        // Freshness should have decayed (but not too much after 30 min of 1 hour)
        assert!(crawl.freshness_score < 1.0);
        assert!(crawl.freshness_score > 0.5);
    }
}
