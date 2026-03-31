pub mod provider;
pub mod types;
pub mod error;
pub mod config;
pub mod tls;
pub mod providers;
pub mod cache;

#[cfg(test)]
pub mod tests;

pub use provider::LlmProvider;
pub use types::{LlmClassificationResponse, SubHubSuggestion};
pub use error::LlmError;
pub use config::LlmClientConfig;
pub use providers::{ClaudeProvider, OpenAiProvider, CustomProvider, MockProvider};
pub use cache::{
	load_cache,
	save_cache,
	cache_file_path,
	key_for_skill,
	get_cached_classification,
	insert_into_map,
	invalidate_key,
	CacheMetrics,
	cache_metrics,
};
