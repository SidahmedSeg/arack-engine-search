//! Retry Logic for Failed Email Provisioning (Phase 2.1)
//!
//! Handles automatic retries for failed provisioning attempts using Redis queue
//! with exponential backoff.

use anyhow::{Context, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{provision_email_account, KratosWebhookPayload, UserIdType};

const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_QUEUE_KEY: &str = "email:provisioning:retry";

/// Retry job for failed provisioning
/// Stores user_id as a string to support both Kratos UUIDs and Zitadel numeric IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryJob {
    /// User ID as string (works for both UUID and Zitadel numeric IDs)
    pub user_id: String,
    pub payload: KratosWebhookPayload,
    pub attempt: u32,
    pub last_error: String,
    pub enqueued_at: i64, // Unix timestamp
}

/// Calculate exponential backoff delay
///
/// Delays: 1min, 5min, 30min for attempts 1, 2, 3
pub fn calculate_backoff_seconds(attempt: u32) -> u64 {
    match attempt {
        1 => 60,      // 1 minute
        2 => 300,     // 5 minutes
        3 => 1800,    // 30 minutes
        _ => 3600,    // 1 hour (fallback)
    }
}

/// Enqueue a failed provisioning job for retry
pub async fn enqueue_retry(
    redis_client: &redis::Client,
    payload: KratosWebhookPayload,
    attempt: u32,
    error_message: String,
) -> Result<()> {
    let user_id_str = payload.identity.id.as_string();

    if attempt >= MAX_RETRY_ATTEMPTS {
        warn!(
            "Maximum retry attempts ({}) reached for user: {}. Not enqueuing.",
            MAX_RETRY_ATTEMPTS, user_id_str
        );
        return Ok(());
    }

    let retry_job = RetryJob {
        user_id: user_id_str.clone(),
        payload,
        attempt: attempt + 1,
        last_error: error_message,
        enqueued_at: chrono::Utc::now().timestamp(),
    };

    let job_json = serde_json::to_string(&retry_job)
        .context("Failed to serialize retry job")?;

    let delay_seconds = calculate_backoff_seconds(retry_job.attempt);

    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .context("Failed to get Redis connection")?;

    // Add to sorted set with score = current_time + delay
    let execute_at = chrono::Utc::now().timestamp() + delay_seconds as i64;

    conn.zadd(RETRY_QUEUE_KEY, job_json, execute_at)
        .await
        .context("Failed to enqueue retry job")?;

    info!(
        "Enqueued retry job for {} (attempt {}/{}, delay: {}s)",
        user_id_str,
        retry_job.attempt,
        MAX_RETRY_ATTEMPTS,
        delay_seconds
    );

    Ok(())
}

/// Dequeue retry jobs that are ready to be processed
pub async fn dequeue_ready_jobs(
    redis_client: &redis::Client,
) -> Result<Vec<RetryJob>> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .context("Failed to get Redis connection")?;

    let now = chrono::Utc::now().timestamp();

    // Get jobs with score <= now (ready to execute)
    let jobs_json: Vec<String> = conn
        .zrangebyscore_limit(RETRY_QUEUE_KEY, 0, now, 0, 10)
        .await
        .context("Failed to fetch ready retry jobs")?;

    if jobs_json.is_empty() {
        return Ok(Vec::new());
    }

    // Remove the jobs from the queue
    for job_json in &jobs_json {
        conn.zrem(RETRY_QUEUE_KEY, job_json)
            .await
            .context("Failed to remove job from retry queue")?;
    }

    // Deserialize jobs
    let mut jobs = Vec::new();
    for job_json in jobs_json {
        match serde_json::from_str::<RetryJob>(&job_json) {
            Ok(job) => jobs.push(job),
            Err(e) => {
                error!("Failed to deserialize retry job: {}", e);
                continue;
            }
        }
    }

    Ok(jobs)
}

/// Start the retry worker
///
/// This runs in the background and processes failed provisioning jobs
/// with exponential backoff.
pub async fn start_retry_worker(
    redis_client: redis::Client,
    db_pool: PgPool,
) {
    info!("Starting email provisioning retry worker...");

    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        match process_retry_jobs(&redis_client, &db_pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Processed {} retry jobs", count);
                }
            }
            Err(e) => {
                error!("Error processing retry jobs: {}", e);
            }
        }
    }
}

/// Process ready retry jobs
async fn process_retry_jobs(
    redis_client: &redis::Client,
    db_pool: &PgPool,
) -> Result<usize> {
    let jobs = dequeue_ready_jobs(redis_client).await?;
    let count = jobs.len();

    for job in jobs {
        info!(
            "Retrying provisioning for {} (attempt {}/{})",
            job.user_id, job.attempt, MAX_RETRY_ATTEMPTS
        );

        match provision_email_account(db_pool, job.payload.clone()).await {
            Ok(response) => {
                info!(
                    "Retry successful for {}: {:?}",
                    job.user_id, response
                );
            }
            Err(e) => {
                error!(
                    "Retry attempt {} failed for {}: {}",
                    job.attempt, job.user_id, e
                );

                // Enqueue for another retry if not at max attempts
                if let Err(enqueue_err) = enqueue_retry(
                    redis_client,
                    job.payload,
                    job.attempt,
                    e.to_string(),
                )
                .await
                {
                    error!("Failed to re-enqueue retry job: {}", enqueue_err);
                }

                // Update provisioning log with failure
                if let Err(db_err) = update_provisioning_failure(
                    db_pool,
                    &job.user_id,
                    job.attempt,
                    &e.to_string(),
                )
                .await
                {
                    error!("Failed to update provisioning log: {}", db_err);
                }
            }
        }
    }

    Ok(count)
}

/// Update provisioning log with retry failure
async fn update_provisioning_failure(
    db_pool: &PgPool,
    user_id: &str,
    attempt: u32,
    error_message: &str,
) -> Result<()> {
    // Try to parse as UUID (Kratos), otherwise use as Zitadel ID
    let kratos_uuid = Uuid::parse_str(user_id).ok();
    let zitadel_id: Option<&str> = if kratos_uuid.is_none() { Some(user_id) } else { None };

    // Use runtime-checked query to avoid need for compile-time database connection
    sqlx::query(
        r#"
        INSERT INTO email.email_provisioning_log
        (kratos_identity_id, zitadel_user_id, action, status, error_message, attempt_count, completed_at)
        VALUES ($1, $2, 'create', 'failed', $3, $4, NOW())
        "#
    )
    .bind(kratos_uuid)
    .bind(zitadel_id)
    .bind(error_message)
    .bind(attempt as i32)
    .execute(db_pool)
    .await
    .context("Failed to log provisioning failure")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        assert_eq!(calculate_backoff_seconds(1), 60);   // 1 min
        assert_eq!(calculate_backoff_seconds(2), 300);  // 5 min
        assert_eq!(calculate_backoff_seconds(3), 1800); // 30 min
    }

    #[test]
    fn test_retry_job_serialization_uuid() {
        use super::super::{KratosIdentity, KratosTraits};

        let uuid = Uuid::new_v4();
        let payload = KratosWebhookPayload {
            identity: KratosIdentity {
                id: UserIdType::Uuid(uuid),
                traits: KratosTraits {
                    email: "test@example.com".to_string(),
                    first_name: "Test".to_string(),
                    last_name: "User".to_string(),
                },
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            },
        };

        let job = RetryJob {
            user_id: payload.identity.id.as_string(),
            payload,
            attempt: 1,
            last_error: "Test error".to_string(),
            enqueued_at: 1234567890,
        };

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: RetryJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.attempt, deserialized.attempt);
        assert_eq!(job.last_error, deserialized.last_error);
        assert_eq!(job.user_id, deserialized.user_id);
    }

    #[test]
    fn test_retry_job_serialization_zitadel() {
        use super::super::{KratosIdentity, KratosTraits};

        let payload = KratosWebhookPayload {
            identity: KratosIdentity {
                id: UserIdType::String("353361647777087498".to_string()),
                traits: KratosTraits {
                    email: "test@arack.io".to_string(),
                    first_name: "Test".to_string(),
                    last_name: "User".to_string(),
                },
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            },
        };

        let job = RetryJob {
            user_id: payload.identity.id.as_string(),
            payload,
            attempt: 1,
            last_error: "Test error".to_string(),
            enqueued_at: 1234567890,
        };

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: RetryJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.attempt, deserialized.attempt);
        assert_eq!(job.user_id, "353361647777087498");
    }
}
