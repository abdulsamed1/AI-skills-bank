use crate::components::llm::types::{LlmClassificationResponse, LlmClassificationContext};
use crate::components::llm::error::LlmError;
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
        context: &LlmClassificationContext,
    ) -> Result<LlmClassificationResponse, LlmError>;

    /// Optional batch classification. Default implementation calls `classify` for each item.
    async fn classify_batch(
        &self,
        items: &[(String, String, Option<String>)],
        context: &LlmClassificationContext,
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        let mut out = Vec::with_capacity(items.len());
        for (skill_id, description, abstract_text) in items.iter() {
            let resp = self
                .classify(skill_id, description, abstract_text.as_deref(), context)
                .await?;
            out.push(resp);
        }
        Ok(out)
    }

    fn name(&self) -> &'static str;
}

pub fn extract_json_substring(s: &str) -> Option<String> {
    // 1. More robust markdown block removal
    let mut cleaned = s.to_string();
    if let Some(start) = s.find("```") {
        // Skip opening fence and optional language tag
        let after_fence = &s[start + 3..];
        let start_content = after_fence.find('\n').map(|i| start + 3 + i + 1).unwrap_or(start + 3);
        
        if let Some(end) = s.rfind("```") {
            if end > start_content {
                cleaned = s[start_content..end].trim().to_string();
            } else {
                cleaned = s[start_content..].trim().to_string();
            }
        } else {
            cleaned = s[start_content..].trim().to_string();
        }
    }

    // 2. Find the FIRST occurrence of '{' or '['
    let first_brace = cleaned.find('{');
    let first_bracket = cleaned.find('[');
    
    let first = match (first_brace, first_bracket) {
        (Some(b), Some(br)) => usize::min(b, br),
        (Some(b), None) => b,
        (None, Some(br)) => br,
        (None, None) => return None,
    };

    // 3. Find the LAST occurrence of '}' or ']' that corresponds to the starting type
    let last = if cleaned.as_bytes()[first] == b'{' {
        cleaned.rfind('}')
    } else {
        cleaned.rfind(']')
    };

    if let Some(last_idx) = last {
        if last_idx >= first {
            return Some(cleaned[first..=last_idx].to_string());
        }
    }
    None
}

pub fn lenient_parse_classification(v: &serde_json::Value) -> Option<LlmClassificationResponse> {
    let mut out = LlmClassificationResponse { ranked_suggestions: vec![] };
    
    // 1. Try to find a list of suggestions anywhere in the object (handle double-nesting)
    if let Some(arr) = find_suggestions_array(v) {
        for item in arr {
            if let Some(sugg) = parse_single_suggestion(item) {
                out.ranked_suggestions.push(sugg);
            }
        }
    } 

    // 2. If no array found, maybe the object itself is a single suggestion
    if out.ranked_suggestions.is_empty() {
        if let Some(sugg) = parse_single_suggestion(v) {
            out.ranked_suggestions.push(sugg);
        }
    }

    if out.ranked_suggestions.is_empty() { None } else { Some(out) }
}

/// Recursively look for a "ranked_suggestions" or "suggestions" array
fn find_suggestions_array(v: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
    if let Some(arr) = v.get("ranked_suggestions").or_else(|| v.get("suggestions")).and_then(|s| s.as_array()) {
        return Some(arr);
    }
    
    if let Some(obj) = v.as_object() {
        for (_k, val) in obj {
            if let Some(found) = find_suggestions_array(val) {
                return Some(found);
            }
        }
    }
    None
}

fn parse_single_suggestion(item: &serde_json::Value) -> Option<crate::components::llm::types::SubHubSuggestion> {
    // If this is an object that contains another suggestion array, recurse into its first item
    if let Some(arr) = item.get("ranked_suggestions").or_else(|| item.get("suggestions")).and_then(|s| s.as_array()) {
        if let Some(first) = arr.first() {
            return parse_single_suggestion(first);
        }
    }

    let hub = item.get("hub")
        .or_else(|| item.get("hubed")) // Handle common model typo
        .or_else(|| item.get("category"))
        .and_then(|v| v.as_str())?;
        
    let sub_hub = item.get("sub_hub")
        .or_else(|| item.get("subhub"))
        .or_else(|| item.get("sub_category"))
        .and_then(|v| v.as_str())?;
        
    let confidence = item.get("confidence")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(100);
        
    let reasoning = item.get("reasoning")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
        
    Some(crate::components::llm::types::SubHubSuggestion {
        hub: hub.to_string(),
        sub_hub: sub_hub.to_string(),
        confidence,
        reasoning: Some(reasoning),
    })
}

pub fn lenient_parse_batch(v: &serde_json::Value) -> Option<Vec<LlmClassificationResponse>> {
    let results = v.get("results")
        .or_else(|| v.get("classifications"))
        .and_then(|r| r.as_array())
        .or_else(|| v.as_array()); // Maybe it's just a raw array

    if let Some(arr) = results {
        let mut out = vec![];
        for item in arr {
            if let Some(parsed) = lenient_parse_classification(item) {
                out.push(parsed);
            }
        }
        if !out.is_empty() { return Some(out); }
    }
    None
}
