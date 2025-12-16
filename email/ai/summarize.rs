//! Email Summarization Feature
//!
//! AI-powered email thread summarization using OpenAI GPT-4o-mini.

use anyhow::Result;

#[cfg(feature = "email")]
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};

/// Email content for summarization
pub struct EmailContent {
    pub from: String,
    pub subject: String,
    pub body: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Summarize an email thread
#[cfg(feature = "email")]
pub async fn summarize_thread(
    openai_client: &Client<OpenAIConfig>,
    emails: Vec<EmailContent>,
) -> Result<SummarizeResult> {
    if emails.is_empty() {
        anyhow::bail!("No emails to summarize");
    }

    // Combine emails chronologically
    let mut combined_text = String::new();
    for email in &emails {
        combined_text.push_str(&format!(
            "\n---\nFrom: {}\nDate: {}\nSubject: {}\n\n{}\n",
            email.from,
            email.timestamp.format("%Y-%m-%d %H:%M"),
            email.subject,
            email.body
        ));
    }

    // Truncate if too long (max ~3000 chars to stay under token limit)
    if combined_text.len() > 3000 {
        combined_text.truncate(3000);
        combined_text.push_str("\n... (truncated for length)");
    }

    let system_prompt = "You are an email summarization assistant. Analyze the email thread and provide:
1. A concise one-paragraph summary
2. Key points (bullet list)
3. Action items (if any)

Be clear and factual. Focus on important information and next steps.";

    let user_message = format!(
        "Please summarize this email thread:\n\n{}",
        combined_text
    );

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
        .max_tokens(200u16)
        .temperature(0.3) // Lower temperature for more factual summaries
        .build()?;

    let response = openai_client.chat().create(request).await?;

    let summary_text = response
        .choices
        .first()
        .and_then(|c| c.message.content.as_ref())
        .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;

    // Parse the response to extract summary, key points, and action items
    let parsed = parse_summary_response(summary_text);

    // Count tokens used (approximate)
    let token_count = response.usage.map(|u| u.total_tokens as usize).unwrap_or(0);

    Ok(SummarizeResult {
        summary: parsed.summary,
        key_points: parsed.key_points,
        action_items: parsed.action_items,
        token_count,
    })
}

/// Parsed summary result
pub struct SummarizeResult {
    pub summary: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub token_count: usize,
}

/// Parse AI response into structured format
fn parse_summary_response(text: &str) -> SummarizeResult {
    let mut summary = String::new();
    let mut key_points = Vec::new();
    let mut action_items = Vec::new();

    let mut current_section = "summary";

    for line in text.lines() {
        let line = line.trim();

        // Detect section headers
        if line.to_lowercase().contains("key points") || line.to_lowercase().contains("main points") {
            current_section = "key_points";
            continue;
        } else if line.to_lowercase().contains("action items") || line.to_lowercase().contains("next steps") {
            current_section = "action_items";
            continue;
        }

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Add content to appropriate section
        match current_section {
            "summary" => {
                if !summary.is_empty() {
                    summary.push(' ');
                }
                summary.push_str(line);
            }
            "key_points" => {
                // Remove bullet points
                let point = line.trim_start_matches(&['-', '*', '•', '·'][..]).trim();
                if !point.is_empty() {
                    key_points.push(point.to_string());
                }
            }
            "action_items" => {
                let item = line.trim_start_matches(&['-', '*', '•', '·'][..]).trim();
                if !item.is_empty() {
                    action_items.push(item.to_string());
                }
            }
            _ => {}
        }
    }

    // If no structured parsing worked, use the whole text as summary
    if summary.is_empty() && key_points.is_empty() && action_items.is_empty() {
        summary = text.to_string();
    }

    SummarizeResult {
        summary,
        key_points,
        action_items,
        token_count: 0, // Will be set by caller
    }
}

#[cfg(not(feature = "email"))]
pub async fn summarize_thread(
    _openai_client: &(),
    _emails: Vec<EmailContent>,
) -> Result<SummarizeResult> {
    anyhow::bail!("Email feature not enabled")
}
