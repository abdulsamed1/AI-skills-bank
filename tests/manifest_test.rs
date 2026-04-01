use skill_manage::components::manifest::*;
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
