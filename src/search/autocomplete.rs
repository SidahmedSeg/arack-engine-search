use anyhow::{Context, Result};
use meilisearch_sdk::client::Client;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

const INDEX_NAME: &str = "documents";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteSuggestion {
    pub query: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteResponse {
    pub suggestions: Vec<AutocompleteSuggestion>,
    pub processing_time_ms: u64,
}

/// Query log-based autocomplete with Redis caching (production approach)
///
/// This implementation mines actual user search queries from the search_history table
/// instead of extracting from page titles. It provides:
/// - Higher diversity (different query intents, not repetitive domains)
/// - Better relevance (based on actual user behavior)
/// - Fast performance (Redis cache + database indexes)
pub struct QueryLogAutocomplete {
    pool: PgPool,
    redis: ConnectionManager,
}

impl QueryLogAutocomplete {
    pub fn new(pool: PgPool, redis: ConnectionManager) -> Self {
        Self { pool, redis }
    }

    /// Get autocomplete suggestions from user search history (with Redis cache)
    ///
    /// Flow:
    /// 1. Try Redis cache first (5-minute TTL)
    /// 2. On cache miss, query PostgreSQL search_history table
    /// 3. Score queries by: frequency (2x) + clicks (3x) + recency boost - age decay
    /// 4. Store result in Redis cache
    /// 5. Return diverse query suggestions
    pub async fn get_suggestions(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<AutocompleteResponse> {
        let start = Instant::now();

        // Guard: Minimum prefix length (avoid expensive scans)
        if prefix.len() < 2 {
            return Ok(AutocompleteResponse {
                suggestions: vec![],
                processing_time_ms: 0,
            });
        }

        // 1. Try Redis cache first
        let cache_key = format!("autocomplete:{}:{}", prefix.to_lowercase(), limit);

        // Clone redis connection for async operations
        let mut redis_conn = self.redis.clone();
        if let Ok(cached_json) = redis_conn.get::<_, String>(&cache_key).await {
            if let Ok(cached_response) = serde_json::from_str::<AutocompleteResponse>(&cached_json) {
                tracing::info!(
                    source = "redis_cache",
                    prefix = prefix,
                    results = cached_response.suggestions.len(),
                    "Autocomplete cache hit"
                );
                return Ok(cached_response);
            }
        }

        // 2. Cache miss - query database with scoring formula
        #[derive(sqlx::FromRow)]
        struct QueryScore {
            query: String,
            search_count: i64,
            click_count: i64,
            score: f64,
        }

        let results = sqlx::query_as::<_, QueryScore>(
            r#"
            WITH ranked_queries AS (
                SELECT
                    query,
                    COUNT(*) as search_count,
                    COALESCE(COUNT(clicked_url), 0) as click_count,
                    MAX(created_at) as last_searched,
                    COALESCE(EXTRACT(EPOCH FROM (NOW() - MAX(created_at))) / 86400.0, 0.0) as days_ago
                FROM search_history
                WHERE
                    LOWER(query) LIKE LOWER($1) || '%'
                    AND query != ''
                    AND LENGTH(query) >= LENGTH($1)
                GROUP BY query
            )
            SELECT
                query,
                search_count::bigint,
                click_count::bigint,
                CAST((
                    (search_count::float * 2.0) +
                    (click_count::float * 3.0) +
                    (CASE WHEN days_ago < 7 THEN 5.0 ELSE 0.0 END) +
                    (-1.0 * LEAST(days_ago * 0.1, 10.0))
                ) AS double precision) as score
            FROM ranked_queries
            WHERE search_count >= 2
            ORDER BY score DESC, search_count DESC
            LIMIT $2
            "#
        )
        .bind(prefix)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query search history for autocomplete")?;

        // Convert to API format
        let suggestions = results
            .into_iter()
            .map(|row| AutocompleteSuggestion {
                query: row.query,
                count: row.search_count as usize,  // Show search frequency
            })
            .collect();

        let processing_time = start.elapsed().as_millis() as u64;

        let response = AutocompleteResponse {
            suggestions,
            processing_time_ms: processing_time,
        };

        // 3. Store in Redis cache (5-minute TTL = 300 seconds)
        if let Ok(response_json) = serde_json::to_string(&response) {
            let mut redis_conn = self.redis.clone();
            let _: Result<(), _> = redis_conn.set_ex(&cache_key, response_json, 300).await;
        }

        tracing::info!(
            source = "query_log",
            prefix = prefix,
            results = response.suggestions.len(),
            latency_ms = processing_time,
            "Autocomplete query log mining (cache miss)"
        );

        Ok(response)
    }
}

pub struct AutocompleteService {
    client: Client,
}

impl AutocompleteService {
    pub fn new(url: &str, api_key: &str) -> Result<Self> {
        let client = Client::new(url, Some(api_key))?;
        Ok(Self { client })
    }

    /// Get autocomplete suggestions based on prefix
    ///
    /// This performs a prefix search in Meilisearch and returns matching queries
    /// with their document counts. Results are sorted by relevance.
    pub async fn get_suggestions(&self, prefix: &str, limit: usize) -> Result<AutocompleteResponse> {
        let start = Instant::now();

        if prefix.is_empty() {
            return Ok(AutocompleteResponse {
                suggestions: vec![],
                processing_time_ms: start.elapsed().as_millis() as u64,
            });
        }

        let index = self.client.index(INDEX_NAME);

        // Search for documents matching the prefix
        // We'll extract unique query patterns from titles and content
        let search_results = index
            .search()
            .with_query(prefix)
            .with_limit(limit * 3) // Get more results to extract diverse suggestions
            .execute::<serde_json::Value>()
            .await?;

        // Extract suggestions from search results
        let mut suggestions = Vec::new();
        let mut seen_queries = std::collections::HashSet::new();

        for hit in search_results.hits {
            if suggestions.len() >= limit {
                break;
            }

            // Try to extract meaningful query suggestions from the title
            if let Some(title) = hit.result.get("title").and_then(|v| v.as_str()) {
                let title_lower = title.to_lowercase();

                // Extract words that match the prefix
                for word in title_lower.split_whitespace() {
                    if word.starts_with(&prefix.to_lowercase()) && word.len() >= prefix.len() {
                        if seen_queries.insert(word.to_string()) && suggestions.len() < limit {
                            // Get approximate count by searching for this word
                            suggestions.push(AutocompleteSuggestion {
                                query: word.to_string(),
                                count: 1, // Simplified count
                            });
                        }
                    }
                }

                // Also add the full query if it's relevant
                if title_lower.contains(&prefix.to_lowercase()) && title.len() < 50 {
                    let normalized_title = title.to_lowercase().trim().to_string();
                    if seen_queries.insert(normalized_title.clone()) && suggestions.len() < limit {
                        suggestions.push(AutocompleteSuggestion {
                            query: normalized_title,
                            count: 1,
                        });
                    }
                }
            }
        }

        // If we didn't get enough suggestions, try common completions
        if suggestions.len() < limit {
            let common_completions = self.get_common_completions(prefix);
            for completion in common_completions {
                if seen_queries.insert(completion.clone()) && suggestions.len() < limit {
                    suggestions.push(AutocompleteSuggestion {
                        query: completion,
                        count: 1,
                    });
                }
            }
        }

        let processing_time = start.elapsed().as_millis() as u64;

        Ok(AutocompleteResponse {
            suggestions,
            processing_time_ms: processing_time,
        })
    }

    /// Get common completions for popular programming-related queries
    /// This is a simple fallback for when we don't have enough data
    fn get_common_completions(&self, prefix: &str) -> Vec<String> {
        let common_queries = vec![
            "rust programming",
            "rust tutorial",
            "rust language",
            "rust web framework",
            "rust async",
            "python tutorial",
            "python programming",
            "javascript tutorial",
            "javascript async",
            "typescript tutorial",
            "go programming",
            "go tutorial",
            "docker tutorial",
            "kubernetes tutorial",
            "api design",
            "rest api",
            "graphql tutorial",
            "database design",
            "sql tutorial",
            "nosql database",
        ];

        common_queries
            .iter()
            .filter(|q| q.starts_with(&prefix.to_lowercase()))
            .take(5)
            .map(|s| s.to_string())
            .collect()
    }
}
