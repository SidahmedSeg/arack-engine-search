//! Email Service Binary (Phase 3: Core Email Features)
//!
//! This is the main entry point for the email service.
//! Currently implements:
//! - Email account provisioning via Kratos webhooks (Phase 2) ✅
//! - Retry logic with exponential backoff (Phase 2.1) ✅
//! - JMAP client for Stalwart integration (Phase 3) ✅
//! - Meilisearch email search (Phase 3) ✅
//! - Centrifugo real-time notifications (Phase 3) ✅
//!
//! Future phases will add:
//! - AI features (Phase 7)
//! - Contact management (Phase 6)

use anyhow::Result;
use meilisearch_sdk::client::Client as MeilisearchClient;
use tracing::info;

// Import shared library modules
use arack_shared::{config, db, email};

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

    info!("Starting Email Service (Phase 3: Core Email Features)...");

    // Load configuration
    let config = config::Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize database connection pool
    let db_pool = db::create_pool(&config.database_url).await?;
    info!("Connected to PostgreSQL at {}", config.database_url);

    // Run database migrations
    db::run_migrations(&db_pool).await?;
    info!("Database migrations completed successfully");

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    info!("Connected to Redis at {}", config.redis_url);

    // Initialize JMAP client for Stalwart
    let stalwart_url = std::env::var("STALWART_URL")
        .unwrap_or_else(|_| "http://stalwart:8080".to_string());
    let jmap_client = email::jmap::JmapClient::new(&stalwart_url);
    info!("JMAP client initialized for Stalwart at {}", stalwart_url);

    // Initialize Meilisearch client and email search
    let meilisearch = MeilisearchClient::new(
        &config.meilisearch_url,
        Some(&config.meilisearch_key),
    )?;
    let search_client = email::search::EmailSearchClient::new(meilisearch.clone(), "emails");
    info!("Meilisearch email search client initialized");

    // Initialize search index
    if let Err(e) = search_client.initialize_index().await {
        tracing::warn!("Failed to initialize Meilisearch email index: {}", e);
    } else {
        info!("Meilisearch email index initialized successfully");
    }

    // Initialize Centrifugo client for real-time notifications
    let centrifugo_url = std::env::var("CENTRIFUGO_URL")
        .unwrap_or_else(|_| "http://centrifugo:8000".to_string());
    let centrifugo_api_key = std::env::var("CENTRIFUGO_API_KEY")
        .unwrap_or_else(|_| "centrifugo-api-key".to_string());
    let centrifugo_client =
        email::centrifugo::CentrifugoClient::new(&centrifugo_url, &centrifugo_api_key);
    info!("Centrifugo client initialized at {}", centrifugo_url);

    // Start retry worker in background (Phase 2.1)
    let retry_worker_redis = redis_client.clone();
    let retry_worker_db = db_pool.clone();
    tokio::spawn(async move {
        email::provisioning::retry::start_retry_worker(retry_worker_redis, retry_worker_db).await;
    });
    info!("Email provisioning retry worker started");

    // Start email indexer worker in background (Phase 3)
    let indexer_db = db_pool.clone();
    let indexer_search = search_client.clone();
    tokio::spawn(async move {
        email::search::indexer::start_indexer_worker(indexer_db, indexer_search).await;
    });
    info!("Email search indexer worker started");

    // Create API router with all clients
    let app = email::api::create_router(
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
    );

    // Start API server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    info!("Starting Email Service API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
