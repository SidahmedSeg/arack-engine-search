use anyhow::Result;
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use tracing::{debug, warn};
use url::Url;

/// Content filtering manager for crawler
#[derive(Clone, Debug)]
pub struct ContentFilter {
    /// Allowed content types (e.g., "text/html", "application/xml")
    allowed_content_types: HashSet<String>,
    /// Maximum file size in bytes (0 = unlimited)
    max_file_size: usize,
    /// URL include patterns (if empty, all URLs allowed)
    url_include_patterns: Vec<Regex>,
    /// URL exclude patterns (takes precedence over include)
    url_exclude_patterns: Vec<Regex>,
    /// Domain whitelist (if empty, all domains allowed)
    domain_whitelist: HashSet<String>,
    /// Domain blacklist (takes precedence over whitelist)
    domain_blacklist: HashSet<String>,
}

impl Default for ContentFilter {
    fn default() -> Self {
        let mut allowed_types = HashSet::new();
        allowed_types.insert("text/html".to_string());
        allowed_types.insert("text/plain".to_string());
        allowed_types.insert("application/xhtml+xml".to_string());
        allowed_types.insert("application/xml".to_string());

        Self {
            allowed_content_types: allowed_types,
            max_file_size: 10 * 1024 * 1024, // 10 MB default
            url_include_patterns: Vec::new(),
            url_exclude_patterns: Vec::new(),
            domain_whitelist: HashSet::new(),
            domain_blacklist: HashSet::new(),
        }
    }
}

impl ContentFilter {
    /// Create a new content filter with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an allowed content type
    pub fn add_allowed_content_type(&mut self, content_type: String) {
        self.allowed_content_types.insert(content_type.to_lowercase());
    }

    /// Set maximum file size in bytes
    pub fn set_max_file_size(&mut self, size: usize) {
        self.max_file_size = size;
    }

    /// Add URL include pattern (regex)
    pub fn add_url_include_pattern(&mut self, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern)?;
        self.url_include_patterns.push(regex);
        Ok(())
    }

    /// Add URL exclude pattern (regex)
    pub fn add_url_exclude_pattern(&mut self, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern)?;
        self.url_exclude_patterns.push(regex);
        Ok(())
    }

    /// Add domain to whitelist
    pub fn add_domain_whitelist(&mut self, domain: String) {
        self.domain_whitelist.insert(domain.to_lowercase());
    }

    /// Add domain to blacklist
    pub fn add_domain_blacklist(&mut self, domain: String) {
        self.domain_blacklist.insert(domain.to_lowercase());
    }

    /// Check if a content type is allowed
    pub fn is_content_type_allowed(&self, content_type: &str) -> bool {
        // Extract base content type (remove charset, etc.)
        let base_type = content_type
            .split(';')
            .next()
            .unwrap_or(content_type)
            .trim()
            .to_lowercase();

        let allowed = self.allowed_content_types.contains(&base_type);

        if !allowed {
            debug!("Content type not allowed: {}", content_type);
        }

        allowed
    }

    /// Check if file size is within limit
    pub fn is_file_size_allowed(&self, size: usize) -> bool {
        if self.max_file_size == 0 {
            return true; // Unlimited
        }

        let allowed = size <= self.max_file_size;

        if !allowed {
            warn!(
                "File size {} bytes exceeds limit of {} bytes",
                size, self.max_file_size
            );
        }

        allowed
    }

    /// Check if a URL matches the filter criteria
    pub fn is_url_allowed(&self, url: &str) -> bool {
        let parsed_url = match Url::parse(url) {
            Ok(u) => u,
            Err(e) => {
                warn!("Invalid URL {}: {}", url, e);
                return false;
            }
        };

        // Check domain filters
        if let Some(domain) = parsed_url.host_str() {
            let domain_lower = domain.to_lowercase();

            // Blacklist takes precedence
            if !self.domain_blacklist.is_empty() && self.domain_blacklist.contains(&domain_lower) {
                debug!("Domain blacklisted: {}", domain);
                return false;
            }

            // If whitelist is not empty, domain must be in it
            if !self.domain_whitelist.is_empty() && !self.domain_whitelist.contains(&domain_lower) {
                debug!("Domain not in whitelist: {}", domain);
                return false;
            }
        }

        // Check URL exclude patterns (takes precedence)
        for pattern in &self.url_exclude_patterns {
            if pattern.is_match(url) {
                debug!("URL matches exclude pattern: {}", url);
                return false;
            }
        }

        // Check URL include patterns (if any are defined)
        if !self.url_include_patterns.is_empty() {
            let mut matches_include = false;
            for pattern in &self.url_include_patterns {
                if pattern.is_match(url) {
                    matches_include = true;
                    break;
                }
            }

            if !matches_include {
                debug!("URL doesn't match any include pattern: {}", url);
                return false;
            }
        }

        true
    }

    /// Get statistics about filter configuration
    pub fn stats(&self) -> FilterStats {
        FilterStats {
            allowed_content_types: self.allowed_content_types.len(),
            max_file_size: self.max_file_size,
            url_include_patterns: self.url_include_patterns.len(),
            url_exclude_patterns: self.url_exclude_patterns.len(),
            domain_whitelist_count: self.domain_whitelist.len(),
            domain_blacklist_count: self.domain_blacklist.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FilterStats {
    pub allowed_content_types: usize,
    pub max_file_size: usize,
    pub url_include_patterns: usize,
    pub url_exclude_patterns: usize,
    pub domain_whitelist_count: usize,
    pub domain_blacklist_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_content_types() {
        let filter = ContentFilter::default();

        assert!(filter.is_content_type_allowed("text/html"));
        assert!(filter.is_content_type_allowed("text/html; charset=utf-8"));
        assert!(filter.is_content_type_allowed("application/xml"));
        assert!(!filter.is_content_type_allowed("application/pdf"));
        assert!(!filter.is_content_type_allowed("image/jpeg"));
    }

    #[test]
    fn test_file_size_limit() {
        let mut filter = ContentFilter::new();
        filter.set_max_file_size(1024); // 1 KB

        assert!(filter.is_file_size_allowed(512));
        assert!(filter.is_file_size_allowed(1024));
        assert!(!filter.is_file_size_allowed(1025));
        assert!(!filter.is_file_size_allowed(2048));
    }

    #[test]
    fn test_url_include_patterns() {
        let mut filter = ContentFilter::new();
        filter.add_url_include_pattern(r"^https://example\.com/blog/.*").unwrap();

        assert!(filter.is_url_allowed("https://example.com/blog/post1"));
        assert!(filter.is_url_allowed("https://example.com/blog/category/tech"));
        assert!(!filter.is_url_allowed("https://example.com/about"));
        assert!(!filter.is_url_allowed("https://other.com/blog/post1"));
    }

    #[test]
    fn test_url_exclude_patterns() {
        let mut filter = ContentFilter::new();
        filter.add_url_exclude_pattern(r".*\.pdf$").unwrap();
        filter.add_url_exclude_pattern(r".*/admin/.*").unwrap();

        assert!(filter.is_url_allowed("https://example.com/page"));
        assert!(!filter.is_url_allowed("https://example.com/document.pdf"));
        assert!(!filter.is_url_allowed("https://example.com/admin/panel"));
        assert!(!filter.is_url_allowed("https://example.com/admin/users/list"));
    }

    #[test]
    fn test_url_patterns_precedence() {
        let mut filter = ContentFilter::new();
        filter.add_url_include_pattern(r"^https://example\.com/.*").unwrap();
        filter.add_url_exclude_pattern(r".*/private/.*").unwrap();

        // Include pattern matches, exclude doesn't
        assert!(filter.is_url_allowed("https://example.com/public/page"));

        // Include matches but exclude takes precedence
        assert!(!filter.is_url_allowed("https://example.com/private/data"));
    }

    #[test]
    fn test_domain_whitelist() {
        let mut filter = ContentFilter::new();
        filter.add_domain_whitelist("example.com".to_string());
        filter.add_domain_whitelist("trusted.org".to_string());

        assert!(filter.is_url_allowed("https://example.com/page"));
        assert!(filter.is_url_allowed("https://trusted.org/page"));
        assert!(!filter.is_url_allowed("https://other.com/page"));
    }

    #[test]
    fn test_domain_blacklist() {
        let mut filter = ContentFilter::new();
        filter.add_domain_blacklist("spam.com".to_string());
        filter.add_domain_blacklist("malware.net".to_string());

        assert!(filter.is_url_allowed("https://example.com/page"));
        assert!(!filter.is_url_allowed("https://spam.com/page"));
        assert!(!filter.is_url_allowed("https://malware.net/page"));
    }

    #[test]
    fn test_domain_blacklist_precedence() {
        let mut filter = ContentFilter::new();
        filter.add_domain_whitelist("example.com".to_string());
        filter.add_domain_blacklist("example.com".to_string()); // Blacklist takes precedence

        assert!(!filter.is_url_allowed("https://example.com/page"));
    }

    #[test]
    fn test_filter_stats() {
        let mut filter = ContentFilter::new();
        filter.add_url_include_pattern(r"^https://.*").unwrap();
        filter.add_url_exclude_pattern(r".*\.pdf$").unwrap();
        filter.add_domain_whitelist("example.com".to_string());
        filter.add_domain_blacklist("spam.com".to_string());

        let stats = filter.stats();
        assert_eq!(stats.allowed_content_types, 4); // Default types
        assert_eq!(stats.url_include_patterns, 1);
        assert_eq!(stats.url_exclude_patterns, 1);
        assert_eq!(stats.domain_whitelist_count, 1);
        assert_eq!(stats.domain_blacklist_count, 1);
    }

    #[test]
    fn test_unlimited_file_size() {
        let mut filter = ContentFilter::new();
        filter.set_max_file_size(0); // Unlimited

        assert!(filter.is_file_size_allowed(1024 * 1024 * 100)); // 100 MB
        assert!(filter.is_file_size_allowed(usize::MAX));
    }
}
