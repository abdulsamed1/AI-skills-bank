use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::LlmProvider;
use crate::components::llm::types::{LlmClassificationResponse, SubHubSuggestion};
use async_trait::async_trait;

pub struct MockProvider {
    pub config: LlmClientConfig,
}

impl MockProvider {
    pub fn new(config: LlmClientConfig) -> Result<Self, LlmError> {
        Ok(Self { config })
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
    ) -> Result<LlmClassificationResponse, LlmError> {
        // Deterministic mock: choose sub_hub based on simple keywords for tests
        let mut suggestions = Vec::new();

        if skill_id.to_lowercase().contains("rust")
            || description.to_lowercase().contains("rust")
            || abstract_text
                .and_then(|s| Some(s.to_lowercase().contains("rust")))
                .unwrap_or(false)
        {
            suggestions.push(SubHubSuggestion {
                hub: "programming".to_string(),
                sub_hub: "rust".to_string(),
                confidence: 95,
                reasoning: Some("mock: matched rust token".to_string()),
            });
        } else {
            suggestions.push(SubHubSuggestion {
                hub: "programming".to_string(),
                sub_hub: "core-concepts".to_string(),
                confidence: 75,
                reasoning: Some("mock: default programming".to_string()),
            });
        }

        Ok(LlmClassificationResponse { ranked_suggestions: suggestions })
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}
