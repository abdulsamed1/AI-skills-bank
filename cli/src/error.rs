use thiserror::Error;

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
