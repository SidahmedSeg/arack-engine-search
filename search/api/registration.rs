//! Custom Registration API
//!
//! Provides a custom registration endpoint that:
//! 1. Creates a user in Zitadel (via Management API)
//! 2. Provisions an email account in Stalwart
//! 3. Stores user preferences in database
//!
//! This enables a custom 3-step registration flow while using Zitadel for SSO.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use validator::Validate;

use crate::zitadel::{
    CreateUserRequest, UserEmail, UserPassword, UserProfile, UserMetadata,
    encode_metadata, to_zitadel_gender,
};

use super::AppState;

/// Validate username format: lowercase letters, numbers, dots, and underscores only
fn validate_username(username: &str) -> Result<(), validator::ValidationError> {
    let re = regex::Regex::new(r"^[a-z0-9._]+$").unwrap();
    if re.is_match(username) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_username");
        err.message = Some("Username can only contain lowercase letters, numbers, dots, and underscores".into());
        Err(err)
    }
}

/// Registration request from frontend
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 1, max = 50, message = "First name must be 1-50 characters"))]
    pub first_name: String,

    #[validate(length(min = 1, max = 50, message = "Last name must be 1-50 characters"))]
    pub last_name: String,

    #[validate(length(min = 1, message = "Date of birth is required"))]
    pub date_of_birth: String,

    #[validate(length(min = 1, message = "Gender is required"))]
    pub gender: String,

    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    #[validate(custom(function = "validate_username"))]
    pub username: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// Registration response
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Handle user registration
///
/// This endpoint:
/// 1. Validates all input fields
/// 2. Checks username is still available
/// 3. Creates user in Zitadel with password
/// 4. Provisions email account via email service
/// 5. Creates user preferences in database
/// 6. Rolls back on any failure
pub async fn register_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Check if Zitadel Management API is configured
    let zitadel_mgmt = match &state.zitadel_mgmt {
        Some(client) => client.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(RegisterResponse {
                    success: false,
                    message: "Custom registration is not configured".to_string(),
                    email: None,
                    user_id: None,
                }),
            );
        }
    };
    // 1. Validate request
    if let Err(errors) = req.validate() {
        let error_messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| {
                    format!("{}: {}", field, e.message.as_ref().map(|m| m.to_string()).unwrap_or_default())
                })
            })
            .collect();

        return (
            StatusCode::BAD_REQUEST,
            Json(RegisterResponse {
                success: false,
                message: error_messages.join(", "),
                email: None,
                user_id: None,
            }),
        );
    }

    let email = format!("{}@arack.io", req.username);

    info!(
        "Processing registration for: {} {} ({})",
        req.first_name, req.last_name, email
    );

    // 2. Check username availability (double-check before creation)
    let username_available = check_username_available(&state.db_pool, &req.username).await;
    if !username_available {
        return (
            StatusCode::CONFLICT,
            Json(RegisterResponse {
                success: false,
                message: "Username is no longer available".to_string(),
                email: None,
                user_id: None,
            }),
        );
    }

    // 3. Check if user already exists in Zitadel
    match zitadel_mgmt.user_exists(&email).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(RegisterResponse {
                    success: false,
                    message: "An account with this email already exists".to_string(),
                    email: None,
                    user_id: None,
                }),
            );
        }
        Ok(false) => {} // Good, continue
        Err(e) => {
            error!("Failed to check user existence in Zitadel: {}", e);
            // Continue anyway, Zitadel will reject duplicates
        }
    }

    // 4. Create user in Zitadel
    let display_name = format!("{} {}", req.first_name, req.last_name);
    let zitadel_request = CreateUserRequest {
        user_name: req.username.clone(),
        profile: UserProfile {
            first_name: req.first_name.clone(),
            last_name: req.last_name.clone(),
            display_name: Some(display_name),
            gender: Some(to_zitadel_gender(&req.gender)),
            preferred_language: Some("en".to_string()),
        },
        email: UserEmail {
            email: email.clone(),
            is_verified: true, // We control @arack.io domain
        },
        password: UserPassword {
            password: req.password.clone(),
            change_required: false,
        },
        metadata: Some(vec![
            UserMetadata {
                key: "date_of_birth".to_string(),
                value: encode_metadata(&req.date_of_birth),
            },
            UserMetadata {
                key: "registration_source".to_string(),
                value: encode_metadata("custom_form"),
            },
        ]),
    };

    let zitadel_user = match zitadel_mgmt.create_user(zitadel_request).await {
        Ok(user) => {
            info!("User created in Zitadel: {}", user.user_id);
            user
        }
        Err(e) => {
            error!("Failed to create user in Zitadel: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    success: false,
                    message: format!("Failed to create account: {}", e),
                    email: None,
                    user_id: None,
                }),
            );
        }
    };

    // 5. Provision email account via email service webhook
    let provision_result = provision_email_account(
        &state.email_service_url,
        &zitadel_user.user_id,
        &email,
        &req.first_name,
        &req.last_name,
    ).await;

    if let Err(e) = provision_result {
        error!("Failed to provision email account: {}", e);

        // Rollback: Delete user from Zitadel
        warn!("Rolling back Zitadel user creation for: {}", zitadel_user.user_id);
        if let Err(rollback_err) = zitadel_mgmt.delete_user(&zitadel_user.user_id).await {
            error!("Failed to rollback Zitadel user: {}", rollback_err);
        }

        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RegisterResponse {
                success: false,
                message: format!("Failed to provision email account: {}", e),
                email: None,
                user_id: None,
            }),
        );
    }

    // 6. Create user preferences in database
    if let Err(e) = create_user_preferences(
        &state.db_pool,
        &zitadel_user.user_id,
        &req.username,
        &req.date_of_birth,
        &req.gender,
    ).await {
        warn!("Failed to create user preferences (non-fatal): {}", e);
        // Don't rollback - user can still use the service
    }

    info!(
        "Registration completed successfully for: {} ({})",
        email, zitadel_user.user_id
    );

    (
        StatusCode::CREATED,
        Json(RegisterResponse {
            success: true,
            message: "Account created successfully. You can now log in.".to_string(),
            email: Some(email),
            user_id: Some(zitadel_user.user_id),
        }),
    )
}

/// Check if username is available in our database
async fn check_username_available(db_pool: &sqlx::PgPool, username: &str) -> bool {
    let result = sqlx::query_scalar!(
        r#"SELECT check_username_available($1) as "available!""#,
        username
    )
    .fetch_one(db_pool)
    .await;

    match result {
        Ok(available) => available,
        Err(e) => {
            error!("Failed to check username availability: {}", e);
            false // Assume not available on error
        }
    }
}

/// Provision email account by calling email service webhook
async fn provision_email_account(
    email_service_url: &str,
    zitadel_user_id: &str,
    email: &str,
    first_name: &str,
    last_name: &str,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    // Build payload matching KratosWebhookPayload structure
    let payload = serde_json::json!({
        "identity": {
            "id": zitadel_user_id,
            "traits": {
                "email": email,
                "first_name": first_name,
                "last_name": last_name
            },
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339()
        }
    });

    let url = format!("{}/internal/mail/provision", email_service_url);

    info!("Calling email provisioning webhook: {}", url);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Email provisioning failed: HTTP {} - {}", status, body);
    }

    info!("Email account provisioned successfully for: {}", email);
    Ok(())
}

/// Create user preferences in database
async fn create_user_preferences(
    db_pool: &sqlx::PgPool,
    zitadel_user_id: &str,
    username: &str,
    date_of_birth: &str,
    gender: &str,
) -> anyhow::Result<()> {
    // Parse date of birth
    let dob = chrono::NaiveDate::parse_from_str(date_of_birth, "%Y-%m-%d").ok();

    // Normalize gender - only allow 'male' or 'female' per database constraint
    let gender_value = match gender.to_lowercase().as_str() {
        "male" | "m" => Some("male"),
        "female" | "f" => Some("female"),
        _ => None, // Other values not allowed by constraint
    };

    // Use runtime query to insert with zitadel_user_id (TEXT column)
    // Zitadel user IDs are numeric strings, not UUIDs
    sqlx::query(
        r#"
        INSERT INTO user_preferences (zitadel_user_id, username, date_of_birth, gender)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (username) DO UPDATE SET
            zitadel_user_id = EXCLUDED.zitadel_user_id,
            date_of_birth = EXCLUDED.date_of_birth,
            gender = EXCLUDED.gender,
            updated_at = NOW()
        "#
    )
    .bind(zitadel_user_id)
    .bind(username)
    .bind(dob)
    .bind(gender_value)
    .execute(db_pool)
    .await?;

    info!("User preferences created for: {} (zitadel_id: {})", username, zitadel_user_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_validation() {
        let valid_req = RegisterRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            date_of_birth: "1990-01-15".to_string(),
            gender: "male".to_string(),
            username: "john.doe".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(valid_req.validate().is_ok());

        // Test short password
        let short_pass = RegisterRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            date_of_birth: "1990-01-15".to_string(),
            gender: "male".to_string(),
            username: "john.doe".to_string(),
            password: "short".to_string(),
        };
        assert!(short_pass.validate().is_err());

        // Test invalid username
        let invalid_username = RegisterRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            date_of_birth: "1990-01-15".to_string(),
            gender: "male".to_string(),
            username: "John.Doe".to_string(), // Contains uppercase
            password: "SecurePass123!".to_string(),
        };
        assert!(invalid_username.validate().is_err());
    }
}
