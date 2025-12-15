use anyhow::Result;
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, info, warn};
use url::Url;

use super::retry::RetryPolicy;

/// Robots.txt rules for a domain
#[derive(Debug, Clone)]
pub struct RobotsRules {
    /// Disallowed paths for our user agent
    disallowed_paths: Vec<String>,
    /// Crawl delay in seconds
    crawl_delay: Option<f64>,
    /// Sitemap URLs
    sitemaps: Vec<String>,
}

impl RobotsRules {
    /// Create a permissive robots rules (allow all)
    fn permissive() -> Self {
        Self {
            disallowed_paths: Vec::new(),
            crawl_delay: None,
            sitemaps: Vec::new(),
        }
    }

    /// Parse robots.txt content
    fn parse(content: &str, user_agent: &str) -> Self {
        let mut disallowed_paths = Vec::new();
        let mut crawl_delay = None;
        let mut sitemaps = Vec::new();
        let mut current_user_agent = String::new();
        let mut matches_our_agent = false;
        let mut has_specific_match = false;

        // First pass: check if there's a specific user-agent match
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                if key == "user-agent" {
                    let agent = value.to_lowercase();
                    if agent == user_agent.to_lowercase() || user_agent.to_lowercase().contains(&agent) && agent != "*" {
                        has_specific_match = true;
                        break;
                    }
                }
            }
        }

        // Second pass: collect rules
        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key-value pairs
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                match key.as_str() {
                    "user-agent" => {
                        current_user_agent = value.to_lowercase();
                        // If we have a specific match, only match that specific agent
                        // If no specific match, use wildcard
                        if has_specific_match {
                            matches_our_agent = current_user_agent == user_agent.to_lowercase()
                                || (user_agent.to_lowercase().contains(&current_user_agent) && current_user_agent != "*");
                        } else {
                            matches_our_agent = current_user_agent == "*";
                        }
                    }
                    "disallow" if matches_our_agent => {
                        if !value.is_empty() {
                            disallowed_paths.push(value.to_string());
                        }
                    }
                    "crawl-delay" if matches_our_agent && crawl_delay.is_none() => {
                        if let Ok(delay) = value.parse::<f64>() {
                            crawl_delay = Some(delay);
                        }
                    }
                    "sitemap" => {
                        sitemaps.push(value.to_string());
                    }
                    _ => {}
                }
            }
        }

        Self {
            disallowed_paths,
            crawl_delay,
            sitemaps,
        }
    }

    /// Check if a path is allowed
    fn is_path_allowed(&self, path: &str) -> bool {
        // If no disallow rules, allow everything
        if self.disallowed_paths.is_empty() {
            return true;
        }

        // Check if path matches any disallow rule
        for disallowed in &self.disallowed_paths {
            if path.starts_with(disallowed) {
                return false;
            }
        }

        true
    }
}

/// Robots.txt manager for respectful crawling
#[derive(Clone)]
pub struct RobotsManager {
    /// Cache of robots.txt rules per domain
    robots_cache: Arc<DashMap<String, RobotsRules>>,
    /// User agent string to use when checking robots.txt
    user_agent: String,
    /// Retry policy for fetching robots.txt
    retry_policy: RetryPolicy,
}

impl RobotsManager {
    /// Create a new robots manager
    pub fn new(user_agent: String) -> Self {
        info!("Initializing RobotsManager with User-Agent: {}", user_agent);

        Self {
            robots_cache: Arc::new(DashMap::new()),
            user_agent,
            retry_policy: RetryPolicy::new(),
        }
    }

    /// Check if a URL is allowed to be crawled according to robots.txt
    pub async fn is_allowed(&self, url: &str) -> Result<bool> {
        let parsed_url = Url::parse(url)?;
        let domain = parsed_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("No host in URL"))?
            .to_string();

        // Get or fetch robots.txt for this domain
        let rules = self.get_or_fetch_robots(&domain).await?;

        // Check if URL is allowed
        let path = parsed_url.path();
        let allowed = rules.is_path_allowed(path);

        if !allowed {
            debug!("URL blocked by robots.txt: {}", url);
        }

        Ok(allowed)
    }

    /// Get crawl delay for a domain from robots.txt
    pub async fn get_crawl_delay(&self, domain: &str) -> Option<f64> {
        match self.get_or_fetch_robots(domain).await {
            Ok(rules) => {
                if let Some(delay) = rules.crawl_delay {
                    debug!("Crawl delay for {}: {}s", domain, delay);
                }
                rules.crawl_delay
            }
            Err(e) => {
                warn!("Failed to get crawl delay for {}: {}", domain, e);
                None
            }
        }
    }

    /// Get sitemap URLs from robots.txt
    pub async fn get_sitemaps(&self, domain: &str) -> Vec<String> {
        match self.get_or_fetch_robots(domain).await {
            Ok(rules) => {
                if !rules.sitemaps.is_empty() {
                    info!("Found {} sitemap(s) for {}", rules.sitemaps.len(), domain);
                }
                rules.sitemaps
            }
            Err(e) => {
                warn!("Failed to get sitemaps for {}: {}", domain, e);
                Vec::new()
            }
        }
    }

    /// Get or fetch robots.txt for a domain
    async fn get_or_fetch_robots(&self, domain: &str) -> Result<RobotsRules> {
        // Check cache first
        if let Some(rules) = self.robots_cache.get(domain) {
            return Ok(rules.clone());
        }

        // Fetch robots.txt
        let robots_url = format!("https://{}/robots.txt", domain);
        debug!("Fetching robots.txt from {}", robots_url);

        match self.fetch_robots_txt(&robots_url).await {
            Ok(content) => {
                let rules = RobotsRules::parse(&content, &self.user_agent);
                self.robots_cache.insert(domain.to_string(), rules.clone());
                info!("Cached robots.txt for {}", domain);
                Ok(rules)
            }
            Err(e) => {
                warn!("Failed to fetch robots.txt for {}: {}", domain, e);
                // On error, create permissive rules (allow all)
                let permissive = RobotsRules::permissive();
                self.robots_cache
                    .insert(domain.to_string(), permissive.clone());
                Ok(permissive)
            }
        }
    }

    /// Fetch robots.txt content from URL with retry logic
    async fn fetch_robots_txt(&self, url: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let user_agent = self.user_agent.clone();
        let url_owned = url.to_string();

        // Use retry policy for HTTP request
        let response = self.retry_policy.execute_http(&url_owned, || {
            let client = client.clone();
            let url = url_owned.clone();
            let user_agent = user_agent.clone();
            async move {
                client
                    .get(&url)
                    .header("User-Agent", user_agent)
                    .send()
                    .await
            }
        }).await?;

        if response.status().is_success() {
            let content = response.text().await?;
            Ok(content)
        } else {
            // If robots.txt doesn't exist (404), allow crawling
            Err(anyhow::anyhow!(
                "robots.txt not found (status: {})",
                response.status()
            ))
        }
    }

    /// Clear robots.txt cache for a domain
    pub fn clear_cache(&self, domain: &str) {
        self.robots_cache.remove(domain);
        info!("Cleared robots.txt cache for {}", domain);
    }

    /// Clear all robots.txt cache
    pub fn clear_all_cache(&self) {
        self.robots_cache.clear();
        info!("Cleared all robots.txt cache");
    }

    /// Get statistics
    pub fn stats(&self) -> RobotsStats {
        RobotsStats {
            cached_domains: self.robots_cache.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RobotsStats {
    pub cached_domains: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robots_manager_creation() {
        let manager = RobotsManager::new("TestBot/1.0".to_string());
        assert_eq!(manager.stats().cached_domains, 0);
    }

    #[tokio::test]
    async fn test_is_allowed_no_robots() {
        let manager = RobotsManager::new("TestBot/1.0".to_string());

        // This should fail to fetch robots.txt and default to allow
        // We can't really test this without a real server
        // So just verify the function signature works
        let _ = manager.is_allowed("https://example.com/test").await;
    }

    #[test]
    fn test_parse_robots_txt() {
        let robots_content = r#"
User-agent: *
Disallow: /admin/
Disallow: /private/
Allow: /public/

Crawl-delay: 1

Sitemap: https://example.com/sitemap.xml
        "#;

        let rules = RobotsRules::parse(robots_content, "TestBot");

        // Test allowed paths
        assert!(rules.is_path_allowed("/public/page"));
        assert!(rules.is_path_allowed("/index.html"));

        // Test disallowed paths
        assert!(!rules.is_path_allowed("/admin/panel"));
        assert!(!rules.is_path_allowed("/private/data"));
    }

    #[test]
    fn test_crawl_delay_extraction() {
        let robots_content = r#"
User-agent: *
Crawl-delay: 2.5
        "#;

        let rules = RobotsRules::parse(robots_content, "TestBot");

        assert_eq!(rules.crawl_delay, Some(2.5));
    }

    #[test]
    fn test_sitemap_extraction() {
        let robots_content = r#"
Sitemap: https://example.com/sitemap.xml
Sitemap: https://example.com/sitemap2.xml
        "#;

        let rules = RobotsRules::parse(robots_content, "TestBot");

        assert_eq!(rules.sitemaps.len(), 2);
        assert!(rules.sitemaps.contains(&"https://example.com/sitemap.xml".to_string()));
        assert!(rules.sitemaps.contains(&"https://example.com/sitemap2.xml".to_string()));
    }

    #[test]
    fn test_user_agent_matching() {
        let robots_content = r#"
User-agent: GoogleBot
Disallow: /private/

User-agent: *
Disallow: /admin/
        "#;

        // Test with GoogleBot
        let rules_google = RobotsRules::parse(robots_content, "GoogleBot");
        assert!(!rules_google.is_path_allowed("/private/test"));
        assert!(rules_google.is_path_allowed("/admin/panel")); // Only private is disallowed for GoogleBot

        // Test with generic bot
        let rules_generic = RobotsRules::parse(robots_content, "TestBot");
        assert!(rules_generic.is_path_allowed("/private/test")); // Not disallowed for us
        assert!(!rules_generic.is_path_allowed("/admin/panel")); // Admin is disallowed for *
    }

    #[test]
    fn test_permissive_rules() {
        let rules = RobotsRules::permissive();

        assert!(rules.is_path_allowed("/anything"));
        assert!(rules.is_path_allowed("/admin/"));
        assert_eq!(rules.crawl_delay, None);
        assert!(rules.sitemaps.is_empty());
    }
}
