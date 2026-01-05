//! Webhook handlers for user lifecycle events
//!
//! These handlers are called by the account-service after user registration.
//! Note: Payload names include "Kratos" for historical reasons but work with custom SSO.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use super::AppState;

/// Webhook payload for user creation (called by account-service)
#[derive(Debug, Deserialize)]
pub struct KratosWebhookPayload {
    pub identity: KratosIdentity,
}

#[derive(Debug, Deserialize)]
pub struct KratosIdentity {
    pub id: Uuid,
    pub traits: KratosTraits,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct KratosTraits {
    pub email: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    // Phase 8: Simplified Registration - new fields
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub date_of_birth: String,
    #[serde(default)]
    pub gender: String,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Handle user creation webhook from account-service
///
/// This endpoint is called after a new user registers.
/// It creates a user_preferences record in the database.
pub async fn handle_user_created(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<KratosWebhookPayload>,
) -> impl IntoResponse {
    let identity_id = payload.identity.id;
    let email = &payload.identity.traits.email;

    info!(
        "Received user-created webhook for identity: {} ({})",
        identity_id, email
    );

    // Create user_preferences in auth schema
    match create_user_preferences(&state, identity_id, &payload.identity.traits).await {
        Ok(_) => {
            info!(
                "User preferences created successfully for {}",
                identity_id
            );
            (
                StatusCode::OK,
                Json(WebhookResponse {
                    success: true,
                    message: Some(format!("User preferences created for {}", identity_id)),
                }),
            )
        }
        Err(e) => {
            error!(
                "Failed to create user preferences for {}: {}",
                identity_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(WebhookResponse {
                    success: false,
                    message: Some(format!("Failed to create user preferences: {}", e)),
                }),
            )
        }
    }
}

/// Create user preferences record in auth schema
async fn create_user_preferences(
    state: &AppState,
    kratos_identity_id: Uuid,
    traits: &KratosTraits,
) -> anyhow::Result<()> {
    // Parse date of birth (YYYY-MM-DD format)
    let date_of_birth = if !traits.date_of_birth.is_empty() {
        Some(NaiveDate::parse_from_str(&traits.date_of_birth, "%Y-%m-%d")?)
    } else {
        None
    };

    // Note: In future migrations, we'll move user_preferences to auth schema
    // For now, it exists in the public schema
    sqlx::query!(
        r#"
        INSERT INTO user_preferences
            (kratos_identity_id, username, date_of_birth, gender, theme, results_per_page)
        VALUES ($1, $2, $3, $4, 'light', 20)
        ON CONFLICT (kratos_identity_id) DO NOTHING
        "#,
        kratos_identity_id,
        if traits.username.is_empty() { None } else { Some(&traits.username) },
        date_of_birth,
        if traits.gender.is_empty() { None } else { Some(&traits.gender) },
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_deserialization() {
        let payload_json = r#"{
            "identity": {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "traits": {
                    "email": "john.doe@arack.io",
                    "username": "john.doe",
                    "first_name": "John",
                    "last_name": "Doe",
                    "date_of_birth": "1990-01-15",
                    "gender": "male"
                },
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        }"#;

        let payload: KratosWebhookPayload = serde_json::from_str(payload_json).unwrap();
        assert_eq!(payload.identity.traits.email, "john.doe@arack.io");
        assert_eq!(payload.identity.traits.username, "john.doe");
        assert_eq!(payload.identity.traits.first_name, "John");
        assert_eq!(payload.identity.traits.date_of_birth, "1990-01-15");
        assert_eq!(payload.identity.traits.gender, "male");
    }
}
