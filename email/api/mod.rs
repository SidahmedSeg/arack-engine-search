//! Email Service API (Phase 3-5)
//!
//! Provides REST API endpoints for email functionality and AI features.

#[cfg(feature = "email")]
pub mod ai;

use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, Method, HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{CorsLayer, AllowOrigin};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{
    centrifugo::{CentrifugoClient, NewEmailNotification},
    jmap::{JmapAuth, JmapClient},
    oauth::OAuthTokenManager,
    provisioning::{self, KratosWebhookPayload, ProvisioningResponse},
    search::EmailSearchClient,
    stalwart::StalwartAdminClient,
    types::*,
};

use crate::ory::KratosClient;

#[cfg(feature = "email")]
use async_openai::{config::OpenAIConfig, Client};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: redis::Client,
    pub jmap_client: JmapClient,
    pub search_client: EmailSearchClient,
    pub centrifugo_client: CentrifugoClient,
    pub stalwart_admin_client: StalwartAdminClient,
    pub default_email_password: String,
    pub kratos_client: KratosClient,
    pub oauth_token_manager: OAuthTokenManager,
    #[cfg(feature = "email")]
    pub openai_client: Client<OpenAIConfig>,
}

/// Create the email service API router
#[cfg(feature = "email")]
pub fn create_router(
    db_pool: PgPool,
    redis_client: redis::Client,
    jmap_client: JmapClient,
    search_client: EmailSearchClient,
    centrifugo_client: CentrifugoClient,
    stalwart_admin_client: StalwartAdminClient,
    default_email_password: String,
    kratos_client: KratosClient,
    oauth_token_manager: OAuthTokenManager,
    openai_client: Client<OpenAIConfig>,
) -> Router {
    let state = Arc::new(AppState {
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
        stalwart_admin_client,
        default_email_password,
        kratos_client,
        oauth_token_manager,
        openai_client,
    });

    // Production CORS using AllowOrigin::mirror_request()
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::mirror_request())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600));

    Router::new()
        // Health check
        .route("/health", get(health_check))

        // Internal provisioning webhook
        .route("/internal/mail/provision", post(provision_webhook_handler))

        // Email account
        .route("/api/mail/account", get(get_account))
        .route("/api/mail/account/me", get(get_my_account))

        // OAuth 2.0 (Phase 8 - OIDC)
        .route("/api/mail/oauth/authorize", get(oauth_authorize_handler))
        .route("/api/mail/oauth/callback", get(oauth_callback_handler))
        .route("/api/mail/oauth/status", get(oauth_status_handler))

        // Mailboxes
        .route("/api/mail/mailboxes", get(list_mailboxes))
        .route("/api/mail/mailboxes", post(create_mailbox))

        // Messages
        .route("/api/mail/messages", get(list_messages))
        .route("/api/mail/messages", post(send_message))
        .route("/api/mail/messages/:id", get(get_message))

        // Search
        .route("/api/mail/search", get(search_emails))

        // AI Features (Phase 5)
        .route("/api/mail/ai/smart-compose", post(ai::smart_compose_handler))
        .route("/api/mail/ai/summarize", post(ai::summarize_handler))
        .route("/api/mail/ai/priority-rank", post(ai::priority_rank_handler))
        .route("/api/mail/ai/quota", get(ai::quota_handler))

        // Real-time connection token
        .route("/api/mail/ws/token", get(get_ws_token))

        .layer(cors)
        .with_state(state)
}

#[cfg(not(feature = "email"))]
pub fn create_router(
    db_pool: PgPool,
    redis_client: redis::Client,
    jmap_client: JmapClient,
    search_client: EmailSearchClient,
    centrifugo_client: CentrifugoClient,
    stalwart_admin_client: StalwartAdminClient,
    default_email_password: String,
    kratos_client: KratosClient,
    oauth_token_manager: OAuthTokenManager,
) -> Router {
    let state = Arc::new(AppState {
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
        stalwart_admin_client,
        default_email_password,
        kratos_client,
        oauth_token_manager,
    });

    // Production CORS using AllowOrigin::mirror_request()
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::mirror_request())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600));

    Router::new()
        // Health check
        .route("/health", get(health_check))

        // Internal provisioning webhook
        .route("/internal/mail/provision", post(provision_webhook_handler))

        // Email account
        .route("/api/mail/account", get(get_account))
        .route("/api/mail/account/me", get(get_my_account))

        // OAuth 2.0 (Phase 8 - OIDC)
        .route("/api/mail/oauth/authorize", get(oauth_authorize_handler))
        .route("/api/mail/oauth/callback", get(oauth_callback_handler))
        .route("/api/mail/oauth/status", get(oauth_status_handler))

        // Mailboxes
        .route("/api/mail/mailboxes", get(list_mailboxes))
        .route("/api/mail/mailboxes", post(create_mailbox))

        // Messages
        .route("/api/mail/messages", get(list_messages))
        .route("/api/mail/messages", post(send_message))
        .route("/api/mail/messages/:id", get(get_message))

        // Search
        .route("/api/mail/search", get(search_emails))

        // Real-time connection token
        .route("/api/mail/ws/token", get(get_ws_token))

        .layer(cors)
        .with_state(state)
}

// ============================================================================
// Health & Status
// ============================================================================

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "email-service",
        "version": "0.3.0",
        "phase": "3"
    }))
}

// ============================================================================
// Provisioning (Phase 2)
// ============================================================================

/// Webhook handler for Kratos/Zitadel user creation
async fn provision_webhook_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<KratosWebhookPayload>,
) -> impl IntoResponse {
    let user_id = payload.identity.id.as_string();
    let email = payload.identity.traits.email.clone();

    info!(
        "Received provisioning webhook for user: {} ({})",
        user_id, email
    );

    // Use full provisioning with Stalwart integration
    match provisioning::provision_email_account_full(
        &state.db_pool,
        &state.stalwart_admin_client,
        &state.jmap_client,
        payload.clone(),
        &state.default_email_password,
    ).await {
        Ok(response) => {
            info!("Email account provisioned successfully for {}", email);
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Failed to provision email account for {}: {}", email, e);

            // Enqueue for retry (Phase 2.1)
            if let Err(retry_err) = provisioning::retry::enqueue_retry(
                &state.redis_client,
                payload,
                0,
                e.to_string(),
            )
            .await
            {
                error!("Failed to enqueue retry job: {}", retry_err);
            } else {
                info!("Enqueued retry job for {}", email);
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProvisioningResponse {
                    success: false,
                    message: Some(format!("Provisioning failed, will retry: {}", e)),
                    email_account_id: None,
                }),
            )
        }
    }
}

// ============================================================================
// Account (Phase 3)
// ============================================================================

#[derive(Deserialize)]
struct AccountQuery {
    kratos_id: Uuid,
}

/// Get email account information and quota
async fn get_account(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccountQuery>,
) -> impl IntoResponse {
    let account = sqlx::query_as!(
        EmailAccount,
        r#"
        SELECT id, kratos_identity_id, email_address, stalwart_user_id,
               COALESCE(storage_quota_bytes, 5368709120) as "storage_quota_bytes!",
               COALESCE(storage_used_bytes, 0) as "storage_used_bytes!",
               COALESCE(is_active, true) as "is_active!"
        FROM email.email_accounts
        WHERE kratos_identity_id = $1
        "#,
        params.kratos_id
    )
    .fetch_optional(&state.db_pool)
    .await;

    match account {
        Ok(Some(acc)) => (StatusCode::OK, Json(json!({
            "account": acc,
            "quota_percentage": (acc.storage_used_bytes as f64 / acc.storage_quota_bytes as f64) * 100.0
        }))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({
            "error": "Email account not found"
        }))),
        Err(e) => {
            error!("Failed to fetch account: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch account"
            })))
        }
    }
}

/// Get current user's email account from Kratos session
async fn get_my_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract Cookie header
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Invalid cookie header"
                    })),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "No session cookie found"
                })),
            )
        }
    };

    // Validate session with Kratos
    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid session"
                })),
            );
        }
    };

    let kratos_id = session.identity.id;

    // Query email account
    let account = sqlx::query_as!(
        EmailAccount,
        r#"
        SELECT id, kratos_identity_id, email_address, stalwart_user_id,
               COALESCE(storage_quota_bytes, 5368709120) as "storage_quota_bytes!",
               COALESCE(storage_used_bytes, 0) as "storage_used_bytes!",
               COALESCE(is_active, true) as "is_active!"
        FROM email.email_accounts
        WHERE kratos_identity_id = $1
        "#,
        kratos_id
    )
    .fetch_optional(&state.db_pool)
    .await;

    match account {
        Ok(Some(acc)) => (
            StatusCode::OK,
            Json(json!({
                "account": acc,
                "quota_percentage": (acc.storage_used_bytes as f64 / acc.storage_quota_bytes as f64) * 100.0
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Email account not found for this user"
            })),
        ),
        Err(e) => {
            error!("Failed to fetch account: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to fetch account"
                })),
            )
        }
    }
}

// ============================================================================
// Mailboxes (Phase 3)
// ============================================================================

// Mailbox query params removed - we use session auth now

/// Helper to get JMAP auth and account ID for a user (Basic Auth with email credentials)
async fn get_jmap_session(
    jmap_client: &JmapClient,
    db_pool: &sqlx::PgPool,
    kratos_identity_id: uuid::Uuid,
    default_password: &str,
) -> Result<(JmapAuth, String), (StatusCode, Json<serde_json::Value>)> {
    // Look up user's email account from database
    let email_account = match sqlx::query!(
        r#"
        SELECT email_address, stalwart_user_id, is_active
        FROM email.email_accounts
        WHERE kratos_identity_id = $1
        "#,
        kratos_identity_id
    )
    .fetch_optional(db_pool)
    .await
    {
        Ok(Some(account)) => account,
        Ok(None) => {
            error!("No email account found for Kratos ID {}", kratos_identity_id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Email account not found. Please contact support." })),
            ));
        }
        Err(e) => {
            error!("Database error fetching email account: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch email account information." })),
            ));
        }
    };

    // Check if account is active
    if !email_account.is_active.unwrap_or(false) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Email account is disabled. Please contact support." })),
        ));
    }

    // Use Basic Auth with email address and default password
    let auth = JmapAuth::Basic {
        username: email_account.email_address.clone(),
        password: default_password.to_string(),
    };

    // Get JMAP session to find account ID
    match jmap_client.get_session(&auth).await {
        Ok(session) => {
            // Get the primary account ID for mail
            let account_id = session
                .primary_accounts
                .get("urn:ietf:params:jmap:mail")
                .cloned()
                .unwrap_or_else(|| {
                    // Fall back to first account
                    session.accounts.keys().next().cloned().unwrap_or_default()
                });

            info!("JMAP session established for user: {}", email_account.email_address);
            Ok((auth, account_id))
        }
        Err(e) => {
            error!("Failed to get JMAP session for {}: {}", email_account.email_address, e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "JMAP authentication failed. Please contact support." })),
            ))
        }
    }
}

/// List all mailboxes (folders)
async fn list_mailboxes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract and validate session
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found"})),
            )
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session"})),
            );
        }
    };

    let kratos_id = session.identity.id;
    info!("Listing mailboxes for Kratos ID: {}", kratos_id);

    // Get JMAP auth and account ID using Basic Auth
    let (auth, account_id) = match get_jmap_session(
        &state.jmap_client,
        &state.db_pool,
        kratos_id,
        &state.default_email_password,
    )
    .await
    {
        Ok(session) => session,
        Err(err) => return err,
    };

    // Fetch real mailboxes from JMAP
    match state.jmap_client.get_mailboxes(&auth, &account_id).await {
        Ok(mailboxes) => {
            let mailbox_list: Vec<serde_json::Value> = mailboxes
                .iter()
                .map(|mb| {
                    json!({
                        "id": mb.id,
                        "name": mb.name,
                        "role": mb.role,
                        "parent_id": mb.parent_id,
                        "sort_order": mb.sort_order,
                        "total_emails": mb.total_emails,
                        "unread_emails": mb.unread_emails,
                        "total_threads": mb.total_threads,
                        "unread_threads": mb.unread_threads
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "mailboxes": mailbox_list,
                    "total": mailboxes.len()
                })),
            )
        }
        Err(e) => {
            error!("Failed to fetch mailboxes: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to fetch mailboxes: {}", e) })),
            )
        }
    }
}

/// Create a new mailbox
async fn create_mailbox(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateMailboxRequest>,
) -> impl IntoResponse {
    info!("Creating mailbox: {}", req.name);

    // Extract and validate Kratos session
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found"})),
            )
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session"})),
            );
        }
    };

    let kratos_id = session.identity.id;

    // Get JMAP auth and account ID using session-based credentials
    let (auth, account_id) = match get_jmap_session(
        &state.jmap_client,
        &state.db_pool,
        kratos_id,
        &state.default_email_password,
    )
    .await
    {
        Ok(session) => session,
        Err(err) => return err,
    };

    // Create mailbox via JMAP
    match state
        .jmap_client
        .create_mailbox(&auth, &account_id, &req.name, req.parent_id.as_deref(), None)
        .await
    {
        Ok(mailbox_id) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "mailbox_id": mailbox_id,
                "name": req.name
            })),
        ),
        Err(e) => {
            error!("Failed to create mailbox: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to create mailbox: {}", e) })),
            )
        }
    }
}

// ============================================================================
// Messages (Phase 3)
// ============================================================================

#[derive(Deserialize)]
struct MessagesQuery {
    mailbox_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

/// List messages in a mailbox
async fn list_messages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MessagesQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50);

    // Extract and validate session
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found"})),
            )
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session"})),
            );
        }
    };

    let kratos_id = session.identity.id;
    info!("Listing messages for Kratos ID: {}", kratos_id);

    // Get JMAP auth and account ID using session-based credentials
    let (auth, account_id) = match get_jmap_session(
        &state.jmap_client,
        &state.db_pool,
        kratos_id,
        &state.default_email_password,
    )
    .await
    {
        Ok(session) => session,
        Err(err) => return err,
    };

    // Fetch real emails from JMAP
    match state.jmap_client.get_emails(&auth, &account_id, params.mailbox_id.as_deref(), Some(limit as usize)).await {
        Ok(emails) => {
            let message_list: Vec<serde_json::Value> = emails
                .iter()
                .map(|email| {
                    json!({
                        "id": email.id,
                        "subject": email.subject,
                        "from": email.from.first().map(|f| json!({
                            "email": f.email,
                            "name": f.name
                        })),
                        "to": email.to.iter().map(|t| json!({
                            "email": t.email,
                            "name": t.name
                        })).collect::<Vec<_>>(),
                        "preview": email.preview,
                        "received_at": email.received_at,
                        "is_read": *email.keywords.get("$seen").unwrap_or(&false),
                        "is_flagged": *email.keywords.get("$flagged").unwrap_or(&false),
                        "has_attachments": email.has_attachment,
                        "mailbox_ids": email.mailbox_ids
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "messages": message_list,
                    "total": emails.len(),
                    "limit": limit
                })),
            )
        }
        Err(e) => {
            error!("Failed to fetch messages: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to fetch messages: {}", e) })),
            )
        }
    }
}

/// Get a single message by ID
async fn get_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    info!("Fetching message: {}", id);

    // Extract and validate Kratos session
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found"})),
            )
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session"})),
            );
        }
    };

    let kratos_id = session.identity.id;

    // Get JMAP auth and account ID using session-based credentials
    let (auth, account_id) = match get_jmap_session(
        &state.jmap_client,
        &state.db_pool,
        kratos_id,
        &state.default_email_password,
    )
    .await
    {
        Ok(session) => session,
        Err(err) => return err,
    };

    // Fetch full email from JMAP
    match state.jmap_client.get_email(&auth, &account_id, &id).await {
        Ok(email) => {
            (StatusCode::OK, Json(json!({
                "id": email.id,
                "subject": email.subject,
                "from": email.from.iter().map(|f| json!({
                    "email": f.email,
                    "name": f.name
                })).collect::<Vec<_>>(),
                "to": email.to.iter().map(|t| json!({
                    "email": t.email,
                    "name": t.name
                })).collect::<Vec<_>>(),
                "cc": email.cc.iter().map(|c| json!({
                    "email": c.email,
                    "name": c.name
                })).collect::<Vec<_>>(),
                "preview": email.preview,
                "body_text": email.text_body(),
                "body_html": email.html_body(),
                "received_at": email.received_at,
                "is_read": *email.keywords.get("$seen").unwrap_or(&false),
                "is_flagged": *email.keywords.get("$flagged").unwrap_or(&false),
                "has_attachments": email.has_attachment,
                "mailbox_ids": email.mailbox_ids
            })))
        }
        Err(e) => {
            error!("Failed to fetch message {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to fetch message: {}", e) })),
            )
        }
    }
}

/// Send a new email
async fn send_message(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SendEmailRequest>,
) -> impl IntoResponse {
    info!("Sending email to {:?}: {}", req.to, req.subject);

    // Extract and validate Kratos session
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found"})),
            )
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session"})),
            );
        }
    };

    let kratos_id = session.identity.id;

    // Get user's email address from Kratos identity
    let from_email = session.identity.traits.email.clone();

    // Get JMAP auth and account ID using session-based credentials
    let (auth, account_id) = match get_jmap_session(
        &state.jmap_client,
        &state.db_pool,
        kratos_id,
        &state.default_email_password,
    )
    .await
    {
        Ok(session) => session,
        Err(err) => return err,
    };

    // Convert to references for the JMAP client
    let to_refs: Vec<&str> = req.to.iter().map(|s| s.as_str()).collect();
    let cc_refs: Option<Vec<&str>> = req.cc.as_ref().map(|cc| cc.iter().map(|s| s.as_str()).collect());

    // Send email via JMAP (identity is fetched automatically)
    match state.jmap_client.send_email(
        &auth,
        &account_id,
        &from_email,  // from address (from Kratos identity)
        to_refs,
        cc_refs,
        &req.subject,
        &req.body_text,
        req.body_html.as_deref(),
    ).await {
        Ok(email_id) => {
            // Notify via Centrifugo for real-time updates
            let notification = NewEmailNotification {
                email_id: email_id.clone(),
                from: from_email.clone(),
                subject: req.subject.clone(),
                preview: req.body_text.chars().take(100).collect(),
            };
            if let Err(e) = state.centrifugo_client.notify_new_email(&account_id, &notification).await {
                warn!("Failed to send Centrifugo notification: {}", e);
            }

            (StatusCode::OK, Json(json!({
                "success": true,
                "email_id": email_id,
                "message": "Email sent successfully"
            })))
        }
        Err(e) => {
            error!("Failed to send email: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Failed to send email: {}", e)
                })),
            )
        }
    }
}

// ============================================================================
// Search (Phase 3)
// ============================================================================

/// Search emails
async fn search_emails(
    State(state): State<Arc<AppState>>,
    Query(req): Query<EmailSearchRequest>,
) -> impl IntoResponse {
    info!("Searching emails: {}", req.query);

    // TODO: For Phase 3, return stub data
    // In production, this would call state.search_client.search_emails()

    (StatusCode::OK, Json(json!({
        "results": [],
        "total": 0,
        "query": req.query
    })))
}

// ============================================================================
// WebSocket / Real-time (Phase 3)
// ============================================================================

/// Generate WebSocket connection token for Centrifugo from session
async fn get_ws_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract Cookie header
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Invalid cookie header"
                    })),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "No session cookie found"
                })),
            )
        }
    };

    // Validate session with Kratos
    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Invalid session"
                })),
            );
        }
    };

    let user_id = session.identity.id.to_string();

    let token = state
        .centrifugo_client
        .generate_connection_token(&user_id)
        .unwrap_or_else(|_| "error".to_string());

    (
        StatusCode::OK,
        Json(json!({
            "token": token,
            "channel": format!("email:user:{}", user_id)
        })),
    )
}

// ============================================================================
// OAuth 2.0 (Phase 8 - OIDC)
// ============================================================================

/// Request parameters for OAuth callback
#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: String,
    state: String,
}

/// Initiate OAuth authorization flow
/// Generates Hydra authorization URL with PKCE and redirects user
async fn oauth_authorize_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    info!("[OAuth] Starting authorization flow");

    // Validate Kratos session first
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found. Please login first."})),
            ))
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("[OAuth] Failed to validate Kratos session: {}", e);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session. Please login first."})),
            ));
        }
    };

    let kratos_id = session.identity.id;

    // Generate OAuth authorization URL with PKCE
    let (auth_url, csrf_token, pkce_verifier) = match state.oauth_token_manager.generate_auth_url() {
        Ok(result) => result,
        Err(e) => {
            error!("[OAuth] Failed to generate auth URL: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to initiate OAuth flow"})),
            ));
        }
    };

    // Store PKCE verifier and CSRF token in Redis temporarily (5 minutes expiry)
    let redis_key = format!("email:oauth:pkce:{}", kratos_id);
    let pkce_data = json!({
        "pkce_verifier": pkce_verifier,
        "csrf_token": csrf_token
    }).to_string();

    let mut redis_conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("[OAuth] Failed to get Redis connection: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to store OAuth state"})),
            ));
        }
    };

    let set_result: Result<(), redis::RedisError> = redis::AsyncCommands::set_ex(
        &mut redis_conn,
        redis_key.clone(),
        pkce_data,
        300, // 5 minutes
    ).await;

    if let Err(e) = set_result {
        error!("[OAuth] Failed to store PKCE data in Redis: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to store OAuth state"})),
        ));
    }

    info!("[OAuth] Redirecting to Hydra authorization URL for Kratos ID: {}", kratos_id);

    // Redirect to Hydra authorization endpoint
    Ok(Redirect::to(&auth_url))
}

/// Handle OAuth callback from Hydra
/// Exchanges authorization code for tokens and stores them
async fn oauth_callback_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OAuthCallbackQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    info!("[OAuth Callback] Received authorization code");

    // Validate Kratos session
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found"})),
            )
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("[OAuth Callback] Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session"})),
            );
        }
    };

    let kratos_id = session.identity.id;

    // Retrieve PKCE verifier and CSRF token from Redis
    let redis_key = format!("email:oauth:pkce:{}", kratos_id);
    let mut redis_conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("[OAuth Callback] Failed to get Redis connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve OAuth state"})),
            );
        }
    };

    let pkce_data: String = match redis::AsyncCommands::get(&mut redis_conn, &redis_key).await {
        Ok(data) => data,
        Err(e) => {
            error!("[OAuth Callback] Failed to retrieve PKCE data from Redis: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "OAuth state expired or not found. Please try again."})),
            );
        }
    };

    let pkce_json: serde_json::Value = match serde_json::from_str(&pkce_data) {
        Ok(json) => json,
        Err(e) => {
            error!("[OAuth Callback] Failed to parse PKCE data: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid OAuth state"})),
            );
        }
    };

    let pkce_verifier = pkce_json["pkce_verifier"].as_str().unwrap_or("");
    let stored_csrf = pkce_json["csrf_token"].as_str().unwrap_or("");

    // Validate CSRF token
    if params.state != stored_csrf {
        error!("[OAuth Callback] CSRF token mismatch");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid state parameter. Possible CSRF attack."})),
        );
    }

    // Exchange authorization code for tokens
    match state.oauth_token_manager.exchange_code(&params.code, pkce_verifier, kratos_id).await {
        Ok(token_pair) => {
            info!("[OAuth Callback] Successfully exchanged code for tokens");

            // Clean up Redis key
            let _: Result<(), _> = redis::AsyncCommands::del(&mut redis_conn, redis_key).await;

            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": "OAuth authorization successful",
                    "expires_at": token_pair.expires_at
                })),
            )
        }
        Err(e) => {
            error!("[OAuth Callback] Failed to exchange code for tokens: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to complete OAuth flow: {}", e)})),
            )
        }
    }
}

/// Check OAuth authorization status for current user
async fn oauth_status_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Validate Kratos session
    let cookie_header = match headers.get("cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid cookie header"})),
                )
            }
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session cookie found"})),
            )
        }
    };

    let session = match state.kratos_client.whoami(cookie_header).await {
        Ok(s) => s,
        Err(e) => {
            error!("[OAuth Status] Failed to validate Kratos session: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session"})),
            );
        }
    };

    let kratos_id = session.identity.id;

    // Check if user has authorized email access
    match state.oauth_token_manager.has_authorization(kratos_id).await {
        Ok(has_auth) => (
            StatusCode::OK,
            Json(json!({
                "connected": has_auth,
                "kratos_id": kratos_id
            })),
        ),
        Err(e) => {
            error!("[OAuth Status] Failed to check authorization: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check OAuth status"})),
            )
        }
    }
}
