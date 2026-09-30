//! Jev-style boolean relevance panels.
//!
//! Design note: jevgrep's accuracy comes from asking a calibrated classifier
//! small yes/no relevance questions (`noul` probability in 0..1) behind strict
//! thresholds, instead of asking a chat model for one argmax JSON object whose
//! `confidence` field is uncalibrated (this codebase's lenient parser even
//! defaults missing confidence to 100). A native TypeSafe `evaluate` transport
//! does not exist in Rust here, so these panels run over the existing
//! OpenAI-compatible chat transport — but they preserve the properties that
//! matter: per-candidate probabilities, strict validation (anything malformed
//! is `None`, never a default), explicit `>` thresholds, and abstention
//! instead of coerced guesses. The prompt version is part of every cache key,
//! mirroring jevgrep's cache identity (endpoint+model+protocol+prompt).

/// Version baked into cache keys. Bump when prompts or decision rules change;
/// old entries then miss once and are re-asked, never silently reused.
pub const RELEVANCE_PROMPT_VERSION: &str = "relevance-panel-1";
/// Version baked into dedup-verdict cache keys.
pub const DEDUP_PROMPT_VERSION: &str = "dedup-verdict-1";
/// Jev-style cutoff: strictly greater wins. Mirrors jevgrep's `> 0.5` gate.
pub const RELEVANCE_THRESHOLD: f64 = 0.5;
/// Duplicate verdicts must clear a high bar: false merges are worse than
/// missed merges (a kept duplicate is visible; a dropped skill is silent loss).
pub const DEDUP_THRESHOLD: f64 = 0.8;

/// Threshold override via env, clamped to (0, 1). Falls back to default.
pub fn relevance_threshold() -> f64 {
    std::env::var("LLM_PANEL_THRESHOLD")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1.0)
        .unwrap_or(RELEVANCE_THRESHOLD)
}

/// Duplicate probability bar via env, clamped to (0, 1).
pub fn dedup_threshold() -> f64 {
    std::env::var("LLM_DEDUP_THRESHOLD")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1.0)
        .unwrap_or(DEDUP_THRESHOLD)
}

/// System prompt for a hub/sub-hub relevance panel. `kind` is "hub" or
/// "sub-hub" and only changes the wording, not the contract.
pub fn build_panel_prompt(candidates: &[String], kind: &str) -> String {
    format!(
        r#"You are a strict relevance judge for a developer skills database, not a routing engine.
For EACH candidate {kind} below, estimate the probability (0.0 to 1.0) that the skill belongs to it, given the skill's name and description.
0.0 = certainly not, 0.5 = genuinely unsure, 1.0 = certainly yes. Calibrate honestly; most candidates should score LOW. Do NOT force a winner — all-low scores are a valid and expected outcome.

Candidates: {cands}

Return ONLY raw JSON, no markdown, no extra text:
{{"scores":[{{"hub":"<exact candidate name>","probability":0.0}}]}}

Rules: score EVERY listed candidate exactly once using its EXACT name; probability must be a finite number in [0,1]."#,
        kind = kind,
        cands = candidates.join(", "),
    )
}

/// Strict parser for panel responses. Returns per-candidate (name, p) in
/// candidate order, or `None` for ANY deviation: missing scores array, wrong
/// length, unknown/duplicate names, non-numeric or out-of-range probability.
/// Never defaults, never coerces — unknown means re-ask or fall back.
pub fn strict_parse_panel(
    v: &serde_json::Value,
    candidates: &[String],
) -> Option<Vec<(String, f64)>> {
    let arr = v.get("scores")?.as_array()?;
    if arr.len() != candidates.len() {
        return None;
    }
    let mut out = Vec::with_capacity(arr.len());
    let mut seen = std::collections::HashSet::new();
    for item in arr {
        let name = item.get("hub").and_then(|h| h.as_str())?;
        if !candidates.iter().any(|c| c == name) || !seen.insert(name.to_string()) {
            return None;
        }
        let p = item.get("probability").and_then(|p| p.as_f64())?;
        if !p.is_finite() || p < 0.0 || p > 1.0 {
            return None;
        }
        out.push((name.to_string(), p));
    }
    Some(out)
}

/// Winner-take-all with abstention: unique max strictly above threshold wins.
/// Exact-threshold hits and ties abstain — a forced guess is worse than none,
/// because downstream coerces guesses into high-confidence facts.
pub fn decide_winner(scores: &[(String, f64)], threshold: f64) -> Option<(String, f64)> {
    let mut best: Option<(&String, f64)> = None;
    let mut tied = false;
    for (name, p) in scores {
        if !p.is_finite() || *p <= threshold {
            continue;
        }
        match best {
            None => best = Some((name, *p)),
            Some((_, bp)) if *p > bp => {
                best = Some((name, *p));
                tied = false;
            }
            Some((_, bp)) if (*p - bp).abs() < f64::EPSILON => {
                tied = true;
            }
            _ => {}
        }
    }
    if tied {
        return None;
    }
    best.map(|(n, p)| (n.clone(), p))
}

/// System prompt for pairwise duplicate verdicts.
pub fn build_dedup_prompt() -> String {
    r#"You are a strict deduplication judge for a developer skills database.
For EACH pair, decide whether skill A and skill B teach/help with the SAME thing (same workflow, same outcome for the user), given names and descriptions. Near-synonyms and cross-repo clones count as duplicates; merely related topics do not.

Return ONLY raw JSON, no markdown, no extra text:
{"verdicts":[{"duplicate":true,"probability":0.0}]}

Rules: one verdict per input pair in input order; duplicate must be a boolean; probability must be a finite number in [0,1] estimating P(they are the same skill). When unsure, answer {"duplicate":false,"probability":0.5} — keeping both is safer than merging."#.to_string()
}

/// Strict parser for dedup verdict batches. `expected` must equal the number
/// of pairs sent. Any deviation yields `None` (keep both, never merge blind).
pub fn strict_parse_dedup(v: &serde_json::Value, expected: usize) -> Option<Vec<(bool, f64)>> {
    let arr = v.get("verdicts")?.as_array()?;
    if arr.len() != expected {
        return None;
    }
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let dup = item.get("duplicate").and_then(|d| d.as_bool())?;
        let p = item.get("probability").and_then(|p| p.as_f64())?;
        if !p.is_finite() || p < 0.0 || p > 1.0 {
            return None;
        }
        out.push((dup, p));
    }
    Some(out)
}

/// Lowercase alphanumeric tokens, used for dedup blocking.
pub fn tokens(s: &str) -> std::collections::HashSet<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() > 1)
        .map(|t| t.to_string())
        .collect()
}

/// Jaccard similarity over token sets. 0.0 when both empty.
pub fn token_jaccard(a: &std::collections::HashSet<String>, b: &std::collections::HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn val(s: &str) -> serde_json::Value {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn panel_accepts_wellformed() {
        let cands = vec!["a".to_string(), "b".to_string()];
        let v = val(r#"{"scores":[{"hub":"a","probability":0.9},{"hub":"b","probability":0.1}]}"#);
        assert_eq!(
            strict_parse_panel(&v, &cands).unwrap(),
            vec![("a".to_string(), 0.9), ("b".to_string(), 0.1)]
        );
    }

    #[test]
    fn panel_rejects_unknown_hub_typo() {
        // The old lenient parser accepted "hubed"/"category" typos; the
        // strict panel must not (a typo'd name is an unknown candidate).
        let cands = vec!["code-quality".to_string()];
        let v = val(r#"{"scores":[{"hub":"code-qualityy","probability":0.9}]}"#);
        assert!(strict_parse_panel(&v, &cands).is_none());
    }

    #[test]
    fn panel_rejects_missing_and_out_of_range() {
        let cands = vec!["a".to_string(), "b".to_string()];
        assert!(strict_parse_panel(&val(r#"{"scores":[{"hub":"a","probability":0.9}]}"#), &cands).is_none());
        assert!(strict_parse_panel(&val(r#"{"scores":[{"hub":"a","probability":1.5},{"hub":"b","probability":0.1}]}"#), &cands).is_none());
        assert!(strict_parse_panel(&val(r#"{"hub":"a"}"#), &cands).is_none());
    }

    #[test]
    fn decide_threshold_is_strict_and_ties_abstain() {
        let scores = vec![("a".to_string(), 0.9), ("b".to_string(), 0.2)];
        assert_eq!(decide_winner(&scores, 0.5).unwrap(), ("a".to_string(), 0.9));
        // Exact 0.5 does NOT clear the gate.
        let edge = vec![("a".to_string(), 0.5)];
        assert!(decide_winner(&edge, 0.5).is_none());
        // Tied maxima abstain rather than pick arbitrarily.
        let tie = vec![("a".to_string(), 0.8), ("b".to_string(), 0.8)];
        assert!(decide_winner(&tie, 0.5).is_none());
        // All below threshold abstains.
        let low = vec![("a".to_string(), 0.4)];
        assert!(decide_winner(&low, 0.5).is_none());
    }

    #[test]
    fn dedup_parse_enforces_length_and_range() {
        let v = val(r#"{"verdicts":[{"duplicate":true,"probability":0.9}]}"#);
        assert_eq!(strict_parse_dedup(&v, 1).unwrap(), vec![(true, 0.9)]);
        assert!(strict_parse_dedup(&v, 2).is_none());
        let bad = val(r#"{"verdicts":[{"duplicate":true,"probability":2.0}]}"#);
        assert!(strict_parse_dedup(&bad, 1).is_none());
        let nonbool = val(r#"{"verdicts":[{"duplicate":"yes","probability":0.9}]}"#);
        assert!(strict_parse_dedup(&nonbool, 1).is_none());
    }

    #[test]
    fn jaccard_behaves() {
        let a = tokens("React component library");
        let b = tokens("react components libraries");
        assert!(token_jaccard(&a, &b) > 0.0);
        assert_eq!(token_jaccard(&a, &a), 1.0);
        assert_eq!(token_jaccard(&tokens("x"), &tokens("yz")), 0.0);
    }
}
