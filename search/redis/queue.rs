use anyhow::Result;
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

/// Job queue manager for background task processing
#[derive(Clone)]
pub struct JobQueue {
    conn: ConnectionManager,
    queue_name: String,
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Crawl job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlJob {
    pub id: Uuid,
    pub collection_id: Option<Uuid>,
    pub urls: Vec<String>,
    pub max_depth: usize,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub pages_crawled: usize,
    pub pages_indexed: usize,
}

impl CrawlJob {
    pub fn new(urls: Vec<String>, max_depth: usize, collection_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            collection_id,
            urls,
            max_depth,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            pages_crawled: 0,
            pages_indexed: 0,
        }
    }
}

impl JobQueue {
    pub fn new(conn: ConnectionManager, queue_name: String) -> Self {
        Self { conn, queue_name }
    }

    /// Enqueue a new crawl job
    pub async fn enqueue(&mut self, job: &CrawlJob) -> Result<()> {
        let job_json = serde_json::to_string(job)?;

        // Add to the queue (FIFO using RPUSH/LPOP)
        self.conn.rpush(&self.queue_name, &job_json).await?;

        // Also store the job details with its ID for status tracking
        let job_key = format!("job:{}", job.id);
        self.conn.set(&job_key, &job_json).await?;
        self.conn.expire(&job_key, 86400).await?; // Expire after 24 hours

        info!("Enqueued job {} with {} URLs", job.id, job.urls.len());
        Ok(())
    }

    /// Dequeue the next job from the queue
    pub async fn dequeue(&mut self) -> Result<Option<CrawlJob>> {
        match self.conn.lpop::<_, Option<String>>(&self.queue_name, None).await {
            Ok(Some(job_json)) => {
                match serde_json::from_str::<CrawlJob>(&job_json) {
                    Ok(mut job) => {
                        job.status = JobStatus::Processing;
                        job.started_at = Some(Utc::now());

                        // Update job status
                        self.update_job(&job).await?;

                        info!("Dequeued job {}", job.id);
                        Ok(Some(job))
                    }
                    Err(e) => {
                        error!("Failed to deserialize job: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to dequeue job: {}", e);
                Err(e.into())
            }
        }
    }

    /// Update job details
    pub async fn update_job(&mut self, job: &CrawlJob) -> Result<()> {
        let job_key = format!("job:{}", job.id);
        let job_json = serde_json::to_string(job)?;
        self.conn.set(&job_key, job_json).await?;
        Ok(())
    }

    /// Mark job as completed
    pub async fn complete_job(&mut self, job_id: Uuid, pages_crawled: usize, pages_indexed: usize) -> Result<()> {
        let job_key = format!("job:{}", job_id);

        if let Some(job_json) = self.conn.get::<_, Option<String>>(&job_key).await? {
            if let Ok(mut job) = serde_json::from_str::<CrawlJob>(&job_json) {
                job.status = JobStatus::Completed;
                job.completed_at = Some(Utc::now());
                job.pages_crawled = pages_crawled;
                job.pages_indexed = pages_indexed;

                let updated_json = serde_json::to_string(&job)?;
                self.conn.set(&job_key, updated_json).await?;

                info!("Marked job {} as completed (crawled: {}, indexed: {})", job_id, pages_crawled, pages_indexed);
            }
        }

        Ok(())
    }

    /// Mark job as failed
    pub async fn fail_job(&mut self, job_id: Uuid, error: String) -> Result<()> {
        let job_key = format!("job:{}", job_id);

        if let Some(job_json) = self.conn.get::<_, Option<String>>(&job_key).await? {
            if let Ok(mut job) = serde_json::from_str::<CrawlJob>(&job_json) {
                job.status = JobStatus::Failed;
                job.completed_at = Some(Utc::now());
                job.error = Some(error.clone());

                let updated_json = serde_json::to_string(&job)?;
                self.conn.set(&job_key, updated_json).await?;

                error!("Marked job {} as failed: {}", job_id, error);
            }
        }

        Ok(())
    }

    /// Get job status by ID
    pub async fn get_job(&mut self, job_id: Uuid) -> Result<Option<CrawlJob>> {
        let job_key = format!("job:{}", job_id);

        match self.conn.get::<_, Option<String>>(&job_key).await? {
            Some(job_json) => {
                let job = serde_json::from_str::<CrawlJob>(&job_json)?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Get queue length
    pub async fn queue_length(&mut self) -> Result<usize> {
        let len: usize = self.conn.llen(&self.queue_name).await?;
        Ok(len)
    }
}
