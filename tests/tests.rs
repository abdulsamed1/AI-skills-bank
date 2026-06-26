#[cfg(test)]
mod tests {
    use skills_bank::components::llm::config::LlmClientConfig;
    use skills_bank::components::llm::provider::LlmProvider;
    use skills_bank::components::llm::providers::MockProvider;
    use skills_bank::components::llm::types::LlmClassificationContext;
    use skills_bank::components::llm::tls;
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
            .classify("example-security-skill", "A skill about security", Some("Abstract mentioning security."), &LlmClassificationContext::default())
            .await
            .expect("classification ok");

        assert!(!resp.ranked_suggestions.is_empty());
        let first = &resp.ranked_suggestions[0];
        assert_eq!(first.hub, "code-quality");
        assert_eq!(first.sub_hub, "security");
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
        use skills_bank::components::llm::cache::{key_for_skill, load_cache, save_cache, insert_into_map, cache_metrics};
        use skills_bank::components::llm::types::{LlmClassificationResponse, SubHubSuggestion};

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
        let key1 = key_for_skill(repo_root, &skill_path, "My Skill", "A short description", None);
        let key2 = key_for_skill(repo_root, &skill_path, "My Skill", "A short description", None);
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 64);

        // Insert and save
        let mut m: std::collections::HashMap<String, LlmClassificationResponse> = std::collections::HashMap::new();
        let suggestion = SubHubSuggestion { hub: "code-quality".to_string(), sub_hub: "security".to_string(), confidence: 90, reasoning: None };
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
        use skills_bank::components::llm::cache::load_cache;

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
        let src = root.path().join("src").join("demo-repo").join("skills").join("security-skill");
        fs::create_dir_all(&src)?;

        let skill_md = r#"---
name: security-audit
description: Learn about security auditing and pentesting.
---

content
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");
        let skills = skills_bank::components::native_pipeline::aggregate_to_output(root.path(), &output, None::<&std::collections::HashSet<String>>, false, false).await?;

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].hub, "code-quality");
        assert_eq!(skills[0].sub_hub, "security");

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
        let src = root.path().join("src").join("demo-bogus").join("skills").join("security-skill");
        fs::create_dir_all(&src)?;

        let skill_md = r#"---
name: security-audit
description: Learn about security auditing and pentesting.
---

content
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");
        let skills = skills_bank::components::native_pipeline::aggregate_to_output(root.path(), &output, None::<&std::collections::HashSet<String>>, false, false).await?;

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].hub, "code-quality");
        assert_eq!(skills[0].sub_hub, "security");

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
        let src = root.path().join("src").join("demo-cache").join("skills").join("security-skill");
        fs::create_dir_all(&src)?;

        let skill_md = r#"---
name: security-audit
description: Learn about security auditing and pentesting.
---

content
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");

        // First run: populate cache (mock provider behaves normally)
        env::remove_var("LLM_MOCK_FAIL");
        let skills1 = skills_bank::components::native_pipeline::aggregate_to_output(root.path(), &output, None::<&std::collections::HashSet<String>>, false, false).await?;
        assert_eq!(skills1.len(), 1);
        assert_eq!(skills1[0].hub, "code-quality");
        assert_eq!(skills1[0].sub_hub, "security");

        // Now simulate provider failing, but cache should be consulted and avoid provider call
        env::set_var("LLM_MOCK_FAIL", "1");
        let skills2 = skills_bank::components::native_pipeline::aggregate_to_output(root.path(), &output, None::<&std::collections::HashSet<String>>, false, false).await?;
        assert_eq!(skills2.len(), 1);
        assert_eq!(skills2[0].hub, "code-quality");
        assert_eq!(skills2[0].sub_hub, "security");

        // Cleanup env
        env::remove_var("LLM_PROVIDER");
        env::remove_var("LLM_API_KEY");
        env::remove_var("LLM_CACHE_PATH");
        env::remove_var("LLM_MOCK_FAIL");

        Ok(())
    }

    #[tokio::test]
    async fn test_llm_max_body_chars_respected() -> Result<(), Box<dyn std::error::Error>> {
        use tempfile::tempdir;
        use std::fs;
        use std::env;

        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("LLM_PROVIDER", "mock");
        env::set_var("LLM_API_KEY", "test-key");
        env::set_var("LLM_MAX_BODY_CHARS", "10");

        let cache_dir = tempdir()?;
        let cache_file = cache_dir.path().join("llm-cache-chars.json");
        env::set_var("LLM_CACHE_PATH", cache_file.to_string_lossy().to_string());

        let root = tempdir()?;
        let src = root.path().join("src").join("demo-chars").join("skills").join("audit-skill");
        fs::create_dir_all(&src)?;

        // The abstract text contains the word "security", but it is past the 10 character mark.
        // Truncated to 10 chars, it will be "Abstract m".
        let skill_md = r#"---
name: audit
description: Learn about audit processes.
---

Abstract mentioning security.
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");
        let skills = skills_bank::components::native_pipeline::aggregate_to_output(
            root.path(),
            &output,
            None::<&std::collections::HashSet<String>>,
            false,
            false,
        )
        .await?;

        assert_eq!(skills.len(), 1);
        // The mock provider returns confidence 75 if "security" is not matched in the abstract.
        // It returns 95 if "security" is matched.
        // Since it is truncated to 10 chars, "security" should not match, and confidence should be 75.
        assert_eq!(skills[0].match_score, Some(75));

        // Let's verify that if we set it to 100, the full body is matched, resulting in 95.
        env::set_var("LLM_MAX_BODY_CHARS", "100");
        // Invalidate the cache to force a new request
        fs::remove_file(&cache_file)?;

        let skills_full = skills_bank::components::native_pipeline::aggregate_to_output(
            root.path(),
            &output,
            None::<&std::collections::HashSet<String>>,
            false,
            false,
        )
        .await?;

        assert_eq!(skills_full.len(), 1);
        assert_eq!(skills_full[0].match_score, Some(95));

        // Cleanup env
        env::remove_var("LLM_PROVIDER");
        env::remove_var("LLM_API_KEY");
        env::remove_var("LLM_CACHE_PATH");
        env::remove_var("LLM_MAX_BODY_CHARS");

        Ok(())
    }
}
