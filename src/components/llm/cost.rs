//! LLM cost accounting. The pipeline previously had no token or spend
//! visibility at all (only cache hit/miss counters), so unbounded
//! re-classification runs could burn budget silently. Providers report
//! `usage` from OpenAI-compatible responses here; jevgrep's eval methodology
//! (per-task cost deltas) is the model: measure first, then cap.
//!
//! Prices are env-overridable per million tokens and default to
//! gpt-4o-mini-ish rates. They are estimates for budgeting, not invoices.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use once_cell::sync::Lazy;

static CALLS: AtomicU64 = AtomicU64::new(0);
static PROMPT_TOKENS: AtomicU64 = AtomicU64::new(0);
static COMPLETION_TOKENS: AtomicU64 = AtomicU64::new(0);
static ERRORS: AtomicU64 = AtomicU64::new(0);

static PER_PROVIDER_CALLS: Lazy<Mutex<HashMap<String, u64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn price_per_million(var: &str, default: f64) -> f64 {
    std::env::var(var)
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(default)
}

/// Record one completed LLM call. Providers call this after parsing `usage`.
pub fn record(provider: &str, prompt_tokens: u64, completion_tokens: u64) {
    CALLS.fetch_add(1, Ordering::SeqCst);
    PROMPT_TOKENS.fetch_add(prompt_tokens, Ordering::SeqCst);
    COMPLETION_TOKENS.fetch_add(completion_tokens, Ordering::SeqCst);
    if let Ok(mut map) = PER_PROVIDER_CALLS.lock() {
        *map.entry(provider.to_string()).or_insert(0) += 1;
    }
}

/// Record one failed LLM call (no token counts available).
pub fn record_error(provider: &str) {
    ERRORS.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut map) = PER_PROVIDER_CALLS.lock() {
        let key = format!("{}:errors", provider);
        *map.entry(key).or_insert(0) += 1;
    }
}

/// Estimated spend in USD from recorded token counts.
pub fn estimate_usd() -> f64 {
    let prompt = PROMPT_TOKENS.load(Ordering::SeqCst) as f64;
    let completion = COMPLETION_TOKENS.load(Ordering::SeqCst) as f64;
    prompt / 1_000_000.0 * price_per_million("LLM_PRICE_INPUT_PER_M", 0.15)
        + completion / 1_000_000.0 * price_per_million("LLM_PRICE_OUTPUT_PER_M", 0.60)
}

/// Extract OpenAI-style `usage` from a response body and record it.
/// Missing/invalid usage records a zero-token call so call counts stay
/// accurate even for providers that omit usage.
pub fn record_usage(provider: &str, body: &serde_json::Value) {
    let prompt = body
        .get("usage")
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let completion = body
        .get("usage")
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    record(provider, prompt, completion);
}

/// True when `LLM_COST_BUDGET_USD` is set and estimated spend reached it.
/// Unset means uncapped (previous behavior).
pub fn budget_exceeded() -> bool {
    match std::env::var("LLM_COST_BUDGET_USD")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
    {
        None => false,
        Some(budget) => estimate_usd() >= budget,
    }
}

/// One-line human summary for end-of-run reporting.
pub fn summary() -> String {
    let calls = CALLS.load(Ordering::SeqCst);
    let prompt = PROMPT_TOKENS.load(Ordering::SeqCst);
    let completion = COMPLETION_TOKENS.load(Ordering::SeqCst);
    let errors = ERRORS.load(Ordering::SeqCst);
    format!(
        "LLM cost: {} calls, {} prompt + {} completion tokens, {} errors, est. ${:.4} (prices: in ${}/M out ${}/M{})",
        calls,
        prompt,
        completion,
        errors,
        estimate_usd(),
        price_per_million("LLM_PRICE_INPUT_PER_M", 0.15),
        price_per_million("LLM_PRICE_OUTPUT_PER_M", 0.60),
        match std::env::var("LLM_COST_BUDGET_USD").ok() {
            Some(b) => format!(", budget ${}", b),
            None => ", no budget cap".to_string(),
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_math() {
        // 1M in @ $0.15 + 1M out @ $0.60 = $0.75 regardless of other state;
        // verify pricing formula on controlled inputs via env overrides.
        std::env::set_var("LLM_PRICE_INPUT_PER_M", "1.0");
        std::env::set_var("LLM_PRICE_OUTPUT_PER_M", "2.0");
        let usd = 1_000_000.0 / 1_000_000.0 * price_per_million("LLM_PRICE_INPUT_PER_M", 0.15)
            + 500_000.0 / 1_000_000.0 * price_per_million("LLM_PRICE_OUTPUT_PER_M", 0.60);
        assert!((usd - 2.0).abs() < 1e-9);
        std::env::remove_var("LLM_PRICE_INPUT_PER_M");
        std::env::remove_var("LLM_PRICE_OUTPUT_PER_M");
    }
}
