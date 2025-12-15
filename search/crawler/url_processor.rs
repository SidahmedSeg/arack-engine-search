use anyhow::Result;
use std::collections::HashSet;
use tracing::debug;
use url::Url;

/// URL processor for normalization and canonicalization
#[derive(Clone, Debug)]
pub struct UrlProcessor {
    /// Query parameters to remove (tracking params)
    remove_params: HashSet<String>,
    /// Whether to remove fragments (#section)
    remove_fragments: bool,
    /// Whether to normalize to lowercase
    lowercase: bool,
    /// Whether to add trailing slash to paths
    trailing_slash: bool,
    /// Whether to sort query parameters
    sort_query_params: bool,
}

impl Default for UrlProcessor {
    fn default() -> Self {
        let mut remove_params = HashSet::new();

        // Common tracking parameters
        remove_params.insert("utm_source".to_string());
        remove_params.insert("utm_medium".to_string());
        remove_params.insert("utm_campaign".to_string());
        remove_params.insert("utm_term".to_string());
        remove_params.insert("utm_content".to_string());
        remove_params.insert("fbclid".to_string());
        remove_params.insert("gclid".to_string());
        remove_params.insert("msclkid".to_string());
        remove_params.insert("mc_cid".to_string());
        remove_params.insert("mc_eid".to_string());

        // Session IDs
        remove_params.insert("sessionid".to_string());
        remove_params.insert("session_id".to_string());
        remove_params.insert("phpsessid".to_string());
        remove_params.insert("jsessionid".to_string());

        Self {
            remove_params,
            remove_fragments: true,
            lowercase: true,
            trailing_slash: false,
            sort_query_params: true,
        }
    }
}

impl UrlProcessor {
    /// Create a new URL processor with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a query parameter to be removed
    pub fn add_remove_param(&mut self, param: String) {
        self.remove_params.insert(param.to_lowercase());
    }

    /// Set whether to remove fragments
    pub fn set_remove_fragments(&mut self, remove: bool) {
        self.remove_fragments = remove;
    }

    /// Set whether to lowercase URLs
    pub fn set_lowercase(&mut self, lowercase: bool) {
        self.lowercase = lowercase;
    }

    /// Set whether to add trailing slashes
    pub fn set_trailing_slash(&mut self, trailing_slash: bool) {
        self.trailing_slash = trailing_slash;
    }

    /// Set whether to sort query parameters
    pub fn set_sort_query_params(&mut self, sort: bool) {
        self.sort_query_params = sort;
    }

    /// Normalize a URL according to configured rules
    pub fn normalize(&self, url: &str) -> Result<String> {
        let mut parsed = Url::parse(url)?;

        // Remove fragments if configured
        if self.remove_fragments {
            parsed.set_fragment(None);
        }

        // Filter query parameters
        if !self.remove_params.is_empty() {
            let filtered_params: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(key, _)| {
                    let key_lower = key.to_lowercase();
                    !self.remove_params.contains(&key_lower)
                })
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            // Clear existing query
            parsed.set_query(None);

            // Add filtered params
            if !filtered_params.is_empty() {
                let query = if self.sort_query_params {
                    let mut sorted = filtered_params.clone();
                    sorted.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                    sorted
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&")
                } else {
                    filtered_params
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&")
                };
                parsed.set_query(Some(&query));
            }
        }

        // Remove default ports by reconstructing URL
        let should_remove_port = if let Some(port) = parsed.port() {
            (parsed.scheme() == "http" && port == 80)
                || (parsed.scheme() == "https" && port == 443)
        } else {
            false
        };

        if should_remove_port {
            let scheme = parsed.scheme();
            let host = parsed.host_str().unwrap_or("");
            let path = parsed.path();
            let query = parsed.query().map(|q| format!("?{}", q)).unwrap_or_default();
            let fragment = parsed.fragment().map(|f| format!("#{}", f)).unwrap_or_default();

            let new_url = format!("{}://{}{}{}{}", scheme, host, path, query, fragment);
            parsed = Url::parse(&new_url)?;
        }

        // Handle trailing slash
        if self.trailing_slash {
            let path = parsed.path();
            if !path.ends_with('/') && !path.contains('.') {
                // Add trailing slash to directory paths (no file extension)
                parsed.set_path(&format!("{}/", path));
            }
        }

        let mut result = parsed.to_string();

        // Lowercase if configured
        if self.lowercase {
            // Only lowercase scheme and host, preserve path case
            if let Ok(parsed) = Url::parse(&result) {
                let scheme = parsed.scheme().to_lowercase();
                let host = parsed.host_str().unwrap_or("").to_lowercase();
                let port = parsed.port().map(|p| format!(":{}", p)).unwrap_or_default();
                let path = parsed.path();
                let query = parsed.query().map(|q| format!("?{}", q)).unwrap_or_default();

                result = format!("{}://{}{}{}{}", scheme, host, port, path, query);
            }
        }

        debug!("Normalized URL: {} -> {}", url, result);
        Ok(result)
    }

    /// Extract canonical URL from HTML
    pub fn extract_canonical(html: &str) -> Option<String> {
        // Simple regex-based extraction for canonical link
        let re = regex::Regex::new(r#"<link[^>]*rel=["']canonical["'][^>]*href=["']([^"']+)["'][^>]*>"#)
            .ok()?;

        if let Some(captures) = re.captures(html) {
            return Some(captures.get(1)?.as_str().to_string());
        }

        // Also try reverse order (href before rel)
        let re2 = regex::Regex::new(r#"<link[^>]*href=["']([^"']+)["'][^>]*rel=["']canonical["'][^>]*>"#)
            .ok()?;

        if let Some(captures) = re2.captures(html) {
            return Some(captures.get(1)?.as_str().to_string());
        }

        None
    }

    /// Check if URL matches a pattern (for deduplication)
    pub fn urls_match(&self, url1: &str, url2: &str) -> bool {
        match (self.normalize(url1), self.normalize(url2)) {
            (Ok(norm1), Ok(norm2)) => norm1 == norm2,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_tracking_params() {
        let processor = UrlProcessor::new();

        let url = "https://example.com/page?utm_source=google&utm_medium=cpc&id=123";
        let normalized = processor.normalize(url).unwrap();

        assert!(normalized.contains("id=123"));
        assert!(!normalized.contains("utm_source"));
        assert!(!normalized.contains("utm_medium"));
    }

    #[test]
    fn test_remove_fragments() {
        let processor = UrlProcessor::new();

        let url = "https://example.com/page#section";
        let normalized = processor.normalize(url).unwrap();

        assert_eq!(normalized, "https://example.com/page");
    }

    #[test]
    fn test_lowercase() {
        let processor = UrlProcessor::new();

        let url = "HTTPS://EXAMPLE.COM/Page";
        let normalized = processor.normalize(url).unwrap();

        assert!(normalized.starts_with("https://example.com"));
        // Path case is preserved
        assert!(normalized.contains("/Page"));
    }

    #[test]
    fn test_remove_default_ports() {
        let processor = UrlProcessor::new();

        let url1 = "http://example.com:80/page";
        let normalized1 = processor.normalize(url1).unwrap();
        assert_eq!(normalized1, "http://example.com/page");

        let url2 = "https://example.com:443/page";
        let normalized2 = processor.normalize(url2).unwrap();
        assert_eq!(normalized2, "https://example.com/page");

        // Non-default ports should be preserved
        let url3 = "http://example.com:8080/page";
        let normalized3 = processor.normalize(url3).unwrap();
        assert!(normalized3.contains(":8080"));
    }

    #[test]
    fn test_sort_query_params() {
        let processor = UrlProcessor::new();

        let url = "https://example.com/page?z=3&a=1&m=2";
        let normalized = processor.normalize(url).unwrap();

        // Parameters should be sorted alphabetically
        assert!(normalized.contains("a=1&m=2&z=3"));
    }

    #[test]
    fn test_trailing_slash() {
        let mut processor = UrlProcessor::new();
        processor.set_trailing_slash(true);

        // Directory path should get trailing slash
        let url1 = "https://example.com/page";
        let normalized1 = processor.normalize(url1).unwrap();
        assert!(normalized1.ends_with("/page/"));

        // File paths (with extension) should not get trailing slash
        let url2 = "https://example.com/page.html";
        let normalized2 = processor.normalize(url2).unwrap();
        assert!(normalized2.ends_with(".html"));
        assert!(!normalized2.ends_with(".html/"));
    }

    #[test]
    fn test_custom_remove_params() {
        let mut processor = UrlProcessor::new();
        processor.add_remove_param("custom_tracker".to_string());

        let url = "https://example.com/page?custom_tracker=abc&id=123";
        let normalized = processor.normalize(url).unwrap();

        assert!(normalized.contains("id=123"));
        assert!(!normalized.contains("custom_tracker"));
    }

    #[test]
    fn test_extract_canonical() {
        let html = r#"
            <html>
            <head>
                <link rel="canonical" href="https://example.com/canonical-page" />
            </head>
            </html>
        "#;

        let canonical = UrlProcessor::extract_canonical(html);
        assert_eq!(canonical, Some("https://example.com/canonical-page".to_string()));
    }

    #[test]
    fn test_extract_canonical_reverse_order() {
        let html = r#"
            <html>
            <head>
                <link href="https://example.com/canonical-page" rel="canonical" />
            </head>
            </html>
        "#;

        let canonical = UrlProcessor::extract_canonical(html);
        assert_eq!(canonical, Some("https://example.com/canonical-page".to_string()));
    }

    #[test]
    fn test_urls_match() {
        let processor = UrlProcessor::new();

        // These should be considered the same
        assert!(processor.urls_match(
            "https://example.com/page?utm_source=google",
            "https://example.com/page"
        ));

        assert!(processor.urls_match(
            "https://EXAMPLE.com/page",
            "https://example.com/page"
        ));

        assert!(processor.urls_match(
            "https://example.com/page#section",
            "https://example.com/page"
        ));

        // These should be different
        assert!(!processor.urls_match(
            "https://example.com/page1",
            "https://example.com/page2"
        ));
    }

    #[test]
    fn test_session_id_removal() {
        let processor = UrlProcessor::new();

        let url = "https://example.com/page?PHPSESSID=abc123&jsessionid=xyz789&id=100";
        let normalized = processor.normalize(url).unwrap();

        assert!(normalized.contains("id=100"));
        assert!(!normalized.contains("PHPSESSID"));
        assert!(!normalized.contains("jsessionid"));
    }

    #[test]
    fn test_complex_normalization() {
        let processor = UrlProcessor::new();

        let url = "HTTPS://EXAMPLE.COM:443/Page?utm_source=fb&z=3&a=1&PHPSESSID=test#section";
        let normalized = processor.normalize(url).unwrap();

        // Should be normalized to lowercase scheme/host
        assert!(normalized.starts_with("https://example.com"));
        // Should remove default port
        assert!(!normalized.contains(":443"));
        // Should remove tracking and session params
        assert!(!normalized.contains("utm_source"));
        assert!(!normalized.contains("PHPSESSID"));
        // Should keep and sort other params
        assert!(normalized.contains("a=1"));
        assert!(normalized.contains("z=3"));
        // Should remove fragment
        assert!(!normalized.contains("#section"));
    }
}
