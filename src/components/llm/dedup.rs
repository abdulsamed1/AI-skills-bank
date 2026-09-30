//! Semantic duplicate verdicts (P2).
//!
//! Exact-match dedup in the aggregator misses renamed cross-repo clones.
//! This module adds a bounded, precision-first semantic pass: block by
//! Step-A (hub, sub-hub) plus token overlap, then ask one boolean question
//! per candidate pair. A skill is dropped only on an explicit
//! `duplicate=true` verdict above a high probability bar; uncertainty keeps
//! both (a kept duplicate is visible, a wrongly dropped skill is silent loss).
//!
//! Verdicts are cached in `~/.skills-bank/dedup-verdicts.json` keyed by the
//! pair's skill cache keys, versioned and TTL'd like classification entries.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use home::home_dir;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::components::llm::provider::LlmProvider;
use crate::components::llm::relevance;
use crate::utils::atomicity::write_file_atomic;

use crate::components::aggregator::SkillMetadata;

/// Max candidate pairs per run (deterministic order). Bounds Jev-style
/// verdict spend: pairs/10 calls of ~10 pairs each.
fn max_pairs() -> usize {
    std::env::var("LLM_DEDUP_MAX_PAIRS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(500)
}

/// Token-overlap floor for blocking.
fn overlap_floor() -> f64 {
    std::env::var("LLM_DEDUP_OVERLAP")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 1.0)
        .unwrap_or(0.30)
}

/// `LLM_SEMANTIC_DEDUP=0/false/no/off` disables the pass. Defaults on when
/// the caller runs it (the caller itself is gated on LLM being enabled).
pub fn enabled() -> bool {
    std::env::var("LLM_SEMANTIC_DEDUP")
        .ok()
        .map(|v| {
            let lo = v.to_ascii_lowercase();
            !(lo == "false" || lo == "0" || lo == "no" || lo == "off")
        })
        .unwrap_or(true)
}

fn body_chars() -> usize {
    std::env::var("LLM_MAX_BODY_CHARS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(400)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    s[..end].to_string()
}

/// Deterministic candidate pairs: same Step-A (hub, sub_hub), token Jaccard
/// over name+description at or above the floor, capped at max_pairs.
pub fn candidate_pairs(skills: &[SkillMetadata]) -> Vec<(usize, usize)> {
    let token_sets: Vec<HashSet<String>> = skills
        .iter()
        .map(|s| relevance::tokens(&format!("{} {}", s.name, s.description)))
        .collect();
    let mut pairs = Vec::new();
    for i in 0..skills.len() {
        for j in (i + 1)..skills.len() {
            if skills[i].hub != skills[j].hub || skills[i].sub_hub != skills[j].sub_hub {
                continue;
            }
            if relevance::token_jaccard(&token_sets[i], &token_sets[j]) < overlap_floor() {
                continue;
            }
            pairs.push((i, j));
            if pairs.len() >= max_pairs() {
                return pairs;
            }
        }
    }
    pairs
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VerdictEntry {
    duplicate: bool,
    probability: f64,
    #[serde(default)]
    inserted_at: Option<String>,
}

fn verdict_file_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("LLM_DEDUP_CACHE_PATH") {
        return Some(PathBuf::from(p));
    }
    home_dir().map(|mut h| {
        h.push(".skills-bank");
        let _ = std::fs::create_dir_all(&h);
        h.push("dedup-verdicts.json");
        h
    })
}

fn verdict_key(key_a: &str, key_b: &str, descs: &str) -> String {
    let (lo, hi) = if key_a <= key_b {
        (key_a, key_b)
    } else {
        (key_b, key_a)
    };
    let payload = format!(
        "{}|{}|{}|{}",
        relevance::DEDUP_PROMPT_VERSION,
        lo,
        hi,
        descs
    );
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{:02x}", b);
    }
    hex
}

fn verdict_ttl_days() -> i64 {
    std::env::var("LLM_CACHE_TTL_DAYS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(7)
}

fn verdict_fresh(inserted_at: &Option<String>, ttl_days: i64) -> bool {
    if ttl_days <= 0 {
        return true;
    }
    let ts = match inserted_at.as_deref() {
        Some(s) => s.to_string(),
        None => return false,
    };
    let parsed = chrono::DateTime::parse_from_rfc3339(&ts).ok();
    match parsed {
        None => false,
        Some(t) => (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_days() < ttl_days,
    }
}

fn load_verdicts() -> HashMap<String, VerdictEntry> {
    let path = match verdict_file_path() {
        Some(p) => p,
        None => return HashMap::new(),
    };
    if !path.exists() {
        return HashMap::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str::<HashMap<String, VerdictEntry>>(&d).ok())
        .unwrap_or_default()
}

fn save_verdicts(map: &HashMap<String, VerdictEntry>) {
    let path = match verdict_file_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_vec_pretty(map) {
        let _ = write_file_atomic(&path, &data);
    }
}

/// Run the semantic dedup pass over `skills` (Step-A hubs already assigned).
/// Returns the number of skills removed. Drops are deterministic: on a
/// qualifying verdict the later index is removed and every drop is logged.
pub async fn semantic_dedup_filter(
    repo_root: &Path,
    skills: &mut Vec<SkillMetadata>,
    provider: &dyn LlmProvider,
) -> usize {
    let pairs = candidate_pairs(skills);
    if pairs.is_empty() {
        return 0;
    }
    println!(
        "Semantic dedup: {} candidate pairs (overlap floor {:.2}, cap {})...",
        pairs.len(),
        overlap_floor(),
        max_pairs()
    );

    let max_body = body_chars();
    let threshold = relevance::dedup_threshold();
    let ttl = verdict_ttl_days();
    let mut verdict_cache = load_verdicts();
    let mut cache_dirty = false;

    // Skill cache keys for stable verdict-key derivation.
    let skill_keys: Vec<String> = skills
        .iter()
        .map(|s| {
            crate::components::llm::cache::key_for_skill(
                repo_root,
                &s.path,
                &s.name,
                &s.description,
                s.content_body.as_deref(),
            )
        })
        .collect();

    // Resolve cached verdicts first; batch only the uncached pairs.
    let mut drop_flags = vec![false; skills.len()];
    let mut uncached: Vec<(usize, usize)> = Vec::new();
    for (i, j) in &pairs {
        let descs = format!(
            "{}|{}|{}|{}",
            skills[*i].name, skills[*i].description, skills[*j].name, skills[*j].description
        );
        let key = verdict_key(&skill_keys[*i], &skill_keys[*j], &descs);
        if let Some(entry) = verdict_cache.get(&key) {
            if verdict_fresh(&entry.inserted_at, ttl)
                && entry.duplicate
                && entry.probability >= threshold
            {
                drop_flags[*j] = true;
            }
            continue;
        }
        uncached.push((*i, *j));
    }

    // Batch uncached pairs through the verdict panel (10 per call).
    // NOTE: indices refer to the pre-drop vec; drops apply afterwards.
    for chunk in uncached.chunks(10) {
        let payload: Vec<(String, String, String, String)> = chunk
            .iter()
            .map(|(i, j)| {
                (
                    skills[*i].name.clone(),
                    truncate(&skills[*i].description, max_body),
                    skills[*j].name.clone(),
                    truncate(&skills[*j].description, max_body),
                )
            })
            .collect();
        match provider.classify_duplicates(&payload).await {
            Ok(verdicts) if verdicts.len() == chunk.len() => {
                for (k, (i, j)) in chunk.iter().enumerate() {
                    let descs = format!(
                        "{}|{}|{}|{}",
                        skills[*i].name, skills[*i].description, skills[*j].name,
                        skills[*j].description
                    );
                    let key = verdict_key(&skill_keys[*i], &skill_keys[*j], &descs);
                    match &verdicts[k] {
                        Some((dup, p)) => {
                            verdict_cache.insert(
                                key,
                                VerdictEntry {
                                    duplicate: *dup,
                                    probability: *p,
                                    inserted_at: Some(chrono::Utc::now().to_rfc3339()),
                                },
                            );
                            cache_dirty = true;
                            if *dup && *p >= threshold {
                                drop_flags[*j] = true;
                            }
                        }
                        None => {
                            // Unparseable verdict: keep both. Never merge blind.
                        }
                    }
                }
            }
            Ok(_) => {
                eprintln!("WARN: dedup verdict length mismatch; keeping skills in this chunk.");
            }
            Err(e) => {
                eprintln!("WARN: dedup panel unavailable ({:?}); keeping skills in this chunk.", e);
            }
        }
    }

    if cache_dirty {
        save_verdicts(&verdict_cache);
    }

    let before = skills.len();
    let mut idx = 0;
    skills.retain(|_| {
        let keep = !drop_flags[idx];
        idx += 1;
        keep
    });
    let removed = before - skills.len();
    if removed > 0 {
        println!("Semantic dedup: removed {} near-duplicate skills.", removed);
    } else {
        println!("Semantic dedup: no qualifying duplicates.");
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_key_is_order_stable() {
        let d = "a|b|c|d";
        assert_eq!(verdict_key("k1", "k2", d), verdict_key("k2", "k1", d));
        assert_ne!(verdict_key("k1", "k2", d), verdict_key("k1", "k3", d));
    }
}
