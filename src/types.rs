use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub min_word_count: Option<usize>,
    pub max_word_count: Option<usize>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    // Phase 7.4: Domain filtering for faceted search
    pub domain: Option<String>,
}

fn default_limit() -> usize {
    20
}

#[derive(Debug, Deserialize)]
pub struct CrawlRequest {
    pub urls: Vec<String>,
    #[serde(default = "default_depth")]
    pub max_depth: usize,
}

fn default_depth() -> usize {
    3
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<serde_json::Value> {
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}
