use skills_bank::components::fetcher::Fetcher;
use skills_bank::components::manifest::{RepoManifest, Repository};
use skills_bank::utils::progress::ProgressManager;
use skills_bank::utils::theme::Theme;
use std::sync::Arc;

#[tokio::test]
async fn test_fetcher_ui_less_mode() {
    let progress = Arc::new(ProgressManager::new(false, false, Arc::new(Theme::new()), None));
    let manifest = RepoManifest {
        repositories: vec![Repository {
            name: "test-repo".to_string(),
            url: "https://github.com/invalid/test-repo".to_string(),
            branch: None,
        }],
    };

    let fetcher = Fetcher::with_manifest(manifest, progress);

    // Dry run should not fail even with invalid URL
    let result = fetcher.fetch(true).await;
    assert!(result.is_ok());
}
