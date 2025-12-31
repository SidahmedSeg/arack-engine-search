use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Cache manager for storing and retrieving cached data
#[derive(Clone)]
pub struct CacheManager {
    conn: ConnectionManager,
    default_ttl: usize, // Time to live in seconds
}

impl CacheManager {
    pub fn new(conn: ConnectionManager, default_ttl: usize) -> Self {
        Self { conn, default_ttl }
    }

    /// Set a value in the cache with the default TTL
    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in the cache with a custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        ttl: usize,
    ) -> Result<()> {
        let serialized = serde_json::to_string(value)?;
        self.conn.set_ex(key, serialized, ttl as u64).await?;
        info!("Cached key: {} with TTL: {}s", key, ttl);
        Ok(())
    }

    /// Get a value from the cache
    pub async fn get<T: for<'de> Deserialize<'de>>(&mut self, key: &str) -> Result<Option<T>> {
        match self.conn.get::<_, Option<String>>(key).await {
            Ok(Some(value)) => {
                match serde_json::from_str::<T>(&value) {
                    Ok(deserialized) => {
                        info!("Cache hit for key: {}", key);
                        Ok(Some(deserialized))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached value for key {}: {}", key, e);
                        // Delete corrupted cache entry
                        let _ = self.delete(key).await;
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                info!("Cache miss for key: {}", key);
                Ok(None)
            }
            Err(e) => {
                warn!("Redis get error for key {}: {}", key, e);
                Ok(None)
            }
        }
    }

    /// Delete a value from the cache
    pub async fn delete(&mut self, key: &str) -> Result<()> {
        self.conn.del(key).await?;
        info!("Deleted cache key: {}", key);
        Ok(())
    }

    /// Check if a key exists in the cache
    pub async fn exists(&mut self, key: &str) -> Result<bool> {
        let exists: bool = self.conn.exists(key).await?;
        Ok(exists)
    }

    /// Clear all keys matching a pattern
    pub async fn clear_pattern(&mut self, pattern: &str) -> Result<usize> {
        let keys: Vec<String> = self.conn.keys(pattern).await?;
        if keys.is_empty() {
            return Ok(0);
        }
        let count = keys.len();
        self.conn.del(&keys).await?;
        info!("Cleared {} keys matching pattern: {}", count, pattern);
        Ok(count)
    }

    /// Generate a cache key for search results
    pub fn search_cache_key(query: &str, limit: usize, offset: usize) -> String {
        // Include all search parameters in the cache key
        format!("search:{}:{}:{}", query, limit, offset)
    }

    /// Generate a cache key for stats
    pub fn stats_cache_key() -> String {
        "stats:latest".to_string()
    }

    /// Get a cloned connection for use in other managers (e.g., AnalyticsManager)
    pub async fn get_connection(&self) -> Result<ConnectionManager> {
        Ok(self.conn.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_cache_key() {
        let key = CacheManager::search_cache_key("rust", 20, 0);
        assert_eq!(key, "search:rust:20:0");
    }

    #[test]
    fn test_stats_cache_key() {
        let key = CacheManager::stats_cache_key();
        assert_eq!(key, "stats:latest");
    }
}
