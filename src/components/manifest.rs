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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepoManifest {
    pub repositories: Vec<Repository>,
}

impl RepoManifest {
    /// Load a manifest from a JSON file
    pub fn load(path: &Path) -> Result<Self, SkillManageError> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&content)
            .map_err(|e| SkillManageError::ManifestParseError(e.to_string()))?;

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

