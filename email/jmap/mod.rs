//! JMAP Client (Phase 3)
//!
//! Client for communicating with Stalwart Mail Server via JMAP protocol.
//! Supports both Bearer token and Basic Auth authentication.

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, error, info, warn};

pub mod types;

pub use types::*;

/// Authentication credentials for JMAP
#[derive(Clone, Debug)]
pub enum JmapAuth {
    /// Bearer token authentication
    Bearer(String),
    /// Basic auth (username, password)
    Basic { username: String, password: String },
}

/// JMAP Client for Stalwart Mail Server
#[derive(Clone)]
pub struct JmapClient {
    base_url: String,
    client: Client,
}

impl JmapClient {
    /// Create a new JMAP client with production-grade HTTP configuration
    pub fn new(base_url: &str) -> Self {
        // Build a properly configured HTTP client for Docker networking
        let client = Client::builder()
            // Set User-Agent (required by many APIs, good practice)
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            // Connection pooling settings
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            // TCP keepalive for long-lived connections
            .tcp_keepalive(Duration::from_secs(60))
            // Timeouts to prevent hanging
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            // Enable HTTP/1.1 connection reuse
            .http1_title_case_headers()
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Apply authentication to a request builder
    fn apply_auth(
        &self,
        request: reqwest::RequestBuilder,
        auth: &JmapAuth,
    ) -> reqwest::RequestBuilder {
        match auth {
            JmapAuth::Bearer(token) => request.bearer_auth(token),
            JmapAuth::Basic { username, password } => request.basic_auth(username, Some(password)),
        }
    }

    /// Get JMAP session (authentication info, capabilities, URLs)
    pub async fn get_session(&self, auth: &JmapAuth) -> Result<JmapSession> {
        let url = format!("{}/jmap/session", self.base_url);

        debug!("Fetching JMAP session from {}", url);

        let request = self.client.get(&url);
        let response = self
            .apply_auth(request, auth)
            .send()
            .await
            .context("Failed to fetch JMAP session")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("JMAP session request failed: {} - {}", status, error_text);
        }

        let session: JmapSession = response
            .json()
            .await
            .context("Failed to parse JMAP session")?;

        info!("JMAP session fetched for user: {}", session.username);

        Ok(session)
    }

    /// Get all mailboxes for an account
    pub async fn get_mailboxes(
        &self,
        auth: &JmapAuth,
        account_id: &str,
    ) -> Result<Vec<JmapMailbox>> {
        let api_url = format!("{}/jmap", self.base_url);

        let jmap_request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
            ],
            method_calls: vec![MethodCall(
                "Mailbox/get".to_string(),
                json!({
                    "accountId": account_id,
                    "ids": null, // Get all mailboxes
                }),
                "0".to_string(),
            )],
        };

        let request = self.client.post(&api_url).json(&jmap_request);
        let response = self
            .apply_auth(request, auth)
            .send()
            .await
            .context("Failed to fetch mailboxes")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Mailbox/get request failed: {} - {}", status, error_text);
        }

        let jmap_response: JmapResponse = response
            .json()
            .await
            .context("Failed to parse JMAP response")?;

        // Extract mailboxes from response
        let mailboxes = self.extract_mailboxes_from_response(&jmap_response)?;

        info!("Fetched {} mailboxes for account {}", mailboxes.len(), account_id);

        Ok(mailboxes)
    }

    /// Get emails from a specific mailbox (or all mailboxes if None)
    pub async fn get_emails(
        &self,
        auth: &JmapAuth,
        account_id: &str,
        mailbox_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<JmapEmail>> {
        let api_url = format!("{}/jmap", self.base_url);
        let limit = limit.unwrap_or(50) as u32;

        // Build filter - if no mailbox specified, get all emails
        let filter = if let Some(mb_id) = mailbox_id {
            json!({ "inMailbox": mb_id })
        } else {
            json!({})
        };

        // First, query for email IDs in the mailbox
        let jmap_request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
            ],
            method_calls: vec![
                // Email/query to get IDs
                MethodCall(
                    "Email/query".to_string(),
                    json!({
                        "accountId": account_id,
                        "filter": filter,
                        "sort": [{ "property": "receivedAt", "isAscending": false }],
                        "limit": limit,
                    }),
                    "0".to_string(),
                ),
                // Email/get to fetch full email data
                MethodCall(
                    "Email/get".to_string(),
                    json!({
                        "accountId": account_id,
                        "#ids": {
                            "resultOf": "0",
                            "name": "Email/query",
                            "path": "/ids"
                        },
                        "properties": [
                            "id", "blobId", "threadId", "mailboxIds", "keywords",
                            "size", "receivedAt", "from", "to", "cc", "subject",
                            "sentAt", "hasAttachment", "preview"
                        ],
                    }),
                    "1".to_string(),
                ),
            ],
        };

        let request = self.client.post(&api_url).json(&jmap_request);
        let response = self
            .apply_auth(request, auth)
            .send()
            .await
            .context("Failed to fetch emails")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Email/query request failed: {} - {}", status, error_text);
        }

        let jmap_response: JmapResponse = response
            .json()
            .await
            .context("Failed to parse JMAP response")?;

        // Extract emails from response (second method response)
        let emails = self.extract_emails_from_response(&jmap_response)?;

        info!("Fetched {} emails from mailbox {:?}", emails.len(), mailbox_id);

        Ok(emails)
    }

    /// Get mailbox ID by role (inbox, sent, drafts, trash, junk)
    pub async fn get_mailbox_by_role(
        &self,
        auth: &JmapAuth,
        account_id: &str,
        role: &str,
    ) -> Result<Option<String>> {
        let mailboxes = self.get_mailboxes(auth, account_id).await?;

        Ok(mailboxes.into_iter().find_map(|mb| {
            match &mb.role {
                Some(r) if r.to_string().to_lowercase() == role.to_lowercase() => Some(mb.id),
                _ => None,
            }
        }))
    }

    /// Get the primary identity for sending emails
    /// Returns the first identity's ID (Stalwart auto-creates identities for users)
    pub async fn get_identity(
        &self,
        auth: &JmapAuth,
        account_id: &str,
    ) -> Result<String> {
        let api_url = format!("{}/jmap", self.base_url);

        let jmap_request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
                "urn:ietf:params:jmap:submission".to_string(),
            ],
            method_calls: vec![MethodCall(
                "Identity/get".to_string(),
                json!({
                    "accountId": account_id
                }),
                "0".to_string(),
            )],
        };

        let request = self.client.post(&api_url).json(&jmap_request);
        let response = self
            .apply_auth(request, auth)
            .send()
            .await
            .context("Identity/get request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Identity/get failed: {} - {}", status, error_text);
        }

        let jmap_response: JmapResponse = response
            .json()
            .await
            .context("Failed to parse Identity/get response")?;

        // Extract identity ID from response
        if let Some(first_response) = jmap_response.method_responses.first() {
            if let Some(list) = first_response.1.get("list") {
                if let Some(identities) = list.as_array() {
                    if let Some(first_identity) = identities.first() {
                        if let Some(id) = first_identity.get("id").and_then(|v| v.as_str()) {
                            debug!("Found identity ID: {}", id);
                            return Ok(id.to_string());
                        }
                    }
                }
            }
        }

        anyhow::bail!("No identity found for account {}", account_id)
    }

    /// Send an email
    pub async fn send_email(
        &self,
        auth: &JmapAuth,
        account_id: &str,
        from: &str,
        to: Vec<&str>,
        cc: Option<Vec<&str>>,
        subject: &str,
        body_text: &str,
        body_html: Option<&str>,
    ) -> Result<String> {
        let api_url = format!("{}/jmap", self.base_url);

        // Get the user's identity ID (required for EmailSubmission)
        let identity_id = self.get_identity(auth, account_id).await?;
        debug!("Using identity ID: {}", identity_id);

        // Get Drafts and Sent mailbox IDs (required by JMAP spec)
        let drafts_mailbox_id = self
            .get_mailbox_by_role(auth, account_id, "drafts")
            .await?
            .context("Drafts mailbox not found")?;

        let sent_mailbox_id = self
            .get_mailbox_by_role(auth, account_id, "sent")
            .await?
            .context("Sent mailbox not found")?;

        debug!("Using mailboxes - Drafts: {}, Sent: {}", drafts_mailbox_id, sent_mailbox_id);

        // Create email draft ID
        let draft_id = "draft1";

        // Build to addresses
        let to_addresses: Vec<serde_json::Value> = to
            .iter()
            .map(|e| json!({ "email": e }))
            .collect();

        // Build cc addresses
        let cc_addresses: Option<Vec<serde_json::Value>> = cc.map(|addrs| {
            addrs.iter().map(|e| json!({ "email": e })).collect()
        });

        // Build body structure based on whether we have HTML
        let (body_values, body_structure) = if let Some(html) = body_html {
            (
                json!({
                    "text": { "value": body_text, "charset": "utf-8" },
                    "html": { "value": html, "charset": "utf-8" }
                }),
                json!({
                    "type": "multipart/alternative",
                    "subParts": [
                        { "type": "text/plain", "partId": "text" },
                        { "type": "text/html", "partId": "html" }
                    ]
                }),
            )
        } else {
            (
                json!({
                    "text": { "value": body_text, "charset": "utf-8" }
                }),
                json!({
                    "type": "text/plain",
                    "partId": "text"
                }),
            )
        };

        // Create email with mailboxIds (required by JMAP spec)
        let mut email_create = json!({
            "mailboxIds": { &drafts_mailbox_id: true },
            "from": [{ "email": from }],
            "to": to_addresses,
            "subject": subject,
            "bodyValues": body_values,
            "bodyStructure": body_structure
        });

        // Add CC if present
        if let Some(cc_addrs) = cc_addresses {
            email_create["cc"] = json!(cc_addrs);
        }

        let jmap_request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
                "urn:ietf:params:jmap:submission".to_string(),
            ],
            method_calls: vec![
                // Email/set to create email in Drafts
                MethodCall(
                    "Email/set".to_string(),
                    json!({
                        "accountId": account_id,
                        "create": {
                            draft_id: email_create
                        }
                    }),
                    "0".to_string(),
                ),
                // EmailSubmission/set to send and move to Sent
                MethodCall(
                    "EmailSubmission/set".to_string(),
                    json!({
                        "accountId": account_id,
                        "create": {
                            "send1": {
                                "emailId": format!("#{}", draft_id),
                                "identityId": identity_id
                            }
                        },
                        "onSuccessUpdateEmail": {
                            "#send1": {
                                "mailboxIds": { &sent_mailbox_id: true }
                            }
                        }
                    }),
                    "1".to_string(),
                ),
            ],
        };

        let request = self.client.post(&api_url).json(&jmap_request);
        let response = self
            .apply_auth(request, auth)
            .send()
            .await
            .context("Failed to send email")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Email send request failed: {} - {}", status, error_text);
        }

        // Parse response to get the actual email ID
        let jmap_response: JmapResponse = response
            .json()
            .await
            .context("Failed to parse send email response")?;

        // Check for errors in the response
        for method_response in &jmap_response.method_responses {
            if method_response.0 == "error" {
                let error_type = method_response.1.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                let error_desc = method_response.1.get("description").and_then(|v| v.as_str()).unwrap_or("No description");
                anyhow::bail!("JMAP error: {} - {}", error_type, error_desc);
            }

            // Check for notCreated errors
            if let Some(not_created) = method_response.1.get("notCreated") {
                if let Some(draft_error) = not_created.get(draft_id) {
                    let error_type = draft_error.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let error_desc = draft_error.get("description").and_then(|v| v.as_str()).unwrap_or("No description");
                    anyhow::bail!("Failed to create email: {} - {}", error_type, error_desc);
                }
            }
        }

        // Extract the created email ID from the first method response
        let email_id = if let Some(first_response) = jmap_response.method_responses.first() {
            if let Some(created) = first_response.1.get("created") {
                if let Some(draft_result) = created.get(draft_id) {
                    draft_result
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(draft_id)
                        .to_string()
                } else {
                    draft_id.to_string()
                }
            } else {
                draft_id.to_string()
            }
        } else {
            draft_id.to_string()
        };

        info!("Email sent from {} to {:?}, ID: {}", from, to, email_id);

        Ok(email_id)
    }

    /// Create a new mailbox
    pub async fn create_mailbox(
        &self,
        auth: &JmapAuth,
        account_id: &str,
        name: &str,
        parent_id: Option<&str>,
        role: Option<&str>,
    ) -> Result<String> {
        let api_url = format!("{}/jmap", self.base_url);

        let mut mailbox_create = json!({
            "name": name,
            "parentId": parent_id
        });

        if let Some(r) = role {
            mailbox_create["role"] = json!(r);
        }

        let jmap_request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
            ],
            method_calls: vec![MethodCall(
                "Mailbox/set".to_string(),
                json!({
                    "accountId": account_id,
                    "create": {
                        "new-mailbox": mailbox_create
                    }
                }),
                "0".to_string(),
            )],
        };

        let request = self.client.post(&api_url).json(&jmap_request);
        let response = self
            .apply_auth(request, auth)
            .send()
            .await
            .context("Failed to create mailbox")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Mailbox/set request failed: {} - {}", status, error_text);
        }

        // Parse response to get the actual mailbox ID
        let jmap_response: JmapResponse = response
            .json()
            .await
            .context("Failed to parse create mailbox response")?;

        let mailbox_id = if let Some(first_response) = jmap_response.method_responses.first() {
            if let Some(created) = first_response.1.get("created") {
                if let Some(new_mailbox) = created.get("new-mailbox") {
                    new_mailbox
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("new-mailbox")
                        .to_string()
                } else {
                    "new-mailbox".to_string()
                }
            } else {
                "new-mailbox".to_string()
            }
        } else {
            "new-mailbox".to_string()
        };

        info!("Created mailbox: {} (ID: {})", name, mailbox_id);

        Ok(mailbox_id)
    }

    /// Get a single email by ID with full body
    pub async fn get_email(
        &self,
        auth: &JmapAuth,
        account_id: &str,
        email_id: &str,
    ) -> Result<JmapEmail> {
        let api_url = format!("{}/jmap", self.base_url);

        let jmap_request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
            ],
            method_calls: vec![MethodCall(
                "Email/get".to_string(),
                json!({
                    "accountId": account_id,
                    "ids": [email_id],
                    "properties": [
                        "id", "blobId", "threadId", "mailboxIds", "keywords",
                        "size", "receivedAt", "from", "to", "cc", "bcc", "replyTo",
                        "subject", "sentAt", "hasAttachment", "preview",
                        "bodyStructure", "bodyValues"
                    ],
                    "fetchAllBodyValues": true
                }),
                "0".to_string(),
            )],
        };

        let request = self.client.post(&api_url).json(&jmap_request);
        let response = self
            .apply_auth(request, auth)
            .send()
            .await
            .context("Failed to fetch email")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Email/get request failed: {} - {}", status, error_text);
        }

        let jmap_response: JmapResponse = response
            .json()
            .await
            .context("Failed to parse JMAP response")?;

        // Extract email from response
        if let Some(first_response) = jmap_response.method_responses.first() {
            if let Some(list) = first_response.1.get("list") {
                let emails: Vec<JmapEmail> = serde_json::from_value(list.clone())
                    .context("Failed to parse email from response")?;
                if let Some(email) = emails.into_iter().next() {
                    return Ok(email);
                }
            }
        }

        anyhow::bail!("Email not found: {}", email_id)
    }

    /// Create default mailboxes (Inbox, Sent, Drafts, Trash, Junk)
    /// Note: Stalwart typically creates these automatically, but this can be used if needed
    pub async fn create_default_mailboxes(
        &self,
        auth: &JmapAuth,
        account_id: &str,
    ) -> Result<()> {
        let default_mailboxes = vec![
            ("Inbox", "inbox"),
            ("Sent", "sent"),
            ("Drafts", "drafts"),
            ("Trash", "trash"),
            ("Junk", "junk"),
        ];

        for (name, role) in default_mailboxes {
            match self.create_mailbox(auth, account_id, name, None, Some(role)).await {
                Ok(_) => info!("Created mailbox: {}", name),
                Err(e) => warn!("Failed to create mailbox {} (may already exist): {}", name, e),
            }
        }

        info!("Default mailboxes setup complete for account {}", account_id);

        Ok(())
    }

    // Helper methods to extract data from JMAP responses

    fn extract_mailboxes_from_response(&self, response: &JmapResponse) -> Result<Vec<JmapMailbox>> {
        // The first method response should be Mailbox/get
        if response.method_responses.is_empty() {
            anyhow::bail!("No method responses in JMAP response");
        }

        let method_response = &response.method_responses[0];
        let list = method_response
            .1
            .get("list")
            .context("No 'list' field in Mailbox/get response")?;

        let mailboxes: Vec<JmapMailbox> = serde_json::from_value(list.clone())
            .context("Failed to parse mailboxes from response")?;

        Ok(mailboxes)
    }

    fn extract_emails_from_response(&self, response: &JmapResponse) -> Result<Vec<JmapEmail>> {
        // The second method response should be Email/get
        if response.method_responses.len() < 2 {
            anyhow::bail!("Expected 2 method responses, got {}", response.method_responses.len());
        }

        let method_response = &response.method_responses[1];
        let list = method_response
            .1
            .get("list")
            .context("No 'list' field in Email/get response")?;

        // Debug: log the raw list to see what's failing
        let emails: Vec<JmapEmail> = match serde_json::from_value(list.clone()) {
            Ok(emails) => emails,
            Err(e) => {
                error!("Failed to parse emails. Error: {}. Raw list: {}", e, list);
                return Err(anyhow::anyhow!("Failed to parse emails from response: {}", e));
            }
        };

        Ok(emails)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jmap_client_creation() {
        let client = JmapClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
    }
}
