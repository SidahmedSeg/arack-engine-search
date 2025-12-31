//! AI Feature Types
//!
//! Request and response types for AI-powered email features.

use serde::{Deserialize, Serialize};

/// Smart compose request
#[derive(Debug, Deserialize)]
pub struct SmartComposeRequest {
    pub partial_text: String,
    pub context: Option<SmartComposeContext>,
}

/// Context for smart compose
#[derive(Debug, Deserialize)]
pub struct SmartComposeContext {
    pub subject: Option<String>,
    pub recipient: Option<String>,
    pub is_reply: bool,
}

/// Smart compose suggestion
#[derive(Debug, Serialize)]
pub struct SmartComposeSuggestion {
    pub text: String,
    pub confidence: f32,
}

/// Smart compose response
#[derive(Debug, Serialize)]
pub struct SmartComposeResponse {
    pub suggestions: Vec<SmartComposeSuggestion>,
}

/// Email summarization request
#[derive(Debug, Deserialize)]
pub struct SummarizeRequest {
    pub thread_id: Option<String>,
    pub email_ids: Vec<String>,
}

/// Email summarization response
#[derive(Debug, Serialize)]
pub struct SummarizeResponse {
    pub summary: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub token_count: usize,
}

/// Priority ranking request
#[derive(Debug, Deserialize)]
pub struct PriorityRankRequest {
    pub mailbox_id: String,
    pub email_ids: Vec<String>,
}

/// Priority email with score
#[derive(Debug, Clone, Serialize)]
pub struct PriorityEmail {
    pub email_id: String,
    pub priority_score: u8, // 1-10
    pub reason: String,
}

/// Priority ranking response
#[derive(Debug, Serialize)]
pub struct PriorityRankResponse {
    pub ranked_emails: Vec<PriorityEmail>,
}

/// AI quota information
#[derive(Debug, Serialize)]
pub struct AiQuota {
    pub smart_compose: QuotaUsage,
    pub summarization: QuotaUsage,
    pub priority_ranking: QuotaUsage,
}

/// Quota usage for a specific feature
#[derive(Debug, Serialize)]
pub struct QuotaUsage {
    pub used: u32,
    pub limit: u32,
    pub reset_at: chrono::DateTime<chrono::Utc>,
}

/// AI feature type for tracking
#[derive(Debug, Clone, Copy)]
pub enum AiFeature {
    SmartCompose,
    Summarization,
    PriorityRanking,
}

impl AiFeature {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiFeature::SmartCompose => "smart_compose",
            AiFeature::Summarization => "summarize",
            AiFeature::PriorityRanking => "priority",
        }
    }

    pub fn daily_limit(&self) -> u32 {
        match self {
            AiFeature::SmartCompose => 50,
            AiFeature::Summarization => 20,
            AiFeature::PriorityRanking => 10,
        }
    }
}
