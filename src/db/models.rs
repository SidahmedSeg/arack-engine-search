use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Collection represents a group of related crawl jobs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub url_pattern: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CreateCollection request
#[derive(Debug, Deserialize)]
pub struct CreateCollection {
    pub name: String,
    pub description: Option<String>,
    pub url_pattern: Option<String>,
}

/// UpdateCollection request
#[derive(Debug, Deserialize)]
pub struct UpdateCollection {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url_pattern: Option<String>,
}

/// CrawlHistory tracks crawl job history
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CrawlHistory {
    pub id: Uuid,
    pub collection_id: Option<Uuid>,
    pub urls: Vec<String>,
    pub status: String,
    pub pages_crawled: i32,
    pub pages_indexed: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}

/// CreateCrawlHistory request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCrawlHistory {
    pub collection_id: Option<Uuid>,
    pub urls: Vec<String>,
}

/// CrawlError represents an error during crawling
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CrawlError {
    pub id: Uuid,
    pub crawl_id: Uuid,
    pub url: String,
    pub error_type: String,
    pub error_message: Option<String>,
    pub occurred_at: DateTime<Utc>,
}
