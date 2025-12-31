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
    ///
    /// For API flows, the session token is stored in a cookie named `ory_kratos_session`,
    /// but Kratos expects it to be sent via X-Session-Token header.
    pub async fn whoami(&self, cookie_header: &str) -> Result<KratosSession> {
        // Extract session token from cookie header
        // Format: "ory_kratos_session=ory_st_...; other_cookie=value"
        let session_token = cookie_header
            .split(';')
            .find_map(|cookie| {
                let cookie = cookie.trim();
                if cookie.starts_with("ory_kratos_session=") {
                    Some(cookie.trim_start_matches("ory_kratos_session="))
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No ory_kratos_session cookie found"))?;

        let response = self
            .client
            .get(format!("{}/sessions/whoami", self.public_url))
            .header("X-Session-Token", session_token)
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

    /// Exchange session token for session cookies
    ///
    /// Calls whoami with X-Session-Token header to get Set-Cookie headers
    /// that can be forwarded to the browser
    pub async fn exchange_token_for_cookies(&self, session_token: &str) -> Result<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/sessions/whoami", self.public_url))
            .header("X-Session-Token", session_token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Token exchange failed: {}", response.status());
        }

        // Extract Set-Cookie headers
        let cookies: Vec<String> = response.headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        Ok(cookies)
    }

    /// Initialize registration flow
    ///
    /// Returns a registration flow that can be used to submit registration data.
    /// The flow contains UI nodes with CSRF tokens and other required fields.
    pub async fn init_registration_flow(&self) -> Result<serde_json::Value> {
        // Use API flow for backend proxy compatibility
        let url = format!("{}/self-service/registration/api", self.public_url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Registration flow init failed: {}", response.status());
        }

        let flow = response.json().await?;
        Ok(flow)
    }

    /// Initialize login flow
    ///
    /// Returns a login flow that can be used to submit login credentials.
    /// The flow contains UI nodes with CSRF tokens and other required fields.
    /// Uses browser flow to enable cookie-based sessions.
    pub async fn init_login_flow(&self) -> Result<serde_json::Value> {
        // Browser flows don't work well with backend proxies.
        // Use API flow but we'll handle cookies manually
        let url = format!("{}/self-service/login/api", self.public_url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Login flow init failed: {}", response.status());
        }

        let flow = response.json().await?;
        Ok(flow)
    }

    /// Initialize logout flow
    ///
    /// Returns a logout URL that can be used to complete the logout process.
    /// The cookie parameter should be the full Cookie header value from the request.
    pub async fn init_logout_flow(&self, cookie: Option<&str>) -> Result<String> {
        let url = format!("{}/self-service/logout/browser", self.public_url);

        let mut request = self.client.get(&url);

        if let Some(cookie_header) = cookie {
            request = request.header("Cookie", cookie_header);
        }

        let response = request
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Logout flow init failed: {}", response.status());
        }

        let logout_data: serde_json::Value = response.json().await?;

        let logout_url = logout_data["logout_url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No logout_url in response"))?
            .to_string();

        Ok(logout_url)
    }

    /// Submit registration data
    ///
    /// Submits registration form to Kratos. Returns session cookies in response.
    pub async fn submit_registration(
        &self,
        flow_id: &str,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
        username: &str,
        date_of_birth: &str,
        gender: &str,
    ) -> Result<(serde_json::Value, Vec<String>)> {
        let url = format!("{}/self-service/registration?flow={}", self.public_url, flow_id);

        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "method": "password",
                "traits": {
                    "email": email,
                    "first_name": first_name,
                    "last_name": last_name,
                    "username": username,
                    "date_of_birth": date_of_birth,
                    "gender": gender
                },
                "password": password
            }))
            .send()
            .await?;

        // Extract Set-Cookie headers as separate strings
        let cookies: Vec<String> = response.headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        if !response.status().is_success() {
            let error_body = response.text().await?;
            anyhow::bail!("Registration failed: {}", error_body);
        }

        let body = response.json().await?;
        Ok((body, cookies))
    }

    /// Submit login credentials
    ///
    /// Submits login form to Kratos. Returns session cookies in response.
    pub async fn submit_login(
        &self,
        flow_id: &str,
        identifier: &str,
        password: &str,
    ) -> Result<(serde_json::Value, Vec<String>)> {
        let url = format!("{}/self-service/login?flow={}", self.public_url, flow_id);

        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "method": "password",
                "identifier": identifier,
                "password": password
            }))
            .send()
            .await?;

        // Extract Set-Cookie headers as separate strings
        let cookies: Vec<String> = response.headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        if !response.status().is_success() {
            let error_body = response.text().await?;
            anyhow::bail!("Login failed: {}", error_body);
        }

        let body = response.json().await?;
        Ok((body, cookies))
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
