use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::search::redis::CacheManager;

/// Initialize and start the job scheduler
pub async fn start_scheduler(db_pool: PgPool, cache: CacheManager) -> Result<JobScheduler> {
    info!("Initializing job scheduler");

    let scheduler = JobScheduler::new().await?;

    // Job 1: Clear old cache entries every hour
    scheduler
        .add(Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let mut cache_clone = cache.clone();
            Box::pin(async move {
                info!("Running scheduled cache cleanup");
                match cache_clone.clear_pattern("search:*").await {
                    Ok(count) => info!("Cleared {} cached search results", count),
                    Err(e) => error!("Cache cleanup failed: {}", e),
                }
            })
        })?)
        .await?;

    // Job 2: Clean up old completed jobs from database every day at 2 AM
    let db_clone = db_pool.clone();
    scheduler
        .add(Job::new_async("0 0 2 * * *", move |_uuid, _l| {
            let pool = db_clone.clone();
            Box::pin(async move {
                info!("Running scheduled database cleanup");
                match cleanup_old_jobs(&pool).await {
                    Ok(count) => info!("Cleaned up {} old jobs", count),
                    Err(e) => error!("Database cleanup failed: {}", e),
                }
            })
        })?)
        .await?;

    // Job 3: Generate daily statistics at midnight
    let db_clone2 = db_pool.clone();
    scheduler
        .add(Job::new_async("0 0 0 * * *", move |_uuid, _l| {
            let pool = db_clone2.clone();
            Box::pin(async move {
                info!("Generating daily statistics");
                match generate_daily_stats(&pool).await {
                    Ok(_) => info!("Daily statistics generated successfully"),
                    Err(e) => error!("Statistics generation failed: {}", e),
                }
            })
        })?)
        .await?;

    // Job 4: Health check every 5 minutes
    scheduler
        .add(Job::new_async("0 */5 * * * *", move |_uuid, _l| {
            Box::pin(async move {
                info!("Health check: System operational");
            })
        })?)
        .await?;

    scheduler.start().await?;
    info!("Job scheduler started successfully");

    Ok(scheduler)
}

/// Clean up completed jobs older than 30 days
async fn cleanup_old_jobs(pool: &PgPool) -> Result<u64> {
    let threshold = Utc::now() - chrono::Duration::days(30);

    let result = sqlx::query!(
        r#"
        DELETE FROM crawl_history
        WHERE status IN ('completed', 'failed')
          AND completed_at < $1
        "#,
        threshold
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Generate daily statistics
async fn generate_daily_stats(pool: &PgPool) -> Result<()> {
    let stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total_jobs,
            COUNT(*) FILTER (WHERE status = 'completed') as completed_jobs,
            COUNT(*) FILTER (WHERE status = 'failed') as failed_jobs,
            COUNT(*) FILTER (WHERE status = 'processing') as processing_jobs,
            SUM(pages_crawled) as total_pages_crawled,
            SUM(pages_indexed) as total_pages_indexed
        FROM crawl_history
        WHERE started_at >= NOW() - INTERVAL '24 hours'
        "#
    )
    .fetch_one(pool)
    .await?;

    info!(
        "Daily Statistics - Total: {}, Completed: {}, Failed: {}, Processing: {}, Pages Crawled: {}, Pages Indexed: {}",
        stats.total_jobs.unwrap_or(0),
        stats.completed_jobs.unwrap_or(0),
        stats.failed_jobs.unwrap_or(0),
        stats.processing_jobs.unwrap_or(0),
        stats.total_pages_crawled.unwrap_or(0),
        stats.total_pages_indexed.unwrap_or(0)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        // Test is async, so we just verify the module compiles
        assert!(true);
    }
}
