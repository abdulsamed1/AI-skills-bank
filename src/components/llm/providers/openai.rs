use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::LlmProvider;
use crate::components::llm::types::LlmClassificationResponse;
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
        // sensible default timeout
        builder = builder.timeout(Duration::from_secs(30));
        let client = builder.build().map_err(|e| LlmError::NetworkError(e.to_string()))?;
        Ok(Self { config, client })
    }
}

fn extract_json_substring(s: &str) -> Option<String> {
    // Remove triple-backtick fences if present
    let cleaned = if let Some(start) = s.find("```") {
        if let Some(end) = s.rfind("```") {
            if end > start {
                let inner = &s[start + 3..end];
                inner.trim().to_string()
            } else {
                s.to_string()
            }
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    };

    if let Some(first) = cleaned.find('{') {
        if let Some(last) = cleaned.rfind('}') {
            return Some(cleaned[first..=last].to_string());
        }
    }
    None
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
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
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let system_prompt = "You are a classification assistant.\nGiven skill metadata, return a JSON object with a top-level key `ranked_suggestions` containing an array of up to 3 objects {\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...}. Return only valid JSON (no additional commentary).";

        let user_payload = serde_json::json!({
            "skill_id": skill_id,
            "description": description,
            "abstract": abstract_text.unwrap_or("")
        });

        let body = serde_json::json!({
            "model": "gpt-4o-mini",
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

        // Try parse JSON from the standard completion structure
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

            // Fallback: sometimes OpenAI includes `choices[0].text`
            if let Some(content) = v
                .get("choices")
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

        // Last resort: try to parse the entire response as JSON
        if let Ok(parsed) = serde_json::from_str::<LlmClassificationResponse>(&text) {
            return Ok(parsed);
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
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        if items.is_empty() {
            return Ok(vec![]);
        }

        let api_url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let system_prompt = "You are a classification assistant.\nGiven an array of skill metadata, return a JSON array (same order) where each element is an object with key `ranked_suggestions` containing an array of up to 3 objects {\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...}. Return only valid JSON (no additional commentary).";

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
            "model": "gpt-4o-mini",
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
                "OpenAI batch request failed: {} - {}",
                status, text
            )));
        }

        // Try parse JSON from choices
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(content) = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    // Try parse as array of LlmClassificationResponse
                    if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                        return Ok(parsed);
                    }

                    // Try parse as object { results: [...] }
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

                        // Try parse as mapping skill_id -> suggestion
                        if parsed_obj.is_object() {
                            let mut out = Vec::new();
                            for it in payload_items.iter() {
                                if let Some(skill_id) = it.get("skill_id").and_then(|s| s.as_str()) {
                                    if let Some(val) = parsed_obj.get(skill_id) {
                                        if let Ok(parsed_item) = serde_json::from_value::<LlmClassificationResponse>(val.clone()) {
                                            out.push(parsed_item);
                                            continue;
                                        }
                                    }
                                }
                                return Err(LlmError::InvalidResponse("Missing mapping for skill_id".into()));
                            }
                            return Ok(out);
                        }
                    }
                }
            }
        }

        // Last resort: try to parse entire text as array
        if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&text) {
            return Ok(parsed);
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse OpenAI batch response as classification JSON".into(),
        ))
    }
}
