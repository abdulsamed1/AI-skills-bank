use crate::components::llm::error::LlmError;
use std::env;

#[derive(Debug, Clone)]
pub struct LlmClientConfig {
    pub provider: String,
    pub api_key: String,
    pub api_url: Option<String>,
    pub ca_cert_path: Option<String>,
}

impl LlmClientConfig {
    pub fn from_env() -> Result<Self, LlmError> {
        let provider = env::var("LLM_PROVIDER").unwrap_or_default();
        if provider.is_empty() {
            return Err(LlmError::ConfigError("LLM_PROVIDER not set".to_string()));
        }

        let api_key = env::var("LLM_API_KEY").map_err(|_| {
            LlmError::ConfigError("LLM_API_KEY not set".to_string())
        })?;

        let api_url = env::var("LLM_API_URL").ok();
        let ca_cert_path = env::var("LLM_CA_CERT_PATH").ok();

        Ok(Self {
            provider,
            api_key,
            api_url,
            ca_cert_path,
        })
    }
}
