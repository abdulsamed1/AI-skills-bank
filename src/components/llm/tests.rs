#[cfg(test)]
mod tests {
    use crate::components::llm::config::LlmClientConfig;
    use crate::components::llm::provider::LlmProvider;
    use crate::components::llm::providers::MockProvider;
    use crate::components::llm::tls;
    use std::env;

    #[tokio::test]
    async fn config_from_env_and_mock_provider() {
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
        std::env::remove_var("LLM_CA_CERT_PATH");
        let builder = tls::build_client_builder().expect("builder created");
        let client = builder.build().expect("client built");
        // basic sanity: client exists
        let _ = client;
    }
}
