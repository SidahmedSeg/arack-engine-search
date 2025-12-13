use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub meilisearch_url: String,
    pub meilisearch_key: String,
    pub crawler_max_depth: usize,
    pub crawler_max_concurrent: usize,
    pub database_url: String,
    pub redis_url: String,
    // Rate Limiting (Phase 6.2)
    pub crawler_requests_per_second: u32,
    pub crawler_min_delay_ms: u64,
    pub crawler_max_retries: u32,
    pub crawler_timeout_seconds: u64,
    // User Agent & Headers (Phase 6.3)
    pub crawler_user_agent: String,
    pub crawler_contact_email: Option<String>,
    pub crawler_bot_url: Option<String>,
    pub crawler_accept_language: String,
    // Ory Kratos URLs (Phase 8.6)
    pub kratos_public_url: String,
    pub kratos_admin_url: String,
    // Qdrant Configuration (Phase 10)
    pub qdrant_url: String,
    pub qdrant_collection_name: String,
}

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        Ok(Config {
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            meilisearch_url: env::var("MEILISEARCH_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:7701".to_string()),
            meilisearch_key: env::var("MEILISEARCH_KEY")
                .unwrap_or_else(|_| "masterKey".to_string()),
            crawler_max_depth: env::var("CRAWLER_MAX_DEPTH")
                .unwrap_or_else(|_| "3".to_string())
                .parse()?,
            crawler_max_concurrent: env::var("CRAWLER_MAX_CONCURRENT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5434/engine_search".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            crawler_requests_per_second: env::var("CRAWLER_REQUESTS_PER_SECOND")
                .unwrap_or_else(|_| "2".to_string())
                .parse()?,
            crawler_min_delay_ms: env::var("CRAWLER_MIN_DELAY_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()?,
            crawler_max_retries: env::var("CRAWLER_MAX_RETRIES")
                .unwrap_or_else(|_| "3".to_string())
                .parse()?,
            crawler_timeout_seconds: env::var("CRAWLER_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            crawler_user_agent: env::var("CRAWLER_USER_AGENT")
                .unwrap_or_else(|_| "EngineSearchBot/1.0".to_string()),
            crawler_contact_email: env::var("CRAWLER_CONTACT_EMAIL").ok(),
            crawler_bot_url: env::var("CRAWLER_BOT_URL").ok(),
            crawler_accept_language: env::var("CRAWLER_ACCEPT_LANGUAGE")
                .unwrap_or_else(|_| "en-US,en;q=0.9".to_string()),
            kratos_public_url: env::var("KRATOS_PUBLIC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4433".to_string()),
            kratos_admin_url: env::var("KRATOS_ADMIN_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4434".to_string()),
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:6334".to_string()),
            qdrant_collection_name: env::var("QDRANT_COLLECTION_NAME")
                .unwrap_or_else(|_| "page_embeddings".to_string()),
        })
    }

    pub fn qdrant(&self) -> QdrantConfig {
        QdrantConfig {
            url: self.qdrant_url.clone(),
            collection_name: self.qdrant_collection_name.clone(),
        }
    }
}
