//! Shared Email Types (Phase 3)
//!
//! Common data structures used across the email service.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Email account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub id: Uuid,
    pub kratos_identity_id: Uuid,
    pub email_address: String,
    pub stalwart_user_id: String,
    pub storage_quota_bytes: i64,
    pub storage_used_bytes: i64,
    pub is_active: bool,
}

/// Mailbox (folder) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub parent_id: Option<String>,
    pub total_emails: u32,
    pub unread_emails: u32,
}

/// Email message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub subject: String,
    pub from: EmailContact,
    pub to: Vec<EmailContact>,
    pub cc: Vec<EmailContact>,
    pub preview: String,
    pub received_at: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub has_attachments: bool,
    pub mailbox_ids: Vec<String>,
}

/// Email contact (from/to/cc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContact {
    pub name: Option<String>,
    pub email: String,
}

/// Request to send an email
#[derive(Debug, Clone, Deserialize)]
pub struct SendEmailRequest {
    /// User's email address for authentication (deprecated - now using OAuth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User's password for authentication (deprecated - now using OAuth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Recipients
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: String,
    pub attachments: Option<Vec<AttachmentInfo>>,
}

/// Attachment information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttachmentInfo {
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub blob_id: Option<String>,
}

/// Response for email operations
#[derive(Debug, Clone, Serialize)]
pub struct EmailOperationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub email_id: Option<String>,
}

/// Mailbox creation request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMailboxRequest {
    /// User's email address for authentication
    pub email: String,
    /// User's password for authentication
    pub password: String,
    pub name: String,
    pub parent_id: Option<String>,
}

/// Email search request
#[derive(Debug, Clone, Deserialize)]
pub struct EmailSearchRequest {
    pub query: String,
    pub mailbox_id: Option<String>,
    pub from: Option<String>,
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
    pub has_attachments: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
