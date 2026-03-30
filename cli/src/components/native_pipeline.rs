use crate::components::aggregator::{Aggregator, SkillMetadata};
use crate::components::CommandResult;
use crate::error::SkillManageError;
use crate::utils::atomicity::{create_link_atomic, sync_dir_atomic, write_file_atomic};
use crate::utils::progress::ProgressManager;
use crate::utils::theme::Theme;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub enum NativeSyncMode {
    Auto,
    Copy,
    Junction,
    SymbolicLink,
}

struct CwdGuard {
    previous: PathBuf,
}

impl CwdGuard {
    fn set(path: &Path) -> Result<Self, SkillManageError> {
        let previous = std::env::current_dir()?;
        std::env::set_current_dir(path)?;
        Ok(Self { previous })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

#[derive(Debug, Serialize)]
struct RoutingRow {
    skill_id: String,
    description: String,
    src_path: String,
}

#[derive(Debug, Serialize)]
struct CatalogRow {
    skill_id: String,
    description: String,
    score: u32,
    phase: u32,
}

#[derive(Debug, Serialize)]
struct SubHubIndexEntry {
    hub: String,
    sub_hub: String,
    skills_count: usize,
    path: String,
}

pub async fn aggregate_to_output(
    repo_root: &Path,
    output_dir: &Path,
    selected_repos: Option<&HashSet<String>>,
    write_global_csv: bool,
    show_progress: bool,
) -> Result<Vec<SkillMetadata>, SkillManageError> {
    let _guard = CwdGuard::set(repo_root)?;

    let theme = Arc::new(Theme::new());
    let progress = Arc::new(ProgressManager::new(show_progress, false, Arc::clone(&theme)));
    let aggregator = Aggregator::new(progress);

    let result = aggregator.aggregate(false).await?;
    let mut skills = match result {
        CommandResult::Aggregate { skills } => skills,
        _ => {
            return Err(SkillManageError::ConfigError(
                "Unexpected aggregate result".to_string(),
            ));
        }
    };

    if let Some(selected) = selected_repos {
        let selected_lower = selected
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<HashSet<_>>();

        skills.retain(|s| {
            skill_repo_name(&s.path)
                .map(|name| selected_lower.contains(&name.to_lowercase()))
                .unwrap_or(false)
        });
    }

    skills.sort_by(|a, b| {
        b.match_score
            .unwrap_or(100)
            .cmp(&a.match_score.unwrap_or(100))
            .then_with(|| a.name.cmp(&b.name))
    });

    if write_global_csv {
        aggregator.generate_csv(skills.clone()).await?;
    }

    write_native_artifacts(repo_root, output_dir, &skills)?;
    Ok(skills)
}

pub fn sync_output_to_targets(
    source_root: &Path,
    targets: &[PathBuf],
    mode: NativeSyncMode,
) -> Result<(), SkillManageError> {
    if !source_root.exists() {
        return Err(SkillManageError::ConfigError(format!(
            "Aggregation output not found: {}",
            source_root.display()
        )));
    }

    for target in targets {
        match mode {
            NativeSyncMode::Copy => sync_dir_atomic(source_root, target)?,
            NativeSyncMode::Junction => {
                #[cfg(windows)]
                {
                    create_link_atomic(source_root, target)?;
                }
                #[cfg(not(windows))]
                {
                    return Err(SkillManageError::ConfigError(
                        "Junction mode is only supported on Windows".to_string(),
                    ));
                }
            }
            NativeSyncMode::SymbolicLink => {
                create_link_atomic(source_root, target)?;
            }
            NativeSyncMode::Auto => {
                if let Err(link_err) = create_link_atomic(source_root, target) {
                    sync_dir_atomic(source_root, target).map_err(|copy_err| {
                        SkillManageError::ConfigError(format!(
                            "Auto sync failed for {}. link error: {}; copy error: {}",
                            target.display(),
                            link_err,
                            copy_err
                        ))
                    })?;
                }
            }
        }
    }

    Ok(())
}

fn write_native_artifacts(
    repo_root: &Path,
    output_dir: &Path,
    skills: &[SkillMetadata],
) -> Result<(), SkillManageError> {
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir)?;
    }
    std::fs::create_dir_all(output_dir)?;

    let mut grouped: BTreeMap<(String, String), Vec<SkillMetadata>> = BTreeMap::new();
    for skill in skills {
        grouped
            .entry((skill.hub.clone(), skill.sub_hub.clone()))
            .or_default()
            .push(skill.clone());
    }

    let mut hub_to_subhubs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut subhub_index = Vec::new();

    for ((hub, sub_hub), group_skills) in grouped {
        hub_to_subhubs
            .entry(hub.clone())
            .or_default()
            .insert(sub_hub.clone());

        let subhub_dir = output_dir.join(&hub).join(&sub_hub);
        std::fs::create_dir_all(&subhub_dir)?;

        let routing_rows = group_skills
            .iter()
            .map(|s| RoutingRow {
                skill_id: s.name.clone(),
                description: sanitize_description(&s.description),
                src_path: normalize_src_path(repo_root, &s.path),
            })
            .collect::<Vec<_>>();
        write_csv_atomic(&subhub_dir.join("routing.csv"), &routing_rows)?;

        let catalog_rows = group_skills
            .iter()
            .map(|s| CatalogRow {
                skill_id: s.name.clone(),
                description: sanitize_description(&s.description),
                score: s.match_score.unwrap_or(100),
                phase: s.phase.unwrap_or(1),
            })
            .collect::<Vec<_>>();

        // For the devops/ci-cd subhub we no longer generate `skills-catalog.csv`.
        // Agents should use `routing.csv` as the source of truth for routing and
        // metadata. Skip writing the catalog file for that specific subhub.
        if !(hub == "devops" && sub_hub == "ci-cd") {
            write_csv_atomic(&subhub_dir.join("skills-catalog.csv"), &catalog_rows)?;
        } else {
            // Ensure any pre-existing catalog file is removed to avoid confusion.
            let catalog_path = subhub_dir.join("skills-catalog.csv");
            if catalog_path.exists() {
                std::fs::remove_file(catalog_path)?;
            }
        }

        let index_json = json!({
            "hub": hub,
            "sub_hub": sub_hub,
            "count": group_skills.len(),
            "skills": group_skills.iter().map(|s| s.name.clone()).collect::<Vec<_>>()
        });
        write_json_atomic(&subhub_dir.join("skills-index.json"), &index_json)?;

        let manifest_json = json!({
            "version": 1,
            "hub": group_skills.first().map(|s| s.hub.clone()).unwrap_or_default(),
            "sub_hub": group_skills.first().map(|s| s.sub_hub.clone()).unwrap_or_default(),
            "generated_at_unix": unix_now(),
            "skills": group_skills.iter().map(|s| json!({
                "skill_id": s.name,
                "description": sanitize_description(&s.description),
                "triggers": s.triggers,
                "score": s.match_score,
                "phase": s.phase,
                "src_path": normalize_src_path(repo_root, &s.path)
            })).collect::<Vec<_>>()
        });
        write_json_atomic(&subhub_dir.join("skills-manifest.json"), &manifest_json)?;

        // Per-subhub SKILL.md files are no longer generated here. Agents should
        // reference the dynamic `skills-aggregated/AGENTS.md` and routing CSVs.

        let rel_path = subhub_dir
            .strip_prefix(output_dir)
            .unwrap_or(&subhub_dir)
            .to_string_lossy()
            .replace('\\', "/");
        subhub_index.push(SubHubIndexEntry {
            hub: group_skills
                .first()
                .map(|s| s.hub.clone())
                .unwrap_or_default(),
            sub_hub: group_skills
                .first()
                .map(|s| s.sub_hub.clone())
                .unwrap_or_default(),
            skills_count: group_skills.len(),
            path: rel_path,
        });
    }

    let hubs_list = hub_to_subhubs.keys().cloned().collect::<Vec<_>>();
    // Master SKILL.md is no longer emitted; the repository-level
    // `skills-aggregated/AGENTS.md` is the canonical, dynamic entrypoint.

    let subhub_json = json!({
        "version": 1,
        "generated_at_unix": unix_now(),
        "subhubs": subhub_index
    });
    write_json_atomic(&output_dir.join("subhub-index.json"), &subhub_json)?;

    let repo_names = skills
        .iter()
        .filter_map(|s| skill_repo_name(&s.path))
        .collect::<BTreeSet<_>>();
    let lock_json = json!({
        "generated_at_unix": unix_now(),
        "source": "native-cli",
        "src_repositories": repo_names.into_iter().map(|name| json!({"name": name})).collect::<Vec<_>>()
    });
    write_json_atomic(&output_dir.join(".skill-lock.json"), &lock_json)?;

    Ok(())
}

fn write_json_atomic(path: &Path, value: &serde_json::Value) -> Result<(), SkillManageError> {
    let data = serde_json::to_vec_pretty(value)
        .map_err(|e| SkillManageError::ConfigError(format!("Failed to serialize JSON: {}", e)))?;
    write_file_atomic(path, &data)
}

fn write_csv_atomic<T: Serialize>(path: &Path, rows: &[T]) -> Result<(), SkillManageError> {
    let mut wtr = csv::Writer::from_writer(Vec::new());
    for row in rows {
        wtr.serialize(row)
            .map_err(|e| SkillManageError::ConfigError(format!("Failed to serialize CSV row: {}", e)))?;
    }
    wtr.flush()
        .map_err(|e| SkillManageError::ConfigError(format!("Failed to flush CSV writer: {}", e)))?;
    let data = wtr
        .into_inner()
        .map_err(|e| SkillManageError::ConfigError(format!("Failed to finalize CSV: {}", e)))?;
    write_file_atomic(path, &data)
}

fn skill_repo_name(skill_path: &Path) -> Option<String> {
    let components = skill_path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    for idx in 0..components.len() {
        if components[idx].eq_ignore_ascii_case("src") && idx + 1 < components.len() {
            return Some(components[idx + 1].clone());
        }
    }

    None
}

fn normalize_src_path(repo_root: &Path, path: &Path) -> String {
    let rel = if path.is_absolute() {
        path.strip_prefix(repo_root).unwrap_or(path)
    } else {
        path
    };
    rel.to_string_lossy().replace('\\', "/")
}

fn sanitize_description(s: &str) -> String {
    // Replace CR/LF with spaces and collapse multiple whitespace into single spaces
    s.replace('\r', " ")
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_native_aggregate_generates_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempdir()?;
        let src = root.path().join("src").join("demo-repo").join("skills").join("sample-skill");
        fs::create_dir_all(&src)?;

        let skill_md = r#"---
name: sample-skill
description: sample skill for testing
---

body
"#;
        fs::write(src.join("SKILL.md"), skill_md)?;

        let output = root.path().join("skills-aggregated");
        let skills = aggregate_to_output(root.path(), &output, None, true, false).await?;

        assert_eq!(skills.len(), 1);
        assert!(output.join("subhub-index.json").exists());

        // Debug: list output directory contents to help diagnose test failures.
        fn print_dir(path: &std::path::Path) {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    println!("OUT: {}", p.display());
                    if p.is_dir() {
                        if let Ok(sub) = std::fs::read_dir(&p) {
                            for s in sub.flatten() {
                                println!("  SUB: {}", s.path().display());
                            }
                        }
                    }
                }
            }
        }

        print_dir(&output);

        // subhub manifest should be present for the subhub (use returned skill's hub/sub_hub)
        let hub = &skills[0].hub;
        let sub_hub = &skills[0].sub_hub;
        assert!(output.join(hub).join(sub_hub).join("skills-manifest.json").exists());
        // SKILL.md is no longer generated by the native pipeline
        assert!(!output.join("SKILL.md").exists());

        Ok(())
    }

    #[test]
    fn test_sync_output_copy_mode() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempdir()?;
        let source = root.path().join("skills-aggregated");
        let target = root.path().join("target").join("skills");

        fs::create_dir_all(source.join("ai").join("llm-agents"))?;
        fs::write(
            source.join("ai").join("llm-agents").join("routing.csv"),
            "skill_id,triggers,score,src_path\n",
        )?;

        sync_output_to_targets(&source, &[target.clone()], NativeSyncMode::Copy)?;
        assert!(target.join("ai").join("llm-agents").join("routing.csv").exists());

        Ok(())
    }
}
