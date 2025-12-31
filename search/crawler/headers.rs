use std::collections::HashMap;
use tracing::info;

/// HTTP Header manager for professional web crawling
#[derive(Clone, Debug)]
pub struct HeaderManager {
    /// User agent string (bot name/version)
    user_agent: String,
    /// Contact email for bot inquiries
    contact_email: Option<String>,
    /// Bot documentation URL
    bot_url: Option<String>,
    /// Accept-Language header value
    accept_language: String,
    /// Accept-Encoding header value (compression support)
    accept_encoding: String,
    /// Accept header value (content types)
    accept: String,
    /// Whether to track and send Referer headers
    send_referer: bool,
}

impl HeaderManager {
    /// Create a new header manager with default settings
    pub fn new(
        user_agent: String,
        contact_email: Option<String>,
        bot_url: Option<String>,
    ) -> Self {
        info!(
            "Initializing HeaderManager - User-Agent: {}, Contact: {:?}",
            user_agent, contact_email
        );

        Self {
            user_agent,
            contact_email,
            bot_url,
            accept_language: "en-US,en;q=0.9".to_string(),
            accept_encoding: "gzip, deflate, br".to_string(),
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
            send_referer: true,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        user_agent: String,
        contact_email: Option<String>,
        bot_url: Option<String>,
        accept_language: String,
    ) -> Self {
        Self {
            user_agent,
            contact_email,
            bot_url,
            accept_language,
            accept_encoding: "gzip, deflate, br".to_string(),
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
            send_referer: true,
        }
    }

    /// Generate the complete User-Agent string with contact info
    pub fn user_agent_string(&self) -> String {
        let mut ua = self.user_agent.clone();

        // Add bot URL and contact email if available
        let mut additions = Vec::new();

        if let Some(url) = &self.bot_url {
            additions.push(format!("+{}", url));
        }

        if let Some(email) = &self.contact_email {
            additions.push(email.clone());
        }

        if !additions.is_empty() {
            ua.push_str(&format!(" ({})", additions.join("; ")));
        }

        ua
    }

    /// Build headers as a HashMap for use with HTTP clients
    pub fn build_headers(&self, referer: Option<&str>) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // User-Agent (most important)
        headers.insert("User-Agent".to_string(), self.user_agent_string());

        // Accept headers for content negotiation
        headers.insert("Accept".to_string(), self.accept.clone());
        headers.insert("Accept-Language".to_string(), self.accept_language.clone());
        headers.insert("Accept-Encoding".to_string(), self.accept_encoding.clone());

        // Referer (optional, helps with navigation tracking)
        if self.send_referer {
            if let Some(ref_url) = referer {
                headers.insert("Referer".to_string(), ref_url.to_string());
            }
        }

        // Connection management
        headers.insert("Connection".to_string(), "keep-alive".to_string());

        // Cache control
        headers.insert("Cache-Control".to_string(), "max-age=0".to_string());

        headers
    }

    /// Get User-Agent without contact info (for logging)
    pub fn user_agent_base(&self) -> &str {
        &self.user_agent
    }

    /// Get contact email
    pub fn contact_email(&self) -> Option<&str> {
        self.contact_email.as_deref()
    }

    /// Get bot documentation URL
    pub fn bot_url(&self) -> Option<&str> {
        self.bot_url.as_deref()
    }

    /// Enable/disable Referer header sending
    pub fn set_send_referer(&mut self, enabled: bool) {
        self.send_referer = enabled;
    }

    /// Update Accept-Language
    pub fn set_accept_language(&mut self, language: String) {
        self.accept_language = language;
    }
}

impl Default for HeaderManager {
    fn default() -> Self {
        Self::new(
            "EngineSearchBot/1.0".to_string(),
            Some("bot@example.com".to_string()),
            Some("https://example.com/bot".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_with_contact() {
        let manager = HeaderManager::new(
            "TestBot/1.0".to_string(),
            Some("test@example.com".to_string()),
            Some("https://example.com/bot".to_string()),
        );

        let ua = manager.user_agent_string();
        assert!(ua.contains("TestBot/1.0"));
        assert!(ua.contains("+https://example.com/bot"));
        assert!(ua.contains("test@example.com"));
    }

    #[test]
    fn test_user_agent_without_contact() {
        let manager = HeaderManager::new("TestBot/1.0".to_string(), None, None);

        let ua = manager.user_agent_string();
        assert_eq!(ua, "TestBot/1.0");
    }

    #[test]
    fn test_headers_contain_essentials() {
        let manager = HeaderManager::default();
        let headers = manager.build_headers(None);

        assert!(headers.contains_key("User-Agent"));
        assert!(headers.contains_key("Accept"));
        assert!(headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("Accept-Encoding"));
    }

    #[test]
    fn test_headers_with_referer() {
        let manager = HeaderManager::default();
        let headers = manager.build_headers(Some("https://example.com/previous"));

        assert_eq!(
            headers.get("Referer"),
            Some(&"https://example.com/previous".to_string())
        );
    }

    #[test]
    fn test_headers_without_referer() {
        let mut manager = HeaderManager::default();
        manager.set_send_referer(false);
        let headers = manager.build_headers(Some("https://example.com/previous"));

        assert!(!headers.contains_key("Referer"));
    }

    #[test]
    fn test_accept_encoding_compression() {
        let manager = HeaderManager::default();
        let headers = manager.build_headers(None);

        let encoding = headers.get("Accept-Encoding").unwrap();
        assert!(encoding.contains("gzip"));
        assert!(encoding.contains("deflate"));
        assert!(encoding.contains("br"));
    }

    #[test]
    fn test_custom_accept_language() {
        let manager = HeaderManager::with_config(
            "TestBot/1.0".to_string(),
            None,
            None,
            "fr-FR,fr;q=0.9".to_string(),
        );

        let headers = manager.build_headers(None);
        assert_eq!(
            headers.get("Accept-Language"),
            Some(&"fr-FR,fr;q=0.9".to_string())
        );
    }
}
