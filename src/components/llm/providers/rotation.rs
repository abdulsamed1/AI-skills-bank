use crate::components::llm::provider::LlmProvider;
use crate::components::llm::types::{LlmClassificationResponse, LlmClassificationContext};
use crate::components::llm::error::LlmError;
use async_trait::async_trait;

pub struct RotationProvider {
    pub providers: Vec<Box<dyn LlmProvider>>,
}

impl RotationProvider {
    pub fn new(providers: Vec<Box<dyn LlmProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl LlmProvider for RotationProvider {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
        context: &LlmClassificationContext,
    ) -> Result<LlmClassificationResponse, LlmError> {
        if self.providers.is_empty() {
            return Err(LlmError::ConfigError("No LLM providers configured in rotation".to_string()));
        }

        let mut errors = Vec::new();
        for provider in &self.providers {
            match provider.classify(skill_id, description, abstract_text, context).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    eprintln!(
                        "WARN: Provider '{}' failed classify for skill '{}': {:?}. Trying next...",
                        provider.name(),
                        skill_id,
                        err
                    );
                    errors.push(err);
                }
            }
        }

        // Prioritize returning RateLimited if any provider was rate limited,
        // so the pipeline triggers its backoff/retry logic.
        let rate_limited = errors.iter().find(|e| matches!(e, LlmError::RateLimited { .. }));
        if let Some(err) = rate_limited {
            return Err(err.clone());
        }

        // Otherwise return the first error
        Err(errors.into_iter().next().unwrap_or(LlmError::ConfigError("All providers failed".into())))
    }

    async fn classify_batch(
        &self,
        items: &[(String, String, Option<String>)],
        context: &LlmClassificationContext,
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        if self.providers.is_empty() {
            return Err(LlmError::ConfigError("No LLM providers configured in rotation".to_string()));
        }

        let mut errors = Vec::new();
        for provider in &self.providers {
            match provider.classify_batch(items, context).await {
                Ok(resps) => return Ok(resps),
                Err(err) => {
                    eprintln!(
                        "WARN: Provider '{}' failed classify_batch: {:?}. Trying next...",
                        provider.name(),
                        err
                    );
                    errors.push(err);
                }
            }
        }

        // Prioritize returning RateLimited if any provider was rate limited,
        // so the pipeline triggers its backoff/retry logic.
        let rate_limited = errors.iter().find(|e| matches!(e, LlmError::RateLimited { .. }));
        if let Some(err) = rate_limited {
            return Err(err.clone());
        }

        // Otherwise return the first error
        Err(errors.into_iter().next().unwrap_or(LlmError::ConfigError("All providers failed".into())))
    }

    fn name(&self) -> &'static str {
        "rotation"
    }
}
