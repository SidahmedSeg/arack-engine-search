//! JMAP Types (Phase 3)
//!
//! Data structures for JMAP protocol communication with Stalwart.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// JMAP Session (from session endpoint)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JmapSession {
    pub capabilities: HashMap<String, serde_json::Value>,
    #[serde(rename = "accounts")]
    pub accounts: HashMap<String, JmapAccount>,
    #[serde(rename = "primaryAccounts")]
    pub primary_accounts: HashMap<String, String>,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,
    #[serde(rename = "eventSourceUrl")]
    pub event_source_url: String,
}

/// JMAP Account
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JmapAccount {
    pub name: String,
    #[serde(rename = "isPersonal")]
    pub is_personal: bool,
    #[serde(rename = "isReadOnly")]
    pub is_read_only: bool,
}

/// JMAP Mailbox
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JmapMailbox {
    pub id: String,
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub role: Option<MailboxRole>,
    #[serde(rename = "sortOrder")]
    pub sort_order: u32,
    #[serde(rename = "totalEmails")]
    pub total_emails: u32,
    #[serde(rename = "unreadEmails")]
    pub unread_emails: u32,
    #[serde(rename = "totalThreads")]
    pub total_threads: u32,
    #[serde(rename = "unreadThreads")]
    pub unread_threads: u32,
}

/// Mailbox Role (standard mailbox types)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MailboxRole {
    Inbox,
    Archive,
    Drafts,
    Sent,
    Trash,
    Junk,
    #[serde(other)]
    Other,
}

/// JMAP Email Message
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JmapEmail {
    pub id: String,
    #[serde(rename = "blobId")]
    pub blob_id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(rename = "mailboxIds")]
    pub mailbox_ids: HashMap<String, bool>,
    pub keywords: HashMap<String, bool>,
    pub size: u64,
    #[serde(rename = "receivedAt")]
    pub received_at: String,
    pub from: Option<Vec<EmailAddress>>,
    pub to: Option<Vec<EmailAddress>>,
    pub cc: Option<Vec<EmailAddress>>,
    pub bcc: Option<Vec<EmailAddress>>,
    #[serde(rename = "replyTo")]
    pub reply_to: Option<Vec<EmailAddress>>,
    pub subject: Option<String>,
    #[serde(rename = "sentAt")]
    pub sent_at: Option<String>,
    #[serde(rename = "hasAttachment")]
    pub has_attachment: bool,
    pub preview: Option<String>,
    #[serde(rename = "bodyStructure")]
    pub body_structure: Option<serde_json::Value>,
    #[serde(rename = "bodyValues")]
    pub body_values: Option<HashMap<String, EmailBodyValue>>,
}

/// Email Address
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub email: String,
}

/// Email Body Value
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailBodyValue {
    pub value: String,
    #[serde(rename = "isEncodingProblem")]
    pub is_encoding_problem: bool,
    #[serde(rename = "isTruncated")]
    pub is_truncated: bool,
}

/// JMAP Request
#[derive(Debug, Clone, Serialize)]
pub struct JmapRequest {
    pub using: Vec<String>,
    #[serde(rename = "methodCalls")]
    pub method_calls: Vec<MethodCall>,
}

/// JMAP Method Call
#[derive(Debug, Clone, Serialize)]
pub struct MethodCall(pub String, pub serde_json::Value, pub String);

/// JMAP Response
#[derive(Debug, Clone, Deserialize)]
pub struct JmapResponse {
    #[serde(rename = "methodResponses")]
    pub method_responses: Vec<MethodResponse>,
    #[serde(rename = "sessionState")]
    pub session_state: String,
}

/// JMAP Method Response
#[derive(Debug, Clone, Deserialize)]
pub struct MethodResponse(pub String, pub serde_json::Value, pub String);

/// Email submission for sending
#[derive(Debug, Clone, Serialize)]
pub struct EmailSubmission {
    #[serde(rename = "emailId")]
    pub email_id: String,
    #[serde(rename = "identityId")]
    pub identity_id: String,
    pub envelope: Option<Envelope>,
}

/// Email envelope
#[derive(Debug, Clone, Serialize)]
pub struct Envelope {
    #[serde(rename = "mailFrom")]
    pub mail_from: MailAddress,
    #[serde(rename = "rcptTo")]
    pub rcpt_to: Vec<MailAddress>,
}

/// Mail address (for envelope)
#[derive(Debug, Clone, Serialize)]
pub struct MailAddress {
    pub email: String,
    pub parameters: Option<HashMap<String, String>>,
}
