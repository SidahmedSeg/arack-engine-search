//! JMAP Client (Phase 3)
//!
//! Client for communicating with Stalwart Mail Server via JMAP protocol.

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, info, warn};

pub mod types;

pub use types::*;

/// JMAP Client for Stalwart Mail Server
#[derive(Clone)]
pub struct JmapClient {
    base_url: String,
    client: Client,
}

impl JmapClient {
    /// Create a new JMAP client
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    /// Get JMAP session (authentication info, capabilities, URLs)
    pub async fn get_session(&self, access_token: &str) -> Result<JmapSession> {
        let url = format!("{}/jmap/session", self.base_url);

        debug!("Fetching JMAP session from {}", url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to fetch JMAP session")?;

        if !response.status().is_success() {
            anyhow::bail!("JMAP session request failed: {}", response.status());
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
        access_token: &str,
        account_id: &str,
    ) -> Result<Vec<JmapMailbox>> {
        let api_url = format!("{}/jmap", self.base_url);

        let request = JmapRequest {
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

        let response = self
            .client
            .post(&api_url)
            .bearer_auth(access_token)
            .json(&request)
            .send()
            .await
            .context("Failed to fetch mailboxes")?;

        if !response.status().is_success() {
            anyhow::bail!("Mailbox/get request failed: {}", response.status());
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

    /// Get emails from a specific mailbox
    pub async fn get_emails(
        &self,
        access_token: &str,
        account_id: &str,
        mailbox_id: &str,
        limit: u32,
    ) -> Result<Vec<JmapEmail>> {
        let api_url = format!("{}/jmap", self.base_url);

        // First, query for email IDs in the mailbox
        let query_request = JmapRequest {
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
                        "filter": {
                            "inMailbox": mailbox_id
                        },
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

        let response = self
            .client
            .post(&api_url)
            .bearer_auth(access_token)
            .json(&query_request)
            .send()
            .await
            .context("Failed to fetch emails")?;

        if !response.status().is_success() {
            anyhow::bail!("Email/query request failed: {}", response.status());
        }

        let jmap_response: JmapResponse = response
            .json()
            .await
            .context("Failed to parse JMAP response")?;

        // Extract emails from response (second method response)
        let emails = self.extract_emails_from_response(&jmap_response)?;

        info!("Fetched {} emails from mailbox {}", emails.len(), mailbox_id);

        Ok(emails)
    }

    /// Send an email
    pub async fn send_email(
        &self,
        access_token: &str,
        account_id: &str,
        from: &str,
        to: Vec<&str>,
        subject: &str,
        body_text: &str,
    ) -> Result<String> {
        let api_url = format!("{}/jmap", self.base_url);

        // Create email draft
        let email_id = "draft1"; // Temporary ID
        let identity_id = account_id; // Use account ID as identity

        let request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
                "urn:ietf:params:jmap:submission".to_string(),
            ],
            method_calls: vec![
                // Email/set to create draft
                MethodCall(
                    "Email/set".to_string(),
                    json!({
                        "accountId": account_id,
                        "create": {
                            email_id: {
                                "mailboxIds": { "sentbox": true }, // TODO: Get actual sentbox ID
                                "from": [{ "email": from }],
                                "to": to.iter().map(|e| json!({ "email": e })).collect::<Vec<_>>(),
                                "subject": subject,
                                "bodyValues": {
                                    "body": {
                                        "value": body_text,
                                        "charset": "utf-8"
                                    }
                                },
                                "bodyStructure": {
                                    "type": "text/plain",
                                    "partId": "body"
                                }
                            }
                        }
                    }),
                    "0".to_string(),
                ),
                // EmailSubmission/set to send
                MethodCall(
                    "EmailSubmission/set".to_string(),
                    json!({
                        "accountId": account_id,
                        "create": {
                            "sub1": {
                                "emailId": format!("#email_{}", email_id),
                                "identityId": identity_id,
                            }
                        }
                    }),
                    "1".to_string(),
                ),
            ],
        };

        let response = self
            .client
            .post(&api_url)
            .bearer_auth(access_token)
            .json(&request)
            .send()
            .await
            .context("Failed to send email")?;

        if !response.status().is_success() {
            anyhow::bail!("Email send request failed: {}", response.status());
        }

        info!("Email sent from {} to {:?}", from, to);

        Ok(email_id.to_string())
    }

    /// Create a new mailbox
    pub async fn create_mailbox(
        &self,
        access_token: &str,
        account_id: &str,
        name: &str,
        parent_id: Option<&str>,
    ) -> Result<String> {
        let api_url = format!("{}/jmap", self.base_url);

        let request = JmapRequest {
            using: vec![
                "urn:ietf:params:jmap:core".to_string(),
                "urn:ietf:params:jmap:mail".to_string(),
            ],
            method_calls: vec![MethodCall(
                "Mailbox/set".to_string(),
                json!({
                    "accountId": account_id,
                    "create": {
                        "new-mailbox": {
                            "name": name,
                            "parentId": parent_id,
                        }
                    }
                }),
                "0".to_string(),
            )],
        };

        let response = self
            .client
            .post(&api_url)
            .bearer_auth(access_token)
            .json(&request)
            .send()
            .await
            .context("Failed to create mailbox")?;

        if !response.status().is_success() {
            anyhow::bail!("Mailbox/set request failed: {}", response.status());
        }

        info!("Created mailbox: {}", name);

        Ok("new-mailbox".to_string()) // TODO: Extract actual ID from response
    }

    /// Create default mailboxes (Inbox, Sent, Drafts, Trash, Spam)
    pub async fn create_default_mailboxes(
        &self,
        access_token: &str,
        account_id: &str,
    ) -> Result<()> {
        let default_mailboxes = vec![
            ("Inbox", MailboxRole::Inbox),
            ("Sent", MailboxRole::Sent),
            ("Drafts", MailboxRole::Drafts),
            ("Trash", MailboxRole::Trash),
            ("Junk", MailboxRole::Junk),
        ];

        for (name, _role) in default_mailboxes {
            self.create_mailbox(access_token, account_id, name, None)
                .await?;
        }

        info!("Created default mailboxes for account {}", account_id);

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

        let emails: Vec<JmapEmail> = serde_json::from_value(list.clone())
            .context("Failed to parse emails from response")?;

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
