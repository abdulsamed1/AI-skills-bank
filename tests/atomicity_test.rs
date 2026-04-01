use skill_manage::utils::atomicity::*;
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
