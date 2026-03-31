use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubHubSuggestion {
    pub hub: String,
    pub sub_hub: String,
    pub confidence: u32,
    #[serde(default)]
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmClassificationResponse {
    pub ranked_suggestions: Vec<SubHubSuggestion>,
}
