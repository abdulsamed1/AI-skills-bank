use crate::components::aggregator::SkillMetadata;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;

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
    "business",
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

    // Programming Hub
    let mut prog_sub = HashMap::new();
    prog_sub.insert(
        "core-concepts",
        SubHubRule {
            keywords: vec!["programming concepts", "data structures", "algorithms", "complexity"],
            anchor_keywords: vec!["programming concepts", "data structures", "algorithms"],
            negative_keywords: vec!["framework", "library", "tooling"],
        },
    );
    prog_sub.insert(
        "java-script",
        SubHubRule {
            keywords: vec!["javascript", "js", "es6", "node", "npm", "v8"],
            anchor_keywords: vec!["javascript", "js", "es6"],
            negative_keywords: vec!["programming concepts", "data structures"],
        },
    );
    prog_sub.insert(
        "typescript",
        SubHubRule {
            keywords: vec!["typescript", "tsconfig", "tsx", "type-system", "ts"],
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

    // AI Hub
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
            keywords: vec![
                "skill-enhancement",
                "skills-factory",
                "llm-skill",
                "prompt-engineering",
            ],
            anchor_keywords: vec![
                "skill-enhancement",
                "skills-factory",
                "agent-skill",
                "llm-skill",
                "prompt-engineering",
            ],
            negative_keywords: vec!["ui", "security", "waf", "ddos", "injection", "vulnerability"],
        },
    );
    hubs.insert(
        "ai",
        HubDefinition {
            name: "ai",
            sub_hubs: ai_sub,
        },
    );

    // Frontend Hub
    let mut fe_sub = HashMap::new();
    fe_sub.insert(
        "web-frameworks",
        SubHubRule {
            keywords: vec!["vue", "vuex", "nuxt", "vue3","react", "nextjs", "jsx", "hooks", "tailwind", "angular", "svelte", "ember"],
            anchor_keywords: vec!["react", "nextjs", "vue", "nuxt"],
            negative_keywords: vec!["sql", "postgres"],
        },
    );
   fe_sub.insert(
        "ui-ux",
        SubHubRule {
            keywords: vec!["html", "css", "tailwind", "styling", "design-systems", "responsive", "design-system", "component-library", "tokens", "storybook", "html", "css", "tailwind", "styling", "ui-ux", "responsive","ux", "user-experience", "usability","ui", "design", "wireframe", "prototype", "user-interface", "user-experience", "stitch"],
            anchor_keywords: vec!["ui", "ux", "design", "css", "tailwind"],
            negative_keywords: vec!["backend", "sql"],
        },
    );
    fe_sub.insert(
        "state-management",
        SubHubRule {
            keywords: vec!["state", "redux", "context", "zustand", "management", "react-query", "tanstack-query", "mobx", "recoil"],
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


    // Backend Hub
    let mut be_sub = HashMap::new();
    be_sub.insert(
        "server-side-frameworks",
        SubHubRule {
            keywords: vec!["node", "hono", "express", "koa", "hapi", "spring", "django", "flask", "fastapi"],
            anchor_keywords: vec!["express", "spring", "django", "fastapi"],
            negative_keywords: vec!["html", "sql", "postgres"],
        },
    );
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
            keywords: vec!["api", "rest", "graphql", "openapi", "swagger", "endpoint", "api-design", "api-development", "api-best-practices", "api-gateway", "api-security", "api-performance", "api-testing", "api-documentation", "api-versioning", "api-error-handling", "api-authentication", "api-authorization", "api-rate-limiting", "api-caching", "api-monitoring", "api-logging", "api-tracing", "api-observability", "api-security", "api-performance", "api-testing", "api-documentation", "api-versioning", "api-error-handling", "api-authentication", "api-authorization", "api-rate-limiting", "api-caching", "api-monitoring", "api-logging", "api-tracing", "api-observability", "web hook", "websocket"],
            anchor_keywords: vec!["api", "rest", "graphql"],
            negative_keywords: vec!["html", "sql", "postgres"],
        },
    );
    
    be_sub.insert(
        "databases",
        SubHubRule {
            keywords: vec!["sql", "postgres", "mongodb", "redis", "nosql", "orm", "supabase", "mysql", "mariadb", "sqlite", "dynamodb", "firestore", "firebase", "prisma", "drizzle", "typeorm", "sequelize", "knex", "sqlx", "diesel", "sqlc"],
            anchor_keywords: vec!["sql", "postgres", "database"],
            negative_keywords: vec!["frontend", "ui"],
        },
    );
    be_sub.insert(
        "microservices",
        SubHubRule {
            keywords: vec!["microservice", "service-mesh", "istio", "distributed systems", "service discovery"],
            anchor_keywords: vec!["microservice", "architecture"],
            negative_keywords: vec!["ui", "frontend"],
        },
    );
    be_sub.insert(
        "serverless-edge",
        SubHubRule {
            keywords: vec!["cloudflare", "serverless", "cloudflare workers", "edge computing", "lambda", "faas", "serverless architecture", "serverless best practices", "cf", "hoku", "vercel edge"],
            anchor_keywords: vec!["serverless", "cloudflare workers"],
            negative_keywords: vec![
                "ui",
                "frontend",
                "security",
                "waf",
                "ddos",
                "injection",
                "vulnerability",
                "pentest",
                "attack",
            ],
        },
    );
    be_sub.insert(
        "caching",
        SubHubRule {
            keywords: vec!["cache", "redis", "memcached", "caching", "performance", "cache invalidation", "cache strategies", "cache best practices"],
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
        "automation testing",
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
            keywords: vec!["auth", "session", "login", "password", "security", "oauth", "jwt", "encryption", "pentest", "infrastructure", "firewall", "network", "vpn", "waf", "vulnerability", "vulnerabilities", "cve", "scanning", "auditing", "red-team", "ddos", "attack", "zero trust", "zero-trust", "sqli", "sql injection", "threat", "browser isolation", "injection", "exploit", "xss", "csrf", "hardening", "kms", "key management", "cryptography"],
            anchor_keywords: vec!["auth", "oauth", "jwt", "security", "pentest", "vulnerability", "vulnerabilities", "infrastructure", "firewall", "red-team", "waf", "ddos", "zero trust", "injection", "exploit", "xss", "csrf", "encryption", "kms", "key management", "threat"],
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

    
    // Business Hub
    let mut bus_sub = HashMap::new();
    bus_sub.insert(
        "product-strategy",
        SubHubRule {
            keywords: vec![
                "product strategy",
                "go-to-market",
                "go to market",
                "roadmap",
                "prd",
                "business model",
                "value proposition",
                "market analysis",
                "competitive analysis",
                "positioning",
                "stakeholder alignment",
                "pricing strategy",
                "porters five forces",
                "swot",
                "pestel",
                "lean canvas",
                "business case",
            ],
            anchor_keywords: vec![
                "product strategy",
                "go-to-market",
                "roadmap",
                "prd",
                "business model",
                "value proposition",
                "market analysis",
                "positioning",
            ],
            negative_keywords: vec![
                "react",
                "nextjs",
                "api",
                "sdk",
                "python",
                "rust",
                "golang",
                "java",
                "kubernetes",
                "docker",
                "sql",
                "database",
                "forensic",
                "malware",
                "vulnerability",
                "injection",
                "exploit",
                "xss",
                "csrf",
                "encryption",
                "auth",
                "oauth",
                "jwt",
            ],
        },
    );
    bus_sub.insert(
        "product",
        SubHubRule {
            keywords: vec![
                "product management",
                "feature prioritization",
                "requirements",
                "user story",
                "backlog",
                "mvp",
                "product discovery",
                "customer discovery",
                "north star metric",
                "feature roadmap",
                "epic",
                "jobs to be done",
                "jtbd",
                "saas"
            ],
            anchor_keywords: vec![
                "saas",
                "product management",
                "requirements",
                "backlog",
                "product discovery",
                "feature prioritization",
                "jtbd",
            ],
            negative_keywords: vec![
                "sales",
                "api",
                "backend",
                "security",
                "python",
                "rust",
                "kubernetes",
                "sql",
                "forensic",
                "malware",
            ],
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
            keywords: vec![
                "business operations",
                "operational excellence",
                "operational process",
                "process improvement",
                "sop",
                "standard operating procedure",
                "workflow optimization",
                "runbook",
                "incident management",
                "service operations",
                "cost optimization",
                "capacity planning",
                "n8n",
                "zapier",
                "ai automation",
                "productivity automation"
            ],
            anchor_keywords: vec!["operations", "operational", "sop", "process improvement", "runbook"],
            negative_keywords: vec![
                "backend",
                "api",
                "sql",
                "database",
                "kubernetes",
                "python",
                "rust",
                "malware",
                "forensic",
                "vulnerability",
                "injection",
                "test"
            ],
        },
    );
    hubs.insert(
        "business",
        HubDefinition {
            name: "business",
            sub_hubs: bus_sub,
        },
    );


    // Marketing Hub
    let mut mark_sub = HashMap::new();
      mark_sub.insert(
        "content-design",
        SubHubRule {
            keywords: vec!["content design", "microcopy", "video script", "image", "generate image"],
            anchor_keywords: vec!["content design", "copywriting"],
            negative_keywords: vec!["backend", "sql"],
        },
    );
 
    mark_sub.insert(
        "strategy",
        SubHubRule {
            keywords: vec!["marketing strategy", "brand strategy", "positioning", "audience", "go-to-market", "gTM strategy", "campaign strategy"],
            anchor_keywords: vec!["marketing strategy", "brand strategy", "positioning", "go-to-market", "campaign strategy"],
            negative_keywords: vec!["python", "rust", "security", "vulnerability", "injection", "forensic", "malware", "microscopy"],
        },
    );
    mark_sub.insert(
        "content",
        SubHubRule {
            keywords: vec!["content marketing", "copywriting", "blog", "editorial", "content strategy", "thought leadership", "landing page copy"],
            anchor_keywords: vec!["content marketing", "copywriting", "blog", "editorial"],
            negative_keywords: vec!["backend", "database", "security", "vulnerability", "vulnerabilities", "injection", "exploit", "forensic", "malware", "microscopy", "dataset", "sdk", "api", "python", "rust", "java", "kubernetes"],
        },
    );
    mark_sub.insert(
        "email",
        SubHubRule {
            keywords: vec!["email", "newsletter", "email campaign", "email marketing", "mailchimp", "sendgrid"],
            anchor_keywords: vec!["email", "newsletter", "email marketing"],
            negative_keywords: vec!["database", "backend", "security", "vulnerability", "forensic", "malware", "sdk", "api"],
        },
    );
    mark_sub.insert(
        "seo",
        SubHubRule {
            keywords: vec!["seo", "search engine optimization", "keyword research", "technical seo", "on-page seo", "serp", "backlinks"],
            anchor_keywords: vec!["seo", "search engine optimization", "serp", "keyword research"],
            negative_keywords: vec!["python", "backend", "security", "vulnerability", "malware", "forensic"],
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
            keywords: vec!["marketing analytics", "google analytics", "conversion rate", "utm", "attribution", "funnel analytics"],
            anchor_keywords: vec!["marketing analytics", "google analytics", "conversion rate", "utm", "attribution"],
            negative_keywords: vec!["backend", "security", "vulnerability", "malware", "forensic"],
        },
    );
    hubs.insert(
        "marketing",
        HubDefinition {
            name: "marketing",
            sub_hubs: mark_sub,
        },
    );

    // Mobile Hub
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

pub static DEFAULT_EXCLUSION_PATTERNS: &[&str] = &[
    "game",
    "legal",
    "medical",
    "hospital",
    "patient",
    "clinical",
    "games",
    "clothing",
    "food",
    "gym",
    "health",
    "fitness",
    "medicine",
    "law", 
    "sports",
    "entertainment",
    "music",
    "art",
    "history",
    "geography",
    "philosophy",
    "religion",
    "social-sciences",
    "natural-sciences",
    "literature",
    "culture",
    "politics",
    "psychology",
    "sociology",
    "anthropology",
    "archaeology",
    "astronomy",
    "astrology",
    "biology",
    "chemistry",
    "physics"
];

static ENV_EXCLUSION_PATTERNS: Lazy<Vec<String>> = Lazy::new(|| {
    let mut out = Vec::new();

    if let Ok(raw) = env::var("SKILL_MANAGE_EXCLUSIONS") {
        for p in raw.split(';') {
            let val = normalize_slug(p);
            if !val.is_empty() {
                out.push(val);
            }
        }
    }

    if out.is_empty() {
        return DEFAULT_EXCLUSION_PATTERNS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
    }

    out
});

static CANONICAL_SUBHUB_ALIASES: &[(&str, &str, &str)] = &[
    ("llm-agents", "ai", "prompting-factory"),
    ("prompting-factory", "ai", "prompting-factory"),
    ("prompting-builder", "ai", "prompting-factory"),
    ("skills-factory", "ai", "skills-factory"),
    ("data-processing", "ai", "skills-factory"),
    ("ml-training", "ai", "skills-factory"),
    ("api-design", "backend", "api-design"),
    ("server-side-frameworks", "backend", "server-side-frameworks"),
    ("databases", "backend", "databases"),
    ("microservices", "backend", "microservices"),
    ("message-queues", "backend", "message-queues"),
    ("ci-cd", "backend", "ci-cd"),
    ("containerization", "backend", "containerization"),
    ("serverless-edge", "backend", "serverless-edge"),
    ("caching", "backend", "caching"),
    ("product-strategy", "business", "product-strategy"),
    ("product", "business", "product"),
    ("sales", "business", "sales"),
    ("operations", "business", "operations"),
    ("ui-ux", "frontend", "ui-ux"),
    ("ux", "frontend", "ui-ux"),
    ("react-nextjs", "frontend", "web-frameworks"),
    ("web-basics", "frontend", "web-frameworks"),
    ("web-frameworks", "frontend", "web-frameworks"),
    ("state-management", "frontend", "state-management"),
    ("strategy", "marketing", "strategy"),
    ("content", "marketing", "content"),
    ("email", "marketing", "email"),
    ("seo", "marketing", "seo"),
    ("social", "marketing", "social-media"),
    ("social-media", "marketing", "social-media"),
    ("analytics", "marketing", "analytics"),
    ("cross-platform", "mobile", "cross-platform"),
    ("ios", "mobile", "ios"),
    ("android", "mobile", "android"),
    ("typescript", "programming", "typescript"),
    ("python", "programming", "python"),
    ("rust", "programming", "rust"),
    ("golang", "programming", "golang"),
    ("java", "programming", "java"),
    ("automation", "testing", "automation testing"),
    ("automation-testing", "testing", "automation testing"),
    ("unit-testing", "testing", "unit-testing"),
    ("e2e-testing", "testing", "e2e-testing"),
    ("performance-testing", "testing", "performance-testing"),
    ("security", "testing", "security"),
    ("core", "testing", "security"),
];

fn normalize_slug(input: &str) -> String {
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

fn default_subhub_for_hub(hub: &str) -> Option<&'static str> {
    match hub {
        "ai" => Some("prompting-factory"),
        "backend" => Some("api-design"),
        "business" => Some("product-strategy"),
        "frontend" => Some("web-frameworks"),
        "marketing" => Some("strategy"),
        "mobile" => Some("cross-platform"),
        "programming" => Some("typescript"),
        "testing" => Some("automation testing"),
        _ => None,
    }
}

fn canonicalize_assignment(hub: &str, sub_hub: &str) -> Option<(String, String)> {
    let hub_norm = normalize_slug(hub);
    let sub_norm = normalize_slug(sub_hub);

    if hub_norm == "security" {
        return Some(("testing".to_string(), "security".to_string()));
    }

    if !sub_norm.is_empty() {
        for (alias, canon_hub, canon_sub) in CANONICAL_SUBHUB_ALIASES {
            if sub_norm == *alias {
                return Some(((*canon_hub).to_string(), (*canon_sub).to_string()));
            }
        }
    }

    if !hub_norm.is_empty() {
        if let Some(hub_def) = SUB_HUB_DEFINITIONS.get(hub_norm.as_str()) {
            if sub_norm.is_empty() {
                if let Some(default_sub) = default_subhub_for_hub(hub_norm.as_str()) {
                    return Some((hub_norm, default_sub.to_string()));
                }
            }
            if hub_def.sub_hubs.contains_key(sub_norm.as_str()) {
                return Some((hub_norm, sub_norm));
            }
        }
    }

    None
}

fn path_components(meta: &SkillMetadata) -> Vec<String> {
    meta.path
        .components()
        .map(|c| normalize_slug(&c.as_os_str().to_string_lossy()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn infer_from_path(meta: &SkillMetadata) -> Option<(String, String)> {
    let components = path_components(meta);
    if components.is_empty() {
        return None;
    }

    // Antigravity's `skill-*` and `writing-skills` packs are meta AI skill
    // authoring workflows and should be grouped under ai/skills-factory.
    let is_antigravity_repo = components
        .iter()
        .any(|c| c == "antigravity-awesome-skills");
    let is_skill_factory_pack = components
        .iter()
        .any(|c| (c.starts_with("skill-") && c != "skill-md") || c == "writing-skills");
    if is_antigravity_repo && is_skill_factory_pack {
        return Some(("ai".to_string(), "skills-factory".to_string()));
    }

    for (alias, hub, sub_hub) in CANONICAL_SUBHUB_ALIASES {
        if components.iter().any(|c| c == alias) {
            return Some(((*hub).to_string(), (*sub_hub).to_string()));
        }
    }

    for component in components {
        if let Some(default_sub) = default_subhub_for_hub(component.as_str()) {
            return Some((component, default_sub.to_string()));
        }
    }

    None
}

pub fn normalize_text(text: &str) -> (String, HashSet<String>) {
    let lower = text.to_lowercase();
    let tokens = TOKEN_REGEX
        .find_iter(&lower)
        .map(|m| m.as_str().to_string())
        .collect();
    (lower, tokens)
}

fn keyword_matches(normalized_text: &str, tokens: &HashSet<String>, keyword: &str) -> bool {
    let kw = keyword.trim().to_ascii_lowercase();
    if kw.is_empty() {
        return false;
    }

    // Short tokens like `ci`/`cd`/`ui` should only match whole tokens.
    if kw.len() <= 2 {
        return tokens.contains(kw.as_str());
    }

    // Single-word keywords should match full tokens only to avoid substring
    // false positives (e.g. `rust` accidentally matching `trust`).
    if kw.chars().all(|c| c.is_ascii_alphanumeric()) {
        return tokens.contains(kw.as_str());
    }

    tokens.contains(kw.as_str()) || normalized_text.contains(kw.as_str())
}

fn rule_matches(normalized_text: &str, tokens: &HashSet<String>, rule: &SubHubRule) -> bool {
    if rule
        .negative_keywords
        .iter()
        .any(|neg| keyword_matches(normalized_text, tokens, neg))
    {
        return false;
    }

    let keyword_hit = rule
        .keywords
        .iter()
        .any(|kw| keyword_matches(normalized_text, tokens, kw));

    if !keyword_hit {
        return false;
    }

    if rule.anchor_keywords.is_empty() {
        return true;
    }

    rule.anchor_keywords
        .iter()
        .any(|anchor| keyword_matches(normalized_text, tokens, anchor))
}

fn matched_keyword_hits(normalized_text: &str, tokens: &HashSet<String>, rule: &SubHubRule) -> usize {
    rule
        .keywords
        .iter()
        .filter(|kw| keyword_matches(normalized_text, tokens, kw))
        .count()
}

fn hub_match_priority(hub: &str) -> usize {
    match hub {
        // Prefer technical/security hubs first to avoid generic business/marketing matches.
        "testing" => 0,
        "backend" => 1,
        "programming" => 2,
        "frontend" => 3,
        "ai" => 4,
        "mobile" => 5,
        "design" => 6,
        "marketing" => 7,
        "business" => 8,
        _ => 99,
    }
}

fn infer_from_rules_ranked_with_min(
    normalized_text: &str,
    tokens: &HashSet<String>,
    min_rule_score: i32,
    excluded_hubs: &[&str],
) -> Option<(String, String)> {

    let mut best: Option<(i32, usize, usize, String, String)> = None;

    let mut hubs = SUB_HUB_DEFINITIONS.keys().cloned().collect::<Vec<_>>();
    hubs.sort_unstable();

    for hub in hubs {
        if excluded_hubs.iter().any(|excluded| hub == *excluded) {
            continue;
        }

        if let Some(hub_def) = SUB_HUB_DEFINITIONS.get(hub) {
            let mut subs = hub_def.sub_hubs.keys().cloned().collect::<Vec<_>>();
            subs.sort_unstable();

            for sub in subs {
                if let Some(rule) = hub_def.sub_hubs.get(sub) {
                    if !rule_matches(normalized_text, tokens, rule) {
                        continue;
                    }

                    let score = get_score_for_subhub(normalized_text, tokens, rule);
                    if score < min_rule_score {
                        continue;
                    }

                    let hits = matched_keyword_hits(normalized_text, tokens, rule);
                    let priority = hub_match_priority(hub);

                    let is_better = match &best {
                        None => true,
                        Some((best_score, best_hits, best_priority, best_hub, best_sub)) => {
                            score > *best_score
                                || (score == *best_score && hits > *best_hits)
                                || (score == *best_score && hits == *best_hits && priority < *best_priority)
                                || (score == *best_score
                                    && hits == *best_hits
                                    && priority == *best_priority
                                    && (hub < best_hub.as_str()
                                        || (hub == best_hub.as_str() && sub < best_sub.as_str())))
                        }
                    };

                    if is_better {
                        best = Some((
                            score,
                            hits,
                            priority,
                            hub.to_string(),
                            sub.to_string(),
                        ));
                    }
                }
            }
        }
    }

    best.and_then(|(_, _, _, hub, sub)| {
        if let Some((canon_hub, canon_sub)) = canonicalize_assignment(&hub, &sub) {
            return Some((canon_hub, canon_sub));
        }
        Some((hub, sub))
    })
}

fn infer_from_rules_ranked(
    normalized_text: &str,
    tokens: &HashSet<String>,
) -> Option<(String, String)> {
    infer_from_rules_ranked_with_min(normalized_text, tokens, 7, &[])
}

pub fn get_score_for_subhub(
    normalized_text: &str,
    tokens: &HashSet<String>,
    rule: &SubHubRule,
) -> i32 {
    let mut score = 0;

    for kw in &rule.keywords {
        if keyword_matches(normalized_text, tokens, kw) {
            score += 4;
        }
    }

    for neg in &rule.negative_keywords {
        if keyword_matches(normalized_text, tokens, neg) {
            score -= 5;
        }
    }

    if !rule.anchor_keywords.is_empty() {
        let mut anchor_hit = false;
        for anchor in &rule.anchor_keywords {
            if keyword_matches(normalized_text, tokens, anchor) {
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
    for pattern in ENV_EXCLUSION_PATTERNS.iter() {
        if tokens.contains(pattern.as_str()) || normalized_text.contains(pattern) {
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
    // Include `triggers` (which may come from frontmatter `triggers` or `tags`)
    // so that keyword-based rules see tag tokens such as `cloudflare`.
    let full_text = format!(
        "{} {} {} {}",
        meta.name,
        meta.description,
        meta.triggers.clone().unwrap_or_default(),
        meta.path.to_string_lossy()
    );
    let (normalized, tokens) = normalize_text(&full_text);

    if is_excluded(&normalized, &tokens) {
        return false;
    }

    // Force Cloudflare-related skills into backend/serverless-edge only when
    // the Cloudflare signal is explicit in name/tags/path and there is no
    // obvious security context that should route to testing/security.
    let cloudflare_signal_text = format!(
        "{} {} {}",
        meta.name,
        meta.triggers.clone().unwrap_or_default(),
        meta.path.to_string_lossy()
    );
    let (cloudflare_norm, cloudflare_tokens) = normalize_text(&cloudflare_signal_text);
    let found_cloudflare_signal = cloudflare_tokens.contains("cloudflare")
        || cloudflare_norm.contains("cloudflare")
        || cloudflare_norm.contains("cloudflare-workers");

    let security_context = tokens.contains("security")
        || tokens.contains("waf")
        || tokens.contains("ddos")
        || tokens.contains("injection")
        || tokens.contains("vulnerability")
        || tokens.contains("pentest")
        || tokens.contains("attack")
        || normalized.contains("zero trust")
        || normalized.contains("sql injection")
        || normalized.contains("browser isolation");

    if found_cloudflare_signal && !security_context {
        meta.hub = "backend".to_string();
        meta.sub_hub = "serverless-edge".to_string();
        meta.match_score = Some(100);
    } else if let Some((hub, sub_hub)) = canonicalize_assignment(&meta.hub, &meta.sub_hub) {
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        meta.match_score = Some(100);
    } else if let Some((hub, sub_hub)) = infer_from_path(meta) {
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        meta.match_score = Some(95);
    } else if let Some((hub, sub_hub)) = infer_from_rules_ranked(&normalized, &tokens) {
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        meta.match_score = Some(80);
    } else if let Some((hub, sub_hub)) =
        infer_from_rules_ranked_with_min(&normalized, &tokens, 4, &["business", "marketing"])
    {
        // Low-confidence salvage pass: prefer technical domains before falling
        // back to a generic business bucket.
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        meta.match_score = Some(70);
    } else {
        meta.hub = "business".to_string();
        meta.sub_hub = "operations".to_string();
        meta.match_score = Some(1);
    }

    if meta.triggers.is_none() || meta.triggers.as_ref().unwrap().is_empty() {
        meta.triggers = Some(generate_triggers(&meta.name));
    }

    meta.phase = Some(match meta.hub.as_str() {
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
    });

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::components::aggregator::SkillMetadata;

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

    #[test]
    fn test_keyword_matching_avoids_substring_false_positive() {
        let (norm, tokens) = normalize_text("zero trust architecture");
        assert!(!keyword_matches(&norm, &tokens, "rust"));
    }

    #[test]
    fn test_keyword_matching_is_case_insensitive() {
        let (norm, tokens) = normalize_text("Create a gtm strategy for launch");
        assert!(keyword_matches(&norm, &tokens, "gTM strategy"));
    }

    #[test]
    fn test_html_injection_routes_to_testing_security() {
        let mut meta = SkillMetadata {
            name: "html-injection-testing".to_string(),
            description: "Identify and exploit HTML injection vulnerabilities in web applications and test input sanitization.".to_string(),
            path: PathBuf::from("lib/antigravity-awesome-skills/skills/html-injection-testing/SKILL.md"),
            hub: String::new(),
            sub_hub: String::new(),
            triggers: None,
            match_score: None,
            phase: None,
            required: None,
            action: None,
        };

        let kept = apply_rules(&mut meta);
        assert!(kept);
        assert_eq!(meta.hub, "testing");
        assert_eq!(meta.sub_hub, "security");
    }

    #[test]
    fn test_omero_not_forced_to_marketing_content() {
        let mut meta = SkillMetadata {
            name: "omero-integration".to_string(),
            description: "Microscopy platform. Access images via Python and analyze high-content screening datasets.".to_string(),
            path: PathBuf::from("lib/K-Dense-AI-claude-scientific-skills/scientific-skills/omero-integration/SKILL.md"),
            hub: String::new(),
            sub_hub: String::new(),
            triggers: None,
            match_score: None,
            phase: None,
            required: None,
            action: None,
        };

        let kept = apply_rules(&mut meta);
        assert!(kept);
        assert!(!(meta.hub == "marketing" && meta.sub_hub == "content"));
    }

    #[test]
    fn test_product_strategy_skill_stays_in_business() {
        let mut meta = SkillMetadata {
            name: "product-strategy-session".to_string(),
            description: "Run an end-to-end product strategy session across positioning, discovery, and roadmap planning.".to_string(),
            path: PathBuf::from("lib/Product-Manager-Skills/skills/product-strategy-session/SKILL.md"),
            hub: String::new(),
            sub_hub: String::new(),
            triggers: None,
            match_score: None,
            phase: None,
            required: None,
            action: None,
        };

        let kept = apply_rules(&mut meta);
        assert!(kept);
        assert_eq!(meta.hub, "business");
        assert_eq!(meta.sub_hub, "product-strategy");
    }

    #[test]
    fn test_security_strategy_does_not_fall_into_business_product_strategy() {
        let mut meta = SkillMetadata {
            name: "implementing-envelope-encryption-with-aws-kms".to_string(),
            description: "Envelope encryption strategy where data encryption keys are managed via AWS KMS for secure systems.".to_string(),
            path: PathBuf::from("lib/mukul975-Anthropic-Cybersecurity-Skills/skills/implementing-envelope-encryption-with-aws-kms/SKILL.md"),
            hub: String::new(),
            sub_hub: String::new(),
            triggers: None,
            match_score: None,
            phase: None,
            required: None,
            action: None,
        };

        let kept = apply_rules(&mut meta);
        assert!(kept);
        assert!(!(meta.hub == "business" && meta.sub_hub == "product-strategy"));
    }

    #[test]
    fn test_antigravity_skill_pack_routes_to_ai_skills_factory() {
        let mut meta = SkillMetadata {
            name: "skill-installer".to_string(),
            description: "Install and validate new skills in the ecosystem.".to_string(),
            path: PathBuf::from("lib/antigravity-awesome-skills/skills/skill-installer/SKILL.md"),
            hub: String::new(),
            sub_hub: String::new(),
            triggers: None,
            match_score: None,
            phase: None,
            required: None,
            action: None,
        };

        let kept = apply_rules(&mut meta);
        assert!(kept);
        assert_eq!(meta.hub, "ai");
        assert_eq!(meta.sub_hub, "skills-factory");
    }
}
