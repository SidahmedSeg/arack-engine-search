use super::models::UserRole;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// Invitation model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub token: String,
    pub invited_by: Uuid,
    pub role: UserRole,
    pub status: InvitationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
}

/// Invitation status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum InvitationStatus {
    #[serde(rename = "pending")]
    #[sqlx(rename = "pending")]
    Pending,
    #[serde(rename = "accepted")]
    #[sqlx(rename = "accepted")]
    Accepted,
    #[serde(rename = "expired")]
    #[sqlx(rename = "expired")]
    Expired,
}

impl std::fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationStatus::Pending => write!(f, "pending"),
            InvitationStatus::Accepted => write!(f, "accepted"),
            InvitationStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Invitation with inviter details
#[derive(Debug, Clone, Serialize)]
pub struct InvitationWithInviter {
    pub id: Uuid,
    pub email: String,
    pub token: String,
    pub role: UserRole,
    pub status: InvitationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub invited_by: InviterInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct InviterInfo {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Create invitation request
#[derive(Debug, Deserialize)]
pub struct CreateInvitationRequest {
    pub email: String,
    pub role: Option<UserRole>,
    pub expires_in_days: Option<i64>,
}

/// Invitation repository
#[derive(Clone)]
pub struct InvitationRepository {
    pool: PgPool,
}

impl InvitationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new invitation
    pub async fn create_invitation(
        &self,
        email: &str,
        invited_by: Uuid,
        role: UserRole,
        expires_in_days: i64,
    ) -> Result<Invitation> {
        // Generate random token
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(expires_in_days);

        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            INSERT INTO invitations (email, token, invited_by, role, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(&token)
        .bind(invited_by)
        .bind(role)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(invitation)
    }

    /// Find invitation by token
    pub async fn find_by_token(&self, token: &str) -> Result<Option<Invitation>> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT * FROM invitations
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(invitation)
    }

    /// List invitations with pagination and filtering
    pub async fn list_invitations(
        &self,
        status: Option<InvitationStatus>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<InvitationWithInviter>, i64)> {
        let offset = (page - 1) * limit;

        // Build the WHERE clause
        let status_clause = if let Some(s) = &status {
            format!(" AND i.status = '{}'", s)
        } else {
            String::new()
        };

        // Query invitations with inviter details
        let query = format!(
            r#"
            SELECT
                i.id, i.email, i.token, i.role, i.status, i.created_at, i.expires_at, i.accepted_at,
                i.invited_by as inviter_id,
                u.email as inviter_email,
                u.first_name as inviter_first_name,
                u.last_name as inviter_last_name
            FROM invitations i
            JOIN users u ON i.invited_by = u.id
            WHERE 1=1 {}
            ORDER BY i.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            status_clause
        );

        let rows = sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let invitations: Vec<InvitationWithInviter> = rows
            .into_iter()
            .map(|row| InvitationWithInviter {
                id: row.get("id"),
                email: row.get("email"),
                token: row.get("token"),
                role: row.get("role"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
                accepted_at: row.get("accepted_at"),
                invited_by: InviterInfo {
                    id: row.get("inviter_id"),
                    email: row.get("inviter_email"),
                    first_name: row.get("inviter_first_name"),
                    last_name: row.get("inviter_last_name"),
                },
            })
            .collect();

        // Count total
        let count_query = format!(
            r#"
            SELECT COUNT(*) FROM invitations i
            WHERE 1=1 {}
            "#,
            status_clause
        );

        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await?;

        Ok((invitations, total))
    }

    /// Mark invitation as accepted
    pub async fn mark_as_accepted(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE invitations
            SET status = 'accepted', accepted_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete invitation
    pub async fn delete_invitation(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM invitations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Cleanup expired invitations (mark as expired)
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET status = 'expired'
            WHERE status = 'pending' AND expires_at < CURRENT_TIMESTAMP
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }
}
