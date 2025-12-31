//! Stalwart Admin API Client (Phase 2)
//!
//! Client for managing accounts via Stalwart's REST Management API.
//! This is separate from the JMAP client which handles email operations.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Stalwart Admin API Client
#[derive(Clone)]
pub struct StalwartAdminClient {
    base_url: String,
    client: Client,
    admin_user: String,
    admin_password: String,
}

/// Principal type for Stalwart accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrincipalType {
    Individual,
    Group,
    Domain,
    List,
    Resource,
    Location,
    Tenant,
    #[serde(rename = "superuser")]
    SuperUser,
    #[serde(rename = "api-key")]
    ApiKey,
}

/// Request to create a new principal (account)
#[derive(Debug, Clone, Serialize)]
pub struct CreatePrincipalRequest {
    #[serde(rename = "type")]
    pub principal_type: PrincipalType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emails: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,
    #[serde(rename = "memberOf", skip_serializing_if = "Option::is_none")]
    pub member_of: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

/// Response from creating a principal
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePrincipalResponse {
    pub data: Option<u64>, // Principal ID
}

/// Error response from Stalwart API
#[derive(Debug, Clone, Deserialize)]
pub struct StalwartErrorResponse {
    pub error: Option<String>,
    pub details: Option<String>,
}

impl StalwartAdminClient {
    /// Create a new Stalwart admin client with production-grade HTTP configuration
    pub fn new(base_url: &str, admin_user: &str, admin_password: &str) -> Self {
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
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            admin_user: admin_user.to_string(),
            admin_password: admin_password.to_string(),
        }
    }

    /// Create a new email account (principal of type individual)
    pub async fn create_account(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
        quota_bytes: Option<u64>,
    ) -> Result<u64> {
        let url = format!("{}/api/principal", self.base_url);

        // Extract username from email (part before @)
        let username = email.split('@').next().unwrap_or(email);

        let request = CreatePrincipalRequest {
            principal_type: PrincipalType::Individual,
            name: username.to_string(),
            description: display_name.map(|s| s.to_string()),
            secrets: Some(vec![password.to_string()]),
            emails: Some(vec![email.to_string()]),
            quota: quota_bytes,
            member_of: None,
            roles: Some(vec!["user".to_string()]), // Default role for email users
        };

        debug!("Creating Stalwart account for {} at URL: {}", email, url);

        let response = match self
            .client
            .post(&url)
            .basic_auth(&self.admin_user, Some(&self.admin_password))
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                warn!("HTTP request failed for create_account ({}): {:?}", email, e);
                warn!("Error kind: {}", e.to_string());
                if e.is_timeout() {
                    warn!("Request timed out");
                }
                if e.is_connect() {
                    warn!("Connection error: {:?}", e.source());
                }
                if e.is_builder() {
                    warn!("Request builder error (possibly malformed)");
                }
                return Err(e).context(format!("Failed to send create account request to {} for {}", url, email));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Create account request failed with status {}: {}",
                status,
                error_text
            );
        }

        let create_response: CreatePrincipalResponse = response
            .json()
            .await
            .context("Failed to parse create account response")?;

        let principal_id = create_response
            .data
            .context("No principal ID in create response")?;

        info!(
            "Created Stalwart account for {} with ID {}",
            email, principal_id
        );

        Ok(principal_id)
    }

    /// Check if an account exists by email
    pub async fn account_exists(&self, email: &str) -> Result<bool> {
        // Extract username from email
        let username = email.split('@').next().unwrap_or(email);
        let url = format!("{}/api/principal/{}", self.base_url, username);

        debug!("Checking if account exists: {}", email);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.admin_user, Some(&self.admin_password))
            .send()
            .await
            .context("Failed to check account existence")?;

        Ok(response.status().is_success())
    }

    /// Delete an account by username
    pub async fn delete_account(&self, username: &str) -> Result<()> {
        let url = format!("{}/api/principal/{}", self.base_url, username);

        debug!("Deleting Stalwart account: {}", username);

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.admin_user, Some(&self.admin_password))
            .send()
            .await
            .context("Failed to delete Stalwart account")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Delete account request failed with status {}: {}",
                status,
                error_text
            );
        }

        info!("Deleted Stalwart account: {}", username);

        Ok(())
    }

    /// Update account password
    pub async fn update_password(&self, username: &str, new_password: &str) -> Result<()> {
        let url = format!("{}/api/principal/{}", self.base_url, username);

        debug!("Updating password for: {}", username);

        let response = self
            .client
            .patch(&url)
            .basic_auth(&self.admin_user, Some(&self.admin_password))
            .json(&serde_json::json!({
                "secrets": [new_password]
            }))
            .send()
            .await
            .context("Failed to update password")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Update password request failed with status {}: {}",
                status,
                error_text
            );
        }

        info!("Updated password for: {}", username);

        Ok(())
    }

    /// Create a domain in Stalwart
    pub async fn create_domain(&self, domain: &str) -> Result<u64> {
        let url = format!("{}/api/principal", self.base_url);

        let request = CreatePrincipalRequest {
            principal_type: PrincipalType::Domain,
            name: domain.to_string(),
            description: Some(format!("Email domain: {}", domain)),
            secrets: None,
            emails: None,
            quota: None,
            member_of: None,
            roles: None,
        };

        debug!("Creating Stalwart domain: {} at URL: {}", domain, url);

        let response = match self
            .client
            .post(&url)
            .basic_auth(&self.admin_user, Some(&self.admin_password))
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                warn!("HTTP request failed for create_domain: {:?}", e);
                warn!("Error kind: {}", e.to_string());
                if e.is_timeout() {
                    warn!("Request timed out");
                }
                if e.is_connect() {
                    warn!("Connection error: {:?}", e.source());
                }
                return Err(e).context(format!("Failed to send create domain request to {}", url));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Domain might already exist, which is OK
            if status.as_u16() == 409 {
                warn!("Domain {} already exists", domain);
                return Ok(0);
            }

            anyhow::bail!(
                "Create domain request failed with status {}: {}",
                status,
                error_text
            );
        }

        let create_response: CreatePrincipalResponse = response
            .json()
            .await
            .context("Failed to parse create domain response")?;

        let principal_id = create_response.data.unwrap_or(0);

        info!("Created Stalwart domain {} with ID {}", domain, principal_id);

        Ok(principal_id)
    }

    /// Health check - verify connection to Stalwart
    pub async fn health_check(&self) -> Result<bool> {
        // Try to access the principal list endpoint with basic auth
        let url = format!("{}/api/principal", self.base_url);

        debug!("Checking Stalwart health at {}", url);

        match self
            .client
            .get(&url)
            .basic_auth(&self.admin_user, Some(&self.admin_password))
            .send()
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("Stalwart health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = StalwartAdminClient::new(
            "http://localhost:8080",
            "admin",
            "password",
        );
        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(client.admin_user, "admin");
    }

    #[test]
    fn test_create_principal_request_serialization() {
        let request = CreatePrincipalRequest {
            principal_type: PrincipalType::Individual,
            name: "testuser".to_string(),
            description: Some("Test User".to_string()),
            secrets: Some(vec!["password123".to_string()]),
            emails: Some(vec!["testuser@arack.com".to_string()]),
            quota: Some(5_368_709_120), // 5GB
            member_of: None,
            roles: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"individual\""));
        assert!(json.contains("\"name\":\"testuser\""));
        assert!(json.contains("\"emails\":[\"testuser@arack.com\"]"));
    }
}
