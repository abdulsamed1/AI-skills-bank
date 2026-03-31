use skill_manage::components::syncer::Syncer;
use skill_manage::utils::progress::ProgressManager;
use skill_manage::utils::theme::Theme;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_sync_logic() -> Result<(), Box<dyn std::error::Error>> {
    let src_root = tempdir()?;
    let dest_root = tempdir()?;

    // Create a mock skill directory
    let skill_dir = src_root.path().join("my-skill");
    fs::create_dir_all(&skill_dir)?;
    fs::write(skill_dir.join("SKILL.md"), "test skill")?;

    // Create a non-skill directory
    let other_dir = src_root.path().join("other");
    fs::create_dir_all(&other_dir)?;
    fs::write(other_dir.join("README.md"), "not a skill")?;

    // Initialize Syncer
    let progress = Arc::new(ProgressManager::new(false, false, Arc::new(Theme::new()), None));
    let syncer = Syncer::new(progress);

    // Change current directory to include lib/ (we'd need to mock the path in Syncer)
    // Actually, let's modify Syncer to accept src_path if needed, or just rely on manual verification for now.
    // Wait, Syncer uses Path::new("src"). Let's try to create a "src" folder in the current dir for the test.

    let current_dir = std::env::current_dir()?;
    let test_src = current_dir.join("src");
    if !test_src.exists() {
        fs::create_dir_all(&test_src)?;
    }

    let test_skill = test_src.join("test-sync-skill");
    fs::create_dir_all(&test_skill)?;
    fs::write(test_skill.join("SKILL.md"), "test")?;

    let dest_path = dest_root.path().to_str().unwrap().to_string();
    let result = syncer.sync(Some(dest_path), false, false).await;

    assert!(result.is_ok());
    assert!(dest_root.path().join("test-sync-skill").exists());
    assert!(dest_root
        .path()
        .join("test-sync-skill")
        .join("SKILL.md")
        .exists());

    // Cleanup
    fs::remove_dir_all(test_skill)?;

    Ok(())
}
