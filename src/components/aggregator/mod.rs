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

fn normalize_identifier(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_dash = false;

    for ch in input.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

fn fallback_name_from_path(path: &Path) -> String {
    let raw = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "skill".to_string());

    let normalized = normalize_identifier(&raw);
    if normalized.is_empty() {
        "skill".to_string()
    } else {
        normalized
    }
}

fn fallback_description_from_content(content: &str) -> String {
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line == "---" || line.starts_with("```") {
            continue;
        }

        if line.starts_with('#') {
            let header = line.trim_start_matches('#').trim();
            if !header.is_empty() {
                return header.to_string();
            }
            continue;
        }

        return line.to_string();
    }

    String::new()
}

fn clean_yaml_like_value(raw: &str) -> String {
    let mut value = raw.trim().to_string();

    if let Some(comment_pos) = value.find(" #") {
        value = value[..comment_pos].trim().to_string();
    }

    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        if value.len() >= 2 {
            return value[1..value.len() - 1].trim().to_string();
        }
    }

    if value.starts_with('[') && value.ends_with(']') && value.len() >= 2 {
        let inner = value[1..value.len() - 1].trim();
        if inner.is_empty() {
            return String::new();
        }

        let parts = inner
            .split(',')
            .map(|part| part.trim().trim_matches('"').trim_matches('\''))
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();

        if !parts.is_empty() {
            return parts.join(";");
        }

        return inner.to_string();
    }

    value
}

fn extract_yaml_like_value(yaml_part: &str, key: &str) -> Option<String> {
    let key_lc = key.to_ascii_lowercase();
    let mut lines = yaml_part.lines().peekable();

    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut split = line.splitn(2, ':');
        let current_key = split.next()?.trim().to_ascii_lowercase();
        if current_key != key_lc {
            continue;
        }

        let rest = split.next().unwrap_or("").trim();
        if !rest.is_empty() {
            return Some(clean_yaml_like_value(rest));
        }

        let mut seq_values = Vec::new();
        while let Some(peek_line) = lines.peek() {
            let next = peek_line.trim();
            if next.starts_with("- ") {
                let v = next.trim_start_matches("- ").trim();
                if !v.is_empty() {
                    seq_values.push(clean_yaml_like_value(v));
                }
                lines.next();
            } else {
                break;
            }
        }

        if !seq_values.is_empty() {
            return Some(seq_values.join(";"));
        }

        return None;
    }

    None
}

fn extract_frontmatter_and_body(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, trimmed);
    }

    let mut parts = trimmed.splitn(3, "---");
    let _ = parts.next();
    let yaml_part = parts.next();
    let body = parts.next().unwrap_or_default();
    (yaml_part, body)
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
        // Parse YAML when possible, but tolerate malformed or missing frontmatter
        // by extracting key/value lines and falling back to path/body heuristics.
        let (yaml_part, body_part) = extract_frontmatter_and_body(&content);

        let parsed_doc = if let Some(yaml) = yaml_part {
            serde_yaml::from_str::<serde_yaml::Value>(yaml).ok()
        } else {
            None
        };

        let get_string = |key: &str| -> Option<String> {
            if let Some(doc) = parsed_doc.as_ref() {
                if let Some(v) = doc.get(key) {
                    let converted = match v {
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
                    };

                    if !converted.trim().is_empty() {
                        return Some(converted.trim().to_string());
                    }
                }
            }

            yaml_part.and_then(|yaml| extract_yaml_like_value(yaml, key))
        };

        let get_u32 = |key: &str| -> Option<u32> {
            if let Some(doc) = parsed_doc.as_ref() {
                if let Some(v) = doc.get(key) {
                    return match v {
                        serde_yaml::Value::Number(n) => n.as_u64().map(|x| x as u32),
                        serde_yaml::Value::String(s) => s.parse::<u32>().ok(),
                        _ => None,
                    };
                }
            }

            yaml_part
                .and_then(|yaml| extract_yaml_like_value(yaml, key))
                .and_then(|v| v.parse::<u32>().ok())
        };

        let name = get_string("name")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| fallback_name_from_path(path));

        let description = get_string("description")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| fallback_description_from_content(body_part));
        // Preserve explicit frontmatter values when provided; classification
        // fallback logic decides final hub/sub_hub when they are absent.
        let hub = get_string("hub").unwrap_or_default();
        let sub_hub = get_string("sub_hub").unwrap_or_default();
        // Prefer an explicit `triggers` field, but fall back to `tags` when present
        // (many SKILL.md use `tags:` in YAML frontmatter). This ensures tag
        // tokens like `cloudflare` are surfaced to the categorization rules.
        let triggers = get_string("triggers").or_else(|| get_string("tags"));
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
        let source_roots = [Path::new("src"), Path::new("lib")]
            .into_iter()
            .filter(|p| p.exists() && p.is_dir())
            .collect::<Vec<_>>();

        if source_roots.is_empty() {
            return Err(SkillManageError::ConfigError(
                "No source directories found (expected 'src' or 'lib').".to_string(),
            ));
        }

        let spinner = self.progress.create_spinner("Scanning skills...");
        if let Some(reporter) = &self.progress.reporter {
            reporter.report(0, 100, "Scanning for SKILL.md files...".to_string());
        }

        let mut paths: Vec<PathBuf> = source_roots
            .iter()
            .flat_map(|root| {
                WalkDir::new(root)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_name() == "SKILL.md")
                    .map(|e| e.path().to_path_buf())
                    .collect::<Vec<_>>()
            })
            .collect();

        if paths.is_empty() {
            return Err(SkillManageError::ConfigError(
                "No SKILL.md files found under source directories (src/lib).".to_string(),
            ));
        }

        // Ensure deterministic traversal order across OS/filesystems.
        paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

        let msg = format!("Found {} SKILL.md files. Applying rules...", paths.len());
        spinner.set_message(msg.clone());
        if let Some(reporter) = &self.progress.reporter {
            reporter.report(20, 100, msg);
        }

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

        let mut seen_names = HashSet::new();
        let mut seen_descriptions = HashSet::new();
        let mut unique_results = Vec::new();

        for meta in results {
            let name_key = meta.name.to_lowercase();
            let desc_key = meta.description.trim().to_lowercase();

            // Skip if we've seen this exact name before
            if !seen_names.insert(name_key) {
                continue;
            }
            // Skip if we've seen this exact description before (different name, same content)
            if !desc_key.is_empty() && !seen_descriptions.insert(desc_key) {
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

