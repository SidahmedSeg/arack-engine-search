//! Search Service Binary
//!
//! This is the main entry point for the search service.
//! It handles web crawling, indexing, and search API operations.

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

// Import shared library modules (which includes search module)
use arack_shared::{config, db, search};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Search Service...");

    // Load configuration
    let config = config::Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize database connection pool
    let db_pool = db::create_pool(&config.database_url).await?;
    info!("Connected to PostgreSQL at {}", config.database_url);

    // Run database migrations
    db::run_migrations(&db_pool).await?;

    // Initialize Redis connection
    let redis_conn = search::redis::create_connection(&config.redis_url).await?;
    info!("Connected to Redis at {}", config.redis_url);

    // Initialize cache manager (TTL: 300 seconds = 5 minutes)
    let cache_manager = search::redis::CacheManager::new(redis_conn.clone(), 300);

    // Initialize job queue
    let job_queue = search::redis::JobQueue::new(redis_conn.clone(), "crawl_jobs".to_string());
    info!("Job queue initialized");

    // Initialize search client with database pool and Redis for query log autocomplete
    let search_client = search::search::SearchClient::new_with_db(
        &config.meilisearch_url,
        &config.meilisearch_key,
        db_pool.clone(),
        redis_conn.clone(),
    )?;
    info!("Connected to Meilisearch at {} (with query log autocomplete)", config.meilisearch_url);

    // Initialize Qdrant service (Phase 10: Semantic Search)
    let qdrant_config = config.qdrant();
    let qdrant_service = match search::qdrant::QdrantService::new(&qdrant_config.url, qdrant_config.collection_name).await {
        Ok(service) => {
            info!("Connected to Qdrant at {}", qdrant_config.url);
            Arc::new(service)
        },
        Err(e) => {
            tracing::warn!("Failed to connect to Qdrant: {}. Using placeholder (semantic search disabled)", e);
            Arc::new(search::qdrant::QdrantService::placeholder())
        }
    };

    // Initialize crawler for workers with rate limiting (Phase 6.2) and headers (Phase 6.3)
    let crawler_config = search::crawler::CrawlerConfig {
        max_depth: config.crawler_max_depth,
        max_concurrent: config.crawler_max_concurrent,
        max_content_length: 10000,
        respect_robots_txt: true,
        requests_per_second: config.crawler_requests_per_second,
        min_delay_ms: config.crawler_min_delay_ms,
        max_retries: config.crawler_max_retries,
    };
    let crawler = search::crawler::Crawler::with_headers(
        crawler_config,
        config.crawler_user_agent.clone(),
        config.crawler_contact_email.clone(),
        config.crawler_bot_url.clone(),
        config.crawler_accept_language.clone(),
    );

    info!("Crawler initialized with User-Agent: {}", crawler.user_agent());

    // Spawn background workers (Phase 5.3)
    search::worker::spawn_workers(
        2, // Number of workers
        crawler,
        search_client.clone(),
        qdrant_service.clone(),
        job_queue.clone(),
        db_pool.clone(),
    )
    .await?;

    // Start job scheduler (Phase 5.4)
    let _scheduler = search::scheduler::start_scheduler(db_pool.clone(), cache_manager.clone()).await?;

    // Start API server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    info!("Starting Search Service API server on {}", addr);

    // Use AccountServiceClient for session validation (no more Zitadel)
    search::api::serve(
        &addr,
        search_client,
        qdrant_service,
        db_pool,
        cache_manager,
        job_queue,
        config.account_service_url,
        config.email_service_url,
    ).await?;

    Ok(())
}
