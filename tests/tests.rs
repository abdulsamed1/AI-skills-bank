#[cfg(test)]
mod tests {
    use crate::components::llm::config::LlmClientConfig;
    use crate::components::llm::provider::LlmProvider;
    use crate::components::llm::providers::MockProvider;
    use crate::components::llm::tls;
    use std::env;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn config_from_env_and_mock_provider() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("LLM_PROVIDER", "mock");
        env::set_var("LLM_API_KEY", "test-key");
        // optional: unset cert path to test builder default
        env::remove_var("LLM_CA_CERT_PATH");

        let cfg = LlmClientConfig::from_env().expect("should load config");
        assert_eq!(cfg.provider, "mock");
        assert_eq!(cfg.api_key, "test-key");

        let provider = MockProvider::new(cfg).expect("mock provider created");
        let resp = provider
            .classify("example-rust-skill", "A skill about Rust", Some("Abstract mentioning Rust."))
            .await
            .expect("classification ok");

        assert!(!resp.ranked_suggestions.is_empty());
        let first = &resp.ranked_suggestions[0];
        assert_eq!(first.hub, "programming");
        assert_eq!(first.sub_hub, "rust");
        assert!(first.confidence >= 75);
    }

    #[test]
    fn tls_builder_without_cert_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("LLM_CA_CERT_PATH");
        let builder = tls::build_client_builder().expect("builder created");
        let client = builder.build().expect("client built");
        // basic sanity: client exists
        let _ = client;
    }

    #[test]
    fn cache_roundtrip_and_key_generation() -> Result<(), Box<dyn std::error::Error>> {
        use tempfile::tempdir;
        use crate::components::llm::cache::{key_for_skill, load_cache, save_cache, insert_into_map, cache_metrics};
        use crate::components::llm::types::{LlmClassificationResponse, SubHubSuggestion};

        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir()?;
        let cache_file = dir.path().join("llm-cache.json");
        std::env::set_var("LLM_CACHE_PATH", cache_file.to_string_lossy().to_string());

        // Ensure empty start
        let initial = load_cache()?;
        assert!(initial.is_empty());

        // Key generation sanity
        let repo_root = dir.path();
        let skill_path = repo_root.join("lib/owner/skill/SKILL.md");
        let key1 = key_for_skill(repo_root, &skill_path, "My Skill", "A short description");
        let key2 = key_for_skill(repo_root, &skill_path, "My Skill", "A short description");
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 64);

        // Insert and save
        let mut m: std::collections::HashMap<String, LlmClassificationResponse> = std::collections::HashMap::new();
        let suggestion = SubHubSuggestion { hub: "programming".to_string(), sub_hub: "rust".to_string(), confidence: 90, reasoning: None };
        let resp = LlmClassificationResponse { ranked_suggestions: vec![suggestion] };
        insert_into_map(&mut m, key1.clone(), resp.clone());
        save_cache(&m)?;

        // Load and validate
        let loaded = load_cache()?;
        assert!(loaded.get(&key1).is_some());
        let metrics = cache_metrics();
        // at least one insert recorded
        assert!(metrics.inserts >= 1);

        Ok(())
    }

    #[test]
    fn cache_corrupt_file_handling() -> Result<(), Box<dyn std::error::Error>> {
        use tempfile::tempdir;
        use std::fs;
        use crate::components::llm::cache::load_cache;

        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir()?;
        let cache_file = dir.path().join("llm-corrupt.json");
        std::env::set_var("LLM_CACHE_PATH", cache_file.to_string_lossy().to_string());

        // Write invalid json
        fs::write(&cache_file, "not-a-json")?;

        let res = load_cache();
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_llm_disabled_uses_rules() -> Result<(), Box<dyn std::error::Error>> {
        use tempfile::tempdir;
        use std::fs;
        use std::env;

        let _guard = ENV_LOCK.lock().unwrap();
        // Disable LLM to force deterministic rules path
        env::set_var("LLM_ENABLED", "false");

        let root = tempdir()?;
        let src = root.path().join("src").join("demo-repo").join("skills").join("rust-skill");
        fs::create_dir_all(&src)?;

        let skill_md = r#"---
name: rust-ownership
description: Learn about Rust ownership and lifetimes.
---

content
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");
        let skills = crate::components::native_pipeline::aggregate_to_output(root.path(), &output, None, false, false).await?;

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].hub, "programming");
        assert_eq!(skills[0].sub_hub, "rust");

        env::remove_var("LLM_ENABLED");
        Ok(())
    }

    #[tokio::test]
    async fn test_unknown_provider_fallback_to_rules() -> Result<(), Box<dyn std::error::Error>> {
        use tempfile::tempdir;
        use std::fs;
        use std::env;

        let _guard = ENV_LOCK.lock().unwrap();
        // Configure an unknown provider so classify step fails and we fallback to rules
        env::set_var("LLM_PROVIDER", "bogus");
        env::set_var("LLM_API_KEY", "x");

        let root = tempdir()?;
        let src = root.path().join("src").join("demo-bogus").join("skills").join("rust-skill");
        fs::create_dir_all(&src)?;

        let skill_md = r#"---
name: rust-ownership
description: Learn about Rust ownership and lifetimes.
---

content
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");
        let skills = crate::components::native_pipeline::aggregate_to_output(root.path(), &output, None, false, false).await?;

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].hub, "programming");
        assert_eq!(skills[0].sub_hub, "rust");

        env::remove_var("LLM_PROVIDER");
        env::remove_var("LLM_API_KEY");
        Ok(())
    }

    #[tokio::test]
    async fn test_cache_used_when_provider_fails() -> Result<(), Box<dyn std::error::Error>> {
        use tempfile::tempdir;
        use std::fs;
        use std::env;

        let _guard = ENV_LOCK.lock().unwrap();
        // Use mock provider and an isolated cache path
        env::set_var("LLM_PROVIDER", "mock");
        env::set_var("LLM_API_KEY", "test-key");

        let cache_dir = tempdir()?;
        let cache_file = cache_dir.path().join("llm-cache.json");
        env::set_var("LLM_CACHE_PATH", cache_file.to_string_lossy().to_string());

        let root = tempdir()?;
        let src = root.path().join("src").join("demo-cache").join("skills").join("rust-skill");
        fs::create_dir_all(&src)?;

        let skill_md = r#"---
name: rust-ownership
description: Learn about Rust ownership and lifetimes.
---

content
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");

        // First run: populate cache (mock provider behaves normally)
        env::remove_var("LLM_MOCK_FAIL");
        let skills1 = crate::components::native_pipeline::aggregate_to_output(root.path(), &output, None, false, false).await?;
        assert_eq!(skills1.len(), 1);
        assert_eq!(skills1[0].hub, "programming");
        assert_eq!(skills1[0].sub_hub, "rust");

        // Now simulate provider failing, but cache should be consulted and avoid provider call
        env::set_var("LLM_MOCK_FAIL", "1");
        let skills2 = crate::components::native_pipeline::aggregate_to_output(root.path(), &output, None, false, false).await?;
        assert_eq!(skills2.len(), 1);
        assert_eq!(skills2[0].hub, "programming");
        assert_eq!(skills2[0].sub_hub, "rust");

        // Cleanup env
        env::remove_var("LLM_PROVIDER");
        env::remove_var("LLM_API_KEY");
        env::remove_var("LLM_CACHE_PATH");
        env::remove_var("LLM_MOCK_FAIL");

        Ok(())
    }
}
