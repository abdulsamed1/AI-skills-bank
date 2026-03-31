use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::LlmProvider;
use crate::components::llm::types::LlmClassificationResponse;
use crate::components::llm::tls;
use async_trait::async_trait;

pub struct OpenAiProvider {
    pub config: LlmClientConfig,
    pub client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(config: LlmClientConfig) -> Result<Self, LlmError> {
        let builder = tls::build_client_builder()?;
        let client = builder.build().map_err(|e| LlmError::NetworkError(e.to_string()))?;
        Ok(Self { config, client })
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn classify(
        &self,
        _skill_id: &str,
        _description: &str,
        _abstract_text: Option<&str>,
    ) -> Result<LlmClassificationResponse, LlmError> {
        Err(LlmError::ProviderUnavailable(
            "OpenAI provider not implemented yet".into(),
        ))
    }

    fn name(&self) -> &'static str {
        "openai"
    }
}
