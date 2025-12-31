//! Email Service Binary (Phase 3: Core Email Features)
//!
//! This is the main entry point for the email service.
//! Currently implements:
//! - Stalwart Admin API client (Phase 2) ✅
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
use arack_shared::{config, db, email, ory::KratosClient};

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

    // Initialize Stalwart Admin client for account management (Phase 2)
    let stalwart_admin_user = std::env::var("STALWART_ADMIN_USER")
        .unwrap_or_else(|_| "admin".to_string());
    let stalwart_admin_password = std::env::var("STALWART_ADMIN_PASSWORD")
        .unwrap_or_else(|_| "adminpassword".to_string());
    let stalwart_admin_client = email::stalwart::StalwartAdminClient::new(
        &stalwart_url,
        &stalwart_admin_user,
        &stalwart_admin_password,
    );
    info!("Stalwart Admin client initialized");

    // Default password for new email accounts (should be changed by user)
    let default_email_password = std::env::var("DEFAULT_EMAIL_PASSWORD")
        .unwrap_or_else(|_| "ChangeMe123!".to_string());

    // Initialize Kratos client for session validation
    let kratos_public_url = std::env::var("KRATOS_PUBLIC_URL")
        .unwrap_or_else(|_| "http://kratos:4433".to_string());
    let kratos_admin_url = std::env::var("KRATOS_ADMIN_URL")
        .unwrap_or_else(|_| "http://kratos:4434".to_string());
    let kratos_client = KratosClient::new(kratos_public_url.clone(), kratos_admin_url.clone());
    info!("Kratos client initialized at {}", kratos_public_url);

    // Initialize OAuth Token Manager (Phase 8 - OIDC)
    let hydra_public_url = std::env::var("HYDRA_PUBLIC_URL")
        .unwrap_or_else(|_| "https://auth.arack.io".to_string());
    let oauth_client_id = std::env::var("OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "email-service".to_string());
    let oauth_client_secret = std::env::var("OAUTH_CLIENT_SECRET")
        .unwrap_or_else(|_| "".to_string());
    let oauth_redirect_uri = std::env::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "https://mail.arack.io/oauth/callback".to_string());
    let oauth_token_manager = email::oauth::OAuthTokenManager::new(
        &hydra_public_url,
        &oauth_client_id,
        &oauth_client_secret,
        &oauth_redirect_uri,
        db_pool.clone(),
    )?;
    info!("OAuth Token Manager initialized for client: {}", oauth_client_id);

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

    // Initialize OpenAI client for AI features (Phase 5)
    #[cfg(feature = "email")]
    let openai_api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("OPENAI_API_KEY not set, AI features will not work");
            String::new()
        });

    #[cfg(feature = "email")]
    let openai_client = if !openai_api_key.is_empty() {
        info!("OpenAI client initialized for AI features");
        email::ai::create_openai_client(&openai_api_key)
    } else {
        tracing::warn!("OpenAI client not initialized - missing API key");
        email::ai::create_openai_client("")
    };

    // Create API router with all clients
    #[cfg(feature = "email")]
    let app = email::api::create_router(
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
        stalwart_admin_client,
        default_email_password,
        kratos_client.clone(),
        oauth_token_manager.clone(),
        openai_client,
    );

    #[cfg(not(feature = "email"))]
    let app = email::api::create_router(
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
        stalwart_admin_client,
        default_email_password,
        kratos_client,
        oauth_token_manager,
    );

    // Start API server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    info!("Starting Email Service API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
