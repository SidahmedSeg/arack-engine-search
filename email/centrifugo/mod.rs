//! Centrifugo Client (Phase 3)
//!
//! Client for real-time push notifications via Centrifugo pub/sub server.

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info};

/// Centrifugo client for real-time messaging
#[derive(Clone)]
pub struct CentrifugoClient {
    api_url: String,
    api_key: String,
    client: Client,
}

impl CentrifugoClient {
    /// Create a new Centrifugo client with production-grade HTTP configuration
    pub fn new(api_url: &str, api_key: &str) -> Self {
        // Build a properly configured HTTP client for Docker networking
        let client = Client::builder()
            // Set User-Agent (required by many APIs, good practice)
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            // Connection pooling settings
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            // TCP keepalive for long-lived connections
            .tcp_keepalive(Duration::from_secs(60))
            // Timeouts to prevent hanging
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            // Enable HTTP/1.1 connection reuse
            .http1_title_case_headers()
            .build()
            .expect("Failed to build HTTP client");

        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            client,
        }
    }

    /// Publish a message to a channel
    pub async fn publish(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        let url = format!("{}/api/publish", self.api_url);

        debug!("Publishing to Centrifugo channel: {}", channel);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("apikey {}", self.api_key))
            .json(&json!({
                "channel": channel,
                "data": data
            }))
            .send()
            .await
            .context("Failed to publish to Centrifugo")?;

        if !response.status().is_success() {
            anyhow::bail!("Centrifugo publish failed: {}", response.status());
        }

        debug!("Successfully published to channel: {}", channel);

        Ok(())
    }

    /// Notify user of new email arrival
    pub async fn notify_new_email(&self, user_id: &str, email: &NewEmailNotification) -> Result<()> {
        let channel = format!("email:user:{}", user_id);

        self.publish(&channel, json!({
            "type": "new_email",
            "email_id": email.email_id,
            "from": email.from,
            "subject": email.subject,
            "preview": email.preview,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })).await
    }

    /// Notify user of email update (read/unread, moved, deleted)
    pub async fn notify_email_updated(
        &self,
        user_id: &str,
        email_id: &str,
        update_type: EmailUpdateType,
    ) -> Result<()> {
        let channel = format!("email:user:{}", user_id);

        self.publish(&channel, json!({
            "type": "email_updated",
            "email_id": email_id,
            "update_type": update_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })).await
    }

    /// Notify user of mailbox changes (new folder, folder renamed, etc.)
    pub async fn notify_mailbox_updated(
        &self,
        user_id: &str,
        mailbox_id: &str,
        action: &str,
    ) -> Result<()> {
        let channel = format!("email:user:{}", user_id);

        self.publish(&channel, json!({
            "type": "mailbox_updated",
            "mailbox_id": mailbox_id,
            "action": action,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })).await
    }

    /// Generate JWT token for WebSocket authentication
    pub fn generate_connection_token(&self, user_id: &str) -> Result<String> {
        // TODO: Implement JWT generation with HMAC secret
        // For now, return a placeholder
        Ok(format!("token_for_{}", user_id))
    }
}

/// New email notification payload
#[derive(Debug, Clone)]
pub struct NewEmailNotification {
    pub email_id: String,
    pub from: String,
    pub subject: String,
    pub preview: String,
}

/// Email update type
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailUpdateType {
    Read,
    Unread,
    Moved,
    Deleted,
    Starred,
    Unstarred,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centrifugo_client_creation() {
        let client = CentrifugoClient::new("http://localhost:8000", "test-api-key");
        assert_eq!(client.api_url, "http://localhost:8000");
        assert_eq!(client.api_key, "test-api-key");
    }

    #[test]
    fn test_generate_connection_token() {
        let client = CentrifugoClient::new("http://localhost:8000", "test-api-key");
        let token = client.generate_connection_token("user123").unwrap();
        assert!(token.contains("user123"));
    }
}
