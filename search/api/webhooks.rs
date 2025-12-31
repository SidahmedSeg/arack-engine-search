//! Webhook handlers for authentication providers
//!
//! These handlers are called by authentication providers after user lifecycle events.
//! Supports both Ory Kratos (legacy) and Zitadel (current).

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use super::AppState;
use crate::zitadel::{ZitadelActionsV2Event, ZitadelWebhookPayload};

/// Kratos webhook payload for user creation
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

/// Handle user creation webhook from Kratos
///
/// This endpoint is called by Kratos after a new user registers.
/// It creates a user_preferences record in the auth schema.
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

/// Handle user creation webhook from Zitadel
///
/// This endpoint is called by Zitadel Actions after a new user registers.
/// The Actions were configured in Phase 2 to call this endpoint.
pub async fn handle_zitadel_user_created(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ZitadelWebhookPayload>,
) -> impl IntoResponse {
    info!(
        "Received Zitadel user-created webhook for user: {} ({})",
        payload.user_id, payload.email
    );

    // Create user_preferences with Zitadel user ID
    match create_zitadel_user_preferences(&state, &payload).await {
        Ok(_) => {
            info!(
                "User preferences created successfully for Zitadel user {}",
                payload.user_id
            );
            (
                StatusCode::OK,
                Json(WebhookResponse {
                    success: true,
                    message: Some(format!("User preferences created for {}", payload.user_id)),
                }),
            )
        }
        Err(e) => {
            error!(
                "Failed to create user preferences for Zitadel user {}: {}",
                payload.user_id, e
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

/// Create user preferences record for Zitadel user
async fn create_zitadel_user_preferences(
    state: &AppState,
    payload: &ZitadelWebhookPayload,
) -> anyhow::Result<()> {
    // Store Zitadel user ID in dedicated column (migration 011)
    // Check if user already exists first (partial unique index doesn't support ON CONFLICT)
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM user_preferences WHERE zitadel_user_id = $1)"
    )
    .bind(&payload.user_id)
    .fetch_one(&state.db_pool)
    .await?;

    if !exists {
        sqlx::query(
            "INSERT INTO user_preferences \
             (zitadel_user_id, username, theme, results_per_page) \
             VALUES ($1, $2, 'light', 20)"
        )
        .bind(&payload.user_id)
        .bind(&payload.username)
        .execute(&state.db_pool)
        .await?;
    }

    Ok(())
}

/// Handle user creation webhook from Zitadel Actions V2
///
/// This endpoint is called by Zitadel Actions V2 after a new user is created.
/// It handles the event-based payload format from Actions V2.
///
/// Event type: user.human.added
pub async fn handle_zitadel_v2_user_created(
    State(state): State<Arc<AppState>>,
    Json(event): Json<ZitadelActionsV2Event>,
) -> impl IntoResponse {
    info!(
        "Received Zitadel Actions V2 event: {} for user: {} ({})",
        event.event_type, event.user_id, event.event_payload.email
    );

    // Verify this is the correct event type
    if event.event_type != "user.human.added" {
        error!(
            "Unexpected event type: {} (expected user.human.added)",
            event.event_type
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(WebhookResponse {
                success: false,
                message: Some(format!("Unexpected event type: {}", event.event_type)),
            }),
        );
    }

    // Create user_preferences with Zitadel user ID
    match create_zitadel_v2_user_preferences(&state, &event).await {
        Ok(_) => {
            info!(
                "User preferences created successfully for Zitadel V2 user {}",
                event.user_id
            );
            (
                StatusCode::OK,
                Json(WebhookResponse {
                    success: true,
                    message: Some(format!("User preferences created for {}", event.user_id)),
                }),
            )
        }
        Err(e) => {
            error!(
                "Failed to create user preferences for Zitadel V2 user {}: {}",
                event.user_id, e
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

/// Create user preferences record for Zitadel Actions V2 event
async fn create_zitadel_v2_user_preferences(
    state: &AppState,
    event: &ZitadelActionsV2Event,
) -> anyhow::Result<()> {
    // Check if user already exists first (partial unique index doesn't support ON CONFLICT)
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM user_preferences WHERE zitadel_user_id = $1)"
    )
    .bind(&event.user_id)
    .fetch_one(&state.db_pool)
    .await?;

    if !exists {
        // Extract username from event payload (prefer userName, fallback to email)
        let username = event
            .event_payload
            .user_name
            .as_ref()
            .unwrap_or(&event.event_payload.email);

        sqlx::query(
            "INSERT INTO user_preferences \
             (zitadel_user_id, username, theme, results_per_page) \
             VALUES ($1, $2, 'light', 20)"
        )
        .bind(&event.user_id)
        .bind(username)
        .execute(&state.db_pool)
        .await?;

        info!(
            "Created user_preferences for Zitadel V2 user {} (username: {})",
            event.user_id, username
        );
    } else {
        info!(
            "User preferences already exist for Zitadel V2 user {}, skipping",
            event.user_id
        );
    }

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
