use crate::components::manifest::{RepoManifest, Repository};
use crate::components::CommandResult;
use crate::error::SkillManageError;
use crate::utils::progress::ProgressManager;
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

const PRIMARY_REPO_CACHE_DIR: &str = "lib";

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
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(std::process::Stdio::null())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SkillManageError::GitError(stderr.to_string()));
        }

        Ok(())
    }

    /// Run a git command asynchronously and return its stdout
    async fn run_git_command_output(args: &[&str], cwd: &Path) -> Result<String, SkillManageError> {
        let output = tokio::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(std::process::Stdio::null())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SkillManageError::GitError(stderr.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if there are updates on the remote repository
    async fn has_updates(repo_path: &Path, branch: Option<&str>) -> Result<bool, SkillManageError> {
        if let Some(branch_name) = branch.filter(|b| !b.trim().is_empty()) {
            Self::run_git_command(&["fetch", "--depth", "1", "origin", branch_name], repo_path).await?;
        } else {
            Self::run_git_command(&["fetch", "--depth", "1"], repo_path).await?;
        }

        let local_head = Self::run_git_command_output(&["rev-parse", "HEAD"], repo_path).await?;
        let remote_head = Self::run_git_command_output(&["rev-parse", "FETCH_HEAD"], repo_path).await?;

        Ok(local_head != remote_head)
    }

    pub fn normalize_repo_url(url: &str) -> String {
        let mut normalized = url.trim().to_ascii_lowercase();
        normalized = normalized.trim_end_matches('/').to_string();
        normalized = normalized.trim_end_matches(".git").to_string();
        normalized
    }

    pub fn dedupe_manifest_repositories(manifest: &RepoManifest) -> Vec<Repository> {
        let mut seen_names = HashSet::new();
        let mut seen_urls = HashSet::new();
        let mut out = Vec::new();

        for repo in &manifest.repositories {
            let name_key = repo.name.trim().to_ascii_lowercase();
            if !seen_names.insert(name_key) {
                continue;
            }

            let url_key = Self::normalize_repo_url(&repo.url);
            if !seen_urls.insert(url_key) {
                continue;
            }

            out.push(repo.clone());
        }

        out
    }

    async fn pull_repository(repo_path: &Path, branch: Option<&str>) -> Result<(), SkillManageError> {
        let _ = Self::run_git_command(&["reset", "--hard"], repo_path).await;
        let _ = Self::run_git_command(&["clean", "-fdx"], repo_path).await;

        if let Some(branch_name) = branch.filter(|b| !b.trim().is_empty()) {
            let args = ["pull", "--ff-only", "origin", branch_name];
            Self::run_git_command(&args, repo_path).await
        } else {
            let args = ["pull", "--ff-only"];
            Self::run_git_command(&args, repo_path).await
        }
    }


    async fn clone_shallow(repo_name: &str, repo_url: &str, branch: Option<&str>) -> Result<(), SkillManageError> {
        let mut args = vec![
            "clone".to_string(),
            "--depth".to_string(),
            "1".to_string(),
            "--single-branch".to_string(),
            "--no-tags".to_string(),
        ];

        if let Some(branch_name) = branch.filter(|b| !b.trim().is_empty()) {
            args.push("--branch".to_string());
            args.push(branch_name.trim().to_string());
        }

        args.push(repo_url.to_string());
        args.push(repo_name.to_string());

        let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        Self::run_git_command(&arg_refs, Path::new(PRIMARY_REPO_CACHE_DIR)).await
    }

    /// Fetch all repositories in the manifest
    pub async fn fetch(&self, dry_run: bool) -> Result<CommandResult, SkillManageError> {
        let manifest = self.manifest.as_ref().ok_or_else(|| {
            SkillManageError::ConfigError("No manifest loaded for fetcher".to_string())
        })?;

        let repositories = Self::dedupe_manifest_repositories(manifest);

        // Ensure canonical repo cache directory exists.
        let lib_dir = Path::new(PRIMARY_REPO_CACHE_DIR);
        if lib_dir.exists() && !lib_dir.is_dir() {
            return Err(SkillManageError::ConfigError(format!(
                "Repository cache path '{}' exists but is not a directory",
                lib_dir.display()
            )));
        }

        if !lib_dir.exists() {
            if !dry_run {
                std::fs::create_dir_all(lib_dir)?;
            }
        }

        let total_repos = repositories.len() as u64;
        let main_pb = self
            .progress
            .create_main_bar(total_repos, "Fetching repositories");

        let semaphore = Arc::new(Semaphore::new(4));
        let cloned = Arc::new(Mutex::new(Vec::new()));
        let updated = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for repo in repositories {
            let sem = Arc::clone(&semaphore);
            let repo_name = repo.name.clone();
            let repo_url = repo.url.clone();
            let repo_branch = repo.branch.clone();
            let progress = Arc::clone(&self.progress);
            let main_pb_clone = main_pb.clone();
            let cloned_ref = Arc::clone(&cloned);
            let updated_ref = Arc::clone(&updated);

            let handle = tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| SkillManageError::GitError(e.to_string()))?;
                let repo_path = Path::new(PRIMARY_REPO_CACHE_DIR).join(&repo_name);
                let branch = repo_branch.as_deref();

                let spinner = progress.create_spinner(&format!("Pending: {}", repo_name));

                let mut fetch_result = Ok(());

                if repo_path.exists() {
                    if !dry_run {
                        spinner.set_message(format!("Checking for updates: {}...", repo_name));
                        match Self::has_updates(&repo_path, branch).await {
                            Ok(true) => {
                                spinner.set_message(format!("Updating {}...", repo_name));
                                if let Err(_e) = Self::pull_repository(&repo_path, branch).await {
                                    let _ = std::fs::remove_dir_all(&repo_path);
                                    if let Err(clone_err) = Self::clone_shallow(&repo_name, &repo_url, branch).await {
                                        fetch_result = Err(clone_err);
                                    } else {
                                        cloned_ref.lock().expect("cloned_ref mutex not poisoned").push(repo_name.clone());
                                    }
                                } else {
                                    updated_ref.lock().expect("updated_ref mutex not poisoned").push(repo_name.clone());
                                }
                            }
                            Ok(false) => {
                                spinner.set_message(format!("Up to date: {}", repo_name));
                            }
                            Err(_e) => {
                                spinner.set_message(format!("Update check failed (falling back): {}...", repo_name));
                                if let Err(_e) = Self::pull_repository(&repo_path, branch).await {
                                    let _ = std::fs::remove_dir_all(&repo_path);
                                    if let Err(clone_err) = Self::clone_shallow(&repo_name, &repo_url, branch).await {
                                        fetch_result = Err(clone_err);
                                    } else {
                                        cloned_ref.lock().expect("cloned_ref mutex not poisoned").push(repo_name.clone());
                                    }
                                } else {
                                    updated_ref.lock().expect("updated_ref mutex not poisoned").push(repo_name.clone());
                                }
                            }
                        }
                    } else {
                        spinner.set_message(format!("Updating {}...", repo_name));
                    }
                } else {
                    spinner.set_message(format!("Cloning {} (shallow)...", repo_name));
                    if !dry_run {
                        if let Err(clone_err) = Self::clone_shallow(&repo_name, &repo_url, branch).await {
                            fetch_result = Err(clone_err);
                        } else {
                            cloned_ref.lock().expect("cloned_ref mutex not poisoned").push(repo_name.clone());
                        }
                    }
                }

                match fetch_result {
                    Ok(()) => {
                        spinner.finish_with_message(format!("Done: {}", repo_name));
                    }
                    Err(e) => {
                        spinner.finish_with_message(format!("❌ Failed: {} ({})", repo_name, e));
                        eprintln!("\n[WARN] Failed to fetch repository '{}': {}", repo_name, e);
                    }
                }

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

        // Extract data and drop locks before constructing result
        let cloned_data = cloned.lock().expect("cloned mutex not poisoned").clone();
        let updated_data = updated.lock().expect("updated mutex not poisoned").clone();

        Ok(CommandResult::Fetch {
            cloned: cloned_data,
            updated: updated_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn run_git(args: &[&str], cwd: &Path) {
        let status = tokio::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .status()
            .await
            .unwrap();
        assert!(status.success());
    }

    #[tokio::test]
    async fn test_has_updates_logic() {
        let temp = tempdir().unwrap();
        let origin_path = temp.path().join("origin");
        let clone_path = temp.path().join("clone");

        std::fs::create_dir_all(&origin_path).unwrap();

        // 1. Initialize origin repository
        run_git(&["init", "-b", "main"], &origin_path).await;
        run_git(&["config", "user.name", "Test User"], &origin_path).await;
        run_git(&["config", "user.email", "test@example.com"], &origin_path).await;

        // Write first commit
        let file_path = origin_path.join("file.txt");
        std::fs::write(&file_path, "initial").unwrap();
        run_git(&["add", "file.txt"], &origin_path).await;
        run_git(&["commit", "-m", "initial commit"], &origin_path).await;

        // 2. Clone repository shallowly
        let origin_url = format!("file://{}", origin_path.to_str().unwrap());
        run_git(&["clone", "--depth", "1", &origin_url, clone_path.to_str().unwrap()], &clone_path.parent().unwrap()).await;

        // 3. Verify has_updates is initially false
        let has_up = Fetcher::has_updates(&clone_path, Some("main")).await.unwrap();
        assert!(!has_up, "Should not have updates initially");

        // 4. Make a new commit in origin
        std::fs::write(&file_path, "updated").unwrap();
        run_git(&["add", "file.txt"], &origin_path).await;
        run_git(&["commit", "-m", "second commit"], &origin_path).await;

        // 5. Verify has_updates is now true
        let has_up = Fetcher::has_updates(&clone_path, Some("main")).await.unwrap();
        assert!(has_up, "Should detect updates after remote changes");

        // 6. Reset clone HEAD to FETCH_HEAD (simulate what pull/update does)
        let remote_head = Fetcher::run_git_command_output(&["rev-parse", "FETCH_HEAD"], &clone_path).await.unwrap();
        Fetcher::run_git_command(&["reset", "--hard", &remote_head], &clone_path).await.unwrap();

        // 7. Verify has_updates is false again
        let has_up = Fetcher::has_updates(&clone_path, Some("main")).await.unwrap();
        assert!(!has_up, "Should be up to date after update");
    }
}

