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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_manifest() {
        let json = r#"{
            "repositories": [
                {
                    "name": "skill-1",
                    "url": "https://github.com/user/skill-1",
                    "branch": "main"
                },
                {
                    "name": "skill-2",
                    "url": "https://github.com/user/skill-2"
                }
            ]
        }"#;

        let manifest: RepoManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.repositories.len(), 2);
        assert_eq!(manifest.repositories[0].name, "skill-1");
        assert_eq!(manifest.repositories[1].branch, None);
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_fields() {
        let manifest = RepoManifest {
            repositories: vec![Repository {
                name: "".to_string(),
                url: "https://github.com/user/repo".to_string(),
                branch: None,
            }],
        };
        assert!(manifest.validate().is_err());

        let manifest = RepoManifest {
            repositories: vec![Repository {
                name: "repo".to_string(),
                url: "  ".to_string(),
                branch: None,
            }],
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_validate_duplicates() {
        let manifest = RepoManifest {
            repositories: vec![
                Repository {
                    name: "repo".to_string(),
                    url: "url1".to_string(),
                    branch: None,
                },
                Repository {
                    name: "repo".to_string(),
                    url: "url2".to_string(),
                    branch: None,
                },
            ],
        };
        assert!(manifest.validate().is_err());

        let manifest = RepoManifest {
            repositories: vec![
                Repository {
                    name: "repo1".to_string(),
                    url: "url".to_string(),
                    branch: None,
                },
                Repository {
                    name: "repo2".to_string(),
                    url: "url".to_string(),
                    branch: None,
                },
            ],
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_load_from_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            r#"{{"repositories": [{{"name": "test", "url": "https://github.com/test"}} ]}}"#
        )?;

        let manifest = RepoManifest::load(file.path())?;
        assert_eq!(manifest.repositories.len(), 1);
        assert_eq!(manifest.repositories[0].name, "test");

        Ok(())
    }
}
