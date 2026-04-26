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
use tokio::sync::Mutex;
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
    let progress = Arc::new(ProgressManager::new(show_progress, false, Arc::clone(&theme), None));
    let aggregator = Aggregator::new(Arc::clone(&progress));

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

    // Load category exclusions from environment early so context can use it
    let exclusions_env = env::var("SKILL_MANAGE_EXCLUSIONS").unwrap_or_default();
    let excluded_cats: Vec<String> = exclusions_env
        .split(';')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

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

    // 1. LLM classification (PRIMARY PROPOSER)
    // Runs before manual overrides, so it operates on raw skills.
    let llm_enabled = env::var("LLM_ENABLED")
        .ok()
        .map(|v| {
            let lo = v.to_ascii_lowercase();
            !(lo == "false" || lo == "0" || lo == "no" || lo == "off")
        })
        .unwrap_or(true);

    if llm_enabled {
        if let Err(e) = classify_skills_with_llm(repo_root, &mut skills, &context, &progress).await {
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

    // 2. Apply explicit manual overrides (Highest User Precedence)
    // This overwrites LLM classifications if there's a match.
    apply_manual_overrides(repo_root, &manual_overrides, &mut skills);

    // 3. Pre-filter skills against static hardcoded exclusions (Regulatory Gate)
    // Runs after LLM to ensure excluded tags always "win".
    for skill in skills.iter_mut() {
        if skill.hub == "excluded" {
            continue;
        }

        let full_text = format!("{} {} {}", skill.name, skill.description, skill.path.to_string_lossy());
        let (norm_text, tokens) = rules::normalize_text(&full_text);

        // EXTRA TRACING
        if skill.name.to_lowercase().contains("resume") || skill.name.to_lowercase().contains("game") {
             println!("DEBUG: Processing skill: '{}' | Excluded? {}", skill.name, rules::is_excluded(&norm_text, &tokens));
        }

        if rules::is_excluded(&norm_text, &tokens) {
            skill.hub = "excluded".to_string();
            skill.sub_hub = "excluded".to_string();
            skill.match_score = Some(100);
        }
    }

    // 4. Apply persisted routing only for low-confidence items.
    let mut migrations = Vec::new();
    for skill in skills.iter_mut() {
        if let Some((prev_hub, prev_sub)) = existing_assignments.by_skill.get(&skill.name) {
            let score = skill.match_score.unwrap_or(0);
            
            // If the new rule matched with confidence (>= 70) and it's DIFFERENT 
            // from the previous assignment, log it as a Migration.
            if score >= 70 && (prev_hub != &skill.hub || prev_sub != &skill.sub_hub) {
                migrations.push(format!(
                    "Migrated `{}`: {}/{} -> {}/{} (score: {})",
                    skill.name, prev_hub, prev_sub, skill.hub, skill.sub_hub, score
                ));
            } else if score < 70 {
                // Otherwise fallback to existing if we have zero confidence.
                skill.hub = prev_hub.clone();
                skill.sub_hub = prev_sub.clone();
                skill.match_score = Some(60); 
            }
        }
    }

    if !migrations.is_empty() {
        let migration_path = output_dir.join("migration_delta.log");
        let content = migrations.join("\n");
        let _ = std::fs::write(migration_path, content);
        println!("INFO: {} skills migrated to new hubs based on refined rules.", migrations.len());
    }

    // Final cleanup: Drop all skills that were explicitly excluded by Code Rules (Step A) or LLM (Step B)
    skills.retain(|s| s.hub != "excluded");

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
        "code-quality" => 1,
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

#[allow(dead_code)]
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
    progress: &ProgressManager,
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

    // Load existing cache (ok to be empty) and wrap in an Arc<Mutex<>> for safe concurrent access.
    let initial_cache = crate::components::llm::load_cache()?;
    let cache = Arc::new(Mutex::new(initial_cache));
    // Snapshot the cache for fast synchronous lookups while building the to_classify list
    let cache_snapshot = {
        let guard = cache.lock().await;
        guard.clone()
    };
    println!("DEBUG: Found {} skills in total to classify", skills.len());

    // Build list of skills that need classification (cache miss and not already handled)
    let mut to_classify: Vec<(usize, String, String, Option<String>, String)> = Vec::new();
    for (idx, skill) in skills.iter_mut().enumerate() {
        if skill.match_score.unwrap_or(0) >= 90 {
            continue;
        }

        // Deterministic cache key: repo+skill_path+content-hash
        let key = crate::components::llm::key_for_skill(_repo_root, &skill.path, &skill.name, &skill.description, skill.content_body.as_deref());

        if let Some(cached) = crate::components::llm::get_cached_classification(&cache_snapshot, &key) {
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
            skill.content_body.clone(),
            key,
        ));
    }

        if !to_classify.is_empty() {
        // Batch size configurable
        let batch_size: usize = env::var("LLM_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(10);

        // Process in chunks concurrently (bounded)
        let total_tc = to_classify.len();
        let concurrency: usize = env::var("LLM_CONCURRENCY")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(10);

        let pb = progress.create_main_bar(total_tc as u64, "Classifying skills (LLM)");
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let provider = Arc::new(provider);
        let mut handles = Vec::new();

        for (_chunk_idx, chunk) in to_classify.chunks(batch_size).enumerate() {
            let chunk_vec: Vec<(usize, String, String, Option<String>, String)> =
                chunk.iter().cloned().collect();

            let provider = Arc::clone(&provider);
            let context = context.clone();
            let semaphore = Arc::clone(&semaphore);
            let pb = pb.clone();

            let max_retries = max_retries;
            let initial_backoff_ms = initial_backoff_ms;
            let max_backoff_ms = max_backoff_ms;

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.map_err(|_| {
                    SkillManageError::ConfigError("LLM concurrency semaphore closed".to_string())
                })?;

                let attempts = if max_retries == 0 { 1 } else { max_retries };
                // try batch classify with retries
                let item_payload: Vec<(String, String, Option<String>)> = chunk_vec
                    .iter()
                    .map(|(_, name, description, abstract_text, _)| {
                        (name.clone(), description.clone(), abstract_text.clone())
                    })
                    .collect();

                let chunk_len = chunk_vec.len();
                // Attempt batch classification with retry/backoff
                let mut batch_ok = false;
                let mut out: Vec<(usize, Option<crate::components::llm::LlmClassificationResponse>, String)> = Vec::new();

                for attempt in 0..attempts {
                    match (&*provider).classify_batch(&item_payload, &context).await {
                        Ok(resps) if resps.len() == item_payload.len() => {
                            for (i, resp) in resps.into_iter().enumerate() {
                                let (idx, _name, _description, _abstract, key) = &chunk_vec[i];
                                out.push((*idx, Some(resp), key.clone()));
                            }
                            batch_ok = true;
                            pb.inc(chunk_len as u64);
                            break;
                        }
                        Ok(resps) => {
                            eprintln!("DEBUG: Batch response length mismatch (expected {}, got {})", item_payload.len(), resps.len());
                            break;
                        }
                        Err(e) => {
                            // Circuit breaker: only 410 Gone (model permanently deprecated) — don't retry
                            if let crate::components::llm::LlmError::ProviderUnavailable(ref msg) = e {
                                if msg.contains("410") {
                                    eprintln!("FATAL: Model permanently unavailable (410 Gone): {}. Skipping all LLM.", msg);
                                    pb.inc(chunk_len as u64);
                                    return Ok(vec![]);
                                }
                            }
                            if let crate::components::llm::LlmError::AuthenticationFailed(text) = &e {
                                eprintln!("CRITICAL: Authentication Failed. Stopping process: {}", text);
                                std::process::exit(1);
                            }

                            if let crate::components::llm::LlmError::RateLimited { retry_after } = &e {
                                if let Some(secs) = retry_after {
                                    let now_ms = (std::time::SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .subsec_millis()) as u64;
                                    let jitter = now_ms % 1000;
                                    let sleep_ms = secs.saturating_mul(1000).saturating_add(jitter);
                                    sleep(StdDuration::from_millis(sleep_ms)).await;
                                    continue;
                                }
                            }

                            if attempt + 1 >= attempts {
                                eprintln!("DEBUG: Batch failed after {} attempts: {:?}", attempts, e);
                                break;
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

                // If batch didn't succeed, fall back to per-item classification with retries
                if !batch_ok {
                    for (idx, name, description, abstract_text, key) in chunk_vec.into_iter() {
                        let mut classified: Option<crate::components::llm::LlmClassificationResponse> = None;
                        for attempt in 0..attempts {
                            match (&*provider).classify(&name, &description, abstract_text.as_deref(), &context).await {
                                Ok(resp) => {
                                    classified = Some(resp);
                                    break;
                                }
                                Err(e) => {
                                    // Circuit breaker: only 410 Gone — permanent
                                    if let crate::components::llm::LlmError::ProviderUnavailable(ref msg) = e {
                                        if msg.contains("410") {
                                            break;
                                        }
                                    }
                                    if let crate::components::llm::LlmError::RateLimited { retry_after } = &e {
                                        if let Some(secs) = retry_after {
                                            let now_ms = (std::time::SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .subsec_millis()) as u64;
                                            let jitter = now_ms % 1000;
                                            let sleep_ms = secs.saturating_mul(1000).saturating_add(jitter);
                                            sleep(StdDuration::from_millis(sleep_ms)).await;
                                            continue;
                                        }
                                    }
                                    if attempt + 1 >= attempts {
                                        break;
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
                        out.push((idx, classified, key));
                        pb.inc(1);
                    }
                }

                Ok::<_, SkillManageError>(out)
            });

            handles.push(handle);
        }

        // Collect results from concurrent tasks and apply them to in-memory skills/cache
        let total_handles = handles.len();
        let mut completed_handles = 0;
        
        for h in handles {
            match h.await {
                Ok(Ok(items)) => {
                    completed_handles += 1;
                    for (idx, opt_resp, key) in items {
                        if let Some(resp) = opt_resp {
                            // Capture top suggestion locally to avoid holding a borrow across an await
                            let top_clone = resp.ranked_suggestions.first().cloned();

                            // Insert into shared cache (lock briefly)
                            {
                                let mut guard = cache.lock().await;
                                crate::components::llm::insert_into_map(&mut *guard, key, resp);
                            }

                            // Apply classification to skill using the cloned top suggestion
                            if let Some(top) = top_clone {
                                let skill = &mut skills[idx];
                                skill.hub = top.hub.clone();
                                skill.sub_hub = top.sub_hub.clone();
                                skill.match_score = Some(top.confidence);
                                skill.phase = Some(phase_for_hub(&skill.hub));
                            }
                        } else {
                            // Fallback to keyword rules for this specific skill
                            rules::apply_rules(&mut skills[idx]);
                        }
                    }
                    
                    if total_handles > 1 && completed_handles % 5 == 0 {
                        if let Some(reporter) = &progress.reporter {
                            reporter.report(
                                (completed_handles * batch_size) as u64,
                                total_tc as u64,
                                format!("Processing LLM results: {}/{}", completed_handles, total_handles),
                            );
                        }
                    }
                }
                Ok(Err(e)) => {
                    completed_handles += 1;
                    eprintln!("LLM batch task error: {}", e);
                }
                Err(join_err) => {
                    completed_handles += 1;
                    eprintln!("LLM batch task join error: {:?}", join_err);
                }
            }
        }
        pb.finish_and_clear();
    }

    // Persist cache
    {
        let guard = cache.lock().await;
        crate::components::llm::save_cache(&*guard)?;
    }

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
        // Handle existing reparse points (junctions/symlinks)
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if let Ok(md) = std::fs::symlink_metadata(target) {
                const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
                if (md.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                    // Only skip if we are in Copy mode to avoid accidental recursion.
                    // For Link-based modes (including Auto), we want to proceed so
                    // create_link_atomic can replace the link.
                    if matches!(mode, NativeSyncMode::Copy) {
                        _logger.warn(&format!("Skipping Copy-sync for {}: destination is a junction/reparse-point", target.display()));
                        continue;
                    }
                }
            }
        }
        #[cfg(not(windows))]
        {
            if let Ok(md) = std::fs::symlink_metadata(target) {
                if md.file_type().is_symlink() {
                    if matches!(mode, NativeSyncMode::Copy) {
                        _logger.warn(&format!("Skipping Copy-sync for {}: destination is a symlink", target.display()));
                        continue;
                    }
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
                    match create_link_atomic(source_root, target) {
                        Ok(()) => {
                            // link created successfully
                        }
                        Err(link_err) => {
                            _logger.warn(&format!(
                                "Junction linking would replace existing target {}: {}. Falling back to non-destructive merge copy.",
                                target.display(), link_err
                            ));
                            sync_dir_atomic(source_root, target)?;
                            used_copy = true;
                        }
                    }
                }
                #[cfg(not(windows))]
                {
                    return Err(SkillManageError::ConfigError(
                        "Junction mode is only supported on Windows".to_string(),
                    ));
                }
            }
            NativeSyncMode::SymbolicLink => {
                match create_link_atomic(source_root, target) {
                    Ok(()) => {}
                    Err(link_err) => {
                        _logger.warn(&format!(
                            "Symbolic linking failed for {}: {}. Falling back to non-destructive merge copy.",
                            target.display(), link_err
                        ));
                        sync_dir_atomic(source_root, target)?;
                        used_copy = true;
                    }
                }
            }
            NativeSyncMode::Auto => {
                // ── Auto Strategy: Try Link, Fallback to Copy ──
                //
                //  skills-aggregated/   ──(junction)──▶  ~/.gemini/skills/
                //                       ──(junction)──▶  ~/.cursor/skills/
                //                       ──(junction)──▶  ~/.claude/skills/
                //
                // Prefer directory links (Junctions on Windows, Symlinks on
                // Unix) so every tool points at the single aggregated output.
                // Only fall back to a full copy when linking fails — e.g. the
                // target lives on a different volume, or the OS denies the
                // privilege.
                match create_link_atomic(source_root, target) {
                    Ok(()) => {
                        // Link created successfully — no copy needed.
                    }
                    Err(link_err) => {
                        _logger.warn(&format!(
                            "Linking failed for {}: {}. Falling back to copy.",
                            target.display(), link_err
                        ));
                        sync_dir_atomic(source_root, target)?;
                        used_copy = true;
                    }
                }
            }
        }

        if rewrite_absolute && used_copy {
            rewrite_routing_csv_to_absolute(source_root, target)?;
        }
        
        let _ = ensure_main_hub_routers(target);
    }

    // ── Post-Sync Cleanup ──
    // Remove any stale .link.tmp directories left over from atomic rename
    // operations (e.g. if the rename was interrupted or failed).
    // This keeps the sync results clean and avoids accumulating temp files.
    cleanup_temp_link_files(source_root, targets)?;

    Ok(())
}

/// Cleanup temporary .link.tmp files that may be left over from atomic link creation.
fn cleanup_temp_link_files(source_root: &Path, targets: &[PathBuf]) -> Result<(), SkillManageError> {
    fn cleanup_temp_link_files_in_dir(dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.ends_with(".link.tmp")
                        && entry
                            .file_type()
                            .ok()
                            .map(|ft| ft.is_dir())
                            .unwrap_or(false)
                    {
                        let path = entry.path();
                        let _ = std::fs::remove_dir_all(&path);
                    }
                }
            }
        }
    }

    // Check source root and its parent. Temp link dirs are created as siblings
    // to the destination path, so parent directories must be scanned as well.
    cleanup_temp_link_files_in_dir(source_root);
    if let Some(parent) = source_root.parent() {
        cleanup_temp_link_files_in_dir(parent);
    }

    // Check each target directory and parent directory for *.link.tmp.
    for target in targets {
        cleanup_temp_link_files_in_dir(target);
        if let Some(parent) = target.parent() {
            cleanup_temp_link_files_in_dir(parent);
        }
    }

    Ok(())
}

fn ensure_main_hub_routers(target_path: &Path) -> Result<(), SkillManageError> {
    let iter = match std::fs::read_dir(target_path) {
        Ok(it) => it,
        Err(_) => return Ok(()),
    };

    for entry in iter.flatten() {
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            || std::fs::metadata(entry.path()).map(|m| m.is_dir()).unwrap_or(false);

        if is_dir {
            let hub_name = entry.file_name();
                let hub_name_str = hub_name.to_string_lossy();
                
                if hub_name_str.starts_with('.') {
                    continue;
                }

                let hub_path = entry.path();
                let skill_path = hub_path.join("SKILL.md");
                if !skill_path.exists() {
                    let mut sub_hubs = Vec::new();
                    if let Ok(sub_iter) = std::fs::read_dir(&hub_path) {
                        for sub_entry in sub_iter.flatten() {
                            if let Ok(sub_ft) = sub_entry.file_type() {
                                if sub_ft.is_dir() && !sub_entry.file_name().to_string_lossy().starts_with('.') {
                                    sub_hubs.push(sub_entry.file_name().to_string_lossy().into_owned());
                                }
                            }
                        }
                    }
                    sub_hubs.sort();

                    let mut table = String::from("| Sub-Hub | Routing |\n|---------|---------|\n");
                    for sh in &sub_hubs {
                        table.push_str(&format!("| {} | {}/routing.csv |\n", sh, sh));
                    }

                    let content = format!(r#"---
name: {}
description: |
  Router for {} skills ({} sub-hubs).
  DO NOT execute from this file. Follow the steps below to load the real skill.
---

# {} Hub

## Sub-Hubs

{}

## How To Use

1. Match user request to a sub-hub from the table above.
2. Open `<sub_hub>/routing.csv` in this directory.
3. Find the `skill_id` row whose `description` best matches the task.
4. Read the full skill from the `src_path` column (the SKILL.md in `lib/`).
5. Follow that SKILL.md as the source of truth.

## Anti-Hallucination

- NEVER guess skill behavior from description alone.
- ALWAYS load the actual SKILL.md from src_path before acting.
- If ambiguous, present top 3 candidates to the user.
"#, hub_name_str, hub_name_str, sub_hubs.len(), hub_name_str, table.trim());

                    std::fs::write(&skill_path, content).map_err(|e| {
                        SkillManageError::ConfigError(format!("Failed to write SKILL.md router: {}", e))
                    })?;
                }
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
        if skill.hub == "excluded" {
            continue;
        }
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

    let mut hub_info: std::collections::BTreeMap<String, Vec<&SubHubIndexEntry>> = std::collections::BTreeMap::new();
    for entry in &subhub_index {
        hub_info.entry(entry.hub.clone()).or_default().push(entry);
    }

    for (hub, entries) in &hub_info {
        let mut total_skills = 0;
        let mut table = String::from("| Sub-Hub | Skills | Routing |\n|---------|--------|---------|\n");
        for entry in entries {
            total_skills += entry.skills_count;
            table.push_str(&format!("| {} | {} | {}/routing.csv |\n", entry.sub_hub, entry.skills_count, entry.sub_hub));
        }

        let content = format!(r#"---
name: {}
description: |
  Router for {} skills ({} skills across {} sub-hubs).
  DO NOT execute from this file. Follow the steps below to load the real skill.
---

# {} Hub

## Sub-Hubs

{}

## How To Use

1. Match user request to a sub-hub from the table above.
2. Open `<sub_hub>/routing.csv` in this directory.
3. Find the `skill_id` row whose `description` best matches the task.
4. Read the full skill from the `src_path` column (the SKILL.md in `lib/`).
5. Follow that SKILL.md as the source of truth.

## Anti-Hallucination

- NEVER guess skill behavior from description alone.
- ALWAYS load the actual SKILL.md from src_path before acting.
- If ambiguous, present top 3 candidates to the user.
"#, hub, hub, total_skills, entries.len(), hub, table.trim());

        write_file_atomic(&output_dir.join(&hub).join("SKILL.md"), content.as_bytes())?;
    }

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

    // Master router: AGENTS.md is the canonical entrypoint for all skill discovery.
    // It lists all hubs and provides instructions for downstream agents.
    let mut master_table = String::from("| Hub | Skills | Router |\n|-----|--------|--------|\n");
    let mut total_master_skills = 0;
    for (hub, entries) in &hub_info {
        let hub_skills: usize = entries.iter().map(|e| e.skills_count).sum();
        total_master_skills += hub_skills;
        master_table.push_str(&format!("| {} | {} | {}/SKILL.md |\n", hub, hub_skills, hub));
    }

    let master_content = format!(r#"---
name: skills-bank
description: |
  Master router for all skills bank agents ({} skills across {} hubs).
  DO NOT execute from this file. Follow the steps below to load the real skill.
---

# Skills Bank Master Router

## Skill Hubs

{}

## How To Use

1. Match user request to a high-level hub from the table above.
2. Open `<hub>/SKILL.md` in this directory.
3. Follow the instructions in that hub's router.

## Anti-Hallucination Guardrails (11 HUBS ONLY)

- NEVER guess skill behavior from description alone.
- ALWAYS load the actual SKILL.md from src_path before acting.
- If ambiguous, present top 3 candidates to the user.
"#, total_master_skills, hub_info.len(), master_table.trim());

    write_file_atomic(&output_dir.join("AGENTS.md"), master_content.as_bytes())?;

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

