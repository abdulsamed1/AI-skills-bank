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
    "devops",
    "business",
    "design",
    "marketing",
    "programming",
];

pub static CSV_COLUMNS: &[&str] = &[
    // Minimal critical manifest columns used by downstream tooling.
    "hub",
    "sub_hub",
    "skill_id",
    "description",
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
    prog_sub.insert(
        "golang",
        SubHubRule {
            keywords: vec!["golang", "go", "goroutine", "channels"],
            anchor_keywords: vec!["golang", "go"],
            negative_keywords: vec!["python", "rust"],
        },
    );
    prog_sub.insert(
        "java",
        SubHubRule {
            keywords: vec!["java", "spring", "maven", "jvm"],
            anchor_keywords: vec!["java", "spring"],
            negative_keywords: vec!["python", "rust"],
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
        "prompting-factory",
        SubHubRule {
            keywords: vec!["prompt", "prompt-engineering", "context-compression"],
            anchor_keywords: vec!["prompt-engineering"],
            negative_keywords: vec!["ui", "css"],
        },
    );
    ai_sub.insert(
        "skills-factory",
        SubHubRule {
            keywords: vec!["skill", "skill-enhancement"],
            anchor_keywords: vec!["skill", "factory"],
            negative_keywords: vec!["ui"],
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
        "frameworks",
        SubHubRule {
            keywords: vec!["vue", "vuex", "nuxt", "vue3","react", "nextjs", "jsx", "hooks", "tailwind", "angular", "svelte", "ember"],
            anchor_keywords: vec!["react", "nextjs", "vue", "nuxt"],
            negative_keywords: vec!["sql", "postgres"],
        },
    );
  
    fe_sub.insert(
        "state-management",
        SubHubRule {
            keywords: vec!["state", "redux", "context", "zustand", "management"],
            anchor_keywords: vec!["state", "redux"],
            negative_keywords: vec!["backend"],
        },
    );
    hubs.insert(
        "frontend",
        HubDefinition {
            name: "frontend",
            sub_hubs: fe_sub,
        },
    );


    // 4. Backend Hub
    let mut be_sub = HashMap::new();
    be_sub.insert(
        "ci-cd",
        SubHubRule {
            keywords: vec!["ci", "cd", "github-actions", "jenkins", "pipeline"],
            anchor_keywords: vec!["ci", "cd", "pipeline"],
            negative_keywords: vec!["ui"],
        },
    );
    be_sub.insert(
        "containerization",
        SubHubRule {
            keywords: vec!["docker", "kubernetes", "k8s", "container"],
            anchor_keywords: vec!["docker", "kubernetes"],
            negative_keywords: vec!["marketing"],
        },
    );
    be_sub.insert(
        "api-design",
        SubHubRule {
            keywords: vec!["api", "rest", "graphql", "openapi", "swagger", "endpoint", "api-design", "api-development", "api-best-practices"],
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
    be_sub.insert(
        "microservices",
        SubHubRule {
            keywords: vec!["microservice", "service-mesh", "istio", "architecture", "scaling"],
            anchor_keywords: vec!["microservice", "architecture"],
            negative_keywords: vec!["ui", "frontend"],
        },
    );
    be_sub.insert(
        "serverless-edge",
        SubHubRule {
            keywords: vec!["serverless", "cloudflare workers", "edge computing", "lambda", "faas", "serverless architecture", "serverless best practices", "cloudflare", "hoku", "vercel edge"],
            anchor_keywords: vec!["serverless", "cloudflare workers"],
            negative_keywords: vec!["ui", "frontend"],
        },
    );
    be_sub.insert(
        "caching",
        SubHubRule {
            keywords: vec!["cache", "redis", "memcached", "caching", "performance"],
            anchor_keywords: vec!["cache", "redis"],
            negative_keywords: vec!["frontend", "ui"],
        },
    );
    be_sub.insert(
        "message-queues",
        SubHubRule {
            keywords: vec!["queue", "kafka", "rabbitmq", "messaging", "async"],
            anchor_keywords: vec!["kafka", "rabbitmq"],
            negative_keywords: vec!["ui", "frontend"],
        },
    );
    hubs.insert(
        "backend",
        HubDefinition {
            name: "backend",
            sub_hubs: be_sub,
        },
    );

    // 5. Testing Hub
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
    test_sub.insert(
        "unit-testing",
        SubHubRule {
            keywords: vec!["unit", "jest", "mocha", "pytest", "unittest"],
            anchor_keywords: vec!["unit", "jest"],
            negative_keywords: vec!["integration"],
        },
    );
    test_sub.insert(
        "e2e-testing",
        SubHubRule {
            keywords: vec!["e2e", "end-to-end", "cypress", "playwright", "selenium"],
            anchor_keywords: vec!["e2e", "cypress", "playwright"],
            negative_keywords: vec!["unit"],
        },
    );
    test_sub.insert(
        "performance-testing",
        SubHubRule {
            keywords: vec!["performance", "load-testing", "k6", "jmeter", "benchmark"],
            anchor_keywords: vec!["performance", "load-testing"],
            negative_keywords: vec!["unit"],
        },
    );
    test_sub.insert(
        "security",
        SubHubRule {
            keywords: vec!["auth", "session", "login", "password", "security", "oauth", "jwt", "encryption", "pentest", "infrastructure", "firewall", "network", "vpn", "waf", "vulnerability", "cve", "scanning", "auditing", "red-team"],
            anchor_keywords: vec!["auth", "oauth", "jwt", "security", "pentest", "vulnerability", "infrastructure", "firewall", "red-team"],
            negative_keywords: vec!["marketing", "seo"],
        },
    );
    hubs.insert(
        "testing",
        HubDefinition {
            name: "testing",
            sub_hubs: test_sub,
        },
    );

    
    // 6. Business Hub
    let mut bus_sub = HashMap::new();
    bus_sub.insert(
        "product-strategy",
        SubHubRule {
            keywords: vec!["product", "strategy", "roadmap", "prd", "stakeholder"],
            anchor_keywords: vec!["product", "strategy", "prd"],
            negative_keywords: vec!["react", "nextjs"],
        },
    );
    bus_sub.insert(
        "product",
        SubHubRule {
            keywords: vec!["product", "management", "features", "requirements"],
            anchor_keywords: vec!["product"],
            negative_keywords: vec!["sales", "backend"],
        },
    );
    bus_sub.insert(
        "sales",
        SubHubRule {
            keywords: vec!["sales", "funnel", "closing", "deal", "pitch"],
            anchor_keywords: vec!["sales"],
            negative_keywords: vec!["product"],
        },
    );
    bus_sub.insert(
        "operations",
        SubHubRule {
            keywords: vec!["operations", "process", "efficiency", "optimization"],
            anchor_keywords: vec!["operations"],
            negative_keywords: vec!["backend"],
        },
    );
    hubs.insert(
        "business",
        HubDefinition {
            name: "business",
            sub_hubs: bus_sub,
        },
    );

    // 7. Design Hub
    let mut des_sub = HashMap::new();
    des_sub.insert(
        "ui-ux",
        SubHubRule {
            keywords: vec![ "ui", "ux", "design", "wireframe", "prototype", "user-interface", "user-experience", "stitch"],
            anchor_keywords: vec!["ui", "ux", "design"],
            negative_keywords: vec!["backend", "sql"],
        },
    );
    des_sub.insert(
        "css-styling",
        SubHubRule {
            keywords: vec!["html", "css", "tailwind", "styling", "design-systems", "responsive"],
            anchor_keywords: vec!["css", "tailwind"],
            negative_keywords: vec!["backend", "database"],
        },
    );
    des_sub.insert(
        "ux",
        SubHubRule {
            keywords: vec!["ux", "user-experience", "research", "usability"],
            anchor_keywords: vec!["ux", "user-experience"],
            negative_keywords: vec!["ui", "backend"],
        },
    );
    des_sub.insert(
        "design-systems",
        SubHubRule {
            keywords: vec!["design-system", "component-library", "tokens", "storybook"],
            anchor_keywords: vec!["design-system"],
            negative_keywords: vec!["backend"],
        },
    );
    hubs.insert(
        "design",
        HubDefinition {
            name: "design",
            sub_hubs: des_sub,
        },
    );

    // 8. Marketing Hub
    let mut mark_sub = HashMap::new();
    mark_sub.insert(
        "strategy",
        SubHubRule {
            keywords: vec!["marketing", "brand", "positioning", "audience", "strategy"],
            anchor_keywords: vec!["marketing", "strategy"],
            negative_keywords: vec!["python", "rust"],
        },
    );
    mark_sub.insert(
        "content",
        SubHubRule {
            keywords: vec!["content", "blog", "copywriting", "seo", "article", "documentation"],
            anchor_keywords: vec!["content", "blog"],
            negative_keywords: vec!["backend", "database"],
        },
    );
    mark_sub.insert(
        "email",
        SubHubRule {
            keywords: vec!["email", "newsletter", "campaign", "automation", "mailchimp", "sendgrid"],
            anchor_keywords: vec!["email", "newsletter"],
            negative_keywords: vec!["database", "backend"],
        },
    );
    mark_sub.insert(
        "seo",
        SubHubRule {
            keywords: vec!["seo", "search", "keywords", "optimization", "rankings", "serp"],
            anchor_keywords: vec!["seo", "keywords"],
            negative_keywords: vec!["python", "backend"],
        },
    );
    mark_sub.insert(
        "social-media",
        SubHubRule {
            keywords: vec!["social", "twitter", "facebook", "instagram", "linkedin", "tiktok"],
            anchor_keywords: vec!["social", "twitter"],
            negative_keywords: vec!["backend", "database"],
        },
    );
    mark_sub.insert(
        "analytics",
        SubHubRule {
            keywords: vec!["analytics", "metrics", "tracking", "google-analytics", "conversion"],
            anchor_keywords: vec!["analytics", "metrics"],
            negative_keywords: vec!["backend"],
        },
    );
    hubs.insert(
        "marketing",
        HubDefinition {
            name: "marketing",
            sub_hubs: mark_sub,
        },
    );

    // 9. Mobile Hub
    let mut mob_sub = HashMap::new();
    mob_sub.insert(
        "cross-platform",
        SubHubRule {
            keywords: vec!["react-native", "flutter", "expo", "mobile"],
            anchor_keywords: vec!["mobile", "react-native", "flutter"],
            negative_keywords: vec!["kubernetes"],
        },
    );
    mob_sub.insert(
        "ios",
        SubHubRule {
            keywords: vec!["ios", "swift", "objective-c", "xcode"],
            anchor_keywords: vec!["ios", "swift"],
            negative_keywords: vec!["android"],
        },
    );
    mob_sub.insert(
        "android",
        SubHubRule {
            keywords: vec!["android", "kotlin", "java", "gradle"],
            anchor_keywords: vec!["android", "kotlin"],
            negative_keywords: vec!["ios", "swift"],
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

    let mut best_hub = "business";
    let mut best_sub = "product-strategy";
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
        meta.hub = "business".to_string();
        meta.sub_hub = "product-strategy".to_string();
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
