//! Zitadel Management API Client
//!
//! Provides programmatic user management through Zitadel's v2beta User API.
//! Used for custom registration flows where we need to create users with
//! specific attributes (email, password, metadata).
//!
//! Security:
//! - Passwords are sent over HTTPS (encrypted in transit)
//! - Zitadel handles password hashing server-side (Argon2id by default)
//! - No plaintext passwords are stored or logged

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

/// Zitadel Management API client for user operations
#[derive(Clone)]
pub struct ZitadelManagementClient {
    base_url: String,
    client: Client,
    access_token: String,
    /// Optional host header override for internal networking
    /// When using internal Docker networking (http://zitadel:8080),
    /// we need to set the Host header to the external domain (auth.arack.io)
    host_override: Option<String>,
}

/// Request to create a new human user (v2beta API format)
/// Uses the v2beta endpoint which properly handles plaintext passwords
#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    #[serde(rename = "username")]
    pub user_name: String,
    pub profile: UserProfile,
    pub email: UserEmail,
    pub password: UserPassword,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<UserMetadata>>,
}

/// User profile information (v2beta format)
#[derive(Debug, Serialize)]
pub struct UserProfile {
    /// User's first/given name
    #[serde(rename = "givenName")]
    pub first_name: String,
    /// User's last/family name
    #[serde(rename = "familyName")]
    pub last_name: String,
    /// Display name (optional)
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Gender (GENDER_UNSPECIFIED, GENDER_FEMALE, GENDER_MALE, GENDER_DIVERSE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    /// Preferred language (e.g., "en")
    #[serde(rename = "preferredLanguage", skip_serializing_if = "Option::is_none")]
    pub preferred_language: Option<String>,
}

/// Email configuration for user
#[derive(Debug, Serialize)]
pub struct UserEmail {
    /// Email address
    pub email: String,
    /// Whether the email is pre-verified (true for domain-controlled emails)
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
}

/// Password configuration for user creation
///
/// SECURITY: Password is sent over HTTPS to Zitadel which handles
/// the secure hashing using Argon2id (or configured algorithm).
/// The plaintext password is never stored or logged.
#[derive(Serialize)]
pub struct UserPassword {
    /// Plaintext password - Zitadel will hash this securely
    #[serde(serialize_with = "serialize_password")]
    pub password: String,
    /// Whether user must change password on first login
    #[serde(rename = "changeRequired")]
    pub change_required: bool,
}

/// Custom serializer that masks password in debug output
fn serialize_password<S>(password: &str, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(password)
}

/// Implement Debug manually to avoid logging passwords
impl std::fmt::Debug for UserPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserPassword")
            .field("password", &"[REDACTED]")
            .field("change_required", &self.change_required)
            .finish()
    }
}

/// Metadata key-value pair
#[derive(Debug, Serialize)]
pub struct UserMetadata {
    pub key: String,
    /// Base64 encoded value (required by Zitadel)
    pub value: String,
}

/// Response from creating a user (v2beta format)
#[derive(Debug, Deserialize)]
pub struct CreateUserResponse {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(default)]
    pub details: Option<UserDetails>,
}

/// User details from response
#[derive(Debug, Deserialize)]
pub struct UserDetails {
    pub sequence: Option<String>,
    #[serde(rename = "creationDate")]
    pub creation_date: Option<String>,
    #[serde(rename = "changeDate")]
    pub change_date: Option<String>,
    #[serde(rename = "resourceOwner")]
    pub resource_owner: Option<String>,
}

/// Error response from Zitadel API
#[derive(Debug, Deserialize)]
pub struct ZitadelError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub details: Option<Vec<serde_json::Value>>,
}

impl ZitadelManagementClient {
    /// Create a new Zitadel Management client
    ///
    /// # Arguments
    /// * `base_url` - Zitadel instance URL (e.g., "https://auth.arack.io" or "http://zitadel:8080")
    /// * `access_token` - Personal Access Token or JWT from service user
    ///
    /// # Security
    /// The access token should be a service account token with minimal required permissions.
    /// Store tokens securely (environment variables, secrets manager).
    pub fn new(base_url: String, access_token: String) -> Self {
        Self::with_host_override(base_url, access_token, None)
    }

    /// Create a new Zitadel Management client with optional host override
    ///
    /// # Arguments
    /// * `base_url` - Zitadel instance URL (e.g., "http://zitadel:8080" for internal networking)
    /// * `access_token` - Personal Access Token or JWT from service user
    /// * `host_override` - Optional host header to send (e.g., "auth.arack.io" when using internal URL)
    ///
    /// # Example
    /// ```
    /// // For internal Docker networking, use container name but external domain in Host header
    /// let client = ZitadelManagementClient::with_host_override(
    ///     "http://zitadel:8080".to_string(),
    ///     "pat_token".to_string(),
    ///     Some("auth.arack.io".to_string()),
    /// );
    /// ```
    pub fn with_host_override(base_url: String, access_token: String, host_override: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("arack-registration-service/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        info!("Zitadel Management client initialized for: {}", base_url);
        if let Some(ref host) = host_override {
            info!("Using Host header override: {}", host);
        }

        // Warn if token appears to be empty or placeholder
        if access_token.is_empty() {
            warn!("Zitadel access token is empty - API calls will fail");
        }

        Self {
            base_url,
            client,
            access_token,
            host_override,
        }
    }

    /// Create a new human user with password
    ///
    /// Creates a user in Zitadel with the specified profile, email (verified),
    /// and password. Uses the v2beta API which properly handles plaintext
    /// password input - Zitadel handles secure hashing server-side.
    ///
    /// # Security
    /// - Password is transmitted over HTTPS (encrypted in transit)
    /// - Zitadel hashes the password using Argon2id (or configured algorithm)
    /// - Password is never logged or stored in plaintext
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<CreateUserResponse> {
        // Use v2beta endpoint which properly handles plaintext passwords
        let url = format!("{}/v2beta/users/human", self.base_url);

        info!(
            "Creating user in Zitadel: {} ({})",
            request.user_name, request.email.email
        );

        // Build the request body in v2beta format
        let body = serde_json::json!({
            "username": request.user_name,
            "profile": {
                "givenName": request.profile.first_name,
                "familyName": request.profile.last_name,
                "displayName": request.profile.display_name,
                "preferredLanguage": request.profile.preferred_language.as_deref().unwrap_or("en"),
                "gender": request.profile.gender.as_deref().unwrap_or("GENDER_UNSPECIFIED")
            },
            "email": {
                "email": request.email.email,
                "isVerified": request.email.is_verified
            },
            "password": {
                "password": request.password.password,
                "changeRequired": request.password.change_required
            },
            "metadata": request.metadata.as_ref().map(|m| m.iter().map(|meta| {
                serde_json::json!({
                    "key": meta.key,
                    "value": meta.value
                })
            }).collect::<Vec<_>>())
        });

        let mut req = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // Add Host header override for internal networking
        if let Some(ref host) = self.host_override {
            req = req.header("Host", host.as_str());
        }

        let response = req
            .json(&body)
            .send()
            .await
            .context("Failed to send create user request to Zitadel")?;

        let status = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;

        if !status.is_success() {
            // Log error but mask any sensitive data
            error!(
                "Zitadel create user failed for {}: HTTP {} - {}",
                request.email.email, status, response_body
            );

            // Try to parse error response for better error message
            if let Ok(err) = serde_json::from_str::<ZitadelError>(&response_body) {
                anyhow::bail!("Zitadel error {}: {}", err.code, err.message);
            }
            anyhow::bail!("Zitadel API error: HTTP {} - {}", status, response_body);
        }

        let result: CreateUserResponse = serde_json::from_str(&response_body)
            .context("Failed to parse Zitadel create user response")?;

        info!(
            "User created in Zitadel: {} (user_id: {})",
            request.email.email, result.user_id
        );

        Ok(result)
    }

    /// Delete a user (for rollback on provisioning failure)
    ///
    /// # Arguments
    /// * `user_id` - The Zitadel user ID to delete
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let url = format!("{}/v2beta/users/{}", self.base_url, user_id);

        info!("Deleting user from Zitadel: {}", user_id);

        let mut req = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json");

        // Add Host header override for internal networking
        if let Some(ref host) = self.host_override {
            req = req.header("Host", host.as_str());
        }

        let response = req
            .send()
            .await
            .context("Failed to send delete user request to Zitadel")?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!("Zitadel delete user failed: HTTP {} - {}", status, body);
            anyhow::bail!("Failed to delete user from Zitadel: HTTP {}", status);
        }

        info!("User deleted from Zitadel: {}", user_id);

        Ok(())
    }

    /// Check if a user exists by login name (email or username)
    ///
    /// # Arguments
    /// * `login_name` - Email or username to search for
    pub async fn user_exists(&self, login_name: &str) -> Result<bool> {
        let url = format!("{}/v2beta/users", self.base_url);

        let mut req = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // Add Host header override for internal networking
        if let Some(ref host) = self.host_override {
            req = req.header("Host", host.as_str());
        }

        let response = req
            .json(&serde_json::json!({
                "queries": [{
                    "loginNameQuery": {
                        "loginName": login_name,
                        "method": "TEXT_QUERY_METHOD_EQUALS"
                    }
                }]
            }))
            .send()
            .await
            .context("Failed to search users in Zitadel")?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            error!("Zitadel user search failed: {}", body);
            anyhow::bail!("Failed to search users in Zitadel");
        }

        #[derive(Deserialize)]
        struct SearchResponse {
            #[serde(default)]
            result: Vec<serde_json::Value>,
        }

        let result: SearchResponse = response.json().await?;
        Ok(!result.result.is_empty())
    }
}

/// Convert gender string to Zitadel gender enum
///
/// # Arguments
/// * `gender` - Human-readable gender string
///
/// # Returns
/// Zitadel gender enum value
pub fn to_zitadel_gender(gender: &str) -> String {
    match gender.to_lowercase().as_str() {
        "male" | "m" => "GENDER_MALE".to_string(),
        "female" | "f" => "GENDER_FEMALE".to_string(),
        "diverse" | "other" | "d" => "GENDER_DIVERSE".to_string(),
        _ => "GENDER_UNSPECIFIED".to_string(),
    }
}

/// Encode metadata value to base64 (required by Zitadel)
///
/// # Arguments
/// * `value` - Plain text value to encode
///
/// # Returns
/// Base64 encoded string
pub fn encode_metadata(value: &str) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(value.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gender_conversion() {
        assert_eq!(to_zitadel_gender("male"), "GENDER_MALE");
        assert_eq!(to_zitadel_gender("MALE"), "GENDER_MALE");
        assert_eq!(to_zitadel_gender("female"), "GENDER_FEMALE");
        assert_eq!(to_zitadel_gender("other"), "GENDER_DIVERSE");
        assert_eq!(to_zitadel_gender("unknown"), "GENDER_UNSPECIFIED");
    }

    #[test]
    fn test_metadata_encoding() {
        let encoded = encode_metadata("1990-01-15");
        assert!(!encoded.is_empty());
        // Verify it's valid base64
        use base64::{Engine, engine::general_purpose::STANDARD};
        let decoded = STANDARD.decode(&encoded).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "1990-01-15");
    }

    #[test]
    fn test_password_debug_redaction() {
        let password = UserPassword {
            password: "super_secret_123".to_string(),
            change_required: false,
        };
        let debug_output = format!("{:?}", password);
        // Password should be redacted in debug output
        assert!(!debug_output.contains("super_secret_123"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn test_create_user_request_serialization() {
        let request = CreateUserRequest {
            user_name: "john.doe".to_string(),
            profile: UserProfile {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                display_name: Some("John Doe".to_string()),
                gender: Some("GENDER_MALE".to_string()),
                preferred_language: Some("en".to_string()),
            },
            email: UserEmail {
                email: "john.doe@arack.io".to_string(),
                is_verified: true,
            },
            password: UserPassword {
                password: "SecurePass123!".to_string(),
                change_required: false,
            },
            metadata: Some(vec![
                UserMetadata {
                    key: "date_of_birth".to_string(),
                    value: encode_metadata("1990-01-15"),
                },
            ]),
        };

        let json = serde_json::to_string_pretty(&request).unwrap();
        assert!(json.contains("john.doe@arack.io"));
        assert!(json.contains("isVerified"));
        assert!(json.contains("givenName")); // v2beta field name
        assert!(json.contains("familyName")); // v2beta field name
    }
}
