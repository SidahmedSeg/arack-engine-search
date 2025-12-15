use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::auth::AuthSession;
use crate::types::ApiResponse;
use super::UserRole;

/// Middleware to require admin role
pub async fn require_admin(
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> Response {
    // Check if user is authenticated
    let user = match auth_session.user {
        Some(user) => user,
        None => {
            let response = ApiResponse::error("Authentication required".to_string());
            return (StatusCode::UNAUTHORIZED, axum::Json(response)).into_response();
        }
    };

    // Check if user has admin role
    if user.role != UserRole::Admin {
        let response = ApiResponse::error("Admin access required".to_string());
        return (StatusCode::FORBIDDEN, axum::Json(response)).into_response();
    }

    // User is admin, continue
    next.run(request).await
}

/// Middleware to require authentication (any role)
pub async fn require_auth(
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> Response {
    // Check if user is authenticated
    if auth_session.user.is_none() {
        let response = ApiResponse::error("Authentication required".to_string());
        return (StatusCode::UNAUTHORIZED, axum::Json(response)).into_response();
    }

    // User is authenticated, continue
    next.run(request).await
}
