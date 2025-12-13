// Phase 8.6: Kratos API Client
// This module provides a client for interacting with Ory Kratos APIs

use anyhow::Result;
use reqwest::Client;
use super::models::{KratosSession, KratosIdentity};

/// Kratos API client
#[derive(Clone)]
pub struct KratosClient {
    client: Client,
    public_url: String,
    admin_url: String,
}

impl KratosClient {
    /// Create a new Kratos client
    pub fn new(public_url: String, admin_url: String) -> Self {
        Self {
            client: Client::new(),
            public_url,
            admin_url,
        }
    }

    /// Validate session using whoami endpoint
    ///
    /// This endpoint checks if a session cookie is valid and returns the session details
    /// including the identity information.
    pub async fn whoami(&self, cookie: &str) -> Result<KratosSession> {
        let response = self
            .client
            .get(format!("{}/sessions/whoami", self.public_url))
            .header("Cookie", cookie)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Whoami request failed: {}", response.status());
        }

        let session: KratosSession = response.json().await?;
        Ok(session)
    }

    /// Get identity by ID (admin API)
    ///
    /// This is an admin endpoint that can fetch any identity by ID.
    /// Useful for debugging or admin operations.
    pub async fn get_identity(&self, identity_id: &uuid::Uuid) -> Result<KratosIdentity> {
        let response = self
            .client
            .get(format!("{}/admin/identities/{}", self.admin_url, identity_id))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Get identity failed: {}", response.status());
        }

        let identity: KratosIdentity = response.json().await?;
        Ok(identity)
    }

    /// Check if Kratos is healthy (for startup checks)
    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/health/ready", self.admin_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kratos_client_creation() {
        let client = KratosClient::new(
            "http://127.0.0.1:4433".to_string(),
            "http://127.0.0.1:4434".to_string(),
        );
        assert_eq!(client.public_url, "http://127.0.0.1:4433");
        assert_eq!(client.admin_url, "http://127.0.0.1:4434");
    }
}
