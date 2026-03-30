use crate::components::manifest::{RepoManifest, Repository};
use crate::components::CommandResult;
use crate::error::SkillManageError;
use crate::utils::progress::ProgressManager;
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

const PRIMARY_REPO_CACHE_DIR: &str = "lib";
const LEGACY_REPO_CACHE_DIR: &str = "repos";

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

    fn normalize_repo_url(url: &str) -> String {
        let mut normalized = url.trim().to_ascii_lowercase();
        normalized = normalized.trim_end_matches('/').to_string();
        normalized = normalized.trim_end_matches(".git").to_string();
        normalized
    }

    fn dedupe_manifest_repositories(manifest: &RepoManifest) -> Vec<Repository> {
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

    fn repo_paths(repo_name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        (
            Path::new(PRIMARY_REPO_CACHE_DIR).join(repo_name),
            Path::new(LEGACY_REPO_CACHE_DIR).join(repo_name),
        )
    }

    async fn pull_repository(repo_path: &Path, branch: Option<&str>) -> Result<(), SkillManageError> {
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
                let (repo_path, legacy_repo_path) = Self::repo_paths(&repo_name);
                let branch = repo_branch.as_deref();

                let spinner = progress.create_spinner(&format!("Pending: {}", repo_name));

                if repo_path.exists() {
                    spinner.set_message(format!("Updating {}...", repo_name));
                    if !dry_run {
                        Self::pull_repository(&repo_path, branch).await?;
                        updated_ref.lock().unwrap().push(repo_name.clone());
                    }
                } else if legacy_repo_path.exists() {
                    spinner.set_message(format!(
                        "Migrating {} from repos/ to lib/ and updating...",
                        repo_name
                    ));
                    if !dry_run {
                        if let Some(parent) = repo_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::rename(&legacy_repo_path, &repo_path)?;
                        Self::pull_repository(&repo_path, branch).await?;
                        updated_ref.lock().unwrap().push(repo_name.clone());
                    }
                } else {
                    spinner.set_message(format!("Cloning {} (shallow)...", repo_name));
                    if !dry_run {
                        Self::clone_shallow(&repo_name, &repo_url, branch).await?;
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

        // Extract data and drop locks before constructing result
        let cloned_data = cloned.lock().unwrap().clone();
        let updated_data = updated.lock().unwrap().clone();

        Ok(CommandResult::Fetch {
            cloned: cloned_data,
            updated: updated_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_repo_url_collapses_common_variants() {
        let a = Fetcher::normalize_repo_url("https://github.com/Org/Repo.git");
        let b = Fetcher::normalize_repo_url("https://github.com/org/repo/");
        assert_eq!(a, b);
    }

    #[test]
    fn test_dedupe_manifest_repositories_skips_duplicate_names_and_urls() {
        let manifest = RepoManifest {
            repositories: vec![
                Repository {
                    name: "repo-a".to_string(),
                    url: "https://github.com/org/repo-a.git".to_string(),
                    branch: None,
                },
                Repository {
                    // Duplicate URL (different formatting) should be skipped.
                    name: "repo-a-alt".to_string(),
                    url: "https://github.com/org/repo-a/".to_string(),
                    branch: None,
                },
                Repository {
                    // Duplicate name should be skipped.
                    name: "repo-a".to_string(),
                    url: "https://github.com/org/repo-b.git".to_string(),
                    branch: None,
                },
                Repository {
                    name: "repo-c".to_string(),
                    url: "https://github.com/org/repo-c.git".to_string(),
                    branch: None,
                },
            ],
        };

        let deduped = Fetcher::dedupe_manifest_repositories(&manifest);
        let names = deduped.iter().map(|r| r.name.as_str()).collect::<Vec<_>>();

        assert_eq!(deduped.len(), 2);
        assert_eq!(names, vec!["repo-a", "repo-c"]);
    }
}
