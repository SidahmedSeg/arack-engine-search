mod circuit_breaker;
mod filters;
mod headers;
mod image_extractor;
mod politeness;
mod rate_limiter;
mod retry;
mod robots;
mod scheduler;
mod url_processor;

pub use circuit_breaker::{CircuitBreakerManager, CircuitBreakerStats, CircuitState, DomainCircuitStats};
pub use filters::{ContentFilter, FilterStats};
pub use headers::HeaderManager;
pub use image_extractor::{ImageData, ImageExtractor};
pub use politeness::{PolitenessManager, PolitenessStats};
pub use rate_limiter::{RateLimiter, RateLimiterStats};
pub use retry::{RetryConfig, RetryPolicy, RetryStats};
pub use robots::{RobotsManager, RobotsStats};
pub use scheduler::{CrawlFrequency, CrawlScheduler, ScheduledCrawl, SchedulerStats};
pub use url_processor::UrlProcessor;

use anyhow::Result;
use chrono::Utc;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use spider::website::Website;
use std::collections::HashSet;
use tracing::{debug, info, warn};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlJob {
    pub id: String,
    pub urls: Vec<String>,
    pub max_depth: usize,
    pub status: CrawlStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CrawlStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawledDocument {
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
    // Phase 9: Favicon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub max_depth: usize,
    pub max_concurrent: usize,
    pub max_content_length: usize,
    pub respect_robots_txt: bool,
    // Phase 6.2: Rate limiting
    pub requests_per_second: u32,
    pub min_delay_ms: u64,
    pub max_retries: u32,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_concurrent: 10,
            max_content_length: 10000,
            respect_robots_txt: true,
            requests_per_second: 2,
            min_delay_ms: 1000,
            max_retries: 3,
        }
    }
}

#[derive(Clone)]
pub struct Crawler {
    config: CrawlerConfig,
    rate_limiter: RateLimiter,
    politeness: PolitenessManager,
    headers: HeaderManager,
    robots: RobotsManager,
    filters: ContentFilter,
    url_processor: UrlProcessor,
    retry_policy: RetryPolicy,
    circuit_breaker: CircuitBreakerManager,
    scheduler: CrawlScheduler,
}

impl Crawler {
    pub fn new(max_depth: usize, max_concurrent: usize) -> Self {
        let config = CrawlerConfig {
            max_depth,
            max_concurrent,
            ..Default::default()
        };

        let rate_limiter = RateLimiter::new(config.requests_per_second, config.min_delay_ms);
        let politeness = PolitenessManager::new(config.min_delay_ms, config.max_retries);
        let headers = HeaderManager::default();
        let robots = RobotsManager::new("EngineSearchBot/1.0".to_string());
        let filters = ContentFilter::default();
        let url_processor = UrlProcessor::default();
        let retry_policy = RetryPolicy::new();
        let circuit_breaker = CircuitBreakerManager::default();
        let scheduler = CrawlScheduler::new();

        Self {
            config,
            rate_limiter,
            politeness,
            headers,
            robots,
            filters,
            url_processor,
            retry_policy,
            circuit_breaker,
            scheduler,
        }
    }

    pub fn with_config(config: CrawlerConfig) -> Self {
        let rate_limiter = RateLimiter::new(config.requests_per_second, config.min_delay_ms);
        let politeness = PolitenessManager::new(config.min_delay_ms, config.max_retries);
        let headers = HeaderManager::default();
        let robots = RobotsManager::new("EngineSearchBot/1.0".to_string());
        let filters = ContentFilter::default();
        let url_processor = UrlProcessor::default();
        let retry_policy = RetryPolicy::new();
        let circuit_breaker = CircuitBreakerManager::default();
        let scheduler = CrawlScheduler::new();

        Self {
            config,
            rate_limiter,
            politeness,
            headers,
            robots,
            filters,
            url_processor,
            retry_policy,
            circuit_breaker,
            scheduler,
        }
    }

    /// Create crawler with full configuration including headers
    pub fn with_headers(
        config: CrawlerConfig,
        user_agent: String,
        contact_email: Option<String>,
        bot_url: Option<String>,
        accept_language: String,
    ) -> Self {
        let rate_limiter = RateLimiter::new(config.requests_per_second, config.min_delay_ms);
        let politeness = PolitenessManager::new(config.min_delay_ms, config.max_retries);
        let robots = RobotsManager::new(user_agent.clone());
        let headers = HeaderManager::with_config(user_agent, contact_email, bot_url, accept_language);
        let filters = ContentFilter::default();
        let url_processor = UrlProcessor::default();
        let retry_policy = RetryPolicy::new();
        let circuit_breaker = CircuitBreakerManager::default();
        let scheduler = CrawlScheduler::new();

        Self {
            config,
            rate_limiter,
            politeness,
            headers,
            robots,
            filters,
            url_processor,
            retry_policy,
            circuit_breaker,
            scheduler,
        }
    }

    pub async fn crawl_urls(&self, urls: Vec<String>) -> Result<(Vec<CrawledDocument>, Vec<ImageData>)> {
        let mut all_documents = Vec::new();
        let mut all_images = Vec::new();
        let mut seen_urls = HashSet::new();

        for url in urls {
            info!("Starting crawl for: {}", url);

            // Normalize URL (Phase 6.5)
            let normalized_url = match self.url_processor.normalize(&url) {
                Ok(u) => u,
                Err(e) => {
                    warn!("Invalid URL {}: {}", url, e);
                    continue;
                }
            };

            // Extract domain for circuit breaker (Phase 6.6)
            let domain = match Url::parse(&normalized_url) {
                Ok(parsed) => parsed.host_str().unwrap_or("").to_string(),
                Err(e) => {
                    warn!("Failed to parse URL {}: {}", normalized_url, e);
                    continue;
                }
            };

            // Check circuit breaker (Phase 6.6)
            if !self.circuit_breaker.can_proceed(&domain) {
                warn!("Circuit breaker is open for domain: {}", domain);
                continue;
            }

            // Check content filters (Phase 6.4)
            if !self.filters.is_url_allowed(&normalized_url) {
                warn!("URL filtered out: {}", normalized_url);
                continue;
            }

            // Check robots.txt (Phase 6.1)
            match self.robots.is_allowed(&normalized_url).await {
                Ok(false) => {
                    warn!("URL blocked by robots.txt: {}", normalized_url);
                    continue;
                }
                Err(e) => {
                    warn!("Error checking robots.txt for {}: {}", normalized_url, e);
                    // Continue on error (permissive approach)
                }
                Ok(true) => {} // Allowed, continue
            }

            // Apply crawl delay from robots.txt
            if let Ok(parsed_url) = Url::parse(&normalized_url) {
                if let Some(domain) = parsed_url.host_str() {
                    if let Some(delay) = self.robots.get_crawl_delay(domain).await {
                        info!("Applying robots.txt crawl delay for {}: {}s", domain, delay);
                        self.politeness.set_crawl_delay(domain, delay);
                    }
                }
            }

            // Apply rate limiting and politeness
            if let Err(e) = self.wait_for_request(&normalized_url).await {
                warn!("Rate limiting error for {}: {}", normalized_url, e);
                continue;
            }

            match self.crawl_single_url(&normalized_url).await {
                Ok((documents, images)) => {
                    info!("Successfully crawled {} pages and {} images from {}", documents.len(), images.len(), normalized_url);

                    // Record success in circuit breaker (Phase 6.6)
                    self.circuit_breaker.record_success(&domain);

                    // Deduplicate documents using normalized URLs (Phase 6.5)
                    for doc in documents {
                        let normalized_doc_url = self.url_processor.normalize(&doc.url)
                            .unwrap_or_else(|_| doc.url.clone());

                        if seen_urls.insert(normalized_doc_url) {
                            all_documents.push(doc);
                        } else {
                            debug!("Skipping duplicate URL: {}", doc.url);
                        }
                    }

                    // Collect all images
                    all_images.extend(images);
                }
                Err(e) => {
                    warn!("Failed to crawl {}: {}", normalized_url, e);
                    // Record failure in circuit breaker (Phase 6.6)
                    self.circuit_breaker.record_failure(&domain);
                }
            }
        }

        info!(
            "Crawl completed. Total unique documents: {}, Total images: {}",
            all_documents.len(),
            all_images.len()
        );
        Ok((all_documents, all_images))
    }

    async fn crawl_single_url(&self, url: &str) -> Result<(Vec<CrawledDocument>, Vec<ImageData>)> {
        let mut website = Website::new(url);

        // Configure spider settings with professional headers
        let user_agent = self.headers.user_agent_string();
        website
            .with_respect_robots_txt(self.config.respect_robots_txt)
            .with_depth(self.config.max_depth)
            .with_user_agent(Some(user_agent.as_str().into()))
            .with_budget(None);

        // Start crawling and subscribe to pages
        let mut rx = website.subscribe(0).unwrap();

        // Start the crawl in the background
        let handle = tokio::spawn(async move {
            website.crawl().await;
        });

        let mut documents = Vec::new();
        let mut images = Vec::new();
        let mut page_count = 0;

        // Collect pages as they arrive
        while let Ok(page) = rx.recv().await {
            page_count += 1;
            let page_url = page.get_url();
            debug!("Processing page {}: {}", page_count, page_url);

            let html = page.get_html();

            // Process page content
            match self.process_page(page_url, &html) {
                Ok(Some(doc)) => {
                    // Extract images from this page
                    match ImageExtractor::extract_images(
                        &html,
                        page_url,
                        &doc.title,
                        &doc.content,
                    ) {
                        Ok(page_images) => {
                            if !page_images.is_empty() {
                                debug!("Extracted {} images from {}", page_images.len(), page_url);
                                images.extend(page_images);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to extract images from {}: {}", page_url, e);
                        }
                    }

                    documents.push(doc);
                }
                Ok(None) => {
                    debug!("Skipped page (empty content): {}", page_url);
                }
                Err(e) => {
                    warn!("Failed to process page {}: {}", page_url, e);
                }
            }
        }

        // Wait for crawl to complete
        handle.await?;

        if documents.is_empty() {
            warn!("No valid documents extracted from {}", url);
        } else {
            info!("Extracted {} documents and {} images from {}", documents.len(), images.len(), url);
        }

        Ok((documents, images))
    }

    fn process_page(&self, url: &str, html: &str) -> Result<Option<CrawledDocument>> {
        let document = Html::parse_document(html);

        // Extract title
        let title = self
            .extract_title(&document)
            .unwrap_or_else(|| url.to_string());

        // Extract meta description
        let description = self.extract_meta_description(&document);

        // Extract keywords
        let keywords = self.extract_keywords(&document);

        // Phase 9: Extract favicon
        let favicon_url = self.extract_favicon(&document, url);

        // Extract main content
        let content = self.extract_content(&document)?;

        // Skip if content is too short (likely error pages or empty pages)
        if content.len() < 50 {
            return Ok(None);
        }

        let word_count = content.split_whitespace().count();

        // Phase 7.4: Extract domain for faceted search
        let domain = Url::parse(url)
            .ok()
            .and_then(|parsed_url| parsed_url.domain().map(|d| d.to_string()));

        Ok(Some(CrawledDocument {
            id: Uuid::new_v4().to_string(),
            url: url.to_string(),
            title,
            content,
            description,
            keywords,
            crawled_at: Utc::now().to_rfc3339(),
            word_count,
            domain,
            favicon_url,
        }))
    }

    fn extract_title(&self, document: &Html) -> Option<String> {
        let title_selector = Selector::parse("title").ok()?;
        let title = document.select(&title_selector).next()?;
        let text = title.text().collect::<String>().trim().to_string();

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn extract_meta_description(&self, document: &Html) -> Option<String> {
        let meta_selector =
            Selector::parse(r#"meta[name="description"], meta[property="og:description"]"#).ok()?;

        for meta in document.select(&meta_selector) {
            if let Some(content) = meta.value().attr("content") {
                let desc = content.trim().to_string();
                if !desc.is_empty() {
                    return Some(desc);
                }
            }
        }

        None
    }

    fn extract_keywords(&self, document: &Html) -> Option<Vec<String>> {
        let meta_selector = Selector::parse(r#"meta[name="keywords"]"#).ok()?;

        for meta in document.select(&meta_selector) {
            if let Some(content) = meta.value().attr("content") {
                let keywords: Vec<String> = content
                    .split(',')
                    .map(|k| k.trim().to_string())
                    .filter(|k| !k.is_empty())
                    .collect();

                if !keywords.is_empty() {
                    return Some(keywords);
                }
            }
        }

        None
    }

    fn extract_favicon(&self, document: &Html, base_url: &str) -> Option<String> {
        // Priority order for favicon extraction:
        // 1. <link rel="icon"> or <link rel="shortcut icon">
        // 2. <link rel="apple-touch-icon">
        // 3. Fallback to /favicon.ico

        let selectors = vec![
            r#"link[rel="icon"]"#,
            r#"link[rel="shortcut icon"]"#,
            r#"link[rel="apple-touch-icon"]"#,
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for link in document.select(&selector) {
                    if let Some(href) = link.value().attr("href") {
                        let href = href.trim();
                        if !href.is_empty() {
                            // Convert relative URL to absolute
                            if let Ok(base) = Url::parse(base_url) {
                                if let Ok(absolute_url) = base.join(href) {
                                    return Some(absolute_url.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: construct /favicon.ico URL
        if let Ok(base) = Url::parse(base_url) {
            if let Ok(favicon_url) = base.join("/favicon.ico") {
                return Some(favicon_url.to_string());
            }
        }

        None
    }

    fn extract_content(&self, document: &Html) -> Result<String> {
        // Remove script and style elements
        let mut html_string = document.html();

        // Remove scripts
        let script_selector = Selector::parse("script, style, noscript").unwrap();
        for element in document.select(&script_selector) {
            let element_html = element.html();
            html_string = html_string.replace(&element_html, "");
        }

        // Parse cleaned HTML
        let cleaned_doc = Html::parse_document(&html_string);

        // Try to extract main content area first
        let content_selectors = vec![
            "main",
            "article",
            "[role='main']",
            ".content",
            "#content",
            "body",
        ];

        for selector_str in content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = cleaned_doc.select(&selector).next() {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    let cleaned = self.clean_text(&text);

                    if cleaned.len() > 100 {
                        // Only use if substantial content
                        return Ok(self.truncate_text(&cleaned, self.config.max_content_length));
                    }
                }
            }
        }

        // Fallback: extract all text from body
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = cleaned_doc.select(&body_selector).next() {
                let text = body.text().collect::<Vec<_>>().join(" ");
                let cleaned = self.clean_text(&text);
                return Ok(self.truncate_text(&cleaned, self.config.max_content_length));
            }
        }

        // Last resort: use html2text
        let text = html2text::from_read(html_string.as_bytes(), self.config.max_content_length);
        Ok(self.clean_text(&text))
    }

    fn clean_text(&self, text: &str) -> String {
        // Remove excessive whitespace
        let cleaned = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // Normalize whitespace
        cleaned
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn truncate_text(&self, text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }

        // Try to truncate at a word boundary
        let truncated = &text[..max_length];
        if let Some(last_space) = truncated.rfind(' ') {
            truncated[..last_space].to_string()
        } else {
            truncated.to_string()
        }
    }

    /// Apply rate limiting and politeness before making a request
    pub async fn wait_for_request(&self, url: &str) -> Result<()> {
        // Apply rate limiter (token bucket)
        self.rate_limiter.wait_for(url).await?;

        // Apply politeness delay
        self.politeness.wait_before_request(url).await?;

        Ok(())
    }

    /// Get rate limiter statistics
    pub fn rate_limiter_stats(&self) -> RateLimiterStats {
        self.rate_limiter.stats()
    }

    /// Get politeness statistics
    pub fn politeness_stats(&self) -> PolitenessStats {
        self.politeness.stats()
    }

    /// Get robots.txt statistics
    pub fn robots_stats(&self) -> RobotsStats {
        self.robots.stats()
    }

    /// Set custom crawl delay for a domain (from robots.txt)
    pub fn set_domain_crawl_delay(&self, domain: &str, delay_secs: f64) {
        self.politeness.set_crawl_delay(domain, delay_secs);
    }

    /// Get the header manager
    pub fn headers(&self) -> &HeaderManager {
        &self.headers
    }

    /// Get current User-Agent string
    pub fn user_agent(&self) -> String {
        self.headers.user_agent_string()
    }

    /// Get the content filter
    pub fn filters(&self) -> &ContentFilter {
        &self.filters
    }

    /// Get mutable reference to content filter for configuration
    pub fn filters_mut(&mut self) -> &mut ContentFilter {
        &mut self.filters
    }

    /// Get filter statistics
    pub fn filter_stats(&self) -> FilterStats {
        self.filters.stats()
    }

    /// Get the retry policy
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// Get the circuit breaker manager
    pub fn circuit_breaker(&self) -> &CircuitBreakerManager {
        &self.circuit_breaker
    }

    /// Get circuit breaker statistics
    pub fn circuit_breaker_stats(&self) -> CircuitBreakerStats {
        self.circuit_breaker.stats()
    }

    /// Manually reset circuit breaker for a domain
    pub fn reset_circuit_breaker(&self, domain: &str) {
        self.circuit_breaker.reset(domain);
    }

    /// Get the crawl scheduler
    pub fn scheduler(&self) -> &CrawlScheduler {
        &self.scheduler
    }

    /// Get scheduler statistics
    pub fn scheduler_stats(&self) -> SchedulerStats {
        self.scheduler.stats()
    }

    /// Schedule a URL for crawling
    pub fn schedule_url(&self, url: String, frequency: CrawlFrequency, priority: u8) -> Result<()> {
        self.scheduler.schedule(url, frequency, priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let crawler = Crawler::new(3, 10);

        let text = "  Hello   World  \n\n  This is  a test  ";
        assert_eq!(crawler.clean_text(text), "Hello World This is a test");
    }

    #[test]
    fn test_truncate_text() {
        let crawler = Crawler::new(3, 10);

        let text = "Hello World This is a test";
        assert_eq!(crawler.truncate_text(text, 15), "Hello World");
        assert_eq!(crawler.truncate_text(text, 100), text);
    }
}
