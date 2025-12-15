pub mod cache;
pub mod queue;

pub use cache::*;
pub use queue::*;

use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::Client;
use tracing::info;

/// Create a Redis connection manager
pub async fn create_connection(redis_url: &str) -> Result<ConnectionManager> {
    info!("Creating Redis connection...");

    let client = Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;

    info!("Redis connection established successfully");
    Ok(manager)
}
