use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("rate limited")]
    RateLimited { retry_after: Option<u64> },
    #[error("timeout")]
    Timeout,
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("config error: {0}")]
    ConfigError(String),
    #[error("provider unavailable: {0}")]
    ProviderUnavailable(String),
}
