//! Zitadel OAuth2/OIDC Client
//!
//! Handles OAuth2 authorization code flow with PKCE for Zitadel

use anyhow::Result;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, TokenResponse, TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use tracing::info;

use super::models::ZitadelUserInfo;

/// Zitadel OAuth2 client for handling authentication flows
#[derive(Clone)]
pub struct ZitadelClient {
    client: BasicClient,
    issuer_url: String,
}

impl ZitadelClient {
    /// Create a new Zitadel client
    ///
    /// # Arguments
    /// * `issuer_url` - Zitadel issuer URL (e.g., "https://auth.arack.io")
    /// * `client_id` - OAuth2 client ID from Zitadel
    /// * `redirect_uri` - OAuth2 redirect URI (must match Zitadel config)
    pub fn new(issuer_url: String, client_id: String, redirect_uri: String) -> Result<Self> {
        let auth_url = AuthUrl::new(format!("{}/oauth/v2/authorize", issuer_url))?;
        let token_url = TokenUrl::new(format!("{}/oauth/v2/token", issuer_url))?;

        let client = BasicClient::new(
            ClientId::new(client_id),
            None, // No client secret - using PKCE
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        info!("Zitadel client initialized with issuer: {}", issuer_url);

        Ok(Self {
            client,
            issuer_url,
        })
    }

    /// Generate authorization URL with PKCE challenge
    ///
    /// Returns: (authorization_url, csrf_token, pkce_verifier)
    /// Store the csrf_token and pkce_verifier in session for later validation
    pub fn authorize_url(&self) -> (String, CsrfToken, PkceCodeVerifier) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("openid".to_string()))
            .add_scope(oauth2::Scope::new("profile".to_string()))
            .add_scope(oauth2::Scope::new("email".to_string()))
            .add_scope(oauth2::Scope::new("offline_access".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        (auth_url.to_string(), csrf_token, pkce_verifier)
    }

    /// Exchange authorization code for access token
    ///
    /// # Arguments
    /// * `code` - Authorization code from OAuth callback
    /// * `pkce_verifier` - PKCE verifier from session (generated during authorize_url)
    ///
    /// Returns: Access token string
    pub async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<String> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;

        Ok(token_response.access_token().secret().to_string())
    }

    /// Fetch user info from Zitadel /oidc/v1/userinfo endpoint
    ///
    /// # Arguments
    /// * `access_token` - Access token from exchange_code
    ///
    /// Returns: User info containing sub, email, name, etc.
    pub async fn get_user_info(&self, access_token: &str) -> Result<ZitadelUserInfo> {
        let userinfo_url = format!("{}/oidc/v1/userinfo", self.issuer_url);

        let client = reqwest::Client::new();
        let response = client
            .get(&userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to fetch user info: HTTP {}",
                response.status()
            );
        }

        let user_info: ZitadelUserInfo = response.json().await?;
        Ok(user_info)
    }

    /// Get JWKS URI for JWT validation
    pub fn jwks_uri(&self) -> String {
        format!("{}/oauth/v2/keys", self.issuer_url)
    }

    /// Get issuer URL
    pub fn issuer(&self) -> &str {
        &self.issuer_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ZitadelClient::new(
            "https://auth.arack.io".to_string(),
            "test-client-id".to_string(),
            "https://api.arack.io/auth/callback".to_string(),
        );

        assert!(client.is_ok());
    }

    #[test]
    fn test_authorize_url_generation() {
        let client = ZitadelClient::new(
            "https://auth.arack.io".to_string(),
            "test-client-id".to_string(),
            "https://api.arack.io/auth/callback".to_string(),
        )
        .unwrap();

        let (auth_url, _csrf, _pkce) = client.authorize_url();

        assert!(auth_url.contains("https://auth.arack.io/oauth/v2/authorize"));
        assert!(auth_url.contains("client_id=test-client-id"));
        assert!(auth_url.contains("scope=openid"));
        assert!(auth_url.contains("code_challenge"));
    }
}
