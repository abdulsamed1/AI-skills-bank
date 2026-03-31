use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::LlmProvider;
use crate::components::llm::types::LlmClassificationResponse;
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
}

use crate::components::llm::provider::extract_json_substring;

#[async_trait]
impl LlmProvider for GeminiProvider {
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
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent");

        let system_prompt = "You are a classification assistant.\nGiven skill metadata, return only a JSON object with a top-level key `ranked_suggestions` containing an array of up to 3 objects {\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...}.";

        let user_payload = serde_json::json!({
            "skill_id": skill_id,
            "description": description,
            "abstract": abstract_text.unwrap_or("")
        });

        let body = serde_json::json!({
            "systemInstruction": {
                "parts": [{"text": system_prompt}]
            },
            "contents": [
                {
                    "parts": [
                        {"text": format!("Classify this skill: {}", user_payload)}
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
                "Gemini request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(candidates) = v.get("candidates").and_then(|c| c.as_array()) {
                if let Some(first_candidate) = candidates.first() {
                    if let Some(finish_reason) = first_candidate.get("finishReason").and_then(|f| f.as_str()) {
                        if finish_reason == "SAFETY" || finish_reason == "RECITATION" {
                            return Err(LlmError::InvalidResponse(format!("Gemini request blocked due to safety filter: {}", text)));
                        }
                    }

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
            "Unable to parse Gemini response as classification JSON".into(),
        ))
    }

    fn name(&self) -> &'static str {
        "gemini"
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
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent");

        let system_prompt = "You are a classification assistant.\nGiven an array of skill metadata, return a JSON array (same order) where each element is an object with key `ranked_suggestions` containing an array of up to 3 objects {\"hub\":..., \"sub_hub\":..., \"confidence\":0-100, \"reasoning\":...}. Return only valid JSON.";

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
            "systemInstruction": {
                "parts": [{"text": system_prompt}]
            },
            "contents": [
                {
                    "parts": [
                        {"text": user_content}
                    ]
                }
            ],
            "generationConfig": {
                "responseMimeType": "application/json",
                "temperature": 0.0,
                "maxOutputTokens": 1500
            }
        });

        println!("Gemini: sending request to {}...", api_url);
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
                "Gemini batch request failed: {} - {}",
                status, text
            )));
        }

        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(candidates) = v.get("candidates").and_then(|c| c.as_array()) {
                if let Some(first_candidate) = candidates.first() {
                    if let Some(finish_reason) = first_candidate.get("finishReason").and_then(|f| f.as_str()) {
                        if finish_reason == "SAFETY" || finish_reason == "RECITATION" {
                            return Err(LlmError::InvalidResponse(format!("Gemini batch request blocked due to safety filter: {}", text)));
                        }
                    }

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
            }
        }

        Err(LlmError::InvalidResponse(
            "Unable to parse Gemini batch response as classification JSON".into(),
        ))
    }
}
