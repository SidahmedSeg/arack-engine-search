// Phase 8.6: Ory User Features Repository
// This module handles all database operations for user features

use anyhow::Result;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use super::models::{
    UserPreferences, SavedSearch, SearchHistory,
    CreateSavedSearchRequest, UpdatePreferencesRequest,
    TrackSearchRequest,
};

/// Repository for Ory user features (preferences, saved searches, history)
#[derive(Clone)]
pub struct OryUserRepository {
    pool: PgPool,
}

impl OryUserRepository {
    /// Create a new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ===== USER PREFERENCES =====

    /// Get or create user preferences
    ///
    /// If preferences don't exist, creates them with default values.
    /// This is called automatically on first access.
    pub async fn get_or_create_preferences(&self, kratos_id: Uuid) -> Result<UserPreferences> {
        // Try to get existing preferences
        let existing = sqlx::query_as::<_, UserPreferences>(
            "SELECT * FROM user_preferences WHERE kratos_identity_id = $1"
        )
        .bind(kratos_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(prefs) = existing {
            return Ok(prefs);
        }

        // Create new with defaults
        let prefs = sqlx::query_as::<_, UserPreferences>(
            r#"
            INSERT INTO user_preferences (kratos_identity_id, theme, results_per_page, analytics_opt_out)
            VALUES ($1, 'light', 20, false)
            RETURNING *
            "#
        )
        .bind(kratos_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(prefs)
    }

    /// Update user preferences
    ///
    /// Only updates fields that are provided in the request.
    /// Returns the updated preferences.
    pub async fn update_preferences(
        &self,
        kratos_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> Result<UserPreferences> {
        // Ensure preferences exist
        self.get_or_create_preferences(kratos_id).await?;

        // Build dynamic update query based on provided fields
        let mut updates = vec![];
        let mut param_count = 2;

        if req.theme.is_some() {
            updates.push(format!("theme = ${}", param_count));
            param_count += 1;
        }
        if req.results_per_page.is_some() {
            updates.push(format!("results_per_page = ${}", param_count));
            param_count += 1;
        }
        if req.analytics_opt_out.is_some() {
            updates.push(format!("analytics_opt_out = ${}", param_count));
        }

        if updates.is_empty() {
            // No updates requested, just return current preferences
            return self.get_or_create_preferences(kratos_id).await;
        }

        let query = format!(
            "UPDATE user_preferences SET {} WHERE kratos_identity_id = $1 RETURNING *",
            updates.join(", ")
        );

        // Execute update with provided values
        let mut query_builder = sqlx::query_as::<_, UserPreferences>(&query).bind(kratos_id);

        if let Some(theme) = req.theme {
            query_builder = query_builder.bind(theme);
        }
        if let Some(rpp) = req.results_per_page {
            query_builder = query_builder.bind(rpp);
        }
        if let Some(opt_out) = req.analytics_opt_out {
            query_builder = query_builder.bind(opt_out);
        }

        let prefs = query_builder.fetch_one(&self.pool).await?;
        Ok(prefs)
    }

    // ===== SAVED SEARCHES =====

    /// Create a new saved search
    pub async fn create_saved_search(
        &self,
        kratos_id: Uuid,
        req: CreateSavedSearchRequest,
    ) -> Result<SavedSearch> {
        // Ensure user_preferences exists (required for foreign key)
        self.get_or_create_preferences(kratos_id).await?;

        let search = sqlx::query_as::<_, SavedSearch>(
            r#"
            INSERT INTO saved_searches (kratos_identity_id, name, query, filters)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(kratos_id)
        .bind(req.name)
        .bind(req.query)
        .bind(req.filters)
        .fetch_one(&self.pool)
        .await?;

        Ok(search)
    }

    /// List all saved searches for a user
    pub async fn list_saved_searches(&self, kratos_id: Uuid) -> Result<Vec<SavedSearch>> {
        let searches = sqlx::query_as::<_, SavedSearch>(
            "SELECT * FROM saved_searches WHERE kratos_identity_id = $1 ORDER BY created_at DESC"
        )
        .bind(kratos_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(searches)
    }

    /// Get a specific saved search
    pub async fn get_saved_search(&self, kratos_id: Uuid, search_id: Uuid) -> Result<Option<SavedSearch>> {
        let search = sqlx::query_as::<_, SavedSearch>(
            "SELECT * FROM saved_searches WHERE id = $1 AND kratos_identity_id = $2"
        )
        .bind(search_id)
        .bind(kratos_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(search)
    }

    /// Delete a saved search
    ///
    /// Only the owner can delete their own searches.
    pub async fn delete_saved_search(&self, kratos_id: Uuid, search_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM saved_searches WHERE id = $1 AND kratos_identity_id = $2"
        )
        .bind(search_id)
        .bind(kratos_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ===== SEARCH HISTORY =====

    /// Track a search query
    ///
    /// Records the search query and its results for analytics and history.
    pub async fn track_search(
        &self,
        kratos_id: Uuid,
        req: TrackSearchRequest,
    ) -> Result<SearchHistory> {
        // Ensure user_preferences exists (required for foreign key)
        self.get_or_create_preferences(kratos_id).await?;

        let history = sqlx::query_as::<_, SearchHistory>(
            r#"
            INSERT INTO search_history (kratos_identity_id, query, filters, result_count)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(kratos_id)
        .bind(req.query)
        .bind(req.filters)
        .bind(req.result_count)
        .fetch_one(&self.pool)
        .await?;

        Ok(history)
    }

    /// Update search history with click information
    ///
    /// When a user clicks a search result, this records which result was clicked.
    pub async fn track_click(
        &self,
        kratos_id: Uuid,
        history_id: Uuid,
        clicked_url: String,
        clicked_position: i32,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE search_history
            SET clicked_url = $3, clicked_position = $4
            WHERE id = $1 AND kratos_identity_id = $2
            "#
        )
        .bind(history_id)
        .bind(kratos_id)
        .bind(clicked_url)
        .bind(clicked_position)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get search history for a user
    ///
    /// Returns the most recent searches up to the specified limit.
    pub async fn get_search_history(
        &self,
        kratos_id: Uuid,
        limit: i64,
    ) -> Result<Vec<SearchHistory>> {
        let history = sqlx::query_as::<_, SearchHistory>(
            r#"
            SELECT * FROM search_history
            WHERE kratos_identity_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(kratos_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// Delete old search history
    ///
    /// Removes search history older than the specified number of days.
    /// Useful for privacy compliance and database maintenance.
    pub async fn delete_old_history(&self, kratos_id: Uuid, days: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM search_history
            WHERE kratos_identity_id = $1
            AND created_at < NOW() - INTERVAL '1 day' * $2
            "#
        )
        .bind(kratos_id)
        .bind(days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get total count of saved searches
    pub async fn count_saved_searches(&self, kratos_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM saved_searches WHERE kratos_identity_id = $1"
        )
        .bind(kratos_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Get total count of search history entries
    pub async fn count_search_history(&self, kratos_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM search_history WHERE kratos_identity_id = $1"
        )
        .bind(kratos_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_creation() {
        // This is a placeholder test - actual tests would require a test database
        // In a real implementation, you'd use sqlx::test or similar
        assert!(true);
    }
}
