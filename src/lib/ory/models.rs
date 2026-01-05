// User Session & Identity Models
// This module defines all types for authentication and user features
// Used by the custom SSO system (account-service)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ===== USER SESSION & IDENTITY MODELS =====

/// User Session - validated session from custom SSO (account-service)
/// Includes access_token for JMAP Bearer authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub active: bool,
    pub identity: UserIdentity,
    pub authenticated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// SSO access token for downstream services (JMAP, etc.)
    #[serde(default)]
    pub access_token: Option<String>,
}

/// User Identity from SSO provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub id: Uuid,
    pub schema_id: String,
    pub traits: IdentityTraits,
    #[serde(default)]
    pub verifiable_addresses: Vec<VerifiableAddress>,
}

/// Identity traits from schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityTraits {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

/// Verifiable address (email verification status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableAddress {
    pub id: Uuid,
    pub value: String,
    pub verified: bool,
    pub via: String, // "email"
    pub status: String,
}

// ===== USER PREFERENCES MODELS =====

/// User preferences model (database)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreferences {
    pub id: Uuid,
    pub kratos_identity_id: Uuid,
    pub theme: String,
    pub results_per_page: i32,
    pub analytics_opt_out: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update preferences request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdatePreferencesRequest {
    #[validate(length(min = 1))]
    pub theme: Option<String>,
    #[validate(range(min = 5, max = 100))]
    pub results_per_page: Option<i32>,
    pub analytics_opt_out: Option<bool>,
}

// ===== SAVED SEARCHES MODELS =====

/// Saved search model (database)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SavedSearch {
    pub id: Uuid,
    pub kratos_identity_id: Uuid,
    pub name: String,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<sqlx::types::JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create saved search request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateSavedSearchRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1))]
    pub query: String,
    pub filters: Option<serde_json::Value>,
}

// ===== SEARCH HISTORY MODELS =====

/// Search history model (database)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchHistory {
    pub id: Uuid,
    pub kratos_identity_id: Uuid,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<sqlx::types::JsonValue>,
    pub result_count: Option<i32>,
    pub clicked_url: Option<String>,
    pub clicked_position: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Track search request
#[derive(Debug, Clone, Deserialize)]
pub struct TrackSearchRequest {
    pub query: String,
    pub filters: Option<serde_json::Value>,
    pub result_count: i32,
}

/// Track click request
#[derive(Debug, Clone, Deserialize)]
pub struct TrackClickRequest {
    pub search_history_id: Uuid,
    pub clicked_url: String,
    pub clicked_position: i32,
}
