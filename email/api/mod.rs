//! Email Service API (Phase 3)
//!
//! Provides REST API endpoints for email functionality.

use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, Method},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{
    centrifugo::CentrifugoClient,
    jmap::JmapClient,
    provisioning::{self, KratosWebhookPayload, ProvisioningResponse},
    search::EmailSearchClient,
    stalwart::StalwartAdminClient,
    types::*,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: redis::Client,
    pub jmap_client: JmapClient,
    pub search_client: EmailSearchClient,
    pub centrifugo_client: CentrifugoClient,
    pub stalwart_admin_client: StalwartAdminClient,
    pub default_email_password: String,
}

/// Create the email service API router
pub fn create_router(
    db_pool: PgPool,
    redis_client: redis::Client,
    jmap_client: JmapClient,
    search_client: EmailSearchClient,
    centrifugo_client: CentrifugoClient,
    stalwart_admin_client: StalwartAdminClient,
    default_email_password: String,
) -> Router {
    let state = Arc::new(AppState {
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
        stalwart_admin_client,
        default_email_password,
    });

    // Configure CORS for frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(health_check))

        // Internal provisioning webhook
        .route("/internal/mail/provision", post(provision_webhook_handler))

        // Email account
        .route("/api/mail/account", get(get_account))

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

/// Webhook handler for Kratos user creation
async fn provision_webhook_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<KratosWebhookPayload>,
) -> impl IntoResponse {
    let kratos_id = payload.identity.id;
    let email = payload.identity.traits.email.clone();

    info!(
        "Received provisioning webhook for Kratos identity: {} ({})",
        kratos_id, email
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
        FROM email_accounts
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

// ============================================================================
// Mailboxes (Phase 3)
// ============================================================================

#[derive(Deserialize)]
struct MailboxQuery {
    account_id: String,
    #[serde(default)]
    access_token: String,
}

/// List all mailboxes (folders)
async fn list_mailboxes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MailboxQuery>,
) -> impl IntoResponse {
    info!("Listing mailboxes for account: {}", params.account_id);

    // TODO: For Phase 3, return stub data
    // In production, this would call state.jmap_client.get_mailboxes()

    let mailboxes = vec![
        json!({
            "id": "inbox",
            "name": "Inbox",
            "role": "inbox",
            "total_emails": 42,
            "unread_emails": 5
        }),
        json!({
            "id": "sent",
            "name": "Sent",
            "role": "sent",
            "total_emails": 128,
            "unread_emails": 0
        }),
        json!({
            "id": "drafts",
            "name": "Drafts",
            "role": "drafts",
            "total_emails": 3,
            "unread_emails": 0
        }),
        json!({
            "id": "trash",
            "name": "Trash",
            "role": "trash",
            "total_emails": 15,
            "unread_emails": 0
        }),
    ];

    (StatusCode::OK, Json(json!({
        "mailboxes": mailboxes
    })))
}

/// Create a new mailbox
async fn create_mailbox(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateMailboxRequest>,
) -> impl IntoResponse {
    info!("Creating mailbox: {}", req.name);

    // TODO: For Phase 3, return stub response
    // In production, this would call state.jmap_client.create_mailbox()

    (StatusCode::OK, Json(json!({
        "success": true,
        "mailbox_id": format!("mailbox_{}", Uuid::new_v4()),
        "name": req.name
    })))
}

// ============================================================================
// Messages (Phase 3)
// ============================================================================

#[derive(Deserialize)]
struct MessagesQuery {
    account_id: String,
    mailbox_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

/// List messages in a mailbox
async fn list_messages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MessagesQuery>,
) -> impl IntoResponse {
    let mailbox_id = params.mailbox_id.unwrap_or_else(|| "inbox".to_string());
    let limit = params.limit.unwrap_or(50);

    info!(
        "Listing messages for account {} in mailbox {}",
        params.account_id, mailbox_id
    );

    // TODO: For Phase 3, return stub data
    // In production, this would call state.jmap_client.get_emails()

    let messages = vec![
        json!({
            "id": "msg1",
            "subject": "Welcome to Arack Mail",
            "from": { "email": "noreply@arack.com", "name": "Arack Team" },
            "preview": "Thank you for signing up! Here's how to get started...",
            "received_at": "2025-12-15T00:00:00Z",
            "is_read": false,
            "is_starred": false,
            "has_attachments": false
        }),
        json!({
            "id": "msg2",
            "subject": "Test Email",
            "from": { "email": "test@example.com", "name": "Test User" },
            "preview": "This is a test email message...",
            "received_at": "2025-12-14T12:00:00Z",
            "is_read": true,
            "is_starred": false,
            "has_attachments": false
        }),
    ];

    (StatusCode::OK, Json(json!({
        "messages": messages,
        "total": 2,
        "limit": limit
    })))
}

/// Get a single message by ID
async fn get_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Fetching message: {}", id);

    // TODO: For Phase 3, return stub data
    // In production, this would fetch full message body from JMAP

    (StatusCode::OK, Json(json!({
        "id": id,
        "subject": "Test Email",
        "from": { "email": "test@example.com", "name": "Test User" },
        "to": [{ "email": "user@arack.com", "name": "You" }],
        "body_text": "This is the full email body...",
        "body_html": "<p>This is the full email body...</p>",
        "received_at": "2025-12-14T12:00:00Z",
        "is_read": true
    })))
}

/// Send a new email
async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SendEmailRequest>,
) -> impl IntoResponse {
    info!("Sending email to {:?}: {}", req.to, req.subject);

    // TODO: For Phase 3, return stub response
    // In production, this would call state.jmap_client.send_email()

    let email_id = format!("sent_{}", Uuid::new_v4());

    // TODO: Notify via Centrifugo
    // state.centrifugo_client.notify_new_email(...).await;

    (StatusCode::OK, Json(json!({
        "success": true,
        "email_id": email_id,
        "message": "Email sent successfully"
    })))
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

#[derive(Deserialize)]
struct WsTokenQuery {
    user_id: String,
}

/// Generate WebSocket connection token for Centrifugo
async fn get_ws_token(
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsTokenQuery>,
) -> impl IntoResponse {
    let token = state
        .centrifugo_client
        .generate_connection_token(&params.user_id)
        .unwrap_or_else(|_| "error".to_string());

    (StatusCode::OK, Json(json!({
        "token": token,
        "channel": format!("email:user:{}", params.user_id)
    })))
}
