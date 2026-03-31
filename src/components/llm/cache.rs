use crate::components::llm::types::LlmClassificationResponse;
use crate::error::SkillManageError;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn default_cache_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("LLM_CACHE_PATH") {
        return Some(PathBuf::from(p));
    }
    home_dir().map(|mut h| {
        h.push(".skill-manage");
        // ensure dir exists
        let _ = fs::create_dir_all(&h);
        h.push("llm-classifications.json");
        h
    })
}

pub fn cache_file_path() -> Result<PathBuf, SkillManageError> {
    default_cache_path().ok_or_else(|| SkillManageError::ConfigError("Unable to resolve home directory for LLM cache; set LLM_CACHE_PATH".to_string()))
}

pub fn load_cache() -> Result<HashMap<String, LlmClassificationResponse>, SkillManageError> {
    let path = cache_file_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let data = fs::read_to_string(&path)?;
    let parsed: HashMap<String, LlmClassificationResponse> = serde_json::from_str(&data)
        .map_err(|e| SkillManageError::ConfigError(format!("Failed parsing LLM cache {}: {}", path.display(), e)))?;
    Ok(parsed)
}

pub fn save_cache(map: &HashMap<String, LlmClassificationResponse>) -> Result<(), SkillManageError> {
    let path = cache_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec_pretty(map).map_err(|e| SkillManageError::ConfigError(format!("Failed serializing LLM cache: {}", e)))?;
    fs::write(&path, &data)?;
    Ok(())
}
