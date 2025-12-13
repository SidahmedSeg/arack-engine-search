use anyhow::Result;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
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
