pub mod rules;

use crate::components::CommandResult;
use crate::error::SkillManageError;
use crate::utils::atomicity::write_file_atomic;
use crate::utils::progress::ProgressManager;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub hub: String,
    pub sub_hub: String,
    #[serde(default)]
    pub triggers: Option<String>,
    #[serde(default)]
    pub match_score: Option<u32>,
    #[serde(default)]
    pub phase: Option<u32>,
    #[serde(default)]
    pub required: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    hub: Option<String>,
    #[serde(default)]
    sub_hub: Option<String>,
    #[serde(default)]
    triggers: Option<String>,
    #[serde(default)]
    match_score: Option<u32>,
    #[serde(default)]
    phase: Option<u32>,
    #[serde(default)]
    required: Option<String>,
    #[serde(default)]
    action: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CsvRow {
    pub hub: String,
    pub sub_hub: String,
    pub skill_id: String,
    pub display_name: String,
    pub description: String,
    pub triggers: String,
    pub match_score: u32,
    pub phase: u32,
    pub after: String,
    pub before: String,
    pub required: String,
    pub action: String,
    pub output_location: String,
    pub outputs: String,
}

impl From<SkillMetadata> for CsvRow {
    fn from(meta: SkillMetadata) -> Self {
        let hub = meta.hub;
        let sub_hub = meta.sub_hub;
        let skill_id = meta.name.clone();
        let display_name = skill_id.replace('-', " ").replace('_', " ");

        Self {
            hub: hub.clone(),
            sub_hub: sub_hub.clone(),
            skill_id: skill_id.clone(),
            display_name,
            description: meta.description,
            triggers: meta.triggers.unwrap_or_else(|| skill_id.replace('-', ";")),
            match_score: meta.match_score.unwrap_or(100),
            phase: meta.phase.unwrap_or(1),
            after: String::new(),
            before: String::new(),
            required: meta.required.unwrap_or_else(|| "true".to_string()),
            action: meta.action.unwrap_or_else(|| "invoke".to_string()),
            output_location: format!("outputs/{}/{}", hub, sub_hub),
            outputs: format!("{}-*", skill_id),
        }
    }
}

pub struct Aggregator {
    pub progress: Arc<ProgressManager>,
}

impl Aggregator {
    pub fn new(progress: Arc<ProgressManager>) -> Self {
        Self { progress }
    }

    pub fn parse_skill_md(path: &Path) -> Result<SkillMetadata, SkillManageError> {
        let content = std::fs::read_to_string(path)?;
        // Extract YAML frontmatter robustly and coerce fields to expected types.
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return Err(SkillManageError::ManifestValidationError(format!(
                "Missing frontmatter in {}",
                path.display()
            )));
        }

        let mut parts = trimmed.splitn(3, "---");
        let _ = parts.next();
        let yaml_part = parts.next().ok_or_else(|| {
            SkillManageError::ManifestValidationError(format!(
                "Missing frontmatter in {}",
                path.display()
            ))
        })?;

        let doc: serde_yaml::Value = serde_yaml::from_str(yaml_part).map_err(|e| {
            SkillManageError::ManifestParseError(format!("YAML error in {}: {}", path.display(), e))
        })?;

        let get_string = |key: &str| -> Option<String> {
            doc.get(key).map(|v| match v {
                serde_yaml::Value::String(s) => s.clone(),
                serde_yaml::Value::Sequence(seq) => seq
                    .iter()
                    .map(|el| match el {
                        serde_yaml::Value::String(s) => s.clone(),
                        other => serde_yaml::to_string(other).unwrap_or_default(),
                    })
                    .collect::<Vec<_>>()
                    .join(";"),
                serde_yaml::Value::Number(n) => n.to_string(),
                serde_yaml::Value::Bool(b) => b.to_string(),
                other => serde_yaml::to_string(other).unwrap_or_default(),
            })
        };

        let get_u32 = |key: &str| -> Option<u32> {
            doc.get(key).and_then(|v| match v {
                serde_yaml::Value::Number(n) => n.as_u64().map(|x| x as u32),
                serde_yaml::Value::String(s) => s.parse::<u32>().ok(),
                _ => None,
            })
        };

        let name = get_string("name").ok_or_else(|| {
            SkillManageError::ManifestParseError(format!("Missing required field `name` in {}", path.display()))
        })?;

        let description = get_string("description").unwrap_or_default();
        // Preserve explicit frontmatter values when provided; classification
        // fallback logic decides final hub/sub_hub when they are absent.
        let hub = get_string("hub").unwrap_or_default();
        let sub_hub = get_string("sub_hub").unwrap_or_default();
        let triggers = get_string("triggers");
        let match_score = get_u32("match_score");
        let phase = get_u32("phase");
        let required = get_string("required").or_else(|| Some("true".to_string()));
        let action = get_string("action").or_else(|| Some("invoke".to_string()));

        Ok(SkillMetadata {
            name,
            description,
            path: path.to_path_buf(),
            hub,
            sub_hub,
            triggers,
            match_score,
            phase,
            required,
            action,
        })
    }

    pub async fn aggregate(&self, _force: bool) -> Result<CommandResult, SkillManageError> {
        let src_path = Path::new("src");
        if !src_path.exists() {
            return Err(SkillManageError::ConfigError(
                "Source directory 'src' not found. Run 'fetch' first.".to_string(),
            ));
        }

        let spinner = self.progress.create_spinner("Scanning skills...");

        let mut paths: Vec<PathBuf> = WalkDir::new(src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "SKILL.md")
            .map(|e| e.path().to_path_buf())
            .collect();
        // Ensure deterministic traversal order across OS/filesystems.
        paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

        spinner.set_message(format!(
            "Found {} SKILL.md files. Applying rules...",
            paths.len()
        ));

        let results: Vec<SkillMetadata> = paths
            .par_iter()
            .filter_map(|path| {
                match Self::parse_skill_md(path) {
                    Ok(mut meta) => {
                        // Apply rules: categorization, exclusions, triggers, phase
                        if rules::apply_rules(&mut meta) {
                            Some(meta)
                        } else {
                            // Excluded by policy
                            None
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        let mut seen = HashSet::new();
        let mut unique_results = Vec::new();
        for meta in results {
            let key = meta.name.to_lowercase();
            if !seen.insert(key) {
                // eprintln!("Warning: Duplicate skill_id found: '{}'. Skipping {}", meta.name, meta.path.display());
                continue;
            }
            unique_results.push(meta);
        }

        spinner.finish_with_message(format!(
            "Aggregation complete. Processed {} unique skills.",
            unique_results.len()
        ));

        Ok(CommandResult::Aggregate {
            skills: unique_results,
        })
    }

    pub async fn generate_csv(&self, skills: Vec<SkillMetadata>) -> Result<(), SkillManageError> {
        let spinner = self.progress.create_spinner("Generating CSV...");

        tokio::task::spawn_blocking(move || {
            let mut wtr = csv::Writer::from_writer(Vec::new());

            // Write only the minimal critical columns requested by users.
            wtr.write_record(rules::CSV_COLUMNS)
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?;

            for meta in skills {
                // Compose the minimal record in the exact order defined by CSV_COLUMNS
                let skill_id = meta.name.clone();
                let outputs = format!("{}-*", skill_id);

                // Sanitize description to avoid newlines and collapse whitespace
                let description = meta
                    .description
                    .replace('\r', " ")
                    .replace('\n', " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                let record = vec![
                    meta.hub,
                    meta.sub_hub,
                    skill_id,
                    description,
                    outputs,
                ];

                wtr.write_record(&record)
                    .map_err(|e| SkillManageError::ConfigError(e.to_string()))?;
            }

            wtr.flush()
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?;
            let data = wtr
                .into_inner()
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?;

            write_file_atomic(Path::new("hub-manifests.csv"), &data)?;
            Ok::<(), SkillManageError>(())
        })
        .await
        .map_err(|e| SkillManageError::ConfigError(e.to_string()))??;

        spinner.finish_with_message("Successfully generated hub-manifests.csv");
        Ok(())
    }
}
