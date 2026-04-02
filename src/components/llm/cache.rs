use crate::components::llm::types::LlmClassificationResponse;
use crate::error::SkillManageError;
use crate::utils::atomicity::write_file_atomic;
use home::home_dir;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn default_cache_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("LLM_CACHE_PATH") {
        return Some(PathBuf::from(p));
    }
    home_dir().map(|mut h| {
        h.push(".skill-manage");
        // ensure dir exists
        let _ = fs::create_dir_all(&h);
        h.push("llm-classifications.json");
        h
    })
}

pub fn cache_file_path() -> Result<PathBuf, SkillManageError> {
    default_cache_path().ok_or_else(|| SkillManageError::ConfigError("Unable to resolve home directory for LLM cache; set LLM_CACHE_PATH".to_string()))
}
#[derive(Serialize, Deserialize, Debug)]
struct CacheFileEntry {
    classification: LlmClassificationResponse,
    #[serde(default)]
    provider_metadata: Option<serde_json::Value>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    skill_path: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description_hash: Option<String>,
    #[serde(default)]
    inserted_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheFile {
    schema_version: u8,
    entries: HashMap<String, CacheFileEntry>,
}

static CACHE_HITS: AtomicUsize = AtomicUsize::new(0);
static CACHE_MISSES: AtomicUsize = AtomicUsize::new(0);
static CACHE_INSERTS: AtomicUsize = AtomicUsize::new(0);
static CACHE_ERRORS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub hits: usize,
    pub misses: usize,
    pub inserts: usize,
    pub errors: usize,
}

pub fn cache_metrics() -> CacheMetrics {
    CacheMetrics {
        hits: CACHE_HITS.load(Ordering::SeqCst),
        misses: CACHE_MISSES.load(Ordering::SeqCst),
        inserts: CACHE_INSERTS.load(Ordering::SeqCst),
        errors: CACHE_ERRORS.load(Ordering::SeqCst),
    }
}

/// Compute a deterministic cache key for a skill using repo origin (if available),
/// relative skill path, and a normalized name+description fingerprint.
pub fn key_for_skill(
    repo_root: &Path,
    skill_path: &Path,
    name: &str,
    description: &str,
    content_body: Option<&str>,
) -> String {
    // repo origin via `git remote get-url origin` when possible
    let repo_origin = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            repo_root
                .canonicalize()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| repo_root.to_string_lossy().to_string())
        });

    let relative = skill_path
        .strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| skill_path.to_string_lossy().replace('\\', "/"));

    let normalize = |s: &str| {
        let mut out = s.trim().to_lowercase();
        // collapse whitespace
        out = out.split_whitespace().collect::<Vec<_>>().join(" ");
        out
    };

    let mut hash_payload = format!("{}|{}", name, description);
    if std::env::var("LLM_CACHE_COMPAT").unwrap_or_default() != "v1" {
        if let Some(body) = content_body {
            hash_payload.push_str(&format!("|{}", body));
        }
    }
    let payload = format!("{}|{}|{}", repo_origin, relative, normalize(&hash_payload));
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let digest = hasher.finalize();
    let hex = digest.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    hex
}

/// Return a cloned classification if present; increments metrics counters.
pub fn get_cached_classification(
    map: &HashMap<String, LlmClassificationResponse>,
    key: &str,
) -> Option<LlmClassificationResponse> {
    if let Some(v) = map.get(key) {
        CACHE_HITS.fetch_add(1, Ordering::SeqCst);
        Some(v.clone())
    } else {
        CACHE_MISSES.fetch_add(1, Ordering::SeqCst);
        None
    }
}

/// Insert into the given map and increment insert metric.
pub fn insert_into_map(
    map: &mut HashMap<String, LlmClassificationResponse>,
    key: String,
    value: LlmClassificationResponse,
) {
    map.insert(key, value);
    CACHE_INSERTS.fetch_add(1, Ordering::SeqCst);
}

pub fn invalidate_key(map: &mut HashMap<String, LlmClassificationResponse>, key: &str) {
    map.remove(key);
}

pub fn load_cache() -> Result<HashMap<String, LlmClassificationResponse>, SkillManageError> {
    let path = cache_file_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let data = fs::read_to_string(&path)?;

    // Try new wrapper format first
    match serde_json::from_str::<CacheFile>(&data) {
        Ok(wrapper) => {
            let mut out = HashMap::new();
            for (k, v) in wrapper.entries.into_iter() {
                out.insert(k, v.classification);
            }
            return Ok(out);
        }
        Err(_e) => {
            // Try legacy format (raw map)
            match serde_json::from_str::<HashMap<String, LlmClassificationResponse>>(&data) {
                Ok(legacy) => {
                    // On next save we'll migrate to wrapper format
                    return Ok(legacy);
                }
                Err(parse_err) => {
                    // Corrupted file: rename and continue with empty cache
                    CACHE_ERRORS.fetch_add(1, Ordering::SeqCst);
                    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                    let corrupt_name = format!("{}.corrupt.{}.json", path.file_name().unwrap().to_string_lossy(), ts);
                    let corrupt_path = path.with_file_name(corrupt_name);
                    let _ = fs::rename(&path, &corrupt_path);
                    let _ = fs::write(&path, "{}"
                        .as_bytes());
                    return Err(SkillManageError::ConfigError(format!("Failed parsing LLM cache {}: {}", path.display(), parse_err)));
                }
            }
        }
    }
}

pub fn save_cache(map: &HashMap<String, LlmClassificationResponse>) -> Result<(), SkillManageError> {
    let path = cache_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try to preserve inserted_at when possible
    let mut existing_entries: HashMap<String, CacheFileEntry> = HashMap::new();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(wrapper) = serde_json::from_str::<CacheFile>(&data) {
                existing_entries = wrapper.entries;
            }
        }
    }

    let mut new_entries: HashMap<String, CacheFileEntry> = HashMap::new();
    let now = chrono::Utc::now().to_rfc3339();
    for (k, v) in map.iter() {
        let mut entry = CacheFileEntry {
            classification: v.clone(),
            provider_metadata: None,
            repo: None,
            skill_path: None,
            name: None,
            description_hash: None,
            inserted_at: Some(now.clone()),
        };

        if let Some(prev) = existing_entries.remove(k) {
            // preserve previous timestamp when possible
            entry.inserted_at = prev.inserted_at.or(entry.inserted_at.clone());
            entry.provider_metadata = prev.provider_metadata.or(entry.provider_metadata);
            entry.repo = prev.repo.or(entry.repo);
            entry.skill_path = prev.skill_path.or(entry.skill_path);
            entry.name = prev.name.or(entry.name);
            entry.description_hash = prev.description_hash.or(entry.description_hash);
        }

        new_entries.insert(k.clone(), entry);
    }

    let wrapper = CacheFile {
        schema_version: 1,
        entries: new_entries,
    };

    let data = serde_json::to_vec_pretty(&wrapper).map_err(|e| SkillManageError::ConfigError(format!("Failed serializing LLM cache: {}", e)))?;

    // Use existing atomic helper
    write_file_atomic(&path, &data)?;
    Ok(())
}
