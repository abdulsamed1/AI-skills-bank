use crate::error::SkillManageError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepoManifest {
    pub repositories: Vec<Repository>,
}

impl RepoManifest {
    /// Load a manifest from a JSON file
    pub fn load(path: &Path) -> Result<Self, SkillManageError> {
        let content = std::fs::read_to_string(path)?;
        let mut manifest: Self = serde_json::from_str(&content)
            .map_err(|e| SkillManageError::ManifestParseError(e.to_string()))?;

        // Automatically deduplicate repositories by URL and Name to be resilient
        let mut unique_repos = Vec::new();
        let mut seen_names = HashSet::new();
        let mut seen_urls = HashSet::new();

        for repo in manifest.repositories {
            let name_lower = repo.name.trim().to_lowercase();
            let url_lower = repo.url.trim().to_lowercase();

            if name_lower.is_empty() || url_lower.is_empty() {
                unique_repos.push(repo);
                continue;
            }

            if seen_names.contains(&name_lower) || seen_urls.contains(&url_lower) {
                // Duplicate - silently skip to prevent failures
                continue;
            }

            seen_names.insert(name_lower);
            seen_urls.insert(url_lower);
            unique_repos.push(repo);
        }

        manifest.repositories = unique_repos;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the structural integrity of the manifest
    pub fn validate(&self) -> Result<(), SkillManageError> {
        let mut names = HashSet::new();
        let mut urls = HashSet::new();

        for repo in &self.repositories {
            if repo.name.trim().is_empty() {
                return Err(SkillManageError::ManifestValidationError(
                    "Repository name cannot be empty".to_string(),
                ));
            }
            if repo.url.trim().is_empty() {
                return Err(SkillManageError::ManifestValidationError(format!(
                    "Repository '{}' has an empty URL",
                    repo.name
                )));
            }

            if !names.insert(&repo.name) {
                return Err(SkillManageError::ManifestValidationError(format!(
                    "Duplicate repository name found: '{}'",
                    repo.name
                )));
            }
            if !urls.insert(&repo.url) {
                return Err(SkillManageError::ManifestValidationError(format!(
                    "Duplicate repository URL found: '{}'",
                    repo.url
                )));
            }
        }

        Ok(())
    }
}

