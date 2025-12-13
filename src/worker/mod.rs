use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    crawler::Crawler,
    qdrant::QdrantService,
    redis::{CrawlJob, JobQueue, JobStatus},
    search::SearchClient,
};

/// Background worker for processing crawl jobs
pub struct Worker {
    crawler: Crawler,
    search_client: SearchClient,
    qdrant_service: Arc<QdrantService>,
    job_queue: JobQueue,
    db_pool: PgPool,
    worker_id: String,
}

impl Worker {
    pub fn new(
        crawler: Crawler,
        search_client: SearchClient,
        qdrant_service: Arc<QdrantService>,
        job_queue: JobQueue,
        db_pool: PgPool,
    ) -> Self {
        let worker_id = format!("worker-{}", Uuid::new_v4());
        Self {
            crawler,
            search_client,
            qdrant_service,
            job_queue,
            db_pool,
            worker_id,
        }
    }

    /// Start the worker loop
    pub async fn start(&mut self) -> Result<()> {
        info!("Worker {} started", self.worker_id);

        loop {
            match self.process_next_job().await {
                Ok(processed) => {
                    if !processed {
                        // No jobs available, wait before checking again
                        sleep(Duration::from_secs(5)).await;
                    }
                }
                Err(e) => {
                    error!("Worker {} error: {}", self.worker_id, e);
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    /// Process the next job from the queue
    async fn process_next_job(&mut self) -> Result<bool> {
        // Dequeue next job
        let job = match self.job_queue.dequeue().await? {
            Some(job) => job,
            None => return Ok(false), // No jobs available
        };

        info!(
            "Worker {} processing job {} with {} URLs",
            self.worker_id,
            job.id,
            job.urls.len()
        );

        // Save job to database
        if let Err(e) = self.save_job_to_db(&job).await {
            warn!("Failed to save job to database: {}", e);
        }

        // Process the job
        match self.process_job(&job).await {
            Ok((pages_crawled, pages_indexed)) => {
                info!(
                    "Job {} completed: crawled {}, indexed {}",
                    job.id, pages_crawled, pages_indexed
                );

                // Mark as completed in Redis
                if let Err(e) = self
                    .job_queue
                    .complete_job(job.id, pages_crawled, pages_indexed)
                    .await
                {
                    error!("Failed to mark job as completed: {}", e);
                }

                // Update database
                if let Err(e) = self.update_job_completed(&job.id, pages_crawled, pages_indexed).await {
                    error!("Failed to update job in database: {}", e);
                }
            }
            Err(e) => {
                let error_msg = format!("Job processing failed: {}", e);
                error!("{}", error_msg);

                // Mark as failed in Redis
                if let Err(e) = self.job_queue.fail_job(job.id, error_msg.clone()).await {
                    error!("Failed to mark job as failed: {}", e);
                }

                // Update database
                if let Err(e) = self.update_job_failed(&job.id, &error_msg).await {
                    error!("Failed to update failed job in database: {}", e);
                }
            }
        }

        Ok(true)
    }

    /// Process a single job
    async fn process_job(&mut self, job: &CrawlJob) -> Result<(usize, usize)> {
        // Crawl the URLs (returns documents and images)
        let (documents, images) = self.crawler.crawl_urls(job.urls.clone()).await?;
        let pages_crawled = documents.len();

        // Index the documents to Meilisearch
        self.search_client.index_documents(documents.clone()).await?;

        // Index the documents to Qdrant (Phase 10: Semantic Search)
        let mut qdrant_indexed = 0;
        for doc in &documents {
            match self.qdrant_service
                .index_page(&doc.id, &doc.url, &doc.title, &doc.content)
                .await
            {
                Ok(_) => {
                    qdrant_indexed += 1;
                }
                Err(e) => {
                    warn!("Failed to index page {} to Qdrant: {}", doc.url, e);
                    // Don't fail the entire job if Qdrant indexing fails
                }
            }
        }

        if qdrant_indexed > 0 {
            info!("Indexed {} documents to Qdrant", qdrant_indexed);
        }

        let pages_indexed = pages_crawled; // Assuming all crawled pages are indexed

        // Index the images
        if !images.is_empty() {
            info!("Indexing {} extracted images", images.len());
            if let Err(e) = self.search_client.index_images(images).await {
                warn!("Failed to index images: {}", e);
                // Don't fail the entire job if image indexing fails
            }
        }

        Ok((pages_crawled, pages_indexed))
    }

    /// Save job to database
    async fn save_job_to_db(&self, job: &CrawlJob) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO crawl_history (id, collection_id, urls, status, pages_crawled, pages_indexed, started_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                status = $4,
                started_at = $7
            "#,
            job.id,
            job.collection_id,
            &job.urls,
            "processing",
            job.pages_crawled as i32,
            job.pages_indexed as i32,
            job.started_at
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Update job as completed in database
    async fn update_job_completed(
        &self,
        job_id: &Uuid,
        pages_crawled: usize,
        pages_indexed: usize,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE crawl_history
            SET status = 'completed',
                pages_crawled = $2,
                pages_indexed = $3,
                completed_at = NOW()
            WHERE id = $1
            "#,
            job_id,
            pages_crawled as i32,
            pages_indexed as i32
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Update job as failed in database
    async fn update_job_failed(&self, job_id: &Uuid, error_message: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE crawl_history
            SET status = 'failed',
                error_message = $2,
                completed_at = NOW()
            WHERE id = $1
            "#,
            job_id,
            error_message
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

/// Spawn multiple workers
pub async fn spawn_workers(
    count: usize,
    crawler: Crawler,
    search_client: SearchClient,
    qdrant_service: Arc<QdrantService>,
    job_queue: JobQueue,
    db_pool: PgPool,
) -> Result<()> {
    info!("Spawning {} background workers", count);

    for i in 0..count {
        let crawler_clone = crawler.clone();
        let search_clone = search_client.clone();
        let qdrant_clone = qdrant_service.clone();
        let queue_clone = job_queue.clone();
        let pool_clone = db_pool.clone();

        tokio::spawn(async move {
            let mut worker = Worker::new(
                crawler_clone,
                search_clone,
                qdrant_clone,
                queue_clone,
                pool_clone,
            );
            if let Err(e) = worker.start().await {
                error!("Worker {} crashed: {}", i, e);
            }
        });
    }

    info!("All workers spawned successfully");
    Ok(())
}
