use std::io::Write;
use tempfile::NamedTempFile;
use skill_manage::components::manifest::RepoManifest;

#[test]
fn test_full_manifest_flow() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    let json = r#"{
        "repositories": [
            {
                "name": "skill-manage",
                "url": "https://github.com/abdulsamed1/AI-skills-bank",
                "branch": "main"
            }
        ]
    }"#;
    writeln!(file, "{}", json)?;

    let manifest = RepoManifest::load(file.path())?;
    assert_eq!(manifest.repositories.len(), 1);
    assert_eq!(manifest.repositories[0].name, "skill-manage");
    assert_eq!(manifest.repositories[0].url, "https://github.com/abdulsamed1/AI-skills-bank");
    assert_eq!(manifest.repositories[0].branch, Some("main".to_string()));

    Ok(())
}

#[test]
fn test_invalid_json_integration() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "invalid json")?;

    let result = RepoManifest::load(file.path());
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_duplicate_validation_integration() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    let json = r#"{
        "repositories": [
            { "name": "dup", "url": "url1" },
            { "name": "dup", "url": "url2" }
        ]
    }"#;
    writeln!(file, "{}", json)?;

    let result = RepoManifest::load(file.path());
    assert!(result.is_err());
    
    Ok(())
}
