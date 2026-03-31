use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::{extract_json_substring, LlmProvider};
use crate::components::llm::types::{LlmClassificationResponse, LlmClassificationContext};
use crate::components::llm::tls;
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

pub struct GeminiProvider {
    pub config: LlmClientConfig,
    pub client: reqwest::Client,
}

impl GeminiProvider {
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
impl LlmProvider for GeminiProvider {
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
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent");

        let system_prompt = self.build_prompt(context, false);

        let body = serde_json::json!({
            "systemInstruction": {
                "parts": [{"text": system_prompt}]
            },
            "contents": [
                {
                    "parts": [
                        {"text": format!("Classify this skill: {}", serde_json::json!({
                            "skill_id": skill_id,
                            "description": description,
                            "abstract": abstract_text.unwrap_or("")
                        }))}
                    ]
                }
            ],
            "generationConfig": {
                "responseMimeType": "application/json",
                "temperature": 0.0,
                "maxOutputTokens": 500
            }
        });

        let resp = self
            .client
            .post(api_url)
            .header("x-goog-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
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
                "Gemini request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(candidates) = v.get("candidates").and_then(|c| c.as_array()) {
                if let Some(first_candidate) = candidates.first() {
                    if let Some(content_text) = first_candidate
                        .get("content")
                        .and_then(|c| c.get("parts"))
                        .and_then(|p| p.as_array())
                        .and_then(|p| p.first())
                        .and_then(|p| p.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        if let Some(json_text) = extract_json_substring(content_text) {
                            if let Ok(parsed) = serde_json::from_str::<LlmClassificationResponse>(&json_text) {
                                return Ok(parsed);
                            }
                        }
                    }
                }
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse Gemini response".into(),
        ))
    }

    fn name(&self) -> &'static str {
        "gemini"
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
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent");

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
            "systemInstruction": {
                "parts": [{"text": system_prompt}]
            },
            "contents": [
                {
                    "parts": [
                        {"text": format!("Classify these skills: {}", serde_json::to_string(&payload_items).unwrap_or_default())}
                    ]
                }
            ],
            "generationConfig": {
                "responseMimeType": "application/json",
                "temperature": 0.0,
                "maxOutputTokens": 1500
            }
        });

        let resp = self
            .client
            .post(api_url)
            .header("x-goog-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
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
                "Gemini batch request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(candidates) = v.get("candidates").and_then(|c| c.as_array()) {
                if let Some(first_candidate) = candidates.first() {
                    if let Some(content_text) = first_candidate
                        .get("content")
                        .and_then(|c| c.get("parts"))
                        .and_then(|p| p.as_array())
                        .and_then(|p| p.first())
                        .and_then(|p| p.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        if let Some(json_text) = extract_json_substring(content_text) {
                            if let Ok(parsed) = serde_json::from_str::<Vec<LlmClassificationResponse>>(&json_text) {
                                return Ok(parsed);
                            }
                        }
                    }
                }
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse Gemini batch response".into(),
        ))
    }
}
