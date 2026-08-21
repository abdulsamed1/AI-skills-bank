use crate::error::SkillManageError;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

fn remove_path_best_effort(path: &Path) {
    if std::fs::symlink_metadata(path).is_err() {
        return;
    }

    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_dir(path);
    let _ = std::fs::remove_dir_all(path);
}

/// Write data to a file atomically by writing to a temporary file and renaming it.
pub fn write_file_atomic(path: &Path, contents: &[u8]) -> Result<(), SkillManageError> {
    let parent = path.parent().ok_or_else(|| {
        SkillManageError::ConfigError(format!("Invalid path: {}", path.display()))
    })?;

    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    // Try atomic write using a temp file in the destination directory. If that
    // is not permitted (e.g., due to restrictive permissions on dotfiles in
    // user home), fall back to a direct write to the target path.
    match NamedTempFile::new_in(parent) {
        Ok(mut temp_file) => {
            {
                let mut writer = BufWriter::new(&mut temp_file);
                writer.write_all(contents)?;
                writer.flush()?;
            }

            match temp_file.persist(path) {
                Ok(_) => Ok(()),
                Err(_e) => {
                    // Persist failed (could be cross-device or permission); fallback
                    // to a best-effort direct write.
                    std::fs::write(path, contents)?;
                    Ok(())
                }
            }
        }
        Err(_e) => {
            // Could not create a temp file in the parent dir (permissions); try
            // writing directly to the destination as a fallback.
            std::fs::write(path, contents)?;
            Ok(())
        }
    }
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

    let dest_meta_exists = std::fs::symlink_metadata(dest).is_ok();
    if dest_meta_exists {
        // If destination is a link/junction (possibly broken), avoid
        // removing it and skip syncing to prevent recursive or
        // permission-error cases.
        let link_flag = is_link(dest);
        if link_flag {
            if temp_dest.exists() {
                let _ = std::fs::remove_dir_all(&temp_dest);
            }
            return Ok(());
        }
        // Non-destructive: We do NOT remove `dest` here. 
    } else {
        std::fs::create_dir_all(dest)?;
    }

    // Perform a merge copy instead of renaming to preserve existing files in `dest`
    let merge_result = merge_copy_recursive(&actual_temp_path, dest);

    // Cleanup temp directory
    if temp_dest.exists() {
        let _ = std::fs::remove_dir_all(&temp_dest);
    }

    merge_result
    }

    /// Recursively merge contents of src into dest.
    fn merge_copy_recursive(src: &Path, dest: &Path) -> Result<(), SkillManageError> {
        if !dest.exists() {
            std::fs::create_dir_all(dest)?;
        }

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if src_path.is_dir() {
                if is_link(&src_path) {
                    let _ = create_link_atomic_force(&src_path, &dest_path);
                } else {
                    merge_copy_recursive(&src_path, &dest_path)?;
                }
            } else {
                std::fs::copy(&src_path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Merges the contents of src into dest. For directories, creates a directory link (junction/symlink).
    /// For files, it copies them. This preserves existing items in dest that don't overlap.
    pub fn sync_contents_as_links(src: &Path, dest: &Path) -> Result<(), SkillManageError> {
        if !dest.exists() {
            std::fs::create_dir_all(dest)?;
        }

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if src_path.is_dir() {
                create_link_atomic_force(&src_path, &dest_path).map_err(|e| {
                    SkillManageError::ConfigError(format!("Failed to link {:?}: {}", src_path, e))
                })?;
            } else {
                std::fs::copy(&src_path, &dest_path).map_err(|e| {
                    SkillManageError::ConfigError(format!("Failed to copy file {:?}: {}", src_path, e))
                })?;
            }
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
        let src_str = src_abs.to_string_lossy().to_string();
        // Standard Windows junctions do not support the \\?\ extended prefix
        // provided by canonicalize(), which causes many tools to see the
        // junction as empty or "File Not Found".
        let src_clean = src_str.trim_start_matches(r"\\?\");
        
        let dest_abs = if temp_dest.is_absolute() {
            temp_dest.clone()
        } else {
            std::env::current_dir()?.join(&temp_dest)
        };
        junction::create(std::path::Path::new(src_clean), &dest_abs)?;
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
    // Use symlink_metadata to detect broken symlinks that Path::exists()
    // would miss (it returns false for dangling links, causing EEXIST later).
    let dest_meta_exists = std::fs::symlink_metadata(dest).is_ok();
    if dest_meta_exists {
        if is_link(dest) {
            #[cfg(windows)]
            std::fs::remove_dir(dest)?;
            #[cfg(unix)]
            std::fs::remove_file(dest)?;
        } else {
            // Destination exists and is not a link: refuse to replace it to
            // avoid destructive behavior that would delete user-managed
            // skills or other files. Caller should handle fallback (e.g.
            // perform a non-destructive merge copy).
            remove_path_best_effort(&temp_dest);
            return Err(SkillManageError::ConfigError(format!(
                "Destination {} exists and is not a link; refusing to replace to avoid data loss",
                dest.display()
            )));
        }
    }

    if let Err(e) = std::fs::rename(&temp_dest, dest) {
        remove_path_best_effort(&temp_dest);
        return Err(SkillManageError::ConfigError(format!(
            "Failed to finalize link {}: {}",
            dest.display(),
            e
        )));
    }

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

/// Check if a path exists as a real directory (not a link or other type).
pub fn is_real_directory(path: &Path) -> bool {
    path.metadata()
        .map(|m| m.is_dir())
        .unwrap_or(false)
        && !is_link(path)
}

/// Create a directory link (Junction on Windows, Symlink on Unix) atomically.
/// If destination exists as a real directory, aggressively replace it with a link
/// to prioritize performance and space efficiency. Use this when linking is preferred.
pub fn create_link_atomic_force(src: &Path, dest: &Path) -> Result<(), SkillManageError> {
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
        let src_str = src_abs.to_string_lossy().to_string();
        let src_clean = src_str.trim_start_matches(r"\\?\");
        
        let dest_abs = if temp_dest.is_absolute() {
            temp_dest.clone()
        } else {
            std::env::current_dir()?.join(&temp_dest)
        };
        junction::create(std::path::Path::new(src_clean), &dest_abs)?;
    }

    #[cfg(unix)]
    {
        let rel_src = pathdiff::diff_paths(src, parent).ok_or_else(|| {
            SkillManageError::ConfigError(
                "Failed to calculate relative path for symlink".to_string(),
            )
        })?;
        std::os::unix::fs::symlink(rel_src, &temp_dest)?;
    }

    // Atomic rename — force replace any existing destination
    let dest_meta_exists = std::fs::symlink_metadata(dest).is_ok();
    if dest_meta_exists {
        // Aggressively replace existing target, whether link or directory
        if is_link(dest) {
            #[cfg(windows)]
            std::fs::remove_dir(dest)?;
            #[cfg(unix)]
            std::fs::remove_file(dest)?;
        } else {
            // Move existing real directory to a timestamped backup instead
            // of deleting it. This preserves user data and allows recovery.
            let fname = dest
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "skills-backup".to_string());
            let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
            let backup_name = format!("{}.bak.{}", fname, ts);
            let backup_path = dest
                .parent()
                .map(|p| p.join(backup_name))
                .unwrap_or_else(|| PathBuf::from(format!("{}.bak.{}", fname, ts)));

            std::fs::rename(dest, &backup_path)?;

            // Record backup location in a local backups log for easy recovery.
            let log_path = backup_path
                .parent()
                .map(|p| p.join(".skills-bank-backups.log"))
                .unwrap_or_else(|| PathBuf::from(".skills-bank-backups.log"));
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
            {
                let entry = format!("{} -> {}\n", dest.display(), backup_path.display());
                let _ = f.write_all(entry.as_bytes());
            }
        }
    }

    // Rename temp link to final destination with error handling
    if let Err(e) = std::fs::rename(&temp_dest, dest) {
        remove_path_best_effort(&temp_dest);
        return Err(SkillManageError::ConfigError(format!(
            "Failed to finalize link {}: {}",
            dest.display(),
            e
        )));
    }

    Ok(())
}

