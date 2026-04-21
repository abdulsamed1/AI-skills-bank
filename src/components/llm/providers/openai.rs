use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::{extract_json_substring, LlmProvider};
use crate::components::llm::types::{LlmClassificationResponse, LlmClassificationContext};
use crate::components::llm::tls;
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;
use std::env;

pub struct OpenAiProvider {
    pub config: LlmClientConfig,
    pub client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(config: LlmClientConfig) -> Result<Self, LlmError> {
        let mut builder = tls::build_client_builder()?;
        // Allow configurable timeout via LLM_TIMEOUT_SECS, default 120 seconds for batch ops
        let timeout_secs = env::var("LLM_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(120);
        builder = builder.timeout(Duration::from_secs(timeout_secs));
        let client = builder.build().map_err(|e| LlmError::NetworkError(e.to_string()))?;
        Ok(Self { config, client })
    }

    fn get_model(&self) -> String {
        std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string())
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
        context: &LlmClassificationContext,
    ) -> Result<LlmClassificationResponse, LlmError> {
        let api_url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let system_prompt = crate::components::llm::build_classification_prompt(context, false);

        let user_payload = serde_json::json!({
            "skill_id": skill_id,
            "description": description,
            "abstract": abstract_text.unwrap_or("")
        });

        let body = serde_json::json!({
            "model": self.get_model(),
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": format!("Classify this skill: {}", user_payload) }
            ],
            "temperature": 0.0,
            "max_tokens": 500,
        });

        let resp = self
            .client
            .post(api_url)
            .bearer_auth(&self.config.api_key)
            .header("User-Agent", "skills-bank/0.1")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout
                } else {
                    LlmError::NetworkError(e.to_string())
                }
            })?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp.text().await.map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(LlmError::AuthenticationFailed(text));
            }
            if status.as_u16() == 429 {
                let retry_after = headers
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                return Err(LlmError::RateLimited { retry_after });
            }
            return Err(LlmError::ProviderUnavailable(format!(
                "OpenAI request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(content) = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(parsed) = serde_json::from_str::<LlmClassificationResponse>(&json_text) {
                        return Ok(parsed);
                    }
                }
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse OpenAI response as classification JSON".into(),
        ))
    }

    fn name(&self) -> &'static str {
        "openai"
    }

    async fn classify_batch(
        &self,
        items: &[(String, String, Option<String>)],
        context: &LlmClassificationContext,
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        if items.is_empty() {
            return Ok(vec![]);
        }

        let api_url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let system_prompt = crate::components::llm::build_classification_prompt(context, true);

        let payload_items: Vec<serde_json::Value> = items
            .iter()
            .map(|(skill_id, description, abstract_text)| {
                serde_json::json!({
                    "skill_id": skill_id,
                    "description": description,
                    "abstract": abstract_text.clone().unwrap_or_default()
                })
            })
            .collect();

        let user_content = format!("Classify these skills: {}",
            serde_json::to_string(&payload_items).unwrap_or_else(|_| "[]".to_string())
        );

        let body = serde_json::json!({
            "model": self.get_model(),
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_content}
            ],
            "temperature": 0.0,
            "max_tokens": 2000
        });

        let resp = self
            .client
            .post(api_url)
            .bearer_auth(&self.config.api_key)
            .header("User-Agent", "skills-bank/0.1")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout
                } else {
                    LlmError::NetworkError(e.to_string())
                }
            })?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp.text().await.map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 || status.as_u16() == 402 {
                return Err(LlmError::AuthenticationFailed(text));
            }
            if status.as_u16() == 429 {
                let retry_after = headers
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                return Err(LlmError::RateLimited { retry_after });
            }
            // Recovery: some providers (SambaNova) return valid model output in
            // error_model_output even when their JSON validator rejects it (400).
            if status.as_u16() == 400 {
                if let Ok(err_body) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(model_output) = err_body.get("error_model_output").and_then(|v| v.as_str()) {
                        if let Some(json_text) = extract_json_substring(model_output) {
                            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_text) {
                                if let Some(results) = value.get("results").and_then(|r| r.as_array()) {
                                    let array_json = serde_json::to_string(results).unwrap_or_else(|_| "[]".to_string());
                                    if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&array_json) {
                                        return Ok(parsed);
                                    }
                                }
                            }
                            if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                                return Ok(parsed);
                            }
                        }
                    }
                }
            }
            return Err(LlmError::ProviderUnavailable(format!(
                "OpenAI batch request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(content) = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_text) {
                        if let Some(results) = value.get("results").and_then(|r| r.as_array()) {
                            // Extract just the results array
                            let array_json = serde_json::to_string(results).unwrap_or_else(|_| "[]".to_string());
                            match serde_json::from_str::<Vec<LlmClassificationResponse>>(&array_json) {
                                Ok(parsed) => return Ok(parsed),
                                Err(e) => {
                                    eprintln!("DEBUG JSON Parse Error: {}. Raw json_text: {}", e, json_text);
                                }
                            }
                        } else {
                            // Fallback if the model still generated just an array
                            match serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                                Ok(parsed) => return Ok(parsed),
                                Err(e) => {
                                    eprintln!("DEBUG JSON Parse Error: Expected 'results' object, got array/other: {}. Raw json_text: {}", e, json_text);
                                }
                            }
                        }
                    } else {
                        eprintln!("DEBUG JSON Parse Error: Could not parse extracted json_text. Raw: {}", json_text);
                    }
                } else {
                    eprintln!("DEBUG JSON Parse Error: Could not extract json substring from content: {}", content);
                }
            } else {
                eprintln!("DEBUG JSON Parse Error: Response missing choices[0].message.content. Full text: {}", text);
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse OpenAI batch response as classification JSON".into(),
        ))
    }
}
