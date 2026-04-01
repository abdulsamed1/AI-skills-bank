use skill_manage::components::aggregator::Aggregator;
use std::fs;
use tempfile::tempdir;

#[test]
fn parse_without_frontmatter_uses_fallbacks() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let skill_dir = root.path().join("skills").join("coding-agent-router");
    fs::create_dir_all(&skill_dir)?;
    let skill_path = skill_dir.join("SKILL.md");

    fs::write(
        &skill_path,
        "# Coding Agent Router\n\nRoutes tasks to the best coding agent.\n",
    )?;

    let meta = Aggregator::parse_skill_md(&skill_path)?;
    assert_eq!(meta.name, "coding-agent-router");
    assert_eq!(meta.description, "Coding Agent Router");
    Ok(())
}

#[test]
fn parse_invalid_yaml_falls_back_to_line_extraction() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let skill_dir = root.path().join("skills").join("build");
    fs::create_dir_all(&skill_dir)?;
    let skill_path = skill_dir.join("SKILL.md");

    fs::write(
        &skill_path,
        "---\nname: build\ndescription: Feature development pipeline\nargument-hint: [subcommand] [name]\n---\n\nbody\n",
    )?;

    let meta = Aggregator::parse_skill_md(&skill_path)?;
    assert_eq!(meta.name, "build");
    assert_eq!(meta.description, "Feature development pipeline");
    Ok(())
}

#[test]
fn parse_missing_name_uses_path_slug() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let skill_dir = root.path().join("skills").join("oral-health-analyzer");
    fs::create_dir_all(&skill_dir)?;
    let skill_path = skill_dir.join("SKILL.md");

    fs::write(
        &skill_path,
        "---\ndescription: Analyzer for oral health\n---\n\nbody\n",
    )?;

    let meta = Aggregator::parse_skill_md(&skill_path)?;
    assert_eq!(meta.name, "oral-health-analyzer");
    assert_eq!(meta.description, "Analyzer for oral health");
    Ok(())
}
