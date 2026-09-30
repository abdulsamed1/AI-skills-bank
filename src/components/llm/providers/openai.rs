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
        self.config
            .model
            .clone()
            .unwrap_or_else(|| {
                std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string())
            })
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

        let mut req_builder = self
            .client
            .post(api_url)
            .bearer_auth(&self.config.api_key)
            .header("User-Agent", "skills-bank/0.1")
            .json(&body);

        if self.config.provider.to_ascii_lowercase() == "github" {
            req_builder = req_builder.header("X-GitHub-Api-Version", "2022-11-28");
        }

        let resp = req_builder
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
            crate::components::llm::cost::record_usage("openai", &v);
            if let Some(content) = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(val) = serde_json::from_str::<Value>(&json_text) {
                        if let Some(parsed) = crate::components::llm::provider::lenient_parse_classification(&val) {
                            return Ok(parsed);
                        }
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
            "max_tokens": 4096
        });

        let mut req_builder = self
            .client
            .post(api_url)
            .bearer_auth(&self.config.api_key)
            .header("User-Agent", "skills-bank/0.1")
            .json(&body);

        if self.config.provider.to_ascii_lowercase() == "github" {
            req_builder = req_builder.header("X-GitHub-Api-Version", "2022-11-28");
        }

        let resp = req_builder
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
                                if let Some(parsed) = crate::components::llm::provider::lenient_parse_batch(&value) {
                                    if parsed.len() == items.len() {
                                        return Ok(parsed);
                                    } else {
                                        return Err(LlmError::InvalidResponse(format!("Batch length mismatch in 400 recovery: expected {}, got {}", items.len(), parsed.len())));
                                    }
                                }
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
            crate::components::llm::cost::record_usage("openai", &v);
            if let Some(content) = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
            {
                if let Some(json_text) = extract_json_substring(content) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_text) {
                        if let Some(parsed) = crate::components::llm::provider::lenient_parse_batch(&value) {
                            if parsed.len() == items.len() {
                                return Ok(parsed);
                            } else {
                                return Err(LlmError::InvalidResponse(format!("Batch length mismatch: expected {}, got {}", items.len(), parsed.len())));
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

    async fn classify_panel(
        &self,
        items: &[(String, String, Option<String>)],
        candidates: &[String],
        context: &LlmClassificationContext,
    ) -> Result<Vec<Option<Vec<(String, f64)>>>, LlmError> {
        if items.is_empty() || candidates.is_empty() {
            return Ok(vec![]);
        }
        let kind = if candidates.iter().all(|c| context.valid_hubs.contains(c)) {
            "hub"
        } else {
            "sub-hub"
        };
        let system_prompt =
            crate::components::llm::relevance::build_panel_prompt(candidates, kind);
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
        let user_content = format!(
            "Score every candidate for each skill, in order. Skills: {}",
            serde_json::to_string(&payload_items).unwrap_or_else(|_| "[]".to_string())
        );
        let body = serde_json::json!({
            "model": self.get_model(),
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_content}
            ],
            "temperature": 0.0,
            "max_tokens": 1024,
        });
        let v = self.post_chat(body).await?;
        let val = Self::panel_content(&v)?;
        // Accept both {"panels":[{"scores":[...]}]} and a bare {"scores":[...]}
        // (treated as a single-item response) — models behind FreeLLMAPI vary.
        let panels: Vec<serde_json::Value> = if let Some(arr) = val.get("panels").and_then(|p| p.as_array()) {
            arr.clone()
        } else if val.get("scores").is_some() {
            vec![val.clone()]
        } else {
            return Err(LlmError::InvalidResponse(
                "panel response missing panels array".to_string(),
            ));
        };
        // Fewer panels than items = model didn't follow batch format.
        // Treat missing items as abstentions (None), not a hard error.
        let mut out: Vec<Option<Vec<(String, f64)>>> = Vec::with_capacity(items.len());
        for i in 0..items.len() {
            out.push(panels.get(i).and_then(|p| {
                crate::components::llm::relevance::strict_parse_panel(p, candidates)
            }));
        }
        Ok(out)
    }

    async fn classify_duplicates(
        &self,
        pairs: &[(String, String, String, String)],
    ) -> Result<Vec<Option<(bool, f64)>>, LlmError> {
        if pairs.is_empty() {
            return Ok(vec![]);
        }
        let system_prompt = crate::components::llm::relevance::build_dedup_prompt();
        let payload_items: Vec<serde_json::Value> = pairs
            .iter()
            .map(|(a_name, a_desc, b_name, b_desc)| {
                serde_json::json!({
                    "a_name": a_name, "a_desc": a_desc,
                    "b_name": b_name, "b_desc": b_desc
                })
            })
            .collect();
        let user_content = format!(
            "Judge each pair in order. Pairs: {}",
            serde_json::to_string(&payload_items).unwrap_or_else(|_| "[]".to_string())
        );
        let body = serde_json::json!({
            "model": self.get_model(),
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_content}
            ],
            "temperature": 0.0,
            "max_tokens": 1024,
        });
        let v = self.post_chat(body).await?;
        let val = Self::panel_content(&v)?;
        match crate::components::llm::relevance::strict_parse_dedup(&val, pairs.len()) {
            Some(verdicts) => Ok(verdicts.into_iter().map(Some).collect()),
            None => Err(LlmError::InvalidResponse(
                "dedup response failed strict validation".to_string(),
            )),
        }
    }
}

impl OpenAiProvider {
    /// Extract the assistant message content and pull out the embedded JSON
    /// object/array. Shared by panel and verdict calls (both strictly
    /// validated downstream — extraction is lenient, validation is not).
    fn panel_content(v: &Value) -> Result<Value, LlmError> {
        let content = v
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                LlmError::InvalidResponse("panel response missing message content".to_string())
            })?;
        let json_text = extract_json_substring(content).ok_or_else(|| {
            LlmError::InvalidResponse("panel response contains no JSON".to_string())
        })?;
        serde_json::from_str(&json_text)
            .map_err(|e| LlmError::InvalidResponse(format!("panel JSON unparsable: {}", e)))
    }

    /// Shared OpenAI-compatible POST with the pipeline's standard error
    /// mapping. Records token usage for cost accounting. No 400-recovery:
    /// panel/verdict calls are strictly validated by contract.
    async fn post_chat(&self, body: serde_json::Value) -> Result<Value, LlmError> {
        let api_url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");
        let mut req_builder = self
            .client
            .post(api_url)
            .bearer_auth(&self.config.api_key)
            .header("User-Agent", "skills-bank/0.1")
            .json(&body);
        if self.config.provider.to_ascii_lowercase() == "github" {
            req_builder = req_builder.header("X-GitHub-Api-Version", "2022-11-28");
        }
        let resp = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                LlmError::Timeout
            } else {
                LlmError::NetworkError(e.to_string())
            }
        })?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp
            .text()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
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
            return Err(LlmError::ProviderUnavailable(format!(
                "OpenAI request failed: {} - {}",
                status, text
            )));
        }
        let v: Value = serde_json::from_str(&text)
            .map_err(|e| LlmError::InvalidResponse(format!("Invalid JSON response: {}", e)))?;
        crate::components::llm::cost::record_usage("openai", &v);
        Ok(v)
    }
}
