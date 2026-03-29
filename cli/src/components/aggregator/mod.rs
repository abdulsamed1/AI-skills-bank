pub mod rules;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use rayon::prelude::*;
use crate::error::SkillManageError;
use crate::components::CommandResult;
use crate::utils::progress::ProgressManager;
use crate::utils::atomicity::write_file_atomic;

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
#[serde(deny_unknown_fields)]
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
        
        Self {
            hub: hub.clone(),
            sub_hub: sub_hub.clone(),
            skill_id: skill_id.clone(),
            display_name: meta.name,
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
        
        let mut parts = content.split("---");
        let _ = parts.next(); 
        let yaml_part = parts.next().ok_or_else(|| {
            SkillManageError::ManifestValidationError(format!("Missing frontmatter in {}", path.display()))
        })?;

        let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_part).map_err(|e| {
            SkillManageError::ManifestParseError(format!("YAML error in {}: {}", path.display(), e))
        })?;

        Ok(SkillMetadata {
            name: frontmatter.name,
            description: frontmatter.description,
            path: path.to_path_buf(),
            hub: frontmatter.hub.unwrap_or_else(|| "ai".to_string()),
            sub_hub: frontmatter.sub_hub.unwrap_or_else(|| "llm-agents".to_string()),
            triggers: frontmatter.triggers,
            match_score: frontmatter.match_score,
            phase: frontmatter.phase,
            required: frontmatter.required,
            action: frontmatter.action,
        })
    }

    pub async fn aggregate(&self, _force: bool) -> Result<CommandResult, SkillManageError> {
        let src_path = Path::new("src");
        if !src_path.exists() {
            return Err(SkillManageError::ConfigError("Source directory 'src' not found. Run 'fetch' first.".to_string()));
        }

        let spinner = self.progress.create_spinner("Scanning skills...");
        
        let paths: Vec<PathBuf> = WalkDir::new(src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "SKILL.md")
            .map(|e| e.path().to_path_buf())
            .collect();

        spinner.set_message(format!("Found {} SKILL.md files. Applying rules...", paths.len()));

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
                    },
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
            if !seen.insert(meta.name.clone()) {
                // eprintln!("Warning: Duplicate skill_id found: '{}'. Skipping {}", meta.name, meta.path.display());
                continue;
            }
            unique_results.push(meta);
        }

        spinner.finish_with_message(format!("Aggregation complete. Processed {} unique skills.", unique_results.len()));
        
        Ok(CommandResult::Aggregate { skills: unique_results })
    }

    pub async fn generate_csv(&self, skills: Vec<SkillMetadata>) -> Result<(), SkillManageError> {
        let spinner = self.progress.create_spinner("Generating CSV...");
        
        tokio::task::spawn_blocking(move || {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            for meta in skills {
                let row = CsvRow::from(meta);
                wtr.serialize(row).map_err(|e| SkillManageError::ConfigError(e.to_string()))?;
            }
            wtr.flush().map_err(|e| SkillManageError::ConfigError(e.to_string()))?;
            let data = wtr.into_inner().map_err(|e| SkillManageError::ConfigError(e.to_string()))?;
            
            write_file_atomic(Path::new("hub-manifests.csv"), &data)?;
            Ok::<(), SkillManageError>(())
        }).await.map_err(|e| SkillManageError::ConfigError(e.to_string()))??;

        spinner.finish_with_message("Successfully generated hub-manifests.csv");
        Ok(())
    }
}
