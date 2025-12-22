//! OAuth 2.0 Token Manager for Email Service (Phase 8 - OIDC)
//!
//! This module handles OAuth 2.0 authentication flow with Ory Hydra:
//! - Authorization code flow initiation
//! - Token exchange (code → access/refresh tokens)
//! - Token storage in PostgreSQL
//! - Automatic token refresh
//! - Token retrieval for JMAP Bearer authentication

use anyhow::{Context, Result};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// OAuth token pair with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub scope: Option<String>,
}

/// PKCE verifier storage (temporary, for code exchange)
#[derive(Debug, Clone)]
pub struct PkceVerifierStore {
    pub verifier: String,
    pub csrf_token: String,
}

/// OAuth Token Manager
#[derive(Clone)]
pub struct OAuthTokenManager {
    oauth_client: BasicClient,
    db_pool: PgPool,
    redirect_uri: String,
}

impl OAuthTokenManager {
    /// Create a new OAuth token manager
    pub fn new(
        hydra_public_url: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        db_pool: PgPool,
    ) -> Result<Self> {
        // Build Hydra OAuth endpoints
        let auth_url = AuthUrl::new(format!("{}/oauth2/auth", hydra_public_url))
            .context("Invalid authorization URL")?;
        let token_url = TokenUrl::new(format!("{}/oauth2/token", hydra_public_url))
            .context("Invalid token URL")?;

        // Create OAuth2 client
        let oauth_client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .context("Invalid redirect URI")?,
        );

        Ok(Self {
            oauth_client,
            db_pool,
            redirect_uri: redirect_uri.to_string(),
        })
    }

    /// Generate OAuth authorization URL with PKCE
    /// Returns (auth_url, csrf_token, pkce_verifier)
    pub fn generate_auth_url(&self) -> Result<(String, String, String)> {
        // Generate PKCE challenge (more secure than just client secret)
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate authorization URL with PKCE
        let (auth_url, csrf_token) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("offline_access".to_string())) // For refresh tokens
            .set_pkce_challenge(pkce_challenge)
            .url();

        info!(
            "Generated OAuth authorization URL with PKCE, CSRF token: {}",
            csrf_token.secret()
        );

        Ok((
            auth_url.to_string(),
            csrf_token.secret().to_string(),
            pkce_verifier.secret().to_string(),
        ))
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
        kratos_identity_id: Uuid,
    ) -> Result<TokenPair> {
        info!(
            "Exchanging authorization code for tokens (Kratos ID: {})",
            kratos_identity_id
        );

        // Exchange code for tokens
        let token_response = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
            .request_async(async_http_client)
            .await
            .context("Failed to exchange authorization code for tokens")?;

        // Extract tokens
        let access_token = token_response.access_token().secret().clone();
        let refresh_token = token_response
            .refresh_token()
            .map(|t| t.secret().clone());
        let expires_in = token_response
            .expires_in()
            .unwrap_or(std::time::Duration::from_secs(3600));
        let scope = token_response
            .scopes()
            .map(|scopes| {
                scopes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            });

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in.as_secs() as i64);

        let token_pair = TokenPair {
            access_token: access_token.clone(),
            refresh_token: refresh_token.clone(),
            expires_at,
            scope,
        };

        // Store tokens in database
        self.store_tokens(kratos_identity_id, &token_pair).await?;

        info!(
            "Successfully exchanged code and stored tokens for Kratos ID: {}",
            kratos_identity_id
        );

        Ok(token_pair)
    }

    /// Get valid access token for a user (refreshing if needed)
    pub async fn get_access_token(&self, kratos_identity_id: Uuid) -> Result<String> {
        // Try to get existing token from database
        let token_record = sqlx::query!(
            r#"
            SELECT access_token, refresh_token, expires_at, scope
            FROM email.email_oauth_tokens
            WHERE kratos_identity_id = $1
            "#,
            kratos_identity_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to query OAuth tokens")?;

        let token_record = token_record
            .ok_or_else(|| anyhow::anyhow!("No OAuth token found for user. User must authorize email access first."))?;

        let expires_at = token_record.expires_at;
        let now = chrono::Utc::now();

        // Check if token is still valid (with 5 minute buffer)
        let buffer = chrono::Duration::minutes(5);
        if expires_at > now + buffer {
            // Token still valid
            debug!(
                "Using cached access token for Kratos ID: {} (expires in {} seconds)",
                kratos_identity_id,
                (expires_at - now).num_seconds()
            );
            return Ok(token_record.access_token);
        }

        // Token expired or about to expire - refresh it
        info!(
            "Access token expired or expiring soon for Kratos ID: {}, refreshing...",
            kratos_identity_id
        );

        let refresh_token = token_record
            .refresh_token
            .ok_or_else(|| anyhow::anyhow!("No refresh token available. User must re-authorize email access."))?;

        self.refresh_access_token(kratos_identity_id, &refresh_token)
            .await
    }

    /// Refresh an expired access token using refresh token
    async fn refresh_access_token(
        &self,
        kratos_identity_id: Uuid,
        refresh_token_str: &str,
    ) -> Result<String> {
        info!(
            "Refreshing access token for Kratos ID: {}",
            kratos_identity_id
        );

        // Request new access token using refresh token
        let token_response = self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token_str.to_string()))
            .request_async(async_http_client)
            .await
            .context("Failed to refresh access token")?;

        // Extract new tokens
        let access_token = token_response.access_token().secret().clone();
        let new_refresh_token = token_response
            .refresh_token()
            .map(|t| t.secret().clone())
            .or_else(|| Some(refresh_token_str.to_string())); // Keep old refresh token if new one not provided
        let expires_in = token_response
            .expires_in()
            .unwrap_or(std::time::Duration::from_secs(3600));
        let scope = token_response
            .scopes()
            .map(|scopes| {
                scopes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            });

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in.as_secs() as i64);

        let token_pair = TokenPair {
            access_token: access_token.clone(),
            refresh_token: new_refresh_token,
            expires_at,
            scope,
        };

        // Update tokens in database
        self.store_tokens(kratos_identity_id, &token_pair).await?;

        info!(
            "Successfully refreshed access token for Kratos ID: {}",
            kratos_identity_id
        );

        Ok(access_token)
    }

    /// Store tokens in database (upsert)
    async fn store_tokens(&self, kratos_identity_id: Uuid, token_pair: &TokenPair) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO email.email_oauth_tokens
                (kratos_identity_id, access_token, refresh_token, expires_at, scope, updated_at)
            VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
            ON CONFLICT (kratos_identity_id)
            DO UPDATE SET
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                expires_at = EXCLUDED.expires_at,
                scope = EXCLUDED.scope,
                updated_at = CURRENT_TIMESTAMP
            "#,
            kratos_identity_id,
            token_pair.access_token,
            token_pair.refresh_token,
            token_pair.expires_at,
            token_pair.scope
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to store OAuth tokens")?;

        debug!(
            "Stored OAuth tokens for Kratos ID: {} (expires at: {})",
            kratos_identity_id, token_pair.expires_at
        );

        Ok(())
    }

    /// Check if user has authorized email access (with valid, non-expired token)
    pub async fn has_authorization(&self, kratos_identity_id: Uuid) -> Result<bool> {
        // Use the same query as get_access_token to leverage cached SQLx query
        let token = sqlx::query!(
            r#"
            SELECT access_token, refresh_token, expires_at, scope
            FROM email.email_oauth_tokens
            WHERE kratos_identity_id = $1
            "#,
            kratos_identity_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to check authorization status")?;

        // Check if token exists and is not expired
        match token {
            Some(t) => Ok(t.expires_at > chrono::Utc::now()),
            None => Ok(false),
        }
    }

    /// Revoke OAuth tokens for a user (logout)
    pub async fn revoke_tokens(&self, kratos_identity_id: Uuid) -> Result<()> {
        info!("Revoking OAuth tokens for Kratos ID: {}", kratos_identity_id);

        // Delete tokens from database
        sqlx::query!(
            r#"
            DELETE FROM email.email_oauth_tokens
            WHERE kratos_identity_id = $1
            "#,
            kratos_identity_id
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to revoke OAuth tokens")?;

        info!(
            "Successfully revoked OAuth tokens for Kratos ID: {}",
            kratos_identity_id
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_pair_serialization() {
        let token_pair = TokenPair {
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at: chrono::Utc::now(),
            scope: Some("openid email profile".to_string()),
        };

        let json = serde_json::to_string(&token_pair).unwrap();
        let deserialized: TokenPair = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.access_token, "test_access_token");
        assert_eq!(deserialized.refresh_token, Some("test_refresh_token".to_string()));
    }
}
