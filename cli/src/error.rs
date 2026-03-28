use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillManageError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Unknown error")]
    Unknown,
}
