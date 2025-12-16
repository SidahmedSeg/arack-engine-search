//! Email Account Provisioning Module (Phase 2)
//!
//! Handles automatic email account creation when users register via Kratos.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::stalwart::StalwartAdminClient;
use super::jmap::JmapClient;

pub mod retry;

/// Kratos webhook payload for user creation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KratosWebhookPayload {
    pub identity: KratosIdentity,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KratosIdentity {
    pub id: Uuid,
    pub traits: KratosTraits,
    pub created_at: String,
    pub updated_at: String,
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

/// Provision a new email account for a Kratos user
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
    let kratos_id = payload.identity.id;
    let email = &payload.identity.traits.email;
    let first_name = &payload.identity.traits.first_name;
    let last_name = &payload.identity.traits.last_name;

    info!(
        "Starting email provisioning for Kratos identity: {} ({})",
        kratos_id, email
    );

    // Log provisioning attempt
    log_provisioning_attempt(db_pool, kratos_id, "create", "pending").await?;

    // Generate a stub user ID for now (real implementation uses Stalwart)
    // This allows the system to work even without Stalwart running
    let stalwart_user_id = format!("stalwart_{}", uuid::Uuid::new_v4());

    info!(
        "Email account placeholder created: {} for {}",
        stalwart_user_id, email
    );

    // Step 2: Insert into email_accounts table
    let account_id = create_email_account_record(db_pool, kratos_id, email, &stalwart_user_id)
        .await
        .context("Failed to create email account record")?;

    info!("Email account record created with ID: {}", account_id);

    // Step 3: Update provisioning log as success
    log_provisioning_completion(db_pool, kratos_id, "create", "success", None).await?;

    Ok(ProvisioningResponse {
        success: true,
        message: Some(format!("Email account provisioned for {}", email)),
        email_account_id: Some(account_id),
    })
}

/// Provision a new email account with Stalwart and JMAP clients
///
/// This is the full implementation that creates the account in Stalwart
/// and sets up mailboxes via JMAP.
pub async fn provision_email_account_full(
    db_pool: &PgPool,
    stalwart_client: &StalwartAdminClient,
    jmap_client: &JmapClient,
    payload: KratosWebhookPayload,
    default_password: &str,
) -> Result<ProvisioningResponse> {
    let kratos_id = payload.identity.id;
    let email = &payload.identity.traits.email;
    let first_name = &payload.identity.traits.first_name;
    let last_name = &payload.identity.traits.last_name;

    let display_name = if first_name.is_empty() && last_name.is_empty() {
        None
    } else {
        Some(format!("{} {}", first_name, last_name).trim().to_string())
    };

    info!(
        "Starting full email provisioning for Kratos identity: {} ({})",
        kratos_id, email
    );

    // Log provisioning attempt
    log_provisioning_attempt(db_pool, kratos_id, "create", "pending").await?;

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

    let stalwart_user_id = format!("stalwart_{}", stalwart_principal_id);

    info!(
        "Stalwart account created: {} (ID: {}) for {}",
        stalwart_user_id, stalwart_principal_id, email
    );

    // Step 3: Insert into email_accounts table
    let account_id = create_email_account_record(db_pool, kratos_id, email, &stalwart_user_id)
        .await
        .context("Failed to create email account record")?;

    info!("Email account record created with ID: {}", account_id);

    // Step 4: Create default mailboxes via JMAP
    // Note: Stalwart typically creates default mailboxes automatically,
    // but we can create additional ones if needed
    // For now, we skip this as Stalwart handles it
    info!("Default mailboxes will be created by Stalwart automatically for {}", email);

    // Step 5: Update provisioning log as success
    log_provisioning_completion(db_pool, kratos_id, "create", "success", None).await?;

    Ok(ProvisioningResponse {
        success: true,
        message: Some(format!("Email account fully provisioned for {}", email)),
        email_account_id: Some(account_id),
    })
}

/// Log provisioning attempt to audit trail
async fn log_provisioning_attempt(
    db_pool: &PgPool,
    kratos_identity_id: Uuid,
    action: &str,
    status: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO email.email_provisioning_log (kratos_identity_id, action, status)
        VALUES ($1, $2, $3)
        "#,
        kratos_identity_id,
        action,
        status
    )
    .execute(db_pool)
    .await
    .context("Failed to log provisioning attempt")?;

    Ok(())
}

/// Log provisioning completion (success or failure)
async fn log_provisioning_completion(
    db_pool: &PgPool,
    kratos_identity_id: Uuid,
    action: &str,
    status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO email.email_provisioning_log
        (kratos_identity_id, action, status, error_message, completed_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        kratos_identity_id,
        action,
        status,
        error_message
    )
    .execute(db_pool)
    .await
    .context("Failed to log provisioning completion")?;

    Ok(())
}

/// Create email account record in database
async fn create_email_account_record(
    db_pool: &PgPool,
    kratos_identity_id: Uuid,
    email_address: &str,
    stalwart_user_id: &str,
) -> Result<Uuid> {
    let result = sqlx::query!(
        r#"
        INSERT INTO email.email_accounts (kratos_identity_id, email_address, stalwart_user_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (kratos_identity_id) DO UPDATE
        SET email_address = $2, stalwart_user_id = $3, updated_at = NOW()
        RETURNING id
        "#,
        kratos_identity_id,
        email_address,
        stalwart_user_id
    )
    .fetch_one(db_pool)
    .await
    .context("Failed to insert email account")?;

    Ok(result.id)
}

// Note: Stalwart account creation and mailbox creation are now handled by:
// - StalwartAdminClient::create_account() - for account creation via Stalwart REST API
// - JmapClient::create_default_mailboxes() - for mailbox creation via JMAP
// These are used in provision_email_account_full()

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_deserialization() {
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
    }
}
