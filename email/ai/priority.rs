//! Priority Inbox Ranking Feature
//!
//! AI-powered email priority scoring using OpenAI GPT-4o-mini.

use super::types::PriorityEmail;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[cfg(feature = "email")]
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};

/// Email information for priority ranking
#[derive(Debug)]
pub struct EmailInfo {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub snippet: String,
    pub has_attachments: bool,
    pub sender_frequency: u32, // How many emails from this sender
}

/// Priority ranking result from AI
#[derive(Debug, Serialize, Deserialize)]
struct AiPriorityResponse {
    priority: u8,    // 1-10
    reason: String,
}

/// Rank emails by priority using AI
#[cfg(feature = "email")]
pub async fn rank_emails(
    openai_client: &Client<OpenAIConfig>,
    emails: Vec<EmailInfo>,
    user_context: Option<&str>,
) -> Result<Vec<PriorityEmail>> {
    if emails.is_empty() {
        return Ok(Vec::new());
    }

    let mut ranked_emails = Vec::new();

    for email in emails {
        match score_email(openai_client, &email, user_context).await {
            Ok(scored) => ranked_emails.push(scored),
            Err(e) => {
                tracing::warn!("Failed to score email {}: {}", email.id, e);
                // Fallback: use basic heuristic scoring
                let fallback_score = calculate_fallback_score(&email);
                ranked_emails.push(PriorityEmail {
                    email_id: email.id,
                    priority_score: fallback_score,
                    reason: "Heuristic scoring (AI unavailable)".to_string(),
                });
            }
        }
    }

    // Sort by priority (highest first)
    ranked_emails.sort_by(|a, b| b.priority_score.cmp(&a.priority_score));

    Ok(ranked_emails)
}

/// Score a single email using AI
#[cfg(feature = "email")]
async fn score_email(
    openai_client: &Client<OpenAIConfig>,
    email: &EmailInfo,
    user_context: Option<&str>,
) -> Result<PriorityEmail> {
    let system_prompt = "You are an email priority assistant. Analyze emails and assign a priority score from 1-10 based on:
- Urgency indicators (urgent, asap, important, deadline)
- Sender importance (frequent contacts are more important)
- Action required (questions, requests, tasks)
- Context relevance
Respond in JSON format: {\"priority\": 1-10, \"reason\": \"brief explanation\"}";

    let mut user_message = format!(
        "Email:\nFrom: {}\nSubject: {}\nPreview: {}\nSender frequency: {} emails\nHas attachments: {}",
        email.from,
        email.subject,
        email.snippet,
        email.sender_frequency,
        if email.has_attachments { "yes" } else { "no" }
    );

    if let Some(context) = user_context {
        user_message.push_str(&format!("\n\nUser context: {}", context));
    }

    let messages = vec![
        ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()?,
        ),
        ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_message)
                .build()?,
        ),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(messages)
        .max_tokens(100u16)
        .temperature(0.2) // Low temperature for consistent scoring
        .build()?;

    let response = openai_client.chat().create(request).await?;

    let content = response
        .choices
        .first()
        .and_then(|c| c.message.content.as_ref())
        .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;

    // Parse JSON response
    let ai_response: AiPriorityResponse = serde_json::from_str(content.trim())
        .or_else(|_| {
            // Fallback: try to extract priority from text
            extract_priority_from_text(content)
        })?;

    Ok(PriorityEmail {
        email_id: email.id.clone(),
        priority_score: ai_response.priority.clamp(1, 10),
        reason: ai_response.reason,
    })
}

/// Extract priority from non-JSON text (fallback)
fn extract_priority_from_text(text: &str) -> Result<AiPriorityResponse> {
    // Try to find a number between 1-10
    for word in text.split_whitespace() {
        if let Ok(num) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
            if (1..=10).contains(&num) {
                return Ok(AiPriorityResponse {
                    priority: num,
                    reason: text.to_string(),
                });
            }
        }
    }
    anyhow::bail!("Could not parse priority from response: {}", text)
}

/// Calculate priority score using heuristics (fallback when AI fails)
fn calculate_fallback_score(email: &EmailInfo) -> u8 {
    let mut score = 5; // Base score

    // Urgency keywords in subject
    let subject_lower = email.subject.to_lowercase();
    if subject_lower.contains("urgent") || subject_lower.contains("asap") {
        score += 3;
    } else if subject_lower.contains("important") || subject_lower.contains("deadline") {
        score += 2;
    }

    // Sender frequency (frequent senders are more important)
    if email.sender_frequency > 20 {
        score += 2;
    } else if email.sender_frequency > 10 {
        score += 1;
    }

    // Attachments might indicate important content
    if email.has_attachments {
        score += 1;
    }

    // Question marks suggest action required
    if email.subject.contains('?') || email.snippet.contains('?') {
        score += 1;
    }

    score.clamp(1, 10)
}

#[cfg(not(feature = "email"))]
pub async fn rank_emails(
    _openai_client: &(),
    _emails: Vec<EmailInfo>,
    _user_context: Option<&str>,
) -> Result<Vec<PriorityEmail>> {
    anyhow::bail!("Email feature not enabled")
}
