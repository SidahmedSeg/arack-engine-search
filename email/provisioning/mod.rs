//! Email Account Provisioning Module (Phase 2)
//!
//! Handles automatic email account creation when users register via Kratos.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

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
/// 2. Creates the account in Stalwart (stub for now)
/// 3. Stores account info in email_accounts table
/// 4. Creates default mailboxes (stub for now)
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

    // Step 1: Create account in Stalwart (stub - will be implemented in Phase 3)
    let stalwart_user_id = create_stalwart_account(email, first_name, last_name).await?;

    info!(
        "Stalwart account created: {} for {}",
        stalwart_user_id, email
    );

    // Step 2: Insert into email_accounts table
    let account_id = create_email_account_record(db_pool, kratos_id, email, &stalwart_user_id)
        .await
        .context("Failed to create email account record")?;

    info!("Email account record created with ID: {}", account_id);

    // Step 3: Create default mailboxes (stub - will be implemented in Phase 3)
    create_default_mailboxes(&stalwart_user_id).await?;

    info!("Default mailboxes created for {}", email);

    // Step 4: Update provisioning log as success
    log_provisioning_completion(db_pool, kratos_id, "create", "success", None).await?;

    Ok(ProvisioningResponse {
        success: true,
        message: Some(format!("Email account provisioned for {}", email)),
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
        INSERT INTO email_provisioning_log (kratos_identity_id, action, status)
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
        INSERT INTO email_provisioning_log
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
        INSERT INTO email_accounts (kratos_identity_id, email_address, stalwart_user_id)
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

/// Create Stalwart account (STUB - Phase 3 implementation)
///
/// In Phase 3, this will make HTTP requests to Stalwart's admin API
/// to create the actual email account.
async fn create_stalwart_account(
    email: &str,
    _first_name: &str,
    _last_name: &str,
) -> Result<String> {
    warn!(
        "Stalwart account creation is stubbed - Phase 3 will implement actual API call for {}",
        email
    );

    // For now, just generate a stub user ID
    let stalwart_user_id = format!("stalwart_{}", uuid::Uuid::new_v4());

    // TODO Phase 3: Implement actual Stalwart API call
    // Example:
    // let client = reqwest::Client::new();
    // let response = client
    //     .post(format!("{}/api/accounts", stalwart_url))
    //     .json(&json!({
    //         "email": email,
    //         "first_name": first_name,
    //         "last_name": last_name,
    //     }))
    //     .send()
    //     .await?;

    Ok(stalwart_user_id)
}

/// Create default mailboxes (STUB - Phase 3 implementation)
///
/// In Phase 3, this will create default mailboxes via JMAP:
/// - Inbox, Sent, Drafts, Trash, Spam
async fn create_default_mailboxes(_stalwart_user_id: &str) -> Result<()> {
    warn!("Default mailbox creation is stubbed - Phase 3 will implement JMAP calls");

    // TODO Phase 3: Implement actual JMAP mailbox creation
    // let jmap_client = JmapClient::new(...);
    // jmap_client.create_mailbox("Inbox", ...).await?;
    // jmap_client.create_mailbox("Sent", ...).await?;
    // etc.

    Ok(())
}

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
