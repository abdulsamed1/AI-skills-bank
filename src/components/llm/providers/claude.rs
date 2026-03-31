use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::{extract_json_substring, LlmProvider};
use crate::components::llm::types::{LlmClassificationResponse, LlmClassificationContext};
use crate::components::llm::tls;
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

pub struct ClaudeProvider {
    pub config: LlmClientConfig,
    pub client: reqwest::Client,
}

impl ClaudeProvider {
    pub fn new(config: LlmClientConfig) -> Result<Self, LlmError> {
        let mut builder = tls::build_client_builder()?;
        builder = builder.timeout(Duration::from_secs(30));
        let client = builder.build().map_err(|e| LlmError::NetworkError(e.to_string()))?;
        Ok(Self { config, client })
    }

    fn build_prompt(&self, context: &LlmClassificationContext, is_batch: bool) -> String {
        let mut prompt = "You are a classification assistant.\n".to_string();
        prompt.push_str("Valid Hubs: ");
        prompt.push_str(&context.valid_hubs.join(", "));
        prompt.push_str("\nValid Sub-Hubs: ");
        prompt.push_str(&context.valid_sub_hubs.join(", "));
        prompt.push_str("\nExcluded Categories: ");
        prompt.push_str(&context.excluded_categories.join(", "));
        prompt.push_str("\n\nInstructions:\n");
        prompt.push_str("1. Classify the skill metadata provided.\n");
        prompt.push_str("2. If a skill matches an Excluded Category, return 'excluded' as the hub and sub_hub.\n");
        prompt.push_str("3. Otherwise, use Valid Hubs and Sub-Hubs.\n");
        if is_batch {
            prompt.push_str("4. Return a JSON array of objects.\n");
        } else {
            prompt.push_str("4. Return a JSON object with 'ranked_suggestions'.\n");
        }
        prompt.push_str("Format: {\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...}.\n");
        prompt.push_str("Return ONLY valid JSON. No commentary.");
        prompt
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
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
            .unwrap_or("https://api.anthropic.com/v1/messages");

        let system_prompt = self.build_prompt(context, false);

        let body = serde_json::json!({
            "model": "claude-3-5-sonnet-20240620",
            "max_tokens": 500,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Classify this skill: {}", serde_json::json!({
                        "skill_id": skill_id,
                        "description": description,
                        "abstract": abstract_text.unwrap_or("")
                    }))
                }
            ],
            "temperature": 0.0
        });

        let resp = self
            .client
            .post(api_url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("User-Agent", "skill-manage/0.1")
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
            if status.as_u16() == 429 {
                let retry_after = headers
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                return Err(LlmError::RateLimited { retry_after });
            }
            return Err(LlmError::ProviderUnavailable(format!(
                "Anthropic request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(content) = v
                .get("content")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("text"))
                .and_then(|t| t.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(parsed) = serde_json::from_str::<LlmClassificationResponse>(&json_text) {
                        return Ok(parsed);
                    }
                }
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse Anthropic response".into(),
        ))
    }

    fn name(&self) -> &'static str {
        "claude"
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
            .unwrap_or("https://api.anthropic.com/v1/messages");

        let system_prompt = self.build_prompt(context, true);

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

        let body = serde_json::json!({
            "model": "claude-3-5-sonnet-20240620",
            "max_tokens": 1500,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Classify these skills: {}", serde_json::to_string(&payload_items).unwrap_or_default())
                }
            ],
            "temperature": 0.0
        });

        let resp = self
            .client
            .post(api_url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("User-Agent", "skill-manage/0.1")
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
            if status.as_u16() == 429 {
                let retry_after = headers
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                return Err(LlmError::RateLimited { retry_after });
            }
            return Err(LlmError::ProviderUnavailable(format!(
                "Anthropic batch request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(content) = v
                .get("content")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("text"))
                .and_then(|t| t.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                        return Ok(parsed);
                    }
                }
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse Anthropic batch response".into(),
        ))
    }
}
