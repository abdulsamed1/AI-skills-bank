use crate::error::SkillManageError;
use std::io::{BufWriter, Write};
use std::path::Path;
use tempfile::NamedTempFile;

/// Write data to a file atomically by writing to a temporary file and renaming it.
pub fn write_file_atomic(path: &Path, contents: &[u8]) -> Result<(), SkillManageError> {
    let parent = path.parent().ok_or_else(|| {
        SkillManageError::ConfigError(format!("Invalid path: {}", path.display()))
    })?;

    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    let mut temp_file = NamedTempFile::new_in(parent)?;
    {
        let mut writer = BufWriter::new(&mut temp_file);
        writer.write_all(contents)?;
        writer.flush()?;
    }

    temp_file.persist(path).map_err(|e| {
        SkillManageError::ConfigError(format!("Failed to persist atomic write: {}", e))
    })?;

    Ok(())
}

/// Synchronize a directory atomically by copying to a temp location and renaming.
pub fn sync_dir_atomic(src: &Path, dest: &Path) -> Result<(), SkillManageError> {
    let parent = dest.parent().ok_or_else(|| {
        SkillManageError::ConfigError(format!("Invalid destination path: {}", dest.display()))
    })?;

    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    // Create a unique temp directory name in the same parent
    let temp_dest_name = format!(
        "{}.tmp",
        dest.file_name().unwrap_or_default().to_string_lossy()
    );
    let temp_dest = parent.join(temp_dest_name);

    if temp_dest.exists() {
        std::fs::remove_dir_all(&temp_dest)?;
    }

    let mut options = fs_extra::dir::CopyOptions::new();
    options.copy_inside = true;
    options.overwrite = true;

    fs_extra::dir::copy(src, &temp_dest, &options).map_err(|e| {
        SkillManageError::ConfigError(format!("Failed to copy to temp location: {}", e))
    })?;

    let src_name = src
        .file_name()
        .ok_or_else(|| SkillManageError::ConfigError("Source has no file name".to_string()))?;

    // fs_extra::dir::copy behavior differs by platform and options: sometimes it
    // creates a nested directory under temp_dest (temp_dest/src_name), other
    // times it copies the contents directly into temp_dest. Detect which one
    // happened and choose the correct path to rename.
    let nested_temp = temp_dest.join(src_name);
    let actual_temp_path = if nested_temp.exists() {
        nested_temp
    } else if temp_dest.exists() {
        temp_dest.clone()
    } else {
        return Err(SkillManageError::ConfigError(format!(
            "Temporary copy location not found: {}",
            temp_dest.display()
        )));
    };

    if dest.exists() {
        std::fs::remove_dir_all(dest)?;
    }

    if let Err(e) = std::fs::rename(&actual_temp_path, dest) {
        if e.to_string().contains("cross-device") || e.kind() == std::io::ErrorKind::Other {
            let mut options = fs_extra::dir::CopyOptions::new();
            options.overwrite = true;
            fs_extra::dir::move_dir(&actual_temp_path, dest, &options).map_err(|e| {
                SkillManageError::ConfigError(format!("Cross-device move failed: {}", e))
            })?;
        } else {
            return Err(e.into());
        }
    }

    if temp_dest.exists() {
        let _ = std::fs::remove_dir_all(&temp_dest);
    }

    Ok(())
}

/// Create a directory link (Junction on Windows, Symlink on Unix) atomically.
pub fn create_link_atomic(src: &Path, dest: &Path) -> Result<(), SkillManageError> {
    let parent = dest.parent().ok_or_else(|| {
        SkillManageError::ConfigError(format!("Invalid destination path: {}", dest.display()))
    })?;

    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    let temp_dest_name = format!(
        "{}.link.tmp",
        dest.file_name().unwrap_or_default().to_string_lossy()
    );
    let temp_dest = parent.join(temp_dest_name);

    if temp_dest.exists() {
        if temp_dest.is_dir() {
            std::fs::remove_dir_all(&temp_dest)?;
        } else {
            std::fs::remove_file(&temp_dest)?;
        }
    }

    // Platform-specific link creation
    #[cfg(windows)]
    {
        let src_abs = std::fs::canonicalize(src)?;
        let dest_abs = if temp_dest.is_absolute() {
            temp_dest.clone()
        } else {
            std::env::current_dir()?.join(&temp_dest)
        };
        junction::create(&src_abs, &dest_abs)?;
    }

    #[cfg(unix)]
    {
        // Calculate relative path for better portability on Unix
        let rel_src = pathdiff::diff_paths(src, parent).ok_or_else(|| {
            SkillManageError::ConfigError(
                "Failed to calculate relative path for symlink".to_string(),
            )
        })?;
        std::os::unix::fs::symlink(rel_src, &temp_dest)?;
    }

    // Atomic rename of the link itself
    if dest.exists() {
        if is_link(dest) {
            #[cfg(windows)]
            std::fs::remove_dir(dest)?;
            #[cfg(unix)]
            std::fs::remove_file(dest)?;
        } else {
            std::fs::remove_dir_all(dest)?;
        }
    }

    std::fs::rename(&temp_dest, dest)?;

    Ok(())
}

/// Check if a path is a directory link (Junction or Symlink).
pub fn is_link(path: &Path) -> bool {
    #[cfg(windows)]
    {
        junction::exists(path).unwrap_or(false)
            || path
                .symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
    }
    #[cfg(unix)]
    {
        path.symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
