use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tracing::{error, info};
use validator::Validate;

use crate::types::ApiResponse;
use super::AppState;

static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();
static RESERVED_USERNAMES: OnceLock<Vec<&str>> = OnceLock::new();

fn get_username_regex() -> &'static Regex {
    USERNAME_REGEX.get_or_init(|| Regex::new(r"^[a-z0-9._]+$").unwrap())
}

fn get_reserved_usernames() -> &'static Vec<&'static str> {
    RESERVED_USERNAMES.get_or_init(|| vec![
        "admin", "administrator", "root", "system",
        "support", "help", "info", "contact",
        "noreply", "no-reply", "postmaster",
        "abuse", "security", "webmaster", "hostmaster",
        "mailer-daemon", "nobody", "www", "ftp",
    ])
}

#[derive(Deserialize, Validate)]
pub struct CheckUsernameQuery {
    #[validate(length(min = 3, max = 30), custom(function = "validate_username_format"))]
    username: String,
}

fn validate_username_format(username: &str) -> Result<(), validator::ValidationError> {
    let lower = username.to_lowercase();

    // Only lowercase letters, numbers, dots, underscores
    if !get_username_regex().is_match(&lower) {
        return Err(validator::ValidationError::new("invalid_format"));
    }

    // No consecutive dots
    if lower.contains("..") {
        return Err(validator::ValidationError::new("consecutive_dots"));
    }

    // No leading/trailing dots or underscores
    if lower.starts_with('.') || lower.ends_with('.')
        || lower.starts_with('_') || lower.ends_with('_')
    {
        return Err(validator::ValidationError::new("invalid_start_end"));
    }

    // Check reserved usernames
    if get_reserved_usernames().contains(&lower.as_str()) {
        return Err(validator::ValidationError::new("reserved"));
    }

    Ok(())
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    available: bool,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

pub async fn check_username_availability(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CheckUsernameQuery>,
) -> impl IntoResponse {
    if let Err(errors) = query.validate() {
        let response = ApiResponse::error(format!("Validation failed: {}", errors));
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    let username = query.username.to_lowercase();
    let email = format!("{}@arack.io", username);

    // Check cache first (5-minute TTL)
    let cached = sqlx::query!(
        r#"
        SELECT available FROM username_availability_cache
        WHERE LOWER(username) = $1
          AND checked_at > NOW() - INTERVAL '5 minutes'
        "#,
        username
    )
    .fetch_optional(&state.db_pool)
    .await;

    if let Ok(Some(cache)) = cached {
        let available = cache.available;
        let response = ApiResponse::success(CheckUsernameResponse {
            available,
            email,
            reason: if available {
                None
            } else {
                Some("Username already taken".to_string())
            },
        });
        return (StatusCode::OK, Json(response)).into_response();
    }

    // Check database using the PostgreSQL function
    let available = match sqlx::query!(
        "SELECT check_username_available($1) as available",
        username
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(result) => result.available.unwrap_or(false),
        Err(e) => {
            error!("Database error checking username: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Update cache (fire and forget)
    let username_clone = username.clone();
    let db_pool = state.db_pool.clone();
    tokio::spawn(async move {
        let _ = sqlx::query!(
            r#"
            INSERT INTO username_availability_cache (username, available)
            VALUES ($1, $2)
            ON CONFLICT (username) DO UPDATE
            SET available = $2, checked_at = NOW()
            "#,
            username_clone,
            available
        )
        .execute(&db_pool)
        .await;
    });

    info!("Username '{}' availability: {}", username, available);

    let response = ApiResponse::success(CheckUsernameResponse {
        available,
        email,
        reason: if available {
            None
        } else {
            Some("Username already taken".to_string())
        },
    });
    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Deserialize, Validate)]
pub struct SuggestUsernamesRequest {
    #[validate(length(min = 1, max = 100))]
    first_name: String,

    #[validate(length(min = 1, max = 100))]
    last_name: String,
}

#[derive(Serialize)]
pub struct SuggestUsernamesResponse {
    suggestions: Vec<UsernameSuggestion>,
}

#[derive(Serialize)]
pub struct UsernameSuggestion {
    username: String,
    email: String,
    available: bool,
}

pub async fn suggest_usernames(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SuggestUsernamesRequest>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        let response = ApiResponse::error(format!("Validation failed: {}", errors));
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    let first = normalize_for_username(&payload.first_name);
    let last = normalize_for_username(&payload.last_name);

    // Generate candidate usernames
    let candidates = vec![
        format!("{}.{}", first, last), // john.doe
        format!("{}{}", first, last),  // johndoe
    ];

    let mut suggestions = Vec::new();
    let mut available_count = 0;

    for username in &candidates {
        if username.len() < 3 || username.len() > 30 {
            continue;
        }
        if !get_username_regex().is_match(username) {
            continue;
        }

        let email = format!("{}@arack.io", username);

        let available = match sqlx::query!(
            "SELECT check_username_available($1) as available",
            username
        )
        .fetch_one(&state.db_pool)
        .await
        {
            Ok(result) => result.available.unwrap_or(false),
            Err(_) => false,
        };

        suggestions.push(UsernameSuggestion {
            username: username.clone(),
            email,
            available,
        });

        if available {
            available_count += 1;
            if available_count >= 2 {
                break;
            }
        }
    }

    // If we don't have 2 available, generate numbered variations
    if available_count < 2 {
        let base = format!("{}{}", first, last);
        for i in 1..=99 {
            let candidate = format!("{}{}", base, i);
            if candidate.len() > 30 {
                break;
            }

            let email = format!("{}@arack.io", candidate);

            let available = match sqlx::query!(
                "SELECT check_username_available($1) as available",
                candidate
            )
            .fetch_one(&state.db_pool)
            .await
            {
                Ok(result) => result.available.unwrap_or(false),
                Err(_) => false,
            };

            if available {
                suggestions.push(UsernameSuggestion {
                    username: candidate,
                    email,
                    available,
                });
                available_count += 1;

                if available_count >= 2 {
                    break;
                }
            }
        }
    }

    info!(
        "Generated {} username suggestions for {} {}",
        suggestions.len(),
        payload.first_name,
        payload.last_name
    );

    let response = ApiResponse::success(SuggestUsernamesResponse { suggestions });
    (StatusCode::OK, Json(response)).into_response()
}

/// Helper: Normalize name for username (remove accents, keep only lowercase ASCII letters)
fn normalize_for_username(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => Some('a'),
            'é' | 'è' | 'ê' | 'ë' => Some('e'),
            'í' | 'ì' | 'î' | 'ï' => Some('i'),
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => Some('o'),
            'ú' | 'ù' | 'û' | 'ü' => Some('u'),
            'ñ' => Some('n'),
            'ç' => Some('c'),
            _ if c.is_ascii_lowercase() => Some(c),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_for_username() {
        assert_eq!(normalize_for_username("José"), "jose");
        assert_eq!(normalize_for_username("François"), "francois");
        assert_eq!(normalize_for_username("María"), "maria");
        assert_eq!(normalize_for_username("O'Brien"), "obrien");
        assert_eq!(normalize_for_username("John-Doe"), "johndoe");
    }

    #[test]
    fn test_validate_username_format() {
        // Valid usernames
        assert!(validate_username_format("john.doe").is_ok());
        assert!(validate_username_format("john_doe").is_ok());
        assert!(validate_username_format("john123").is_ok());

        // Invalid: too short
        assert!(validate_username_format("ab").is_err());

        // Invalid: consecutive dots
        assert!(validate_username_format("john..doe").is_err());

        // Invalid: leading dot
        assert!(validate_username_format(".johndoe").is_err());

        // Invalid: trailing underscore
        assert!(validate_username_format("johndoe_").is_err());

        // Invalid: reserved
        assert!(validate_username_format("admin").is_err());
        assert!(validate_username_format("support").is_err());
    }
}
