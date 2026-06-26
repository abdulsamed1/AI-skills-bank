use crate::components::llm::config::LlmClientConfig;
use crate::components::llm::error::LlmError;
use crate::components::llm::provider::LlmProvider;
use crate::components::llm::types::{LlmClassificationResponse, LlmClassificationContext, SubHubSuggestion};
use crate::components::llm::tls;
use async_trait::async_trait;
use serde_json::json;

pub struct CustomProvider {
    pub config: LlmClientConfig,
    pub client: reqwest::Client,
}

impl CustomProvider {
    pub fn new(config: LlmClientConfig) -> Result<Self, LlmError> {
        let builder = tls::build_client_builder()?;
        let client = builder.build().map_err(|e| LlmError::NetworkError(e.to_string()))?;
        Ok(Self { config, client })
    }

    /// Make a request to OpenAI-compatible API (works with freellmapi, OpenAI, Anthropic, etc.)
    async fn openai_compat_request(
        &self,
        messages: Vec<serde_json::Value>,
    ) -> Result<String, LlmError> {
        let api_url = self.config.api_url.as_ref()
            .ok_or_else(|| LlmError::ConfigError("api_url not set for custom provider".to_string()))?;

        // Accept either the full completions URL or the base /v1 URL. If the
        // configured api_url contains a chat/completions path, use it as-is.
        // Otherwise derive the completions endpoint from the base (e.g. /v1 -> /v1/chat/completions)
        let completions_url = if api_url.contains("/chat") {
            api_url.clone()
        } else if api_url.ends_with("/v1") || api_url.ends_with("/v1/") {
            format!("{}/chat/completions", api_url.trim_end_matches('/'))
        } else if api_url.ends_with('/') {
            format!("{}chat/completions", api_url)
        } else {
            format!("{}/chat/completions", api_url)
        };

        let model = self.config.model.as_ref()
            .unwrap_or(&"auto".to_string())
            .clone();

        let body = json!({
            "model": model,
            "messages": messages,
            "temperature": 0.3,
            "top_p": 0.9,
            "max_tokens": 4096,
        });

        let response = self.client
            .post(&completions_url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.status().map(|s| s.as_u16()) == Some(429) {
                    LlmError::RateLimited { retry_after: None }
                } else if e.is_timeout() {
                    LlmError::NetworkError("Request timeout".to_string())
                } else {
                    LlmError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();
        if status.as_u16() == 429 {
            return Err(LlmError::RateLimited { retry_after: None });
        }
        if status.as_u16() == 401 || status.as_u16() == 403 {
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::AuthenticationFailed(text));
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            // Transient gateway errors should be retried via the pipeline's backoff logic
            if status.as_u16() == 502 || status.as_u16() == 503 || status.as_u16() == 504 {
                return Err(LlmError::ProviderUnavailable(format!("{} Bad Gateway: {}", status.as_u16(), text)));
            }
            return Err(LlmError::InvalidResponse(format!("{}: {}", status, text)));
        }

        let resp_json: serde_json::Value = response.json().await
            .map_err(|e| LlmError::InvalidResponse(format!("Invalid JSON response: {}", e)))?;

        let content_val = resp_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .ok_or_else(|| LlmError::InvalidResponse("No content in response".to_string()))?;

        let content = match content_val {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        Ok(content)
    }
}

#[async_trait]
impl LlmProvider for CustomProvider {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
        _context: &LlmClassificationContext,
    ) -> Result<LlmClassificationResponse, LlmError> {
        let abstract_part = abstract_text
            .map(|a| format!("Abstract/Body:\n{}\n\n", a))
            .unwrap_or_default();

        let prompt = format!(
            "Classify this AI skill into ONE primary hub and ONE primary sub-hub.\n\n\
Skill ID: {}\n\
Description: {}\n\
{}\
\n\
Return ONLY valid JSON with this exact structure (no markdown, no extra text):\n\
{{\n\
  \"hub\": \"<hub_name>\",\n\
  \"sub_hub\": \"<sub_hub_name>\",\n\
  \"confidence\": <0-100>,\n\
  \"reasoning\": \"<brief reason>\"\n\
}}\n\n\
Valid hubs: code-quality, server-side, frontend, business\n\
Valid sub-hubs (examples): testing-qa, security, performance, product-management, ux-design, marketing",
            skill_id, description, abstract_part
        );

        let messages = vec![
            json!({
                "role": "system",
                "content": "You are a JSON classification API. Your entire response must be a single valid JSON object. Do NOT output any reasoning, thinking, analysis, numbered steps, markdown, or explanatory text. Output ONLY the raw JSON object and nothing else."
            }),
            json!({
                "role": "user",
                "content": prompt
            }),
        ];

        let response = self.openai_compat_request(messages).await?;

        // Robust JSON extraction: models behind FreeLLMAPI may prefix JSON
        // with reasoning text or wrap it in markdown fences. Find the first
        // valid JSON object in the response.
        let parsed: serde_json::Value = extract_json_object(&response)
            .ok_or_else(|| LlmError::InvalidResponse(format!(
                "Failed to parse JSON response: no valid JSON object found in: {}",
                &response[..response.len().min(200)]
            )))?;

        let suggestion = SubHubSuggestion {
            hub: parsed.get("hub").and_then(|v| v.as_str()).unwrap_or("unclassified").to_string(),
            sub_hub: parsed.get("sub_hub").and_then(|v| v.as_str()).unwrap_or("general").to_string(),
            confidence: parsed.get("confidence").and_then(|v| v.as_u64()).unwrap_or(50) as u32,
            reasoning: parsed.get("reasoning").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };

        Ok(LlmClassificationResponse {
            ranked_suggestions: vec![suggestion],
        })
    }

    async fn classify_batch(
        &self,
        items: &[(String, String, Option<String>)],
        _context: &LlmClassificationContext,
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        let mut results = Vec::new();
        
        for (skill_id, description, abstract_text) in items {
            match self.classify(skill_id, description, abstract_text.as_deref(), _context).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Transient upstream errors (502/503/504, rate limits) should
                    // abort the batch immediately so the pipeline retries with backoff.
                    if matches!(&e, LlmError::ProviderUnavailable(_) | LlmError::RateLimited { .. } | LlmError::Timeout | LlmError::NetworkError(_)) {
                        eprintln!("Batch classify transient error for {}: {:?}. Aborting batch for retry.", skill_id, e);
                        return Err(e);
                    }
                    // Non-transient errors (InvalidResponse from reasoning models):
                    // log and insert a fallback so remaining items aren't wasted.
                    eprintln!("Batch classify parse error for {}: {:?}. Using fallback.", skill_id, e);
                    results.push(LlmClassificationResponse {
                        ranked_suggestions: vec![SubHubSuggestion {
                            hub: "unclassified".to_string(),
                            sub_hub: "general".to_string(),
                            confidence: 0,
                            reasoning: Some("LLM response was not parseable JSON".to_string()),
                        }],
                    });
                }
            }
        }
        
        Ok(results)
    }

    fn name(&self) -> &'static str {
        "custom"
    }
}

/// Extract the first valid JSON object from a string that may contain
/// surrounding text (reasoning, markdown fences, etc.).
fn extract_json_object(text: &str) -> Option<serde_json::Value> {
    let s = text.trim();

    // Fast path: entire string is valid JSON
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
        if v.is_object() {
            return Some(v);
        }
    }

    // Strip markdown fences and retry
    let stripped = s
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(stripped) {
        if v.is_object() {
            return Some(v);
        }
    }

    // Scan for the first '{' and try progressively larger substrings
    // ending at each '}'. This handles reasoning prefixes.
    let bytes = s.as_bytes();
    for (start, _) in bytes.iter().enumerate().filter(|(_, &b)| b == b'{') {
        let mut depth = 0i32;
        for end in start..bytes.len() {
            match bytes[end] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s[start..=end]) {
                            if v.is_object() {
                                return Some(v);
                            }
                        }
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    None
}
