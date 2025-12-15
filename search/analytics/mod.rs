use anyhow::Result;
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

// Phase 7.6: Search Analytics - Track search queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvent {
    pub query: String,
    pub result_count: usize,
    pub processing_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}

// Phase 7.7: Click Tracking - Track result clicks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent {
    pub query: String,
    pub clicked_url: String,
    pub position: usize,
    pub timestamp: DateTime<Utc>,
}

// Phase 7.8: Analytics aggregated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub query: String,
    pub search_count: usize,
    pub avg_result_count: f64,
    pub avg_processing_time_ms: f64,
    pub click_count: usize,
    pub click_through_rate: f64,
    pub last_searched: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_searches: usize,
    pub total_clicks: usize,
    pub avg_click_through_rate: f64,
    pub top_queries: Vec<QueryStats>,
    pub zero_result_queries: Vec<String>,
    pub popular_results: Vec<PopularResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularResult {
    pub url: String,
    pub title: String,
    pub click_count: usize,
}

#[derive(Clone)]
pub struct AnalyticsManager {
    redis: ConnectionManager,
}

impl AnalyticsManager {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    /// Phase 7.6: Track a search query
    pub async fn track_search(&mut self, event: SearchEvent) -> Result<()> {
        let key = format!("analytics:search:{}", event.timestamp.format("%Y-%m-%d"));
        let value = serde_json::to_string(&event)?;

        // Store individual search event
        let _: () = self.redis.lpush(&key, &value).await?;

        // Set expiry to 90 days
        let _: () = self.redis.expire(&key, 90 * 24 * 60 * 60).await?;

        // Increment query counter
        let query_key = format!("analytics:query_count:{}", event.query.to_lowercase());
        let _: () = self.redis.incr(&query_key, 1).await?;
        let _: () = self.redis.expire(&query_key, 90 * 24 * 60 * 60).await?;

        // Track zero-result queries
        if event.result_count == 0 {
            let zero_key = "analytics:zero_results".to_string();
            let _: () = self.redis.lpush(&zero_key, &event.query).await?;
            let _: () = self.redis.ltrim(&zero_key, 0, 99).await?; // Keep last 100
        }

        Ok(())
    }

    /// Phase 7.7: Track a result click
    pub async fn track_click(&mut self, event: ClickEvent) -> Result<()> {
        let key = format!("analytics:click:{}", event.timestamp.format("%Y-%m-%d"));
        let value = serde_json::to_string(&event)?;

        // Store individual click event
        let _: () = self.redis.lpush(&key, &value).await?;

        // Set expiry to 90 days
        let _: () = self.redis.expire(&key, 90 * 24 * 60 * 60).await?;

        // Increment click counter for this URL
        let url_key = format!("analytics:url_clicks:{}", event.clicked_url);
        let _: () = self.redis.incr(&url_key, 1).await?;
        let _: () = self.redis.expire(&url_key, 90 * 24 * 60 * 60).await?;

        // Track query-specific clicks
        let query_click_key = format!("analytics:query_clicks:{}", event.query.to_lowercase());
        let _: () = self.redis.incr(&query_click_key, 1).await?;
        let _: () = self.redis.expire(&query_click_key, 90 * 24 * 60 * 60).await?;

        Ok(())
    }

    /// Phase 7.8: Get analytics summary
    pub async fn get_summary(&mut self, days: usize) -> Result<AnalyticsSummary> {
        let mut total_searches = 0;
        let mut total_clicks = 0;
        let mut query_searches: HashMap<String, Vec<SearchEvent>> = HashMap::new();
        let mut query_clicks: HashMap<String, usize> = HashMap::new();

        // Collect data for the last N days
        for i in 0..days {
            let date = Utc::now() - chrono::Duration::days(i as i64);
            let date_str = date.format("%Y-%m-%d").to_string();

            // Get search events
            let search_key = format!("analytics:search:{}", date_str);
            if let Ok(events) = self.get_search_events(&search_key).await {
                total_searches += events.len();
                for event in events {
                    query_searches.entry(event.query.clone()).or_insert_with(Vec::new).push(event);
                }
            }

            // Get click events
            let click_key = format!("analytics:click:{}", date_str);
            if let Ok(clicks) = self.get_click_events(&click_key).await {
                total_clicks += clicks.len();
                for click in clicks {
                    *query_clicks.entry(click.query.clone()).or_insert(0) += 1;
                }
            }
        }

        // Calculate top queries
        let mut top_queries: Vec<QueryStats> = query_searches
            .iter()
            .map(|(query, events)| {
                let search_count = events.len();
                let avg_result_count = events.iter().map(|e| e.result_count).sum::<usize>() as f64 / search_count as f64;
                let avg_processing_time_ms = events.iter().map(|e| e.processing_time_ms).sum::<u64>() as f64 / search_count as f64;
                let click_count = query_clicks.get(query).copied().unwrap_or(0);
                let click_through_rate = if search_count > 0 {
                    (click_count as f64 / search_count as f64) * 100.0
                } else {
                    0.0
                };
                let last_searched = events.iter().map(|e| e.timestamp).max().unwrap_or_else(Utc::now);

                QueryStats {
                    query: query.clone(),
                    search_count,
                    avg_result_count,
                    avg_processing_time_ms,
                    click_count,
                    click_through_rate,
                    last_searched,
                }
            })
            .collect();

        // Sort by search count
        top_queries.sort_by(|a, b| b.search_count.cmp(&a.search_count));
        top_queries.truncate(20);

        // Get zero-result queries
        let zero_result_queries = self.get_zero_result_queries().await.unwrap_or_default();

        // Calculate avg CTR
        let avg_click_through_rate = if total_searches > 0 {
            (total_clicks as f64 / total_searches as f64) * 100.0
        } else {
            0.0
        };

        Ok(AnalyticsSummary {
            total_searches,
            total_clicks,
            avg_click_through_rate,
            top_queries,
            zero_result_queries,
            popular_results: Vec::new(), // TODO: Implement popular results
        })
    }

    async fn get_search_events(&mut self, key: &str) -> Result<Vec<SearchEvent>> {
        let events: Vec<String> = self.redis.lrange(key, 0, -1).await?;
        let parsed: Vec<SearchEvent> = events
            .iter()
            .filter_map(|e| serde_json::from_str(e).ok())
            .collect();
        Ok(parsed)
    }

    async fn get_click_events(&mut self, key: &str) -> Result<Vec<ClickEvent>> {
        let events: Vec<String> = self.redis.lrange(key, 0, -1).await?;
        let parsed: Vec<ClickEvent> = events
            .iter()
            .filter_map(|e| serde_json::from_str(e).ok())
            .collect();
        Ok(parsed)
    }

    async fn get_zero_result_queries(&mut self) -> Result<Vec<String>> {
        let queries: Vec<String> = self.redis.lrange("analytics:zero_results", 0, 19).await?;
        Ok(queries)
    }
}
