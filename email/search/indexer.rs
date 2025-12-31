//! Email Indexer Worker (Phase 3)
//!
//! Background worker that indexes emails to Meilisearch for search functionality.

use anyhow::Result;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

use super::{EmailDocument, EmailSearchClient};

/// Start the background email indexer worker
///
/// This worker runs every 60 seconds and indexes emails that haven't been
/// indexed yet (indexed_in_meilisearch = false).
pub async fn start_indexer_worker(db_pool: PgPool, search_client: EmailSearchClient) {
    info!("Starting email indexer worker...");

    let mut tick_interval = interval(Duration::from_secs(60));

    loop {
        tick_interval.tick().await;

        match process_unindexed_emails(&db_pool, &search_client).await {
            Ok(count) => {
                if count > 0 {
                    info!("Indexed {} emails to Meilisearch", count);
                }
            }
            Err(e) => {
                error!("Error processing unindexed emails: {}", e);
            }
        }
    }
}

/// Process emails that need to be indexed
async fn process_unindexed_emails(
    db_pool: &PgPool,
    search_client: &EmailSearchClient,
) -> Result<usize> {
    // Fetch unindexed emails from database
    let unindexed_emails = sqlx::query!(
        r#"
        SELECT
            id,
            account_id,
            jmap_id,
            subject,
            from_address,
            from_name,
            to_addresses,
            cc_addresses,
            body_preview,
            EXTRACT(EPOCH FROM received_at)::bigint as "received_at!",
            has_attachments,
            is_read,
            is_starred,
            mailbox_ids,
            keywords
        FROM email.email_metadata
        WHERE indexed_in_meilisearch = false
        LIMIT 100
        "#
    )
    .fetch_all(db_pool)
    .await?;

    if unindexed_emails.is_empty() {
        return Ok(0);
    }

    let count = unindexed_emails.len();

    // Convert to EmailDocument and index in batch
    let mut email_documents = Vec::new();

    for email in &unindexed_emails {
        let email_doc = EmailDocument {
            id: email.id.to_string(),
            account_id: email.account_id.to_string(),
            jmap_id: email.jmap_id.clone(),
            subject: email.subject.clone().unwrap_or_default(),
            from_address: email.from_address.clone().unwrap_or_default(),
            from_name: email.from_name.clone(),
            to_addresses: email.to_addresses.clone().unwrap_or_default(),
            cc_addresses: email.cc_addresses.clone().unwrap_or_default(),
            body_preview: email.body_preview.clone().unwrap_or_default(),
            received_at: email.received_at,
            has_attachments: email.has_attachments.unwrap_or(false),
            is_read: email.is_read.unwrap_or(false),
            is_starred: email.is_starred.unwrap_or(false),
            mailbox_ids: email.mailbox_ids.clone().unwrap_or_default(),
            keywords: email.keywords.clone().unwrap_or_default(),
        };

        email_documents.push(email_doc);
    }

    // Index all documents in batch
    search_client.index_emails_batch(email_documents).await?;

    // Mark emails as indexed in database
    for email in &unindexed_emails {
        sqlx::query!(
            r#"
            UPDATE email.email_metadata
            SET indexed_in_meilisearch = true, indexed_at = NOW()
            WHERE id = $1
            "#,
            email.id
        )
        .execute(db_pool)
        .await?;
    }

    Ok(count)
}

/// Index a single email immediately (used when new email arrives)
pub async fn index_email_immediately(
    db_pool: &PgPool,
    search_client: &EmailSearchClient,
    email_id: uuid::Uuid,
) -> Result<()> {
    let email = sqlx::query!(
        r#"
        SELECT
            id,
            account_id,
            jmap_id,
            subject,
            from_address,
            from_name,
            to_addresses,
            cc_addresses,
            body_preview,
            EXTRACT(EPOCH FROM received_at)::bigint as "received_at!",
            has_attachments,
            is_read,
            is_starred,
            mailbox_ids,
            keywords
        FROM email.email_metadata
        WHERE id = $1
        "#,
        email_id
    )
    .fetch_one(db_pool)
    .await?;

    let email_doc = EmailDocument {
        id: email.id.to_string(),
        account_id: email.account_id.to_string(),
        jmap_id: email.jmap_id.clone(),
        subject: email.subject.clone().unwrap_or_default(),
        from_address: email.from_address.clone().unwrap_or_default(),
        from_name: email.from_name.clone(),
        to_addresses: email.to_addresses.clone().unwrap_or_default(),
        cc_addresses: email.cc_addresses.clone().unwrap_or_default(),
        body_preview: email.body_preview.clone().unwrap_or_default(),
        received_at: email.received_at,
        has_attachments: email.has_attachments.unwrap_or(false),
        is_read: email.is_read.unwrap_or(false),
        is_starred: email.is_starred.unwrap_or(false),
        mailbox_ids: email.mailbox_ids.clone().unwrap_or_default(),
        keywords: email.keywords.clone().unwrap_or_default(),
    };

    search_client.index_email(&email_doc).await?;

    // Mark as indexed
    sqlx::query!(
        r#"
        UPDATE email.email_metadata
        SET indexed_in_meilisearch = true, indexed_at = NOW()
        WHERE id = $1
        "#,
        email_id
    )
    .execute(db_pool)
    .await?;

    info!("Immediately indexed email: {}", email_id);

    Ok(())
}
