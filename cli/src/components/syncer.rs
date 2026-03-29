use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;
use crate::error::SkillManageError;
use crate::utils::progress::ProgressManager;
use crate::utils::paths::{expand_home, get_default_destination};
use crate::utils::atomicity::{sync_dir_atomic, create_link_atomic, is_link};

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
}

impl Skill {
    /// Create a Skill from a directory path if it contains a SKILL.md file.
    pub fn from_path(path: &Path) -> Option<Self> {
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() && skill_md.is_file() {
                let name = path.file_name()?.to_string_lossy().into_owned();
                return Some(Self {
                    name,
                    path: path.to_path_buf(),
                });
            }
        }
        None
    }
}

pub struct Syncer {
    pub progress: Arc<ProgressManager>,
}

impl Syncer {
    pub fn new(progress: Arc<ProgressManager>) -> Self {
        Self { progress }
    }

    /// Recursively find skills in the source directory.
    fn find_skills(&self, src_path: &Path) -> Vec<Skill> {
        let mut skills = Vec::new();
        for entry in WalkDir::new(src_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(skill) = Skill::from_path(entry.path()) {
                skills.push(skill);
            }
        }
        skills
    }

    /// Synchronize skills to the target destination.
    pub async fn sync(&self, destination: Option<String>, link: bool, dry_run: bool) -> Result<(), SkillManageError> {
        let target_base = match destination {
            Some(d) => expand_home(&d),
            None => get_default_destination(),
        };

        if !dry_run && !target_base.exists() {
            std::fs::create_dir_all(&target_base)?;
        }

        let src_path = Path::new("src");
        if !src_path.exists() {
            return Err(SkillManageError::ConfigError("Source directory 'src' not found. Run 'fetch' first.".to_string()));
        }

        let skills = self.find_skills(src_path);
        let total_skills = skills.len() as u64;
        let main_pb = self.progress.create_main_bar(total_skills, "Synchronizing skills");

        for skill in skills {
            let target_path = target_base.join(&skill.name);
            let spinner = self.progress.create_spinner(&format!("Syncing: {}", skill.name));

            if dry_run {
                let mode = if link { "link" } else { "sync" };
                println!("[Dry Run] Would {} {} to {}", mode, skill.path.display(), target_path.display());
            } else {
                // Conflict detection
                if target_path.exists() {
                    let existing_is_link = is_link(&target_path);
                    if link && !existing_is_link {
                        return Err(SkillManageError::ConfigError(format!(
                            "Conflict: Target '{}' exists and is a directory, but --link was requested. Remove it manually or run without --link.",
                            target_path.display()
                        )));
                    }
                    if !link && existing_is_link {
                        return Err(SkillManageError::ConfigError(format!(
                            "Conflict: Target '{}' exists and is a link, but full sync was requested. Remove it manually or run with --link.",
                            target_path.display()
                        )));
                    }
                }

                if link {
                    create_link_atomic(&skill.path, &target_path)?;
                } else {
                    sync_dir_atomic(&skill.path, &target_path)?;
                }
            }

            spinner.finish_with_message(format!("Synced: {}", skill.name));
            main_pb.inc(1);
        }

        main_pb.finish_with_message("Synchronization complete.");
        Ok(())
    }
}
