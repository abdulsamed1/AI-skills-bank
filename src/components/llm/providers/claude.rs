use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::LlmProvider;
use crate::components::llm::types::LlmClassificationResponse;
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
}

fn extract_json_substring(s: &str) -> Option<String> {
    if let Some(start) = s.find("```") {
        if let Some(end) = s.rfind("```") {
            if end > start {
                return Some(s[start + 3..end].trim().to_string());
            }
        }
    }
    if let Some(first) = s.find('{') {
        if let Some(last) = s.rfind('}') {
            return Some(s[first..=last].to_string());
        }
    }
    None
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
    ) -> Result<LlmClassificationResponse, LlmError> {
        let api_url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1/complete");

        let prompt = format!(
            "<|system|>You are a classification assistant. Return only JSON: {{\"ranked_suggestions\": [ {{\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...}} ] }}<|end|>\n\nSkill metadata: {}",
            serde_json::json!({"skill_id": skill_id, "description": description, "abstract": abstract_text.unwrap_or("")})
        );

        let body = serde_json::json!({
            "model": "claude-3-5-sonnet",
            "prompt": prompt,
            "max_tokens": 500,
            "temperature": 0.0
        });

        let resp = self
            .client
            .post(api_url)
            .header("x-api-key", &self.config.api_key)
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
                "Anthropic request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            // Anthropic older responses may include `completion` key
            if let Some(content) = v.get("completion").and_then(|c| c.as_str()) {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(parsed) = serde_json::from_str::<LlmClassificationResponse>(&json_text) {
                        return Ok(parsed);
                    }
                }
            }

            // Newer APIs may have nested structures
            if let Some(content) = v
                .get("completion")
                .and_then(|c| c.get("content"))
                .and_then(|cc| cc.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(parsed) = serde_json::from_str::<LlmClassificationResponse>(&json_text) {
                        return Ok(parsed);
                    }
                }
            }
        }

        // Try to extract JSON directly from text
        if let Some(json_text) = extract_json_substring(&text) {
            if let Ok(parsed) = serde_json::from_str::<LlmClassificationResponse>(&json_text) {
                return Ok(parsed);
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse Anthropic response as classification JSON".into(),
        ))
    }

    fn name(&self) -> &'static str {
        "claude"
    }

    async fn classify_batch(
        &self,
        items: &[(String, String, Option<String>)],
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        if items.is_empty() {
            return Ok(vec![]);
        }

        let api_url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1/complete");

        let prompt_prefix = "<|system|>You are a classification assistant. Return only JSON: {\"ranked_suggestions\": [ {\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...} ] }<|end|>\n\n";

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

        let prompt = format!("{}Skill metadata: {}", prompt_prefix, serde_json::to_string(&payload_items).unwrap_or_else(|_| "[]".to_string()));

        let body = serde_json::json!({
            "model": "claude-3-5-sonnet",
            "prompt": prompt,
            "max_tokens": 1500,
            "temperature": 0.0
        });

        let resp = self
            .client
            .post(api_url)
            .header("x-api-key", &self.config.api_key)
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
                "Anthropic batch request failed: {} - {}",
                status, text
            )));
        }

        // Try to extract JSON
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            // older responses may include `completion` key
            if let Some(content) = v.get("completion").and_then(|c| c.as_str()) {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                        return Ok(parsed);
                    }
                    if let Ok(parsed_obj) = serde_json::from_str::<serde_json::Value>(&json_text) {
                        if let Some(arr) = parsed_obj.get("results").and_then(|r| r.as_array()) {
                            let mut out = Vec::new();
                            for item in arr {
                                if let Ok(parsed_item) = serde_json::from_value::<LlmClassificationResponse>(item.clone()) {
                                    out.push(parsed_item);
                                } else {
                                    return Err(LlmError::InvalidResponse("Invalid item in results array".into()));
                                }
                            }
                            return Ok(out);
                        }
                    }
                }
            }
        }

        if let Some(json_text) = extract_json_substring(&text) {
            if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                return Ok(parsed);
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse Anthropic batch response as classification JSON".into(),
        ))
    }
}
