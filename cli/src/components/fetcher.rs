use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use crate::components::manifest::RepoManifest;
use crate::components::CommandResult;
use crate::error::SkillManageError;
use crate::utils::progress::ProgressManager;

pub struct Fetcher {
    pub manifest: Option<RepoManifest>,
    pub progress: Arc<ProgressManager>,
}

impl Fetcher {
    pub fn new(progress: Arc<ProgressManager>) -> Self {
        Self {
            manifest: None,
            progress,
        }
    }

    pub fn with_manifest(manifest: RepoManifest, progress: Arc<ProgressManager>) -> Self {
        Self {
            manifest: Some(manifest),
            progress,
        }
    }

    /// Run a git command asynchronously
    async fn run_git_command(args: &[&str], cwd: &Path) -> Result<(), SkillManageError> {
        let output = tokio::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SkillManageError::GitError(stderr.to_string()));
        }

        Ok(())
    }

    /// Fetch all repositories in the manifest
    pub async fn fetch(&self, dry_run: bool) -> Result<CommandResult, SkillManageError> {
        let manifest = self.manifest.as_ref().ok_or_else(|| {
            SkillManageError::ConfigError("No manifest loaded for fetcher".to_string())
        })?;

        // Ensure src directory exists
        let src_dir = Path::new("src");
        if !src_dir.exists() {
            if !dry_run {
                std::fs::create_dir_all(src_dir)?;
            }
        }

        let total_repos = manifest.repositories.len() as u64;
        let main_pb = self.progress.create_main_bar(total_repos, "Fetching repositories");

        let semaphore = Arc::new(Semaphore::new(4));
        let cloned = Arc::new(Mutex::new(Vec::new()));
        let updated = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for repo in manifest.repositories.clone() {
            let sem = Arc::clone(&semaphore);
            let repo_name = repo.name.clone();
            let repo_url = repo.url.clone();
            let progress = Arc::clone(&self.progress);
            let main_pb_clone = main_pb.clone();
            let cloned_ref = Arc::clone(&cloned);
            let updated_ref = Arc::clone(&updated);
            
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.map_err(|e| SkillManageError::GitError(e.to_string()))?;
                let repo_path = Path::new("src").join(&repo_name);
                
                let spinner = progress.create_spinner(&format!("Pending: {}", repo_name));

                if repo_path.exists() {
                    spinner.set_message(format!("Updating {}...", repo_name));
                    if !dry_run {
                        Self::run_git_command(&["pull"], &repo_path).await?;
                        updated_ref.lock().unwrap().push(repo_name.clone());
                    }
                } else {
                    spinner.set_message(format!("Cloning {}...", repo_name));
                    if !dry_run {
                        let args = ["clone", "--depth", "1", &repo_url, &repo_name];
                        Self::run_git_command(&args, Path::new("src")).await?;
                        cloned_ref.lock().unwrap().push(repo_name.clone());
                    }
                }
                
                spinner.finish_with_message(format!("Done: {}", repo_name));
                main_pb_clone.inc(1);
                Ok::<(), SkillManageError>(())
            });
            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(result) => result?,
                Err(e) => return Err(SkillManageError::GitError(e.to_string())),
            }
        }

        main_pb.finish_with_message("All repositories fetched successfully.");
        
        Ok(CommandResult::Fetch {
            cloned: cloned.lock().unwrap().clone(),
            updated: updated.lock().unwrap().clone(),
        })
    }
}
