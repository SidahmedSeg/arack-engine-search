//! AI Feature API Endpoints (Phase 5)
//!
//! REST API handlers for AI-powered email features.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::super::{
    ai::{self, priority::{self, EmailInfo}, smart_compose, summarize, types::*},
};

use super::AppState;

// ============================================================================
// Smart Compose
// ============================================================================

/// Smart compose API endpoint
pub async fn smart_compose_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccountQuery>,
    Json(req): Json<SmartComposeRequest>,
) -> impl IntoResponse {
    info!("Smart compose request for account: {}", params.account_id);

    // Check quota
    match ai::check_quota(&state.db_pool, &params.account_id, AiFeature::SmartCompose).await {
        Ok(has_quota) => {
            if !has_quota {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": "Daily quota exceeded for smart compose",
                        "limit": AiFeature::SmartCompose.daily_limit()
                    })),
                );
            }
        }
        Err(e) => {
            error!("Failed to check quota: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to check quota" })),
            );
        }
    }

    // Generate suggestions
    match smart_compose::generate_suggestions(
        &state.openai_client,
        &req.partial_text,
        req.context.as_ref(),
    )
    .await
    {
        Ok(suggestions) => {
            // Count tokens (approximate)
            let token_count = smart_compose::count_tokens(&req.partial_text)
                + suggestions
                    .iter()
                    .map(|s| smart_compose::count_tokens(&s.text))
                    .sum::<usize>();

            // Record usage
            if let Err(e) = ai::record_usage(
                &state.db_pool,
                &params.account_id,
                AiFeature::SmartCompose,
                token_count as i32,
                None, // Cost calculation would be more precise with actual token counts from OpenAI
            )
            .await
            {
                warn!("Failed to record AI usage: {}", e);
            }

            (
                StatusCode::OK,
                Json(json!({
                    "suggestions": suggestions
                })),
            )
        }
        Err(e) => {
            error!("Failed to generate smart compose suggestions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to generate suggestions" })),
            )
        }
    }
}

// ============================================================================
// Email Summarization
// ============================================================================

/// Summarize email thread endpoint
pub async fn summarize_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccountQuery>,
    Json(req): Json<SummarizeRequest>,
) -> impl IntoResponse {
    info!(
        "Summarize request for account: {} ({} emails)",
        params.account_id,
        req.email_ids.len()
    );

    // Check quota
    match ai::check_quota(&state.db_pool, &params.account_id, AiFeature::Summarization).await {
        Ok(has_quota) => {
            if !has_quota {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": "Daily quota exceeded for summarization",
                        "limit": AiFeature::Summarization.daily_limit()
                    })),
                );
            }
        }
        Err(e) => {
            error!("Failed to check quota: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to check quota" })),
            );
        }
    }

    // Fetch email contents from database
    // TODO: Implement actual email fetching from JMAP or database
    // For now, return a stub response
    let emails = Vec::new(); // Placeholder

    if emails.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No emails found to summarize" })),
        );
    }

    // Generate summary
    match summarize::summarize_thread(&state.openai_client, emails).await {
        Ok(result) => {
            // Record usage
            if let Err(e) = ai::record_usage(
                &state.db_pool,
                &params.account_id,
                AiFeature::Summarization,
                result.token_count as i32,
                None,
            )
            .await
            {
                warn!("Failed to record AI usage: {}", e);
            }

            (
                StatusCode::OK,
                Json(json!({
                    "summary": result.summary,
                    "key_points": result.key_points,
                    "action_items": result.action_items,
                    "token_count": result.token_count
                })),
            )
        }
        Err(e) => {
            error!("Failed to summarize thread: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to summarize thread" })),
            )
        }
    }
}

// ============================================================================
// Priority Ranking
// ============================================================================

/// Priority ranking endpoint
pub async fn priority_rank_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccountQuery>,
    Json(req): Json<PriorityRankRequest>,
) -> impl IntoResponse {
    info!(
        "Priority rank request for account: {} ({} emails)",
        params.account_id,
        req.email_ids.len()
    );

    // Check quota
    match ai::check_quota(&state.db_pool, &params.account_id, AiFeature::PriorityRanking).await {
        Ok(has_quota) => {
            if !has_quota {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": "Daily quota exceeded for priority ranking",
                        "limit": AiFeature::PriorityRanking.daily_limit()
                    })),
                );
            }
        }
        Err(e) => {
            error!("Failed to check quota: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to check quota" })),
            );
        }
    }

    // Fetch email metadata
    // TODO: Implement actual email fetching with sender frequency
    let emails: Vec<EmailInfo> = Vec::new(); // Placeholder

    if emails.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No emails found to rank" })),
        );
    }

    // Rank emails
    match priority::rank_emails(&state.openai_client, emails, None).await {
        Ok(ranked) => {
            // Record usage (one call per email)
            if let Err(e) = ai::record_usage(
                &state.db_pool,
                &params.account_id,
                AiFeature::PriorityRanking,
                (req.email_ids.len() * 100) as i32, // Approximate: 100 tokens per email
                None,
            )
            .await
            {
                warn!("Failed to record AI usage: {}", e);
            }

            (
                StatusCode::OK,
                Json(json!({
                    "ranked_emails": ranked
                })),
            )
        }
        Err(e) => {
            error!("Failed to rank emails: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to rank emails" })),
            )
        }
    }
}

// ============================================================================
// Quota
// ============================================================================

#[derive(Deserialize)]
pub struct AccountQuery {
    pub account_id: Uuid,
}

/// Get AI quota for user
pub async fn quota_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccountQuery>,
) -> impl IntoResponse {
    info!("Quota request for account: {}", params.account_id);

    match ai::get_quota(&state.db_pool, &params.account_id).await {
        Ok(quota) => (StatusCode::OK, Json(json!(quota))),
        Err(e) => {
            error!("Failed to get quota: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get quota" })),
            )
        }
    }
}
