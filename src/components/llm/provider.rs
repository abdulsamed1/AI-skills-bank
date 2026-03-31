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

    /// Optional batch classification. Default implementation calls `classify` for each item.
    async fn classify_batch(
        &self,
        items: &[(String, String, Option<String>)],
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        let mut out = Vec::with_capacity(items.len());
        for (skill_id, description, abstract_text) in items.iter() {
            let resp = self
                .classify(skill_id, description, abstract_text.as_deref())
                .await?;
            out.push(resp);
        }
        Ok(out)
    }

    fn name(&self) -> &'static str;
}
