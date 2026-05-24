use crate::components::llm::error::LlmError;
use reqwest::Certificate;
use reqwest::ClientBuilder;
use std::fs;

/// Build a reqwest ClientBuilder configured with optional CA cert from `LLM_CA_CERT_PATH`.
pub fn build_client_builder() -> Result<ClientBuilder, LlmError> {
    let mut builder = reqwest::Client::builder();

    if let Ok(path) = std::env::var("LLM_CA_CERT_PATH") {
        let pem = fs::read(path.clone()).map_err(|e| {
            LlmError::ConfigError(format!("Failed to read CA cert file {}: {}", path, e))
        })?;
        let cert = Certificate::from_pem(&pem).map_err(|e| {
            LlmError::ConfigError(format!("Failed to parse CA cert {}: {}", path, e))
        })?;
        builder = builder.add_root_certificate(cert);
    }

    Ok(builder)
}
