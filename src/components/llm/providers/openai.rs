use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::{extract_json_substring, LlmProvider};
use crate::components::llm::types::{LlmClassificationResponse, LlmClassificationContext};
use crate::components::llm::tls;
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

pub struct OpenAiProvider {
    pub config: LlmClientConfig,
    pub client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(config: LlmClientConfig) -> Result<Self, LlmError> {
        let mut builder = tls::build_client_builder()?;
        builder = builder.timeout(Duration::from_secs(30));
        let client = builder.build().map_err(|e| LlmError::NetworkError(e.to_string()))?;
        Ok(Self { config, client })
    }

    fn get_model(&self) -> String {
        std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string())
    }

    fn build_system_prompt(&self, context: &LlmClassificationContext, is_batch: bool) -> String {
        let mut prompt = "You are a classification assistant.\n".to_string();
        prompt.push_str("Valid Hubs: ");
        prompt.push_str(&context.valid_hubs.join(", "));
        prompt.push_str("\nValid Sub-Hubs: ");
        prompt.push_str(&context.valid_sub_hubs.join(", "));
        prompt.push_str("\nExcluded Categories: ");
        prompt.push_str(&context.excluded_categories.join(", "));
        prompt.push_str("\n\nInstructions:\n");
        prompt.push_str("1. Classify the skill metadata provided.\n");
        prompt.push_str("2. If a skill matches an Excluded Category, return 'excluded' for both hub and sub_hub.\n");
        prompt.push_str("3. Use the provided Valid Hubs and Sub-Hubs otherwise.\n");
        if is_batch {
            prompt.push_str("4. Return a JSON array of objects (one per skill).\n");
        } else {
            prompt.push_str("4. Return a JSON object with 'ranked_suggestions' array.\n");
        }
        prompt.push_str("Each object: {\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...}.\n");
        prompt.push_str("Return ONLY valid JSON.");
        prompt
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

        let system_prompt = self.build_system_prompt(context, false);

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

        let system_prompt = self.build_system_prompt(context, true);

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
            "max_tokens": 1500,
        });

        let resp = self
            .client
            .post(api_url)
            .bearer_auth(&self.config.api_key)
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
                    if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                        return Ok(parsed);
                    }
                }
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse OpenAI batch response as classification JSON".into(),
        ))
    }
}
