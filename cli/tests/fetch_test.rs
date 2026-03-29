use std::sync::Arc;
use skill_manage::components::fetcher::Fetcher;
use skill_manage::components::manifest::{RepoManifest, Repository};
use skill_manage::utils::progress::ProgressManager;

#[tokio::test]
async fn test_fetcher_ui_less_mode() {
    let progress = Arc::new(ProgressManager::new(false));
    let manifest = RepoManifest {
        repositories: vec![
            Repository {
                name: "test-repo".to_string(),
                url: "https://github.com/invalid/test-repo".to_string(),
                branch: None,
            }
        ],
    };
    
    let fetcher = Fetcher::with_manifest(manifest, progress);
    
    // Dry run should not fail even with invalid URL
    let result = fetcher.fetch(true).await;
    assert!(result.is_ok());
}
