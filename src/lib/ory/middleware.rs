// Phase 8.6: Ory Authentication Middleware
// This module provides middleware functions for validating Ory sessions

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tracing::error;

use super::models::KratosSession;
use crate::search::api::AppState;
use crate::types::ApiResponse;

/// Extension for storing Kratos session in request
#[derive(Clone)]
pub struct OrySession(pub KratosSession);

/// Middleware to require Ory authentication
///
/// This middleware extracts the session cookie, validates it with Kratos,
/// and inserts the session into request extensions for handlers to use.
/// Returns 401 if authentication fails.
pub async fn require_ory_auth(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract session cookie
    let cookie_header = match headers.get("cookie") {
        Some(h) => h.to_str().unwrap_or(""),
        None => {
            let response = ApiResponse::error("Authentication required".to_string());
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    // Validate session with Kratos whoami endpoint
    match state.kratos.whoami(cookie_header).await {
        Ok(session) => {
            if session.active {
                // Insert session into request extensions for handlers to access
                request.extensions_mut().insert(OrySession(session));
                next.run(request).await
            } else {
                error!("Session is not active");
                let response = ApiResponse::error("Session expired".to_string());
                (StatusCode::UNAUTHORIZED, Json(response)).into_response()
            }
        }
        Err(e) => {
            error!("Session validation failed: {}", e);
            let response = ApiResponse::error("Invalid session".to_string());
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}

/// Optional auth middleware - doesn't fail if not authenticated
///
/// This middleware attempts to validate the session but continues
/// even if authentication fails. Useful for endpoints that work
/// differently for authenticated vs. unauthenticated users.
pub async fn optional_ory_auth(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and validate session
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            if let Ok(session) = state.kratos.whoami(cookie_str).await {
                if session.active {
                    request.extensions_mut().insert(OrySession(session));
                }
            }
        }
    }

    // Continue regardless of authentication status
    next.run(request).await
}
