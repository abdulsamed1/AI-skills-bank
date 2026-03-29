use thiserror::Error;
use serde_json::json;

#[derive(Error, Debug)]
pub enum SkillManageError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Manifest parse error: {0}")]
    ManifestParseError(String),
    
    #[error("Manifest validation error: {0}")]
    ManifestValidationError(String),
    
    #[error("Git error: {0}")]
    GitError(String),
    
    #[error("Unknown error")]
    Unknown,
}

impl SkillManageError {
    pub fn to_json(&self) -> serde_json::Value {
        let (code, message) = match self {
            Self::ConfigError(m) => ("CONFIG_ERROR", m.clone()),
            Self::IoError(e) => ("IO_ERROR", e.to_string()),
            Self::ManifestParseError(m) => ("MANIFEST_PARSE_ERROR", m.clone()),
            Self::ManifestValidationError(m) => ("MANIFEST_VALIDATION_ERROR", m.clone()),
            Self::GitError(m) => ("GIT_ERROR", m.clone()),
            Self::Unknown => ("UNKNOWN_ERROR", "An unknown error occurred".to_string()),
        };

        json!({
            "status": "error",
            "error": {
                "code": code,
                "message": message
            }
        })
    }
}
