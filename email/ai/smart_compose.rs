//! Smart Compose Feature
//!
//! AI-powered email completion suggestions using OpenAI GPT-4o-mini.

use super::types::{SmartComposeContext, SmartComposeSuggestion};
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

/// Generate smart compose suggestions
#[cfg(feature = "email")]
pub async fn generate_suggestions(
    openai_client: &Client<OpenAIConfig>,
    partial_text: &str,
    context: Option<&SmartComposeContext>,
) -> Result<Vec<SmartComposeSuggestion>> {
    // Build system prompt based on context
    let system_prompt = if let Some(ctx) = context {
        if ctx.is_reply {
            "You are an email writing assistant. Continue this email reply naturally and professionally. Keep the tone consistent with the partial text provided."
        } else {
            "You are an email writing assistant. Continue this email naturally and professionally. Be concise and clear."
        }
    } else {
        "You are an email writing assistant. Continue this email naturally and professionally."
    };

    // Build user message with context
    let mut user_message = String::new();
    if let Some(ctx) = context {
        if let Some(subject) = &ctx.subject {
            user_message.push_str(&format!("Subject: {}\n", subject));
        }
        if let Some(recipient) = &ctx.recipient {
            user_message.push_str(&format!("To: {}\n", recipient));
        }
        user_message.push_str("\n");
    }
    user_message.push_str("Partial email:\n");
    user_message.push_str(partial_text);
    user_message.push_str("\n\nProvide 3 different natural completions for this email (just the next 1-2 sentences). Make them diverse in style and length.");

    // Create messages
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

    // Generate 3 different completions with temperature for diversity
    let mut suggestions = Vec::new();

    for i in 0..3 {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(messages.clone())
            .max_tokens(100u16)
            .temperature(0.7 + (i as f32 * 0.1)) // Vary temperature for diversity
            .n(1_u8)
            .build()?;

        match openai_client.chat().create(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    if let Some(content) = &choice.message.content {
                        let confidence = 0.95 - (i as f32 * 0.07); // Higher confidence for first suggestion
                        suggestions.push(SmartComposeSuggestion {
                            text: content.trim().to_string(),
                            confidence,
                        });
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to generate smart compose suggestion {}: {}", i, e);
            }
        }
    }

    Ok(suggestions)
}

/// Count tokens in text (approximate)
#[cfg(feature = "email")]
pub fn count_tokens(text: &str) -> usize {
    // Approximate: ~4 characters per token for English text
    // More accurate would use tiktoken-rs
    (text.len() as f64 / 4.0).ceil() as usize
}

#[cfg(not(feature = "email"))]
pub async fn generate_suggestions(
    _openai_client: &(),
    _partial_text: &str,
    _context: Option<&SmartComposeContext>,
) -> Result<Vec<SmartComposeSuggestion>> {
    anyhow::bail!("Email feature not enabled")
}

#[cfg(not(feature = "email"))]
pub fn count_tokens(_text: &str) -> usize {
    0
}
