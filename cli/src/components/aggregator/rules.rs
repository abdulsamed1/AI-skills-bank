use crate::components::aggregator::SkillMetadata;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub struct SubHubRule {
    pub keywords: Vec<&'static str>,
    pub anchor_keywords: Vec<&'static str>,
    pub negative_keywords: Vec<&'static str>,
}

pub struct HubDefinition {
    pub name: &'static str,
    pub sub_hubs: HashMap<&'static str, SubHubRule>,
}

static TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-z0-9]+").unwrap());

pub static VALID_HUBS: &[&str] = &[
    "backend",
    "frontend",
    "ai",
    "testing",
    "mobile",
    "security",
    "devops",
    "business",
    "design",
    "marketing",
    "programming",
];

pub static CSV_COLUMNS: &[&str] = &[
    "hub",
    "sub_hub",
    "skill_id",
    "display_name",
    "description",
    "triggers",
    "match_score",
    "phase",
    "after",
    "before",
    "required",
    "action",
    "output_location",
    "outputs",
];

pub static SUB_HUB_DEFINITIONS: Lazy<HashMap<&'static str, HubDefinition>> = Lazy::new(|| {
    let mut hubs = HashMap::new();

    // 1. Programming Hub
    let mut prog_sub = HashMap::new();
    prog_sub.insert(
        "typescript",
        SubHubRule {
            keywords: vec!["typescript", "tsconfig", "tsx", "type-system", "javascript"],
            anchor_keywords: vec!["typescript", "tsconfig"],
            negative_keywords: vec!["python", "rust", "golang"],
        },
    );
    prog_sub.insert(
        "python",
        SubHubRule {
            keywords: vec!["python", "py", "django", "fastapi", "pandas", "numpy"],
            anchor_keywords: vec!["python", "fastapi"],
            negative_keywords: vec!["typescript", "rust"],
        },
    );
    prog_sub.insert(
        "rust",
        SubHubRule {
            keywords: vec!["rust", "cargo", "ownership", "lifetimes"],
            anchor_keywords: vec!["rust", "cargo"],
            negative_keywords: vec!["python", "typescript"],
        },
    );
    hubs.insert(
        "programming",
        HubDefinition {
            name: "programming",
            sub_hubs: prog_sub,
        },
    );

    // 2. AI Hub
    let mut ai_sub = HashMap::new();
    ai_sub.insert(
        "llm-agents",
        SubHubRule {
            keywords: vec!["llm", "gpt", "prompt", "rag", "agent", "claude"],
            anchor_keywords: vec!["llm", "agent", "rag"],
            negative_keywords: vec!["seo", "marketing"],
        },
    );
    ai_sub.insert(
        "prompting-builder",
        SubHubRule {
            keywords: vec!["prompt", "prompt-engineering", "context-compression"],
            anchor_keywords: vec!["prompt-engineering"],
            negative_keywords: vec!["ui", "css"],
        },
    );
    hubs.insert(
        "ai",
        HubDefinition {
            name: "ai",
            sub_hubs: ai_sub,
        },
    );

    // 3. Frontend Hub
    let mut fe_sub = HashMap::new();
    fe_sub.insert(
        "react-nextjs",
        SubHubRule {
            keywords: vec!["react", "nextjs", "jsx", "hooks", "tailwind"],
            anchor_keywords: vec!["react", "nextjs"],
            negative_keywords: vec!["sql", "postgres"],
        },
    );
    hubs.insert(
        "frontend",
        HubDefinition {
            name: "frontend",
            sub_hubs: fe_sub,
        },
    );

    // 4. Security Hub
    let mut sec_sub = HashMap::new();
    sec_sub.insert(
        "core",
        SubHubRule {
            keywords: vec!["security", "auth", "oauth", "jwt", "encryption", "pentest"],
            anchor_keywords: vec!["security", "pentest", "vulnerability"],
            negative_keywords: vec!["marketing", "seo"],
        },
    );
    hubs.insert(
        "security",
        HubDefinition {
            name: "security",
            sub_hubs: sec_sub,
        },
    );

    // 5. Backend Hub
    let mut be_sub = HashMap::new();
    be_sub.insert(
        "api-design",
        SubHubRule {
            keywords: vec!["api", "rest", "graphql", "openapi", "swagger"],
            anchor_keywords: vec!["api", "rest", "graphql"],
            negative_keywords: vec!["react", "nextjs"],
        },
    );
    be_sub.insert(
        "databases",
        SubHubRule {
            keywords: vec!["sql", "postgres", "mongodb", "redis", "nosql", "orm"],
            anchor_keywords: vec!["sql", "postgres", "database"],
            negative_keywords: vec!["frontend", "ui"],
        },
    );
    hubs.insert(
        "backend",
        HubDefinition {
            name: "backend",
            sub_hubs: be_sub,
        },
    );

    // 6. Testing Hub
    let mut test_sub = HashMap::new();
    test_sub.insert(
        "automation",
        SubHubRule {
            keywords: vec![
                "testing",
                "test",
                "unit-test",
                "integration-test",
                "cypress",
                "playwright",
            ],
            anchor_keywords: vec!["testing", "test", "qa"],
            negative_keywords: vec!["marketing"],
        },
    );
    hubs.insert(
        "testing",
        HubDefinition {
            name: "testing",
            sub_hubs: test_sub,
        },
    );

    // 7. DevOps Hub
    let mut devops_sub = HashMap::new();
    devops_sub.insert(
        "ci-cd",
        SubHubRule {
            keywords: vec!["ci", "cd", "github-actions", "jenkins", "pipeline"],
            anchor_keywords: vec!["ci", "cd", "pipeline"],
            negative_keywords: vec!["ui"],
        },
    );
    devops_sub.insert(
        "containerization",
        SubHubRule {
            keywords: vec!["docker", "kubernetes", "k8s", "container"],
            anchor_keywords: vec!["docker", "kubernetes"],
            negative_keywords: vec!["marketing"],
        },
    );
    hubs.insert(
        "devops",
        HubDefinition {
            name: "devops",
            sub_hubs: devops_sub,
        },
    );

    // 8. Business Hub
    let mut bus_sub = HashMap::new();
    bus_sub.insert(
        "product-strategy",
        SubHubRule {
            keywords: vec!["product", "strategy", "roadmap", "prd", "stakeholder"],
            anchor_keywords: vec!["product", "strategy", "prd"],
            negative_keywords: vec!["react", "nextjs"],
        },
    );
    hubs.insert(
        "business",
        HubDefinition {
            name: "business",
            sub_hubs: bus_sub,
        },
    );

    // 9. Design Hub
    let mut des_sub = HashMap::new();
    des_sub.insert(
        "ui-ux",
        SubHubRule {
            keywords: vec!["ui", "ux", "design", "figma", "wireframe"],
            anchor_keywords: vec!["ui", "ux", "design"],
            negative_keywords: vec!["backend", "sql"],
        },
    );
    hubs.insert(
        "design",
        HubDefinition {
            name: "design",
            sub_hubs: des_sub,
        },
    );

    // 10. Marketing Hub
    let mut mark_sub = HashMap::new();
    mark_sub.insert(
        "strategy",
        SubHubRule {
            keywords: vec!["marketing", "brand", "positioning", "audience"],
            anchor_keywords: vec!["marketing", "strategy"],
            negative_keywords: vec!["python", "rust"],
        },
    );
    hubs.insert(
        "marketing",
        HubDefinition {
            name: "marketing",
            sub_hubs: mark_sub,
        },
    );

    // 11. Mobile Hub
    let mut mob_sub = HashMap::new();
    mob_sub.insert(
        "cross-platform",
        SubHubRule {
            keywords: vec!["react-native", "flutter", "expo", "mobile"],
            anchor_keywords: vec!["mobile", "react-native", "flutter"],
            negative_keywords: vec!["kubernetes"],
        },
    );
    hubs.insert(
        "mobile",
        HubDefinition {
            name: "mobile",
            sub_hubs: mob_sub,
        },
    );

    hubs
});

pub static EXCLUSION_PATTERNS: &[&str] = &[
    "game", "law", "legal", "medicine", "medical", "hospital", "patient", "clinical",
];

pub fn normalize_text(text: &str) -> (String, HashSet<String>) {
    let lower = text.to_lowercase();
    let tokens = TOKEN_REGEX
        .find_iter(&lower)
        .map(|m| m.as_str().to_string())
        .collect();
    (lower, tokens)
}

pub fn get_score_for_subhub(
    normalized_text: &str,
    tokens: &HashSet<String>,
    rule: &SubHubRule,
) -> i32 {
    let mut score = 0;

    for kw in &rule.keywords {
        if tokens.contains(*kw) {
            score += 4;
        } else if normalized_text.contains(kw) {
            score += 2;
        }
    }

    for neg in &rule.negative_keywords {
        if tokens.contains(*neg) || normalized_text.contains(neg) {
            score -= 5;
        }
    }

    if !rule.anchor_keywords.is_empty() {
        let mut anchor_hit = false;
        for anchor in &rule.anchor_keywords {
            if tokens.contains(*anchor) || normalized_text.contains(anchor) {
                anchor_hit = true;
                break;
            }
        }
        if anchor_hit {
            score += 3;
        } else {
            score -= 3;
        }
    }

    score
}

pub fn is_excluded(normalized_text: &str, tokens: &HashSet<String>) -> bool {
    for pattern in EXCLUSION_PATTERNS {
        if tokens.contains(*pattern) || normalized_text.contains(pattern) {
            return true;
        }
    }
    false
}

pub fn generate_triggers(skill_id: &str) -> String {
    let tokens: Vec<String> = TOKEN_REGEX
        .find_iter(&skill_id.to_lowercase())
        .map(|m| m.as_str().to_string())
        .take(5)
        .collect();
    tokens.join(";")
}

pub fn apply_rules(meta: &mut SkillMetadata) -> bool {
    let full_text = format!("{} {}", meta.name, meta.description);
    let (normalized, tokens) = normalize_text(&full_text);

    if is_excluded(&normalized, &tokens) {
        return false;
    }

    let mut best_hub = "productivity";
    let mut best_sub = "workflow-automation";
    let mut best_score = 0;

    for (hub_name, hub_def) in SUB_HUB_DEFINITIONS.iter() {
        for (sub_name, rule) in hub_def.sub_hubs.iter() {
            let score = get_score_for_subhub(&normalized, &tokens, rule);
            if score > best_score {
                best_score = score;
                best_hub = hub_name;
                best_sub = sub_name;
            }
        }
    }

    if best_score >= 4 {
        meta.hub = best_hub.to_string();
        meta.sub_hub = best_sub.to_string();
        meta.match_score = Some(best_score as u32);
    } else {
        meta.hub = "productivity".to_string();
        meta.sub_hub = "workflow-automation".to_string();
        meta.match_score = Some(100);
    }

    if meta.triggers.is_none() || meta.triggers.as_ref().unwrap().is_empty() {
        meta.triggers = Some(generate_triggers(&meta.name));
    }

    meta.phase = Some(match meta.hub.as_str() {
        "programming" => 1,
        "frontend" => 1,
        "backend" => 2,
        "devops" => 3,
        "security" => 4,
        "testing" => 5,
        "ai" => 6,
        _ => 1,
    });

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let (_, tokens) = normalize_text("React-Next.js best practices!");
        assert!(tokens.contains("react"));
        assert!(tokens.contains("next"));
        assert!(tokens.contains("js"));
    }

    #[test]
    fn test_exclusion() {
        let (norm, tokens) = normalize_text("My medical records");
        assert!(is_excluded(&norm, &tokens));
    }

    #[test]
    fn test_trigger_gen() {
        let triggers = generate_triggers("my-awesome-skill");
        assert_eq!(triggers, "my;awesome;skill");
    }
}
