use crate::components::llm::types::LlmClassificationResponse;
use crate::components::llm::error::LlmError;
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn classify(
        &self,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
    ) -> Result<LlmClassificationResponse, LlmError>;

    /// Optional batch classification. Default implementation calls `classify` for each item.
    async fn classify_batch(
        &self,
        items: &[(String, String, Option<String>)],
    ) -> Result<Vec<LlmClassificationResponse>, LlmError> {
        let mut out = Vec::with_capacity(items.len());
        for (skill_id, description, abstract_text) in items.iter() {
            let resp = self
                .classify(skill_id, description, abstract_text.as_deref())
                .await?;
            out.push(resp);
        }
        Ok(out)
    }

    fn name(&self) -> &'static str;
}

pub fn extract_json_substring(s: &str) -> Option<String> {
    // Remove triple-backtick fences if present
    let cleaned = if let Some(start) = s.find("```") {
        // Find the next newline after ``` to strip language tag if present
        let start_content = s[start + 3..].find('\n').map(|i| start + 3 + i).unwrap_or(start + 3);
        if let Some(end) = s.rfind("```") {
            if end > start_content {
                s[start_content..end].trim().to_string()
            } else {
                s[start_content..].trim().to_string()
            }
        } else {
            s[start_content..].trim().to_string()
        }
    } else {
        s.to_string()
    };

    // Find the first JSON-like object or array
    let first_brace = cleaned.find('{');
    let first_bracket = cleaned.find('[');
    
    let first = match (first_brace, first_bracket) {
        (Some(b), Some(br)) => usize::min(b, br),
        (Some(b), None) => b,
        (None, Some(br)) => br,
        (None, None) => return None,
    };

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
