//! Zitadel Data Models
//!
//! Data structures for Zitadel OIDC integration

use serde::{Deserialize, Serialize};

/// Zitadel user info from /oidc/v1/userinfo endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZitadelUserInfo {
    pub sub: String, // Zitadel user ID
    pub email: String,
    pub email_verified: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
}

/// Webhook payload from Zitadel Actions V1
/// This matches the payload sent by the Actions we configured in Phase 2
#[derive(Debug, Deserialize, Serialize)]
pub struct ZitadelWebhookPayload {
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Zitadel Actions V2 Event Payload
/// This is the complete event payload sent by Actions V2 on user.human.added events
#[derive(Debug, Deserialize, Serialize)]
pub struct ZitadelActionsV2Event {
    #[serde(rename = "aggregateID")]
    pub aggregate_id: String,
    #[serde(rename = "aggregateType")]
    pub aggregate_type: String,
    #[serde(rename = "resourceOwner")]
    pub resource_owner: String,
    #[serde(rename = "instanceID")]
    pub instance_id: String,
    pub version: String,
    pub sequence: u64,
    pub event_type: String, // "user.human.added"
    pub created_at: String,
    #[serde(rename = "userID")]
    pub user_id: String,
    pub event_payload: ZitadelUserEventPayload,
}

/// Event payload nested within the V2 event
#[derive(Debug, Deserialize, Serialize)]
pub struct ZitadelUserEventPayload {
    pub email: String,
    #[serde(rename = "firstName", default)]
    pub first_name: Option<String>,
    #[serde(rename = "lastName", default)]
    pub last_name: Option<String>,
    #[serde(rename = "userName", default)]
    pub user_name: Option<String>,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "preferredLanguage", default)]
    pub preferred_language: Option<String>,
    #[serde(default)]
    pub gender: Option<String>, // "GENDER_UNSPECIFIED", "GENDER_FEMALE", "GENDER_MALE", etc.
}

/// OAuth state for CSRF protection and PKCE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub csrf_token: String,
    pub pkce_verifier: String,
    #[serde(default)]
    pub redirect_after: Option<String>,
}

/// JWT Claims from Zitadel ID token
#[derive(Debug, Serialize, Deserialize)]
pub struct ZitadelClaims {
    pub sub: String,          // User ID
    pub exp: usize,           // Expiration time
    pub iat: usize,           // Issued at
    pub iss: String,          // Issuer
    pub aud: Vec<String>,     // Audience
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub email_verified: Option<bool>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
}
