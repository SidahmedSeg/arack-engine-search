use anyhow::Result;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::search::Selectors;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;
use tracing::{info, warn};

use crate::search::crawler::{CrawledDocument, ImageData};
use crate::types::SearchQuery;

pub mod autocomplete;
pub use autocomplete::{AutocompleteResponse, AutocompleteSuggestion, AutocompleteService, QueryLogAutocomplete};

const INDEX_NAME: &str = "documents";
const IMAGES_INDEX_NAME: &str = "images";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub url: String,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    pub crawled_at: String,
    pub word_count: usize,
    // Phase 7.4: Domain for faceted search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    // Phase 7.3: Highlighted/formatted fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _formatted: Option<FormattedResult>,
    // Phase 9: Image count per document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_count: Option<usize>,
    // Phase 9: Favicon URL extracted during crawl
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchResult>,
    pub query: String,
    pub processing_time_ms: u64,
    pub total_hits: usize,
    // Phase 7.2: Search suggestions for typos/zero results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
    // Phase 7.4: Facet distribution for filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<std::collections::HashMap<String, std::collections::HashMap<String, usize>>>,
}

#[derive(Clone)]
pub struct SearchClient {
    client: Client,
    db_pool: Option<PgPool>,
    redis_conn: Option<ConnectionManager>,
}

impl SearchClient {
    /// Create SearchClient without database/Redis (backward compatibility)
    pub fn new(url: &str, api_key: &str) -> Result<Self> {
        let client = Client::new(url, Some(api_key))?;
        Ok(Self {
            client,
            db_pool: None,
            redis_conn: None,
        })
    }

    /// Create SearchClient with database pool and Redis for query log autocomplete
    pub fn new_with_db(url: &str, api_key: &str, db_pool: PgPool, redis_conn: ConnectionManager) -> Result<Self> {
        let client = Client::new(url, Some(api_key))?;
        Ok(Self {
            client,
            db_pool: Some(db_pool),
            redis_conn: Some(redis_conn),
        })
    }

    pub async fn initialize_index(&self) -> Result<()> {
        info!("Initializing search index: {}", INDEX_NAME);

        match self.client.create_index(INDEX_NAME, Some("id")).await {
            Ok(_) => {
                info!("Index created successfully");
                self.configure_index().await?;
            }
            Err(e) => {
                warn!("Index might already exist: {}", e);
            }
        }

        // Also initialize images index
        info!("Initializing images index: {}", IMAGES_INDEX_NAME);
        match self.client.create_index(IMAGES_INDEX_NAME, Some("id")).await {
            Ok(_) => {
                info!("Images index created successfully");
                self.configure_images_index().await?;
            }
            Err(e) => {
                warn!("Images index might already exist: {}", e);
            }
        }

        Ok(())
    }

    async fn configure_index(&self) -> Result<()> {
        let index = self.client.index(INDEX_NAME);

        // Configure searchable attributes (order matters for ranking)
        index
            .set_searchable_attributes(&[
                "title",
                "description",
                "keywords",
                "content",
                "url",
            ])
            .await?;

        // Configure displayed attributes
        index
            .set_displayed_attributes(&[
                "id",
                "url",
                "title",
                "content",
                "description",
                "keywords",
                "crawled_at",
                "word_count",
                "domain", // Phase 7.4: Domain for faceting
            ])
            .await?;

        // Configure filterable attributes (Phase 7.4: Added domain for faceted search)
        index
            .set_filterable_attributes(&["crawled_at", "word_count", "domain"])
            .await?;

        // Configure sortable attributes
        index
            .set_sortable_attributes(&["crawled_at", "word_count"])
            .await?;

        // Configure ranking rules
        index
            .set_ranking_rules(&[
                "words",
                "typo",
                "proximity",
                "attribute",
                "sort",
                "exactness",
            ])
            .await?;

        // Phase 7.5: Configure synonyms for better search quality
        let mut synonyms = std::collections::HashMap::new();
        synonyms.insert("js".to_string(), vec!["javascript".to_string()]);
        synonyms.insert("javascript".to_string(), vec!["js".to_string(), "ecmascript".to_string()]);
        synonyms.insert("ts".to_string(), vec!["typescript".to_string()]);
        synonyms.insert("typescript".to_string(), vec!["ts".to_string()]);
        synonyms.insert("py".to_string(), vec!["python".to_string()]);
        synonyms.insert("python".to_string(), vec!["py".to_string()]);
        synonyms.insert("golang".to_string(), vec!["go".to_string()]);
        synonyms.insert("go".to_string(), vec!["golang".to_string()]);
        synonyms.insert("react".to_string(), vec!["reactjs".to_string(), "react.js".to_string()]);
        synonyms.insert("vue".to_string(), vec!["vuejs".to_string(), "vue.js".to_string()]);
        synonyms.insert("ml".to_string(), vec!["machine learning".to_string(), "machine-learning".to_string()]);
        synonyms.insert("ai".to_string(), vec!["artificial intelligence".to_string()]);
        synonyms.insert("db".to_string(), vec!["database".to_string()]);
        synonyms.insert("api".to_string(), vec!["application programming interface".to_string()]);

        if let Err(e) = index.set_synonyms(&synonyms).await {
            warn!("Failed to set synonyms: {}", e);
        }

        // Phase 7.5: Configure stop words (common words to ignore)
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "with", "to", "for",
            "of", "on", "at", "from", "by", "about", "as", "into", "through",
            "during", "before", "after", "above", "below", "between", "under",
            "again", "further", "then", "once"
        ];
        if let Err(e) = index.set_stop_words(&stop_words).await {
            warn!("Failed to set stop words: {}", e);
        }

        // Phase 7.5: Typo tolerance is enabled by default in Meilisearch
        // It will automatically handle fuzzy matching for typos

        info!("Index configured successfully with optimizations");
        Ok(())
    }

    async fn configure_images_index(&self) -> Result<()> {
        let index = self.client.index(IMAGES_INDEX_NAME);

        // Configure searchable attributes for images
        // Priority 1: Include figcaption as highly valuable semantic text
        index
            .set_searchable_attributes(&[
                "figcaption",     // Priority 1: Rich semantic descriptions
                "alt_text",
                "title",
                "page_title",
                "page_content",
            ])
            .await?;

        // Configure displayed attributes
        index
            .set_displayed_attributes(&[
                "id",
                "image_url",
                "source_url",
                "alt_text",
                "title",
                "width",
                "height",
                "page_title",
                "page_content",   // Required by ImageData struct
                "domain",
                "crawled_at",
                "is_og_image",    // Priority 1: Flag for high-quality OG images
                "figcaption",     // Priority 1: Rich semantic caption
                "srcset_url",     // Priority 1: Highest resolution URL
            ])
            .await?;

        // Configure filterable attributes
        // Priority 1: Add is_og_image filter for high-quality images
        // Phase 9: Add source_url for counting images per page
        index
            .set_filterable_attributes(&["domain", "width", "height", "crawled_at", "is_og_image", "source_url"])
            .await?;

        // Configure sortable attributes
        index
            .set_sortable_attributes(&["crawled_at", "width", "height"])
            .await?;

        // Configure ranking rules
        index
            .set_ranking_rules(&[
                "words",
                "typo",
                "proximity",
                "attribute",
                "sort",
                "exactness",
            ])
            .await?;

        info!("Images index configured successfully");
        Ok(())
    }

    pub async fn index_documents(&self, documents: Vec<CrawledDocument>) -> Result<()> {
        if documents.is_empty() {
            return Ok(());
        }

        info!("Indexing {} documents", documents.len());
        let index = self.client.index(INDEX_NAME);

        index.add_documents(&documents, Some("id")).await?;
        info!("Documents indexed successfully");

        Ok(())
    }

    pub async fn index_images(&self, images: Vec<ImageData>) -> Result<()> {
        if images.is_empty() {
            return Ok(());
        }

        info!("Indexing {} images", images.len());
        let index = self.client.index(IMAGES_INDEX_NAME);

        index.add_documents(&images, Some("id")).await?;
        info!("Images indexed successfully");

        Ok(())
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<SearchResponse> {
        let index = self.client.index(INDEX_NAME);

        // Phase 7.3: Enable highlighting and cropping
        let attributes_to_highlight = vec!["title", "content", "description"];
        let attributes_to_crop = vec![("content", Some(200))];

        let search_results = index
            .search()
            .with_query(query)
            .with_limit(limit)
            .with_attributes_to_highlight(Selectors::Some(&attributes_to_highlight))
            .with_attributes_to_crop(Selectors::Some(&attributes_to_crop))
            .with_highlight_pre_tag("<mark>")
            .with_highlight_post_tag("</mark>")
            .with_show_matches_position(true)
            .execute::<SearchResult>()
            .await?;

        let hits: Vec<SearchResult> = search_results.hits.into_iter().map(|h| h.result).collect();
        let total_hits = search_results.estimated_total_hits.unwrap_or(0);

        // Phase 7.2: Generate suggestions if zero results
        let suggestions = if hits.is_empty() {
            self.generate_suggestions(query).await.ok()
        } else {
            None
        };

        Ok(SearchResponse {
            hits,
            query: query.to_string(),
            processing_time_ms: search_results.processing_time_ms as u64,
            total_hits,
            suggestions,
            facets: None, // Basic search doesn't use facets
        })
    }

    pub async fn search_with_params(&self, params: SearchQuery) -> Result<SearchResponse> {
        let index = self.client.index(INDEX_NAME);

        let mut search = index.search();
        search.with_query(&params.q);
        search.with_limit(params.limit);
        search.with_offset(params.offset);

        // Phase 7.3: Enable highlighting and cropping
        let attributes_to_highlight = vec!["title", "content", "description"];
        let attributes_to_crop = vec![("content", Some(200))];
        search.with_attributes_to_highlight(Selectors::Some(&attributes_to_highlight));
        search.with_attributes_to_crop(Selectors::Some(&attributes_to_crop));
        search.with_highlight_pre_tag("<mark>");
        search.with_highlight_post_tag("</mark>");
        search.with_show_matches_position(true);

        // Build filter string - needs to live for the lifetime of search
        let mut filters = Vec::new();

        // Word count filters
        if let Some(min) = params.min_word_count {
            filters.push(format!("word_count >= {}", min));
        }
        if let Some(max) = params.max_word_count {
            filters.push(format!("word_count <= {}", max));
        }

        // Date filters
        if let Some(from) = params.from_date.as_ref() {
            filters.push(format!("crawled_at >= '{}'", from));
        }
        if let Some(to) = params.to_date.as_ref() {
            filters.push(format!("crawled_at <= '{}'", to));
        }

        // Phase 7.4: Domain filter for faceted search
        if let Some(domain) = params.domain.as_ref() {
            filters.push(format!("domain = '{}'", domain));
        }

        // Apply filters if any
        let filter_str = filters.join(" AND ");
        if !filters.is_empty() {
            search.with_filter(&filter_str);
        }

        // Phase 7.4: Enable facets for domain distribution
        search.with_facets(Selectors::Some(&["domain"]));

        // Build sort string outside the if block so it lives long enough
        let sort_str = params.sort_by.as_ref().map(|sort_by| {
            let sort_order = params.sort_order.as_deref().unwrap_or("asc");
            format!("{}:{}", sort_by, sort_order)
        });

        // Apply sorting if sort_str exists
        let sort_array;
        if let Some(ref s) = sort_str {
            sort_array = vec![s.as_str()];
            search.with_sort(&sort_array);
        }

        let search_results = search.execute::<SearchResult>().await?;

        let hits: Vec<SearchResult> = search_results.hits.into_iter().map(|h| h.result).collect();
        let total_hits = search_results.estimated_total_hits.unwrap_or(0);

        // Phase 7.2: Generate suggestions if zero results
        let suggestions = if hits.is_empty() {
            self.generate_suggestions(&params.q).await.ok()
        } else {
            None
        };

        // Phase 7.4: Extract facet distribution
        let facets = search_results.facet_distribution;

        Ok(SearchResponse {
            hits,
            query: params.q.clone(),
            processing_time_ms: search_results.processing_time_ms as u64,
            total_hits,
            suggestions,
            facets,
        })
    }

    /// Generate query suggestions for typos or zero results
    /// Phase 7.2: Search Suggestions
    async fn generate_suggestions(&self, query: &str) -> Result<Vec<String>> {
        // Try variations of the query
        let mut suggestions = Vec::new();

        // Split query into words and try different combinations
        let words: Vec<&str> = query.split_whitespace().collect();

        if words.len() > 1 {
            // Try removing one word at a time
            for i in 0..words.len() {
                let mut modified_words = words.clone();
                modified_words.remove(i);
                let suggested_query = modified_words.join(" ");

                // Check if this suggestion returns results
                if self.has_results(&suggested_query).await.unwrap_or(false) {
                    suggestions.push(suggested_query);
                    if suggestions.len() >= 3 {
                        break;
                    }
                }
            }
        }

        // Try the query without special characters
        let cleaned_query = query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();

        if cleaned_query != query && self.has_results(&cleaned_query).await.unwrap_or(false) {
            suggestions.push(cleaned_query);
        }

        // Try common tech term corrections
        let common_corrections = vec![
            ("javascript", vec!["java script", "js"]),
            ("typescript", vec!["type script", "ts"]),
            ("python", vec!["pyton", "phyton"]),
            ("golang", vec!["go lang", "go"]),
        ];

        for (correct, typos) in common_corrections {
            if typos.iter().any(|t| query.to_lowercase().contains(t)) {
                suggestions.push(correct.to_string());
                break;
            }
        }

        Ok(suggestions.into_iter().take(3).collect())
    }

    /// Check if a query returns any results
    async fn has_results(&self, query: &str) -> Result<bool> {
        let index = self.client.index(INDEX_NAME);
        let search_results = index
            .search()
            .with_query(query)
            .with_limit(1)
            .execute::<SearchResult>()
            .await?;

        Ok(search_results.estimated_total_hits.unwrap_or(0) > 0)
    }

    pub async fn clear_index(&self) -> Result<()> {
        info!("Clearing index: {}", INDEX_NAME);
        let index = self.client.index(INDEX_NAME);
        index.delete_all_documents().await?;
        info!("Index cleared successfully");
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let index = self.client.index(INDEX_NAME);
        let stats = index.get_stats().await?;

        // Manually create JSON since IndexStats doesn't implement Serialize
        Ok(serde_json::json!({
            "numberOfDocuments": stats.number_of_documents,
            "isIndexing": stats.is_indexing,
            "fieldDistribution": stats.field_distribution,
        }))
    }

    /// Get image index statistics
    pub async fn get_image_stats(&self) -> Result<serde_json::Value> {
        let index = self.client.index(IMAGES_INDEX_NAME);
        let stats = index.get_stats().await?;

        // Manually create JSON since IndexStats doesn't implement Serialize
        Ok(serde_json::json!({
            "numberOfImages": stats.number_of_documents,
            "isIndexing": stats.is_indexing,
            "fieldDistribution": stats.field_distribution,
        }))
    }

    /// Get autocomplete suggestions for a query prefix
    /// Phase 7.1 ENHANCED: Hybrid autocomplete (query log + Meilisearch fallback)
    ///
    /// Strategy:
    /// 1. Try query log autocomplete first (if db_pool + redis_conn available)
    /// 2. If insufficient results (<3), add Meilisearch fallback
    /// 3. Deduplicate and return up to `limit` suggestions
    pub async fn autocomplete(&self, prefix: &str, limit: usize) -> Result<AutocompleteResponse> {
        let start = Instant::now();
        let mut suggestions = Vec::new();

        // 1. Try query log autocomplete first (primary source, with Redis caching)
        if let (Some(ref pool), Some(ref redis)) = (&self.db_pool, &self.redis_conn) {
            let query_log = QueryLogAutocomplete::new(pool.clone(), redis.clone());
            match query_log.get_suggestions(prefix, limit).await {
                Ok(response) => {
                    suggestions.extend(response.suggestions);
                    tracing::info!(
                        "Query log autocomplete: {} suggestions for '{}'",
                        suggestions.len(),
                        prefix
                    );
                }
                Err(e) => {
                    tracing::warn!("Query log autocomplete failed: {}, using fallback", e);
                }
            }
        }

        // 2. If insufficient results, add Meilisearch fallback
        if suggestions.len() < 3 {
            let needed = limit.saturating_sub(suggestions.len());
            match self.meilisearch_autocomplete(prefix, needed).await {
                Ok(fallback) => {
                    let fallback_count = fallback.suggestions.len();
                    suggestions.extend(fallback.suggestions);
                    tracing::info!(
                        "Meilisearch fallback: {} suggestions for '{}'",
                        fallback_count,
                        prefix
                    );
                }
                Err(e) => {
                    tracing::error!("Meilisearch autocomplete fallback failed: {}", e);
                }
            }
        }

        // 3. Deduplicate by query (case-insensitive)
        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|s| seen.insert(s.query.to_lowercase()));

        // 4. Truncate to requested limit
        suggestions.truncate(limit);

        let total_time = start.elapsed().as_millis() as u64;

        Ok(AutocompleteResponse {
            suggestions,
            processing_time_ms: total_time,
        })
    }

    /// Meilisearch-based autocomplete (fallback for rare queries)
    async fn meilisearch_autocomplete(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<AutocompleteResponse> {
        let autocomplete_service = AutocompleteService::new(
            &self.client.get_host(),
            self.client.get_api_key().unwrap_or(""),
        )?;
        autocomplete_service.get_suggestions(prefix, limit).await
    }

    /// Search images index
    pub async fn search_images(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        min_width: Option<u32>,
        min_height: Option<u32>,
        domain: Option<String>,
    ) -> Result<serde_json::Value> {
        let index = self.client.index(IMAGES_INDEX_NAME);

        // Build filter string
        let mut filters = Vec::new();
        if let Some(width) = min_width {
            filters.push(format!("width >= {}", width));
        }
        if let Some(height) = min_height {
            filters.push(format!("height >= {}", height));
        }
        if let Some(d) = domain {
            filters.push(format!("domain = \"{}\"", d));
        }

        let filter_str = if filters.is_empty() {
            String::new()
        } else {
            filters.join(" AND ")
        };

        let mut search_request = index.search();
        search_request
            .with_query(query)
            .with_limit(limit)
            .with_offset(offset);

        if !filter_str.is_empty() {
            search_request.with_filter(&filter_str);
        }

        let search_results = search_request.execute::<ImageData>().await?;

        // Extract the actual ImageData from search results
        let hits: Vec<ImageData> = search_results.hits.into_iter().map(|h| h.result).collect();

        Ok(serde_json::json!({
            "hits": hits,
            "query": query,
            "processing_time_ms": search_results.processing_time_ms,
            "total_hits": search_results.estimated_total_hits.unwrap_or(0),
        }))
    }

    /// Get image counts grouped by source URL
    pub async fn get_image_counts_by_url(&self, urls: Vec<String>) -> Result<std::collections::HashMap<String, usize>> {
        let index = self.client.index(IMAGES_INDEX_NAME);
        let mut counts = std::collections::HashMap::new();

        // For each URL, count how many images have that source_url
        for url in urls {
            let filter = format!("source_url = \"{}\"", url);
            let search_result = index
                .search()
                .with_query("")
                .with_filter(&filter)
                .with_limit(0) // We only need the count
                .execute::<ImageData>()
                .await?;

            let count = search_result.estimated_total_hits.unwrap_or(0);
            counts.insert(url, count);
        }

        Ok(counts)
    }
}
