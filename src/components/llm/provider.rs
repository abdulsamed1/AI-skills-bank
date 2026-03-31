use crate::components::llm::types::LlmClassificationResponse;
use crate::components::llm::error::LlmError;
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
    ) -> Result<LlmClassificationResponse, LlmError>;

    fn name(&self) -> &'static str;
}
