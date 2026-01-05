// Account Service Client
// Custom SSO session validation via account.arack.io
// See CUSTOM_SSO_SYSTEM.md for full documentation
//
// Features:
// - Session validation via arack_session cookie
// - Bearer token support for client-side API calls
// - JWT self-validation using JWKS from account-service

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use tracing::{info, warn, debug};

use super::models::{UserSession, UserIdentity, IdentityTraits};

/// Response from account-service /api/session endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct AccountServiceSession {
    pub user_id: String,
    pub email: String,
    pub name: String,
    #[serde(default)]
    pub picture: Option<String>,
    pub access_token: String,
}

/// JWT Claims from account-service tokens
#[derive(Debug, Clone, Deserialize)]
pub struct JwtClaims {
    pub sub: String,           // User ID
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    pub iss: String,           // Issuer
    pub aud: serde_json::Value, // Audience (can be string or array)
    pub exp: i64,              // Expiration time
    pub iat: i64,              // Issued at
    #[serde(default)]
    pub jti: Option<String>,   // JWT ID
}

/// JWKS (JSON Web Key Set) response
#[derive(Debug, Clone, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
    pub kty: String,           // Key type (RSA)
    #[serde(default)]
    pub use_: Option<String>,  // Usage (sig)
    pub kid: String,           // Key ID
    pub alg: String,           // Algorithm (RS256)
    pub n: String,             // RSA modulus
    pub e: String,             // RSA exponent
}

/// Cached JWKS with expiration
#[derive(Clone)]
struct CachedJwks {
    jwks: JwksResponse,
    fetched_at: std::time::Instant,
}

/// Account Service client for session validation
#[derive(Clone)]
pub struct AccountServiceClient {
    client: Client,
    base_url: String,
    jwks_cache: Arc<RwLock<Option<CachedJwks>>>,
    jwks_cache_ttl: std::time::Duration,
}

impl AccountServiceClient {
    /// Create a new Account Service client
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            jwks_cache: Arc::new(RwLock::new(None)),
            jwks_cache_ttl: std::time::Duration::from_secs(3600), // Cache JWKS for 1 hour
        }
    }

    /// Fetch JWKS from account-service (with caching)
    async fn get_jwks(&self) -> Result<JwksResponse> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.jwks_cache_ttl {
                    debug!("[JWT] Using cached JWKS");
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Fetch fresh JWKS
        info!("[JWT] Fetching JWKS from {}", self.base_url);
        let response = self
            .client
            .get(format!("{}/.well-known/jwks.json", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch JWKS: {} - {}", status, body);
        }

        let jwks: JwksResponse = response.json().await?;
        info!("[JWT] Fetched JWKS with {} keys", jwks.keys.len());

        // Update cache
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some(CachedJwks {
                jwks: jwks.clone(),
                fetched_at: std::time::Instant::now(),
            });
        }

        Ok(jwks)
    }

    /// Get decoding key for a specific key ID
    async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey> {
        let jwks = self.get_jwks().await?;

        let jwk = jwks.keys.iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| anyhow::anyhow!("Key ID '{}' not found in JWKS", kid))?;

        if jwk.kty != "RSA" {
            anyhow::bail!("Unsupported key type: {}", jwk.kty);
        }

        // Create decoding key from RSA components (n, e)
        DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| anyhow::anyhow!("Failed to create decoding key: {}", e))
    }

    /// Validate session using account-service /api/session endpoint
    ///
    /// This endpoint checks if the arack_session cookie is valid and returns
    /// the session details as UserSession.
    pub async fn whoami(&self, cookie_header: &str) -> Result<UserSession> {
        // Extract session token from cookie header
        // Format: "arack_session=session_id_here; other_cookie=value"
        let session_id = cookie_header
            .split(';')
            .find_map(|cookie| {
                let cookie = cookie.trim();
                if cookie.starts_with("arack_session=") {
                    Some(cookie.trim_start_matches("arack_session="))
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No arack_session cookie found"))?;

        // Call account-service API with the session cookie
        let response = self
            .client
            .get(format!("{}/api/session", self.base_url))
            .header("Cookie", format!("arack_session={}", session_id))
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            anyhow::bail!("Session expired or invalid");
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Session validation failed: {} - {}", status, body);
        }

        let session: AccountServiceSession = response.json().await?;

        // Convert to UserSession format
        Ok(self.to_user_session(session))
    }

    /// Get the access token from session (for JMAP/Stalwart authentication)
    pub async fn get_access_token(&self, cookie_header: &str) -> Result<String> {
        let session_id = cookie_header
            .split(';')
            .find_map(|cookie| {
                let cookie = cookie.trim();
                if cookie.starts_with("arack_session=") {
                    Some(cookie.trim_start_matches("arack_session="))
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No arack_session cookie found"))?;

        let response = self
            .client
            .get(format!("{}/api/session", self.base_url))
            .header("Cookie", format!("arack_session={}", session_id))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get access token: {}", response.status());
        }

        let session: AccountServiceSession = response.json().await?;
        Ok(session.access_token)
    }

    /// Validate Bearer token using local JWT validation
    /// Phase 10: Validates JWT locally using JWKS from account-service
    /// This is used for client-side API calls that send Authorization: Bearer <token>
    pub async fn validate_bearer_token(&self, token: &str) -> Result<UserSession> {
        // Decode JWT header to get key ID
        let header = decode_header(token)
            .map_err(|e| anyhow::anyhow!("Invalid JWT header: {}", e))?;

        let kid = header.kid
            .ok_or_else(|| anyhow::anyhow!("JWT missing key ID (kid)"))?;

        debug!("[JWT] Validating token with kid: {}", kid);

        // Get the decoding key for this key ID
        let decoding_key = self.get_decoding_key(&kid).await?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["https://account.arack.io"]);
        // Don't validate audience for now (account-service might use different values)
        validation.validate_aud = false;

        // Decode and validate the token
        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)
            .map_err(|e| {
                warn!("[JWT] Token validation failed: {}", e);
                anyhow::anyhow!("Bearer token invalid or expired: {}", e)
            })?;

        let claims = token_data.claims;
        debug!("[JWT] Token validated for user: {} ({})", claims.sub, claims.email);

        // Convert claims to UserSession
        Ok(self.claims_to_user_session(claims, token.to_string()))
    }

    /// Convert AccountServiceSession to UserSession
    /// Phase 9: Now includes access_token for JMAP Bearer authentication
    fn to_user_session(&self, session: AccountServiceSession) -> UserSession {
        // Parse user_id as UUID, fallback to nil UUID if parse fails
        let user_uuid = Uuid::parse_str(&session.user_id)
            .unwrap_or_else(|_| Uuid::nil());

        // Split name into first_name and last_name
        let name_parts: Vec<&str> = session.name.splitn(2, ' ').collect();
        let first_name = name_parts.first().unwrap_or(&"").to_string();
        let last_name = name_parts.get(1).unwrap_or(&"").to_string();

        UserSession {
            id: session.user_id.clone(),
            active: true, // If we got here, session is active
            identity: UserIdentity {
                id: user_uuid,
                schema_id: "default".to_string(),
                traits: IdentityTraits {
                    email: session.email,
                    first_name,
                    last_name,
                },
                verifiable_addresses: vec![],
            },
            // These are approximations since account-service doesn't return them
            authenticated_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(30),
            // Phase 9: Include access token for JMAP Bearer auth
            access_token: Some(session.access_token),
        }
    }

    /// Convert JWT claims to UserSession
    fn claims_to_user_session(&self, claims: JwtClaims, access_token: String) -> UserSession {
        // Parse user_id as UUID, fallback to nil UUID if parse fails
        let user_uuid = Uuid::parse_str(&claims.sub)
            .unwrap_or_else(|_| Uuid::nil());

        // Split name into first_name and last_name
        let name = claims.name.unwrap_or_default();
        let name_parts: Vec<&str> = name.splitn(2, ' ').collect();
        let first_name = name_parts.first().unwrap_or(&"").to_string();
        let last_name = name_parts.get(1).unwrap_or(&"").to_string();

        // Convert exp timestamp to DateTime
        let expires_at = DateTime::from_timestamp(claims.exp, 0)
            .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1));

        UserSession {
            id: claims.sub.clone(),
            active: true,
            identity: UserIdentity {
                id: user_uuid,
                schema_id: "default".to_string(),
                traits: IdentityTraits {
                    email: claims.email,
                    first_name,
                    last_name,
                },
                verifiable_addresses: vec![],
            },
            authenticated_at: DateTime::from_timestamp(claims.iat, 0)
                .unwrap_or_else(Utc::now),
            expires_at,
            access_token: Some(access_token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_service_client_creation() {
        let client = AccountServiceClient::new("http://account-service:3002".to_string());
        assert_eq!(client.base_url, "http://account-service:3002");
    }
}
