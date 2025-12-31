use super::models::{User, UserRole};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// User repository for database operations
#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        first_name: &str,
        last_name: &str,
        role: UserRole,
        invited_by: Option<Uuid>,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, first_name, last_name, role, invited_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(first_name)
        .bind(last_name)
        .bind(role)
        .bind(invited_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE email = $1 AND is_active = true
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Update last login timestamp
    pub async fn update_last_login(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET last_login = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update user password
    pub async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update user active status
    pub async fn update_active_status(&self, id: Uuid, is_active: bool) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET is_active = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(is_active)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update user role
    pub async fn update_role(&self, id: Uuid, role: UserRole) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET role = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(role)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List all users with pagination
    pub async fn list_users(
        &self,
        role: Option<UserRole>,
        is_active: Option<bool>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<User>, i64)> {
        let offset = (page - 1) * limit;

        // Build query conditionally
        let mut query = String::from("SELECT * FROM users WHERE 1=1");
        let mut count_query = String::from("SELECT COUNT(*) FROM users WHERE 1=1");

        if role.is_some() {
            query.push_str(" AND role = $1");
            count_query.push_str(" AND role = $1");
        }
        if is_active.is_some() {
            let param_num = if role.is_some() { "$2" } else { "$1" };
            query.push_str(&format!(" AND is_active = {}", param_num));
            count_query.push_str(&format!(" AND is_active = {}", param_num));
        }

        query.push_str(" ORDER BY created_at DESC");

        // Add pagination
        let limit_offset_start = if role.is_some() && is_active.is_some() {
            "$3"
        } else if role.is_some() || is_active.is_some() {
            "$2"
        } else {
            "$1"
        };
        let limit_offset_end = if role.is_some() && is_active.is_some() {
            "$4"
        } else if role.is_some() || is_active.is_some() {
            "$3"
        } else {
            "$2"
        };

        query.push_str(&format!(" LIMIT {} OFFSET {}", limit_offset_start, limit_offset_end));

        // Execute queries
        let mut users_query = sqlx::query_as::<_, User>(&query);
        let mut total_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(r) = role {
            users_query = users_query.bind(r.clone());
            total_query = total_query.bind(r);
        }
        if let Some(active) = is_active {
            users_query = users_query.bind(active);
            total_query = total_query.bind(active);
        }

        users_query = users_query.bind(limit).bind(offset);

        let users = users_query.fetch_all(&self.pool).await?;
        let total = total_query.fetch_one(&self.pool).await?;

        Ok((users, total))
    }

    /// Update user (generic update method)
    pub async fn update_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET first_name = $1, last_name = $2, role = $3, is_active = $4, updated_at = CURRENT_TIMESTAMP
            WHERE id = $5
            "#,
        )
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.role)
        .bind(user.is_active)
        .bind(user.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a user
    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if email exists
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
