//! Email Search Module (Phase 3)
//!
//! Meilisearch integration for indexing and searching emails.

use anyhow::{Context, Result};
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use tracing::{debug, info};
use uuid::Uuid;

pub mod indexer;

/// Email document for Meilisearch indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDocument {
    pub id: String,
    pub account_id: String,
    pub jmap_id: String,
    pub subject: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub body_preview: String,
    pub received_at: i64, // Unix timestamp for sorting
    pub has_attachments: bool,
    pub is_read: bool,
    pub is_starred: bool,
    pub mailbox_ids: Vec<String>,
    pub keywords: Vec<String>,
}

/// Email search client
#[derive(Clone)]
pub struct EmailSearchClient {
    meilisearch: Client,
    index_name: String,
}

impl EmailSearchClient {
    /// Create a new email search client
    pub fn new(meilisearch: Client, index_name: &str) -> Self {
        Self {
            meilisearch,
            index_name: index_name.to_string(),
        }
    }

    /// Initialize the email search index with proper settings
    pub async fn initialize_index(&self) -> Result<()> {
        let index = self.meilisearch.index(&self.index_name);

        // Configure searchable attributes
        index
            .set_searchable_attributes(&[
                "subject",
                "from_name",
                "from_address",
                "to_addresses",
                "body_preview",
            ])
            .await
            .context("Failed to set searchable attributes")?;

        // Configure filterable attributes
        index
            .set_filterable_attributes(&[
                "account_id",
                "from_address",
                "received_at",
                "has_attachments",
                "is_read",
                "is_starred",
                "mailbox_ids",
                "keywords",
            ])
            .await
            .context("Failed to set filterable attributes")?;

        // Configure sortable attributes
        index
            .set_sortable_attributes(&["received_at", "subject"])
            .await
            .context("Failed to set sortable attributes")?;

        // Configure ranking rules
        index
            .set_ranking_rules(&[
                "sort",
                "words",
                "typo",
                "proximity",
                "attribute",
                "exactness",
            ])
            .await
            .context("Failed to set ranking rules")?;

        info!("Email search index '{}' initialized successfully", self.index_name);

        Ok(())
    }

    /// Index a single email
    pub async fn index_email(&self, email: &EmailDocument) -> Result<()> {
        let index = self.meilisearch.index(&self.index_name);

        debug!("Indexing email: {} ({})", email.subject, email.jmap_id);

        index
            .add_documents(&[email], Some("id"))
            .await
            .context("Failed to index email")?;

        Ok(())
    }

    /// Index multiple emails in batch
    pub async fn index_emails_batch(&self, emails: Vec<EmailDocument>) -> Result<()> {
        if emails.is_empty() {
            return Ok(());
        }

        let index = self.meilisearch.index(&self.index_name);

        info!("Indexing {} emails in batch", emails.len());

        index
            .add_documents(&emails, Some("id"))
            .await
            .context("Failed to index emails batch")?;

        Ok(())
    }

    /// Search emails with filters
    pub async fn search_emails(
        &self,
        query: &str,
        account_id: &str,
        filters: Option<SearchFilters>,
        limit: usize,
        offset: usize,
    ) -> Result<SearchResults> {
        let index = self.meilisearch.index(&self.index_name);

        // Build filter string
        let mut filter_parts = vec![format!("account_id = '{}'", account_id)];

        if let Some(f) = filters {
            if let Some(from) = f.from {
                filter_parts.push(format!("from_address = '{}'", from));
            }
            if let Some(mailbox_id) = f.mailbox_id {
                filter_parts.push(format!("mailbox_ids = '{}'", mailbox_id));
            }
            if let Some(is_read) = f.is_read {
                filter_parts.push(format!("is_read = {}", is_read));
            }
            if let Some(is_starred) = f.is_starred {
                filter_parts.push(format!("is_starred = {}", is_starred));
            }
            if let Some(has_attachments) = f.has_attachments {
                filter_parts.push(format!("has_attachments = {}", has_attachments));
            }
        }

        let filter_string = filter_parts.join(" AND ");

        debug!("Searching emails with query='{}', filter='{}'", query, filter_string);

        let results = index
            .search()
            .with_query(query)
            .with_filter(&filter_string)
            .with_limit(limit)
            .with_offset(offset)
            .with_sort(&["received_at:desc"])
            .execute::<EmailDocument>()
            .await
            .context("Failed to search emails")?;

        Ok(SearchResults {
            hits: results.hits.into_iter().map(|h| h.result).collect(),
            total: results.estimated_total_hits.unwrap_or(0),
            query: query.to_string(),
        })
    }

    /// Delete an email from the index
    pub async fn delete_email(&self, email_id: &str) -> Result<()> {
        let index = self.meilisearch.index(&self.index_name);

        debug!("Deleting email from index: {}", email_id);

        index
            .delete_document(email_id)
            .await
            .context("Failed to delete email from index")?;

        Ok(())
    }

    /// Update indexed email (e.g., mark as read)
    pub async fn update_email(&self, email: &EmailDocument) -> Result<()> {
        // In Meilisearch, updates are the same as adding documents
        self.index_email(email).await
    }
}

/// Search filters
#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub from: Option<String>,
    pub mailbox_id: Option<String>,
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
    pub has_attachments: Option<bool>,
}

/// Search results
#[derive(Debug, Clone, Serialize)]
pub struct SearchResults {
    pub hits: Vec<EmailDocument>,
    pub total: usize,
    pub query: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_document_serialization() {
        let email = EmailDocument {
            id: "test-id".to_string(),
            account_id: "acc-123".to_string(),
            jmap_id: "jmap-456".to_string(),
            subject: "Test Email".to_string(),
            from_address: "sender@example.com".to_string(),
            from_name: Some("Sender Name".to_string()),
            to_addresses: vec!["recipient@example.com".to_string()],
            cc_addresses: vec![],
            body_preview: "This is a test email...".to_string(),
            received_at: 1234567890,
            has_attachments: false,
            is_read: false,
            is_starred: false,
            mailbox_ids: vec!["inbox".to_string()],
            keywords: vec![],
        };

        let json = serde_json::to_string(&email).unwrap();
        assert!(json.contains("Test Email"));
    }
}
