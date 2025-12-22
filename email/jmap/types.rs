//! JMAP Types (Phase 3)
//!
//! Data structures for JMAP protocol communication with Stalwart.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Deserialize null or missing values as empty Vec
/// This is needed because JMAP returns null for absent headers (cc, bcc, etc.)
fn null_to_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

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
    /// Session state - used for detecting changes
    #[serde(default)]
    pub state: Option<String>,
}

/// JMAP Account
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JmapAccount {
    pub name: String,
    #[serde(rename = "isPersonal")]
    pub is_personal: bool,
    #[serde(rename = "isReadOnly")]
    pub is_read_only: bool,
    /// Account capabilities - varies by server, so we use serde_json::Value
    #[serde(rename = "accountCapabilities", default)]
    pub account_capabilities: HashMap<String, serde_json::Value>,
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

impl std::fmt::Display for MailboxRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailboxRole::Inbox => write!(f, "inbox"),
            MailboxRole::Archive => write!(f, "archive"),
            MailboxRole::Drafts => write!(f, "drafts"),
            MailboxRole::Sent => write!(f, "sent"),
            MailboxRole::Trash => write!(f, "trash"),
            MailboxRole::Junk => write!(f, "junk"),
            MailboxRole::Other => write!(f, "other"),
        }
    }
}

/// JMAP Email Message
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JmapEmail {
    pub id: String,
    #[serde(rename = "blobId", default)]
    pub blob_id: String,
    #[serde(rename = "threadId", default)]
    pub thread_id: String,
    #[serde(rename = "mailboxIds", default)]
    pub mailbox_ids: HashMap<String, bool>,
    #[serde(default)]
    pub keywords: HashMap<String, bool>,
    #[serde(default)]
    pub size: u64,
    #[serde(rename = "receivedAt", default)]
    pub received_at: String,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub from: Vec<EmailAddress>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub to: Vec<EmailAddress>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub cc: Vec<EmailAddress>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub bcc: Vec<EmailAddress>,
    #[serde(rename = "replyTo", default, deserialize_with = "null_to_empty_vec")]
    pub reply_to: Vec<EmailAddress>,
    #[serde(default)]
    pub subject: String,
    #[serde(rename = "sentAt")]
    pub sent_at: Option<String>,
    #[serde(rename = "hasAttachment", default)]
    pub has_attachment: bool,
    #[serde(default)]
    pub preview: String,
    #[serde(rename = "bodyStructure")]
    pub body_structure: Option<serde_json::Value>,
    #[serde(rename = "bodyValues")]
    pub body_values: Option<HashMap<String, EmailBodyValue>>,
}

impl JmapEmail {
    /// Extract text body from body_values
    /// Searches bodyStructure for text/plain part and retrieves its content
    pub fn text_body(&self) -> Option<String> {
        let body_values = self.body_values.as_ref()?;

        // First try our own partId convention (for emails we sent)
        if let Some(v) = body_values.get("text") {
            return Some(v.value.clone());
        }

        // Search bodyStructure for text/plain partId
        if let Some(structure) = &self.body_structure {
            if let Some(part_id) = Self::find_part_id(structure, "text/plain") {
                if let Some(v) = body_values.get(&part_id) {
                    return Some(v.value.clone());
                }
            }
        }

        // Fallback: return first body value that looks like plain text
        // (doesn't contain HTML tags)
        for (_, body_value) in body_values.iter() {
            if !body_value.value.contains("<html") && !body_value.value.contains("<body") {
                return Some(body_value.value.clone());
            }
        }

        None
    }

    /// Extract HTML body from body_values
    /// Searches bodyStructure for text/html part and retrieves its content
    pub fn html_body(&self) -> Option<String> {
        let body_values = self.body_values.as_ref()?;

        // First try our own partId convention (for emails we sent)
        if let Some(v) = body_values.get("html") {
            return Some(v.value.clone());
        }

        // Search bodyStructure for text/html partId
        if let Some(structure) = &self.body_structure {
            if let Some(part_id) = Self::find_part_id(structure, "text/html") {
                if let Some(v) = body_values.get(&part_id) {
                    return Some(v.value.clone());
                }
            }
        }

        // Fallback: return first body value that looks like HTML
        for (_, body_value) in body_values.iter() {
            if body_value.value.contains("<html") || body_value.value.contains("<body") || body_value.value.contains("<div") {
                return Some(body_value.value.clone());
            }
        }

        None
    }

    /// Recursively search bodyStructure for a part with the given content type
    /// Returns the partId if found
    fn find_part_id(structure: &serde_json::Value, content_type: &str) -> Option<String> {
        // Check if this part matches the content type
        if let Some(part_type) = structure.get("type").and_then(|t| t.as_str()) {
            if part_type.eq_ignore_ascii_case(content_type) {
                // Return the partId
                return structure.get("partId").and_then(|p| p.as_str()).map(|s| s.to_string());
            }
        }

        // Check subParts recursively
        if let Some(sub_parts) = structure.get("subParts").and_then(|s| s.as_array()) {
            for sub_part in sub_parts {
                if let Some(part_id) = Self::find_part_id(sub_part, content_type) {
                    return Some(part_id);
                }
            }
        }

        None
    }
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
