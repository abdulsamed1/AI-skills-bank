pub mod provider;
pub mod prompt;
pub mod types;
pub mod error;
pub mod config;
pub mod tls;
pub mod providers;
pub mod cache;
pub mod relevance;
pub mod cost;
pub mod dedup;

// #[cfg(test)]
// pub mod tests;

pub use prompt::build_classification_prompt;
pub use provider::LlmProvider;
pub use types::{LlmClassificationResponse, SubHubSuggestion};
pub use error::LlmError;
pub use config::LlmClientConfig;
pub use providers::{ClaudeProvider, OpenAiProvider, CustomProvider, MockProvider, GeminiProvider, GroqProvider, RotationProvider};
pub use cache::{
	load_cache,
	load_cache_entries,
	save_cache,
	cache_file_path,
	key_for_skill,
	key_for_skill_versioned,
	cache_entry_usable,
	cache_ttl_days,
	get_cached_classification,
	insert_into_map,
	invalidate_key,
	CacheMetrics,
	cache_metrics,
	CacheFileEntry,
};
