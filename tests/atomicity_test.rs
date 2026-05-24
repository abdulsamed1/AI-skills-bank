use skills_bank::utils::atomicity::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_write_file_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join("test.txt");
    let content = b"hello atomic";

    write_file_atomic(&file_path, content)?;

    assert_eq!(fs::read(&file_path)?, content);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_sync_dir_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let src = root.path().join("src");
    let dest = root.path().join("dest");

    fs::create_dir_all(&src)?;
    fs::write(src.join("file.txt"), "content")?;

    sync_dir_atomic(&src, &dest)?;

    assert!(dest.exists());
    assert_eq!(fs::read_to_string(dest.join("file.txt"))?, "content");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_create_link_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let src = root.path().join("src");
    let dest = root.path().join("dest");

    fs::create_dir_all(&src)?;
    fs::write(src.join("skill.txt"), "skill data")?;

    create_link_atomic(&src, &dest)?;

    assert!(dest.exists());
    assert!(is_link(&dest));
    assert_eq!(fs::read_to_string(dest.join("skill.txt"))?, "skill data");

    // Verify deletion safety: removing dest should not remove src/skill.txt
    if cfg!(windows) {
        fs::remove_dir(&dest)?;
    } else {
        fs::remove_file(&dest)?;
    }

    assert!(src.join("skill.txt").exists());
    Ok(())
}

#[test]
fn test_create_link_atomic_cleans_temp_on_existing_directory() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let src = root.path().join("src");
    let dest = root.path().join("dest");
    let temp_link_path = root.path().join("dest.link.tmp");

    fs::create_dir_all(&src)?;
    fs::write(src.join("skill.txt"), "skill data")?;

    // Existing destination simulates local skills directories that should be
    // preserved and merged, not replaced by a junction/symlink.
    fs::create_dir_all(&dest)?;
    fs::write(dest.join("existing.txt"), "keep")?;

    let result = create_link_atomic(&src, &dest);
    assert!(result.is_err());
    assert!(!temp_link_path.exists(), "temp link path should be cleaned on failure");
    assert!(dest.join("existing.txt").exists());

    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_create_link_atomic_with_trailing_slash() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let src = root.path().join("src");
    let dest_raw = root.path().join("dest");

    fs::create_dir_all(&src)?;
    fs::write(src.join("skill.txt"), "skill data")?;

    // Simulate a config target with a trailing slash (the cause of ENOTDIR).
    // The caller (sync_output_to_targets) is responsible for stripping this,
    // but create_link_atomic should still succeed with a clean path.
    let cleaned = dest_raw
        .to_string_lossy()
        .trim_end_matches('/')
        .to_string();
    let dest = std::path::PathBuf::from(cleaned);

    create_link_atomic(&src, &dest)?;

    assert!(dest.exists());
    assert!(is_link(&dest));
    assert_eq!(fs::read_to_string(dest.join("skill.txt"))?, "skill data");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_create_link_atomic_recovers_broken_symlink() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let src = root.path().join("src");
    let dest = root.path().join("dest");
    let nonexistent = root.path().join("gone");

    fs::create_dir_all(&src)?;
    fs::write(src.join("skill.txt"), "new data")?;

    // Create a broken symlink: dest -> nonexistent (which does not exist)
    std::os::unix::fs::symlink(&nonexistent, &dest)?;
    assert!(!dest.exists(), "broken symlink should report !exists()");
    assert!(
        fs::symlink_metadata(&dest).is_ok(),
        "broken symlink inode must be detectable via symlink_metadata"
    );

    // create_link_atomic must detect the broken symlink and replace it
    create_link_atomic(&src, &dest)?;

    assert!(dest.exists());
    assert!(is_link(&dest));
    assert_eq!(fs::read_to_string(dest.join("skill.txt"))?, "new data");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_sync_dir_atomic_handles_broken_symlink_gracefully() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let src = root.path().join("src");
    let dest = root.path().join("dest");
    let nonexistent = root.path().join("gone");

    fs::create_dir_all(&src)?;
    fs::write(src.join("file.txt"), "content")?;

    // Create a broken symlink at dest
    std::os::unix::fs::symlink(&nonexistent, &dest)?;

    // sync_dir_atomic should detect the broken symlink via symlink_metadata
    // and skip (since it's a link), rather than attempting create_dir_all
    // which would fail with EEXIST.
    let result = sync_dir_atomic(&src, &dest);
    assert!(result.is_ok());
    Ok(())
}
