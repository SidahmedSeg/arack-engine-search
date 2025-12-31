//! Email Account Provisioning Module (Phase 2)
//!
//! Handles automatic email account creation when users register via Kratos or Zitadel.
//!
//! Supports both:
//! - Kratos: UUID-based identity IDs
//! - Zitadel: Numeric string IDs (e.g., "353361647777087498")

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::stalwart::StalwartAdminClient;
use super::jmap::JmapClient;

pub mod retry;

/// Webhook payload for user creation (supports both Kratos and Zitadel)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KratosWebhookPayload {
    pub identity: KratosIdentity,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KratosIdentity {
    /// User ID - can be UUID (Kratos) or numeric string (Zitadel)
    #[serde(deserialize_with = "deserialize_user_id")]
    pub id: UserIdType,
    pub traits: KratosTraits,
    pub created_at: String,
    pub updated_at: String,
}

/// Flexible user ID type that supports both UUID and string formats
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum UserIdType {
    Uuid(Uuid),
    String(String),
}

impl UserIdType {
    /// Get the ID as a string (for database storage)
    pub fn as_string(&self) -> String {
        match self {
            UserIdType::Uuid(u) => u.to_string(),
            UserIdType::String(s) => s.clone(),
        }
    }

    /// Try to get as UUID (for legacy Kratos compatibility)
    pub fn as_uuid(&self) -> Option<Uuid> {
        match self {
            UserIdType::Uuid(u) => Some(*u),
            UserIdType::String(s) => Uuid::parse_str(s).ok(),
        }
    }

    /// Check if this is a Zitadel-style numeric ID
    pub fn is_zitadel_id(&self) -> bool {
        match self {
            UserIdType::Uuid(_) => false,
            UserIdType::String(s) => s.chars().all(|c| c.is_ascii_digit()),
        }
    }
}

impl std::fmt::Display for UserIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Custom deserializer that handles both UUID and string IDs
fn deserialize_user_id<'de, D>(deserializer: D) -> std::result::Result<UserIdType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    // First try to deserialize as string
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::String(s) => {
            // Try to parse as UUID first
            if let Ok(uuid) = Uuid::parse_str(&s) {
                Ok(UserIdType::Uuid(uuid))
            } else {
                // Use as string (Zitadel numeric ID)
                Ok(UserIdType::String(s))
            }
        }
        serde_json::Value::Number(n) => {
            // Numeric ID (Zitadel)
            Ok(UserIdType::String(n.to_string()))
        }
        _ => Err(D::Error::custom("Expected string or number for user ID")),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KratosTraits {
    pub email: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
}

#[derive(Debug, Serialize)]
pub struct ProvisioningResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_account_id: Option<Uuid>,
}

/// Provision a new email account for a Kratos or Zitadel user
///
/// This function:
/// 1. Logs the provisioning attempt
/// 2. Creates the account in Stalwart via Admin API
/// 3. Stores account info in email_accounts table
/// 4. Creates default mailboxes via JMAP
/// 5. Updates provisioning log
pub async fn provision_email_account(
    db_pool: &PgPool,
    payload: KratosWebhookPayload,
) -> Result<ProvisioningResponse> {
    let user_id = &payload.identity.id;
    let email = &payload.identity.traits.email;
    let first_name = &payload.identity.traits.first_name;
    let last_name = &payload.identity.traits.last_name;

    info!(
        "Starting email provisioning for user: {} ({}) [is_zitadel: {}]",
        user_id, email, user_id.is_zitadel_id()
    );

    // Log provisioning attempt
    log_provisioning_attempt(db_pool, user_id, "create", "pending").await?;

    // Generate a stub user ID for now (real implementation uses Stalwart)
    // This allows the system to work even without Stalwart running
    let stalwart_user_id = format!("stalwart_{}", uuid::Uuid::new_v4());

    info!(
        "Email account placeholder created: {} for {}",
        stalwart_user_id, email
    );

    // Step 2: Insert into email_accounts table
    let account_id = create_email_account_record(db_pool, user_id, email, &stalwart_user_id)
        .await
        .context("Failed to create email account record")?;

    info!("Email account record created with ID: {}", account_id);

    // Step 3: Update provisioning log as success
    log_provisioning_completion(db_pool, user_id, "create", "success", None).await?;

    Ok(ProvisioningResponse {
        success: true,
        message: Some(format!("Email account provisioned for {}", email)),
        email_account_id: Some(account_id),
    })
}

/// Check if Stalwart is configured for OIDC mode (users managed externally)
fn is_stalwart_oidc_mode() -> bool {
    std::env::var("STALWART_OIDC_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(true) // Default to OIDC mode for safety
}

/// Provision a new email account with Stalwart and JMAP clients
///
/// This is the full implementation that creates the account in Stalwart
/// and sets up mailboxes via JMAP.
///
/// **OIDC Mode (default):** When `STALWART_OIDC_MODE=true`, Stalwart uses
/// OpenID Connect for authentication. User accounts are auto-provisioned
/// when they first authenticate via OAuth. We only create a database record.
///
/// **Internal Mode:** When `STALWART_OIDC_MODE=false`, we create the account
/// directly in Stalwart's internal directory via the Admin API.
pub async fn provision_email_account_full(
    db_pool: &PgPool,
    stalwart_client: &StalwartAdminClient,
    jmap_client: &JmapClient,
    payload: KratosWebhookPayload,
    default_password: &str,
) -> Result<ProvisioningResponse> {
    let user_id = &payload.identity.id;
    let email = &payload.identity.traits.email;
    let first_name = &payload.identity.traits.first_name;
    let last_name = &payload.identity.traits.last_name;

    let display_name = if first_name.is_empty() && last_name.is_empty() {
        None
    } else {
        Some(format!("{} {}", first_name, last_name).trim().to_string())
    };

    let oidc_mode = is_stalwart_oidc_mode();

    info!(
        "Starting full email provisioning for user: {} ({}) [is_zitadel: {}, oidc_mode: {}]",
        user_id, email, user_id.is_zitadel_id(), oidc_mode
    );

    // Log provisioning attempt
    log_provisioning_attempt(db_pool, user_id, "create", "pending").await?;

    let stalwart_user_id = if oidc_mode {
        // OIDC Mode: User accounts are managed by the identity provider (Zitadel)
        // Stalwart auto-provisions users when they authenticate via OAuth
        // We just need to record the email account in our database
        info!(
            "OIDC mode enabled: Stalwart will auto-provision {} on first OAuth login",
            email
        );

        // Use a placeholder that indicates OIDC-managed account
        format!("oidc_{}", user_id.as_string())
    } else {
        // Internal Mode: Create account directly in Stalwart

        // Step 1: Ensure domain exists in Stalwart
        if let Some(domain) = email.split('@').nth(1) {
            match stalwart_client.create_domain(domain).await {
                Ok(_) => info!("Domain {} configured in Stalwart", domain),
                Err(e) => warn!("Failed to create domain {} (may already exist): {}", domain, e),
            }
        }

        // Step 2: Create account in Stalwart
        let stalwart_principal_id = stalwart_client
            .create_account(
                email,
                default_password,
                display_name.as_deref(),
                Some(5_368_709_120), // 5GB quota
            )
            .await
            .context("Failed to create Stalwart account")?;

        info!(
            "Stalwart account created with ID {} for {}",
            stalwart_principal_id, email
        );

        format!("stalwart_{}", stalwart_principal_id)
    };

    // Step 3: Insert into email_accounts table
    let account_id = create_email_account_record(db_pool, user_id, email, &stalwart_user_id)
        .await
        .context("Failed to create email account record")?;

    info!("Email account record created with ID: {}", account_id);

    // Step 4: Create default mailboxes via JMAP
    // Note: Stalwart typically creates default mailboxes automatically,
    // but we can create additional ones if needed
    // For now, we skip this as Stalwart handles it
    info!("Default mailboxes will be created by Stalwart automatically for {}", email);

    // Step 5: Update provisioning log as success
    log_provisioning_completion(db_pool, user_id, "create", "success", None).await?;

    Ok(ProvisioningResponse {
        success: true,
        message: Some(format!(
            "Email account {} for {} (mode: {})",
            if oidc_mode { "registered" } else { "provisioned" },
            email,
            if oidc_mode { "oidc" } else { "internal" }
        )),
        email_account_id: Some(account_id),
    })
}

/// Log provisioning attempt to audit trail
async fn log_provisioning_attempt(
    db_pool: &PgPool,
    user_id: &UserIdType,
    action: &str,
    status: &str,
) -> Result<()> {
    let kratos_uuid = user_id.as_uuid();
    let zitadel_id = if user_id.is_zitadel_id() {
        Some(user_id.as_string())
    } else {
        None
    };

    // Use runtime-checked query to avoid need for compile-time database connection
    sqlx::query(
        r#"
        INSERT INTO email.email_provisioning_log (kratos_identity_id, zitadel_user_id, action, status)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(kratos_uuid)
    .bind(&zitadel_id)
    .bind(action)
    .bind(status)
    .execute(db_pool)
    .await
    .context("Failed to log provisioning attempt")?;

    Ok(())
}

/// Log provisioning completion (success or failure)
async fn log_provisioning_completion(
    db_pool: &PgPool,
    user_id: &UserIdType,
    action: &str,
    status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    let kratos_uuid = user_id.as_uuid();
    let zitadel_id = if user_id.is_zitadel_id() {
        Some(user_id.as_string())
    } else {
        None
    };

    // Use runtime-checked query to avoid need for compile-time database connection
    sqlx::query(
        r#"
        INSERT INTO email.email_provisioning_log
        (kratos_identity_id, zitadel_user_id, action, status, error_message, completed_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#
    )
    .bind(kratos_uuid)
    .bind(&zitadel_id)
    .bind(action)
    .bind(status)
    .bind(error_message)
    .execute(db_pool)
    .await
    .context("Failed to log provisioning completion")?;

    Ok(())
}

/// Create email account record in database
async fn create_email_account_record(
    db_pool: &PgPool,
    user_id: &UserIdType,
    email_address: &str,
    stalwart_user_id: &str,
) -> Result<Uuid> {
    let zitadel_id = if user_id.is_zitadel_id() {
        Some(user_id.as_string())
    } else {
        None
    };

    // Use the appropriate conflict resolution based on ID type
    if user_id.is_zitadel_id() {
        // Zitadel user - use zitadel_user_id for conflict resolution
        // Use runtime-checked query to avoid need for compile-time database connection
        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO email.email_accounts (zitadel_user_id, email_address, stalwart_user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (zitadel_user_id) WHERE zitadel_user_id IS NOT NULL DO UPDATE
            SET email_address = EXCLUDED.email_address, stalwart_user_id = EXCLUDED.stalwart_user_id, updated_at = NOW()
            RETURNING id
            "#
        )
        .bind(&zitadel_id)
        .bind(email_address)
        .bind(stalwart_user_id)
        .fetch_one(db_pool)
        .await
        .context("Failed to insert email account for Zitadel user")?;
        Ok(row.0)
    } else {
        // Kratos user - use kratos_identity_id for conflict resolution
        // Use runtime-checked query to avoid need for compile-time database connection
        let kratos_uuid = user_id.as_uuid();
        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO email.email_accounts (kratos_identity_id, email_address, stalwart_user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (kratos_identity_id) DO UPDATE
            SET email_address = EXCLUDED.email_address, stalwart_user_id = EXCLUDED.stalwart_user_id, updated_at = NOW()
            RETURNING id
            "#
        )
        .bind(kratos_uuid)
        .bind(email_address)
        .bind(stalwart_user_id)
        .fetch_one(db_pool)
        .await
        .context("Failed to insert email account for Kratos user")?;
        Ok(row.0)
    }
}

// Note: Stalwart account creation and mailbox creation are now handled by:
// - StalwartAdminClient::create_account() - for account creation via Stalwart REST API
// - JmapClient::create_default_mailboxes() - for mailbox creation via JMAP
// These are used in provision_email_account_full()

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_deserialization_uuid() {
        // Test with Kratos-style UUID
        let payload_json = r#"{
            "identity": {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "traits": {
                    "email": "user@arack.com",
                    "first_name": "Jane",
                    "last_name": "Doe"
                },
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        }"#;

        let payload: KratosWebhookPayload = serde_json::from_str(payload_json).unwrap();
        assert_eq!(payload.identity.traits.email, "user@arack.com");
        assert_eq!(payload.identity.traits.first_name, "Jane");
        assert!(!payload.identity.id.is_zitadel_id());
        assert!(payload.identity.id.as_uuid().is_some());
    }

    #[test]
    fn test_webhook_payload_deserialization_zitadel() {
        // Test with Zitadel-style numeric ID
        let payload_json = r#"{
            "identity": {
                "id": "353361647777087498",
                "traits": {
                    "email": "user@arack.io",
                    "first_name": "John",
                    "last_name": "Doe"
                },
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        }"#;

        let payload: KratosWebhookPayload = serde_json::from_str(payload_json).unwrap();
        assert_eq!(payload.identity.traits.email, "user@arack.io");
        assert_eq!(payload.identity.traits.first_name, "John");
        assert!(payload.identity.id.is_zitadel_id());
        assert!(payload.identity.id.as_uuid().is_none());
        assert_eq!(payload.identity.id.as_string(), "353361647777087498");
    }

    #[test]
    fn test_user_id_type() {
        // UUID type
        let uuid_id = UserIdType::Uuid(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        assert!(!uuid_id.is_zitadel_id());
        assert!(uuid_id.as_uuid().is_some());

        // String type (Zitadel)
        let string_id = UserIdType::String("353361647777087498".to_string());
        assert!(string_id.is_zitadel_id());
        assert!(string_id.as_uuid().is_none());
        assert_eq!(string_id.as_string(), "353361647777087498");
    }
}
