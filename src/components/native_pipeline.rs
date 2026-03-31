use crate::components::aggregator::{Aggregator, SkillMetadata};
use crate::components::aggregator::rules;
use crate::components::CommandResult;
use crate::error::SkillManageError;
use crate::utils::atomicity::{create_link_atomic, sync_dir_atomic, write_file_atomic};
use home::home_dir;
use crate::utils::progress::ProgressManager;
use crate::utils::theme::Theme;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;
use walkdir::WalkDir;
// `crate::components::llm` referenced via fully-qualified paths where needed
use std::env;
use tokio::time::sleep;
use std::time::Duration as StdDuration;

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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize)]
struct ReviewCandidate {
    skill_id: String,
    hub: String,
    sub_hub: String,
    score: u32,
    src_path: String,
}

const REVIEW_BAND_MIN_SCORE: u32 = 40;
const REVIEW_BAND_MAX_SCORE: u32 = 80;
const DEFAULT_MIN_SKILLS_PER_SUBHUB: usize = 1;

#[derive(Debug, Default)]
struct ExistingAssignments {
    by_skill: HashMap<String, (String, String)>,
    by_src: HashMap<String, (String, String)>,
}

#[derive(Debug, Deserialize)]
struct ExistingRoutingRow {
    skill_id: String,
    #[serde(default)]
    src_path: String,
}

pub async fn aggregate_to_output(
    repo_root: &Path,
    output_dir: &Path,
    selected_repos: Option<&HashSet<String>>,
    write_global_csv: bool,
    show_progress: bool,
) -> Result<Vec<SkillMetadata>, SkillManageError> {
    let _guard = CwdGuard::set(repo_root)?;
    let existing_assignments = load_existing_assignments(output_dir)?;
    let manual_overrides = load_manual_overrides(repo_root)?;

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

    // Apply any explicit manual overrides first (highest precedence)
    apply_manual_overrides(repo_root, &manual_overrides, &mut skills);

    // Load category exclusions from environment
    let exclusions_env = env::var("SKILL_MANAGE_EXCLUSIONS").unwrap_or_default();
    let excluded_cats: Vec<String> = exclusions_env
        .split(';')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    // Pre-filter skills against exclusions
    for skill in skills.iter_mut() {
        if skill.match_score.unwrap_or(0) >= 100 {
            continue;
        }

        let norm_text = format!("{} {}", skill.name, skill.description).to_lowercase();
        // Simple tokenization for exclusion check
        let tokens: HashSet<String> = norm_text
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if rules::is_excluded(&norm_text, &tokens) {
            skill.hub = "excluded".to_string();
            skill.sub_hub = "excluded".to_string();
            skill.match_score = Some(100); // Mark as fully handled
        }
    }

    // Build LLM classification context
    let mut context = crate::components::llm::types::LlmClassificationContext {
        valid_hubs: rules::VALID_HUBS.iter().map(|s| s.to_string()).collect(),
        excluded_categories: excluded_cats,
        ..Default::default()
    };
    
    // Collect all unique sub-hubs from definitions
    let mut all_sub_hubs = HashSet::new();
    for hub_def in rules::SUB_HUB_DEFINITIONS.values() {
        for sub_hub in hub_def.sub_hubs.keys() {
            all_sub_hubs.insert(sub_hub.to_string());
        }
    }
    context.valid_sub_hubs = all_sub_hubs.into_iter().collect();

    // LLM classification (primary). Honor the `LLM_ENABLED` env flag.
    let llm_enabled = env::var("LLM_ENABLED")
        .ok()
        .map(|v| {
            let lo = v.to_ascii_lowercase();
            !(lo == "false" || lo == "0" || lo == "no" || lo == "off")
        })
        .unwrap_or(true);

    if llm_enabled {
        if let Err(e) = classify_skills_with_llm(repo_root, &mut skills, &context).await {
            eprintln!("LLM classification system error: {}. Falling back to keywords for all unclassified skills.", e);
            for skill in skills.iter_mut() {
                if skill.match_score.unwrap_or(0) >= 100 {
                    continue;
                }
                rules::apply_rules(skill);
            }
        }
    } else {
        for skill in skills.iter_mut() {
            if skill.match_score.unwrap_or(0) >= 100 {
                continue;
            }
            rules::apply_rules(skill);
        }
    }

    // Then apply persisted routing only for low-confidence items.
    apply_existing_assignments(repo_root, &existing_assignments, &mut skills);

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

fn normalize_path_key(input: &str) -> String {
    input.trim().replace('\\', "/").to_lowercase()
}

fn phase_for_hub(hub: &str) -> u32 {
    match hub {
        "programming" => 1,
        "frontend" => 1,
        "backend" => 2,
        "testing" => 3,
        "ai" => 4,
        "business" => 5,
        "marketing" => 5,
        "design" => 5,
        "mobile" => 5,
        _ => 6,
    }
}

fn load_existing_assignments(output_dir: &Path) -> Result<ExistingAssignments, SkillManageError> {
    let mut out = ExistingAssignments::default();
    if !output_dir.exists() {
        return Ok(out);
    }

    let mut routing_files = WalkDir::new(output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name() == "routing.csv")
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    routing_files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    for routing_file in routing_files {
        let subhub_dir = match routing_file.parent() {
            Some(dir) => dir,
            None => continue,
        };

        let rel = match subhub_dir.strip_prefix(output_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut parts = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        if parts.len() < 2 {
            continue;
        }

        let hub = parts.remove(0).to_lowercase();
        let sub_hub = parts.remove(0).to_lowercase();

        let mut rdr = csv::Reader::from_path(&routing_file).map_err(|e| {
            SkillManageError::ConfigError(format!(
                "Failed reading existing routing file {}: {}",
                routing_file.display(),
                e
            ))
        })?;

        for row in rdr.deserialize::<ExistingRoutingRow>() {
            let row = row.map_err(|e| {
                SkillManageError::ConfigError(format!(
                    "Failed parsing existing routing row in {}: {}",
                    routing_file.display(),
                    e
                ))
            })?;

            let skill_key = row.skill_id.trim().to_lowercase();
            if !skill_key.is_empty() {
                out.by_skill
                    .entry(skill_key)
                    .or_insert_with(|| (hub.clone(), sub_hub.clone()));
            }

            let src_key = normalize_path_key(&row.src_path);
            if !src_key.is_empty() {
                out.by_src
                    .entry(src_key)
                    .or_insert_with(|| (hub.clone(), sub_hub.clone()));
            }
        }
    }

    Ok(out)
}

fn apply_existing_assignments(
    repo_root: &Path,
    existing: &ExistingAssignments,
    skills: &mut [SkillMetadata],
) {
    for skill in skills {
        // Keep high-confidence deterministic assignments (explicit/path-based)
        // and only use persisted routing for weaker fallback classifications.
        if skill.match_score.unwrap_or(0) >= 90 {
            continue;
        }

        let src_key = normalize_path_key(&normalize_src_path(repo_root, &skill.path));
        if let Some((hub, sub_hub)) = existing.by_src.get(&src_key) {
            skill.hub = hub.clone();
            skill.sub_hub = sub_hub.clone();
            skill.match_score = Some(100);
            skill.phase = Some(phase_for_hub(hub));
            continue;
        }

        let skill_key = skill.name.to_lowercase();
        if let Some((hub, sub_hub)) = existing.by_skill.get(&skill_key) {
            skill.hub = hub.clone();
            skill.sub_hub = sub_hub.clone();
            skill.match_score = Some(100);
            skill.phase = Some(phase_for_hub(hub));
        }
    }
}

#[derive(Debug, Deserialize)]
struct ManualOverrideRow {
    skill_id: String,
    hub: String,
    sub_hub: String,
    #[serde(default)]
    score: Option<u32>,
}

fn load_manual_overrides(
    repo_root: &Path,
) -> Result<HashMap<String, (String, String, Option<u32>)>, SkillManageError> {
    let mut out: HashMap<String, (String, String, Option<u32>)> = HashMap::new();

    let candidates = vec![
        repo_root.join("config").join("manual_overrides.csv"),
        repo_root.join("manual_overrides.csv"),
    ];

    for p in candidates {
        if p.exists() {
            let mut rdr = csv::Reader::from_path(&p).map_err(|e| {
                SkillManageError::ConfigError(format!(
                    "Failed reading manual overrides {}: {}",
                    p.display(),
                    e
                ))
            })?;

            for row in rdr.deserialize::<ManualOverrideRow>() {
                let row = row.map_err(|e| {
                    SkillManageError::ConfigError(format!(
                        "Failed parsing override row in {}: {}",
                        p.display(),
                        e
                    ))
                })?;

                out.insert(
                    row.skill_id.trim().to_lowercase(),
                    (row.hub.trim().to_lowercase(), row.sub_hub.trim().to_lowercase(), row.score),
                );
            }

            // Respect first-found overrides file
            break;
        }
    }

    Ok(out)
}

fn apply_manual_overrides(
    repo_root: &Path,
    overrides: &HashMap<String, (String, String, Option<u32>)>,
    skills: &mut [SkillMetadata],
) {
    if overrides.is_empty() {
        return;
    }

    for skill in skills.iter_mut() {
        let key = skill.name.to_lowercase();
        if let Some((hub, sub, score)) = overrides.get(&key) {
            skill.hub = hub.clone();
            skill.sub_hub = sub.clone();
            skill.match_score = Some(score.unwrap_or(100));
            skill.phase = Some(phase_for_hub(hub));
            continue;
        }

        // fallback: match by normalized src path
        let src_key = normalize_path_key(&normalize_src_path(repo_root, &skill.path));
        if let Some((hub, sub, score)) = overrides.get(&src_key) {
            skill.hub = hub.clone();
            skill.sub_hub = sub.clone();
            skill.match_score = Some(score.unwrap_or(100));
            skill.phase = Some(phase_for_hub(hub));
        }
    }
}

async fn classify_skills_with_llm(
    _repo_root: &Path,
    skills: &mut [SkillMetadata],
    context: &crate::components::llm::types::LlmClassificationContext,
) -> Result<(), SkillManageError> {
    // Load LLM client config from env; if absent, return an error so caller
    // falls back to deterministic rules.
    let config = match crate::components::llm::LlmClientConfig::from_env() {
        Ok(c) => c,
        Err(e) => return Err(SkillManageError::ConfigError(e.to_string())),
    };

    // Create provider instance
    let provider_name = config.provider.to_ascii_lowercase();
    let provider: Box<dyn crate::components::llm::LlmProvider> = match provider_name.as_str() {
        "openai" => Box::new(
            crate::components::llm::OpenAiProvider::new(config.clone())
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?,
        ),
        "claude" => Box::new(
            crate::components::llm::ClaudeProvider::new(config.clone())
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?,
        ),
        "custom" => Box::new(
            crate::components::llm::CustomProvider::new(config.clone())
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?,
        ),
        "mock" => Box::new(
            crate::components::llm::MockProvider::new(config.clone())
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?,
        ),
        "gemini" => Box::new(
            crate::components::llm::GeminiProvider::new(config.clone())
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?,
        ),
        "groq" => Box::new(
            crate::components::llm::GroqProvider::new(config.clone())
                .map_err(|e| SkillManageError::ConfigError(e.to_string()))?,
        ),
        other => {
            return Err(SkillManageError::ConfigError(format!(
                "Unknown LLM provider: {}",
                other
            )))
        }
    };

    // Retry/backoff configuration
    let max_retries: u32 = env::var("LLM_MAX_RETRIES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(3);

    let initial_backoff_ms: u64 = env::var("LLM_INITIAL_BACKOFF_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(500);

    let max_backoff_ms: u64 = env::var("LLM_MAX_BACKOFF_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60_000);

    // Load existing cache (ok to be empty)
    let mut cache = crate::components::llm::load_cache()?;
    println!("DEBUG: Found {} skills in total to classify", skills.len());

    // Helper performs classify with retries/backoff
    async fn classify_with_retry(
        provider: &dyn crate::components::llm::LlmProvider,
        skill_id: &str,
        description: &str,
        abstract_text: Option<&str>,
        context: &crate::components::llm::types::LlmClassificationContext,
        max_retries: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
    ) -> Result<crate::components::llm::LlmClassificationResponse, SkillManageError> {
        let attempts = if max_retries == 0 { 1 } else { max_retries };
        for attempt in 0..attempts {
            match provider
                .classify(skill_id, description, abstract_text, context)
                .await
            {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    // If rate limited and server provided a retry_after, honor it
                    if let crate::components::llm::LlmError::RateLimited { retry_after } = &e {
                        if let Some(secs) = retry_after {
                            let now_ms = (std::time::SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .subsec_millis()) as u64;
                            let jitter = now_ms % 1000;
                            let sleep_ms = (secs.saturating_mul(1000)).saturating_add(jitter);
                            sleep(StdDuration::from_millis(sleep_ms)).await;
                            continue;
                        }
                    }

                    if attempt + 1 >= attempts {
                        return Err(SkillManageError::ConfigError(e.to_string()));
                    }

                    // Exponential backoff with simple time-derived jitter
                    let multiplier = 1u64.checked_shl(attempt as u32).unwrap_or(u64::MAX);
                    let mut backoff = initial_backoff_ms.saturating_mul(multiplier);
                    if backoff > max_backoff_ms {
                        backoff = max_backoff_ms;
                    }
                    let now_ms = (std::time::SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .subsec_millis()) as u64;
                    let jitter = if backoff > 1 { now_ms % (backoff / 2 + 1) } else { 0 };
                    let sleep_ms = backoff / 2 + jitter;
                    sleep(StdDuration::from_millis(sleep_ms)).await;
                    continue;
                }
            }
        }

        Err(SkillManageError::ConfigError(
            "Exceeded retries for LLM classify".to_string(),
        ))
    }

    // Build list of skills that need classification (cache miss and not already handled)
    let mut to_classify: Vec<(usize, String, String, Option<String>, String)> = Vec::new();
    for (idx, skill) in skills.iter_mut().enumerate() {
        if skill.match_score.unwrap_or(0) >= 100 {
            continue;
        }

        // Deterministic cache key: repo+skill_path+content-hash
        let key = crate::components::llm::key_for_skill(_repo_root, &skill.path, &skill.name, &skill.description);

        if let Some(cached) = crate::components::llm::get_cached_classification(&cache, &key) {
            if let Some(top) = cached.ranked_suggestions.first() {
                skill.hub = top.hub.clone();
                skill.sub_hub = top.sub_hub.clone();
                skill.match_score = Some(top.confidence);
                skill.phase = Some(phase_for_hub(&skill.hub));
            }
            continue;
        }

        to_classify.push((
            idx,
            skill.name.clone(),
            skill.description.clone(),
            skill.triggers.clone(),
            key,
        ));
    }

    if !to_classify.is_empty() {
        // Batch size configurable
        let batch_size: usize = env::var("LLM_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(8);

        // Helper: batch classify with retries/backoff
        async fn classify_batch_with_retry(
            provider: &dyn crate::components::llm::LlmProvider,
            items: &[(String, String, Option<String>)],
            context: &crate::components::llm::types::LlmClassificationContext,
            max_retries: u32,
            initial_backoff_ms: u64,
            max_backoff_ms: u64,
        ) -> Result<Vec<crate::components::llm::LlmClassificationResponse>, SkillManageError> {
            let attempts = if max_retries == 0 { 1 } else { max_retries };
            for attempt in 0..attempts {
                match provider.classify_batch(items, context).await {
                    Ok(resps) => return Ok(resps),
                    Err(e) => {
                        if let crate::components::llm::LlmError::RateLimited { retry_after } = &e {
                            if let Some(secs) = retry_after {
                                let now_ms = (std::time::SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .subsec_millis()) as u64;
                                let jitter = now_ms % 1000;
                                let sleep_ms = (secs.saturating_mul(1000)).saturating_add(jitter);
                                sleep(StdDuration::from_millis(sleep_ms)).await;
                                continue;
                            }
                        }

                        if attempt + 1 >= attempts {
                            return Err(SkillManageError::ConfigError(e.to_string()));
                        }

                        let multiplier = 1u64.checked_shl(attempt as u32).unwrap_or(u64::MAX);
                        let mut backoff = initial_backoff_ms.saturating_mul(multiplier);
                        if backoff > max_backoff_ms {
                            backoff = max_backoff_ms;
                        }
                        let now_ms = (std::time::SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .subsec_millis()) as u64;
                        let jitter = if backoff > 1 { now_ms % (backoff / 2 + 1) } else { 0 };
                        let sleep_ms = backoff / 2 + jitter;
                        sleep(StdDuration::from_millis(sleep_ms)).await;
                        continue;
                    }
                }
            }
            Err(SkillManageError::ConfigError(
                "Exceeded retries for LLM batch classify".to_string(),
            ))
        }

        // Process in chunks
        for chunk in to_classify.chunks(batch_size) {
            let items: Vec<(String, String, Option<String>)> = chunk
                .iter()
                .map(|(_, name, description, abstract_text, _key)| {
                    (name.clone(), description.clone(), abstract_text.clone())
                })
                .collect();

            // Try batch classification
            let batch_result = classify_batch_with_retry(
                &*provider,
                &items,
                context,
                max_retries,
                initial_backoff_ms,
                max_backoff_ms,
            )
            .await;

            match batch_result {
                Ok(resps) if resps.len() == items.len() => {
                    for (i, resp) in resps.into_iter().enumerate() {
                        let (idx, _name, _description, _abstract, key) = &chunk[i];
                        if let Some(top) = resp.ranked_suggestions.first() {
                            let skill = &mut skills[*idx];
                            skill.hub = top.hub.clone();
                            skill.sub_hub = top.sub_hub.clone();
                            skill.match_score = Some(top.confidence);
                            skill.phase = Some(phase_for_hub(&skill.hub));
                        }
                        crate::components::llm::insert_into_map(&mut cache, key.clone(), resp);
                    }
                }
                Ok(_resps) => {
                    // unexpected length; fall back to per-item
                    for (j, (_idx, name, description, abstract_text, key)) in chunk.iter().enumerate() {
                        let resp = classify_with_retry(
                            &*provider,
                            name,
                            description,
                            abstract_text.as_deref(),
                            context,
                            max_retries,
                            initial_backoff_ms,
                            max_backoff_ms,
                        )
                        .await;

                        match resp {
                            Ok(resp) => {
                                if let Some(top) = resp.ranked_suggestions.first() {
                                    let skill = &mut skills[chunk[j].0];
                                    skill.hub = top.hub.clone();
                                    skill.sub_hub = top.sub_hub.clone();
                                    skill.match_score = Some(top.confidence);
                                    skill.phase = Some(phase_for_hub(&skill.hub));
                                }
                                crate::components::llm::insert_into_map(&mut cache, key.clone(), resp);
                            }
                            Err(_) => {
                                // Fallback to keyword rules for this specific skill
                                rules::apply_rules(&mut skills[chunk[j].0]);
                            }
                        }
                    }
                }
                Err(_e) => {
                    // Batch failed entirely; fallback to per-item classification
                    for (idx, name, description, abstract_text, key) in chunk.iter() {
                        let resp = classify_with_retry(
                            &*provider,
                            name,
                            description,
                            abstract_text.as_deref(),
                            context,
                            max_retries,
                            initial_backoff_ms,
                            max_backoff_ms,
                        )
                        .await;

                        match resp {
                            Ok(resp) => {
                                if let Some(top) = resp.ranked_suggestions.first() {
                                    let skill = &mut skills[*idx];
                                    skill.hub = top.hub.clone();
                                    skill.sub_hub = top.sub_hub.clone();
                                    skill.match_score = Some(top.confidence);
                                    skill.phase = Some(phase_for_hub(&skill.hub));
                                }
                                crate::components::llm::insert_into_map(&mut cache, key.clone(), resp);
                            }
                            Err(_) => {
                                // Fallback to keyword rules for this specific skill
                                rules::apply_rules(&mut skills[*idx]);
                            }
                        }
                    }
                }
            }
        }
    }

    // Persist cache
    crate::components::llm::save_cache(&cache)?;

    Ok(())
}

fn compute_git_info(repo_dir: &Path) -> Option<serde_json::Value> {
    // Only attempt git commands if a .git directory exists
    if !repo_dir.join(".git").exists() {
        return None;
    }

    let head = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let status = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    match (head, status) {
        (Some(h), Some(s)) => Some(json!({"head": h, "porcelain": s})),
        (Some(h), None) => Some(json!({"head": h})),
        _ => None,
    }
}

fn is_global_target(path: &Path) -> bool {
    if let Some(home) = home_dir() {
        return path.starts_with(home);
    }
    false
}

fn is_absolute_src_path(path: &str) -> bool {
    let p = Path::new(path);
    if p.is_absolute() {
        return true;
    }

    // Handle Windows-style absolute paths in normalized slash form (e.g. C:/...)
    let bytes = path.as_bytes();
    bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn infer_repo_root_from_output(source_root: &Path) -> PathBuf {
    if source_root
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("skills-aggregated"))
        .unwrap_or(false)
    {
        if let Some(parent) = source_root.parent() {
            return parent.to_path_buf();
        }
    }

    source_root.to_path_buf()
}

fn rewrite_routing_csv_to_absolute(
    source_root: &Path,
    target_root: &Path,
) -> Result<usize, SkillManageError> {
    let repo_root = infer_repo_root_from_output(source_root);
    let mut updated_files = 0usize;

    let mut routing_files = WalkDir::new(target_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name() == "routing.csv")
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();
    routing_files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    for routing_file in routing_files {
        let mut rdr = csv::Reader::from_path(&routing_file).map_err(|e| {
            SkillManageError::ConfigError(format!(
                "Failed reading routing for absolute rewrite {}: {}",
                routing_file.display(),
                e
            ))
        })?;

        let mut rows = Vec::new();
        let mut changed = false;
        for row in rdr.deserialize::<RoutingRow>() {
            let mut row = row.map_err(|e| {
                SkillManageError::ConfigError(format!(
                    "Failed parsing routing row in {}: {}",
                    routing_file.display(),
                    e
                ))
            })?;

            let src = row.src_path.trim().to_string();
            if src.is_empty() || is_absolute_src_path(&src) {
                rows.push(row);
                continue;
            }

            // Rewrite only known relative roots that point to local repository files.
            if !(src.starts_with("lib/") || src.starts_with("src/")) {
                rows.push(row);
                continue;
            }

            let absolute = repo_root.join(src.replace('/', "\\"));
            let normalized = absolute.to_string_lossy().replace('\\', "/");
            row.src_path = normalized;
            changed = true;
            rows.push(row);
        }

        if changed {
            write_csv_atomic(&routing_file, &rows)?;
            updated_files += 1;
        }
    }

    Ok(updated_files)
}

pub fn sync_output_to_targets(
    source_root: &Path,
    targets: &[PathBuf],
    mode: NativeSyncMode,
) -> Result<(), SkillManageError> {
    // use the CLI logger for user-visible messages
    let _logger = crate::utils::log::Logger::new(false, false, Arc::new(Theme::new()));
    if !source_root.exists() {
        return Err(SkillManageError::ConfigError(format!(
            "Aggregation output not found: {}",
            source_root.display()
        )));
    }

    for target in targets {
        let rewrite_absolute = is_global_target(target);
        let mut used_copy = false;
        // internal debug left intentionally minimal
        // If the destination already exists and is a reparse point (junction/symlink),
        // skip syncing to avoid copying into an existing junction that may point
        // back to the source (which causes recursive copy errors) or to external paths.
        // Prefer reading symlink metadata first (doesn't follow links). This is
        // important because `Path::exists()` follows symlinks and can return false
        // if the link target is inaccessible — but the link itself still exists.
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            match std::fs::symlink_metadata(target) {
                Ok(md) => {
                    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
                    if (md.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                        _logger.warn(&format!("Skipping sync for {}: destination is a junction/reparse-point", target.display()));
                        continue;
                    }
                }
                Err(_) => {
                    // If we cannot read metadata, fall back to `exists()` below.
                }
            }
        }
        #[cfg(not(windows))]
        {
            match std::fs::symlink_metadata(target) {
                    Ok(md) if md.file_type().is_symlink() => {
                        _logger.warn(&format!("Skipping sync for {}: destination is a symlink", target.display()));
                        continue;
                    }
                Ok(_) => {}
                Err(_e) => {
                    _logger.warn(&format!("Skipping sync for {}: unable to inspect target metadata", target.display()));
                    continue;
                }
            }
        }

        if target.exists() {
            // If the destination exists as a regular file/dir, proceed normally.
        }

        match mode {
            NativeSyncMode::Copy => {
                sync_dir_atomic(source_root, target)?;
                used_copy = true;
            }
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
                // To make the CLI safe to run for non-admin users across platforms
                // (especially Windows where creating junctions/symlinks may require
                // elevated privileges or developer mode), default `Auto` to copy-only
                // behavior. This avoids attempts to create junctions/symlinks that can
                // fail under restricted permissions. Users who want link-based
                // deployment can explicitly use `NativeSyncMode::Junction` or
                // `NativeSyncMode::SymbolicLink` via future CLI flags.
                sync_dir_atomic(source_root, target)?;
                used_copy = true;
            }
        }

        if rewrite_absolute && used_copy {
            rewrite_routing_csv_to_absolute(source_root, target)?;
        }
    }

    Ok(())
}

fn min_skills_per_subhub() -> usize {
    std::env::var("SKILL_MANAGE_MIN_SKILLS_PER_SUBHUB")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_MIN_SKILLS_PER_SUBHUB)
}

fn regroup_small_subhubs(
    grouped: BTreeMap<(String, String), Vec<SkillMetadata>>,
) -> BTreeMap<(String, String), Vec<SkillMetadata>> {
    let min_required = min_skills_per_subhub();
    if min_required <= 1 {
        return grouped;
    }

    let mut out: BTreeMap<(String, String), Vec<SkillMetadata>> = BTreeMap::new();
    let mut fallback_by_hub: BTreeMap<String, Vec<SkillMetadata>> = BTreeMap::new();

    for ((hub, sub_hub), mut skill_list) in grouped.into_iter() {
        if skill_list.len() < min_required {
            fallback_by_hub
                .entry(hub)
                .or_default()
                .append(&mut skill_list);
        } else {
            out.entry((hub, sub_hub)).or_default().append(&mut skill_list);
        }
    }

    for (hub, mut skill_list) in fallback_by_hub.into_iter() {
        out.entry((hub, "general".to_string()))
            .or_default()
            .append(&mut skill_list);
    }

    out
}

fn write_review_band(
    repo_root: &Path,
    output_dir: &Path,
    skills: &[SkillMetadata],
) -> Result<(), SkillManageError> {
    let candidates = skills
        .iter()
        .filter_map(|s| {
            let score = s.match_score.unwrap_or(0);
            if score < REVIEW_BAND_MIN_SCORE || score > REVIEW_BAND_MAX_SCORE {
                return None;
            }

            Some(ReviewCandidate {
                skill_id: s.name.clone(),
                hub: s.hub.clone(),
                sub_hub: s.sub_hub.clone(),
                score,
                src_path: normalize_src_path(repo_root, &s.path),
            })
        })
        .collect::<Vec<_>>();

    let payload = json!({
        "version": 1,
        "generated_at_unix": unix_now(),
        "min_score": REVIEW_BAND_MIN_SCORE,
        "max_score": REVIEW_BAND_MAX_SCORE,
        "candidates": candidates,
    });

    write_json_atomic(&output_dir.join("review-band.json"), &payload)
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
    grouped = regroup_small_subhubs(grouped);

    // Ensure marketing hub contains all defined sub-hubs (even if empty).
    // This makes it easier to navigate and consume many repos under `lib/`
    // by presenting a stable directory structure for marketing sub-hubs.
    if let Some(marking_def) = rules::SUB_HUB_DEFINITIONS.get("marketing") {
        for (sub_name, _rule) in marking_def.sub_hubs.iter() {
            let key = ("marketing".to_string(), sub_name.to_string());
            if !grouped.contains_key(&key) {
                // create directory and write empty artifacts
                let subhub_dir = output_dir.join("marketing").join(sub_name);
                std::fs::create_dir_all(&subhub_dir)?;

                // empty routing
                let empty_routing: Vec<RoutingRow> = Vec::new();
                write_csv_atomic(&subhub_dir.join("routing.csv"), empty_routing.as_slice())?;

                // empty catalog
                let empty_catalog: Vec<CatalogRow> = Vec::new();
                write_csv_atomic(&subhub_dir.join("skills-catalog.csv"), empty_catalog.as_slice())?;

                // index and manifest
                let index_json = json!({
                    "hub": "marketing",
                    "sub_hub": sub_name,
                    "count": 0,
                    "skills": []
                });
                write_json_atomic(&subhub_dir.join("skills-index.json"), &index_json)?;

                let manifest_json = json!({
                    "version": 1,
                    "hub": "marketing",
                    "sub_hub": sub_name,
                    "generated_at_unix": unix_now(),
                    "skills": []
                });
                write_json_atomic(&subhub_dir.join("skills-manifest.json"), &manifest_json)?;

                // (Placeholder index entries are added later once `subhub_index`
                // is available in scope to avoid borrow/move issues.)
            }
        }
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

    // Ensure marketing hub contains placeholder entries in the subhub index
    // for any defined sub-hubs that ended up empty so they are discoverable.
    if let Some(marking_def) = rules::SUB_HUB_DEFINITIONS.get("marketing") {
        for (sub_name, _rule) in marking_def.sub_hubs.iter() {
            let exists = subhub_index
                .iter()
                .any(|e| e.hub == "marketing" && e.sub_hub == *sub_name);
            if !exists {
                subhub_index.push(SubHubIndexEntry {
                    hub: "marketing".to_string(),
                    sub_hub: sub_name.to_string(),
                    skills_count: 0,
                    path: format!("marketing/{}", sub_name),
                });
            }
        }
    }

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

    let mut repo_objs = Vec::new();
    for name in repo_names.into_iter() {
        let repo_dir = resolve_repo_dir(repo_root, &name);
        if let Some(git) = compute_git_info(&repo_dir) {
            repo_objs.push(json!({"name": name, "git": git}));
        } else {
            repo_objs.push(json!({"name": name}));
        }
    }

    let review_candidates = skills
        .iter()
        .filter(|s| {
            let score = s.match_score.unwrap_or(0);
            score >= REVIEW_BAND_MIN_SCORE && score <= REVIEW_BAND_MAX_SCORE
        })
        .count();

    let exclude_categories = std::env::var("SKILL_MANAGE_EXCLUSIONS")
        .ok()
        .map(|v| {
            v.split(';')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let lock_json = json!({
        "generated_at_unix": unix_now(),
        "source": "native-cli",
        "src_repositories": repo_objs,
        "exclude_categories": exclude_categories,
        "min_skills_per_subhub": min_skills_per_subhub(),
        "review_band": {
            "min_score": REVIEW_BAND_MIN_SCORE,
            "max_score": REVIEW_BAND_MAX_SCORE,
            "candidates": review_candidates
        }
    });
    write_json_atomic(&output_dir.join(".skill-lock.json"), &lock_json)?;

    write_review_band(repo_root, output_dir, skills)?;

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

fn resolve_repo_dir(repo_root: &Path, repo_name: &str) -> PathBuf {
    let lib_dir = repo_root.join("lib").join(repo_name);
    if lib_dir.exists() {
        return lib_dir;
    }

    let src_dir = repo_root.join("src").join(repo_name);
    if src_dir.exists() {
        return src_dir;
    }

    lib_dir
}

fn skill_repo_name(skill_path: &Path) -> Option<String> {
    let components = skill_path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    for idx in 0..components.len() {
        if (components[idx].eq_ignore_ascii_case("src")
            || components[idx].eq_ignore_ascii_case("lib"))
            && idx + 1 < components.len()
        {
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
