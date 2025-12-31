//! AI Features Module (Phase 5)
//!
//! AI-powered email features using OpenAI:
//! - Smart compose suggestions
//! - Email thread summarization
//! - Priority inbox ranking

pub mod priority;
pub mod smart_compose;
pub mod summarize;
pub mod types;

use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use types::{AiFeature, AiQuota, QuotaUsage};

#[cfg(feature = "email")]
use async_openai::{config::OpenAIConfig, Client};

/// Check if user has quota remaining for a feature
pub async fn check_quota(
    db_pool: &PgPool,
    account_id: &uuid::Uuid,
    feature: AiFeature,
) -> Result<bool> {
    let today = Utc::now().date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);

    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count
        FROM email.email_ai_interactions
        WHERE account_id = $1
          AND feature = $2
          AND created_at >= $3
          AND created_at < $4
        "#,
    )
    .bind(account_id)
    .bind(feature.as_str())
    .bind(today)
    .bind(tomorrow)
    .fetch_one(db_pool)
    .await?;

    let used = count.0 as u32;
    let limit = feature.daily_limit();

    Ok(used < limit)
}

/// Record AI usage
pub async fn record_usage(
    db_pool: &PgPool,
    account_id: &uuid::Uuid,
    feature: AiFeature,
    tokens_used: i32,
    cost_usd: Option<f64>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO email.email_ai_interactions
        (account_id, feature, tokens_used, cost_usd)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(account_id)
    .bind(feature.as_str())
    .bind(tokens_used)
    .bind(cost_usd)
    .execute(db_pool)
    .await?;

    Ok(())
}

/// Get AI quota for user
pub async fn get_quota(db_pool: &PgPool, account_id: &uuid::Uuid) -> Result<AiQuota> {
    let today = Utc::now().date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);
    let reset_at = tomorrow
        .and_hms_opt(0, 0, 0)
        .unwrap_or_default()
        .and_utc();

    // Get usage for each feature
    let usage_rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT feature, COUNT(*) as count
        FROM email.email_ai_interactions
        WHERE account_id = $1
          AND created_at >= $2
          AND created_at < $3
        GROUP BY feature
        "#,
    )
    .bind(account_id)
    .bind(today)
    .bind(tomorrow)
    .fetch_all(db_pool)
    .await?;

    // Convert to map for easy lookup
    let mut usage_map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for (feature, count) in usage_rows {
        usage_map.insert(feature, count as u32);
    }

    let smart_compose_used = *usage_map.get("smart_compose").unwrap_or(&0);
    let summarize_used = *usage_map.get("summarize").unwrap_or(&0);
    let priority_used = *usage_map.get("priority").unwrap_or(&0);

    Ok(AiQuota {
        smart_compose: QuotaUsage {
            used: smart_compose_used,
            limit: AiFeature::SmartCompose.daily_limit(),
            reset_at,
        },
        summarization: QuotaUsage {
            used: summarize_used,
            limit: AiFeature::Summarization.daily_limit(),
            reset_at,
        },
        priority_ranking: QuotaUsage {
            used: priority_used,
            limit: AiFeature::PriorityRanking.daily_limit(),
            reset_at,
        },
    })
}

/// Calculate cost for GPT-4o-mini
/// Input: $0.15 per 1M tokens
/// Output: $0.60 per 1M tokens
pub fn calculate_cost(input_tokens: usize, output_tokens: usize) -> f64 {
    let input_cost = (input_tokens as f64 / 1_000_000.0) * 0.15;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * 0.60;
    input_cost + output_cost
}

/// Create OpenAI client from API key
#[cfg(feature = "email")]
pub fn create_openai_client(api_key: &str) -> Client<OpenAIConfig> {
    let config = OpenAIConfig::new().with_api_key(api_key);
    Client::with_config(config)
}

#[cfg(not(feature = "email"))]
pub fn create_openai_client(_api_key: &str) -> () {
    ()
}
