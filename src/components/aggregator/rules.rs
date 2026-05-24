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


pub static CSV_COLUMNS: &[&str] = &[
    // Minimal critical manifest columns used by downstream tooling.
    "hub",
    "sub_hub",
    "skill_id",
    "description",
    "outputs",
];

/// Valid hub names derived from SUB_HUB_DEFINITIONS.
pub static VALID_HUBS: &[&str] = &[
    "business",
    "code-quality",
    "frontend",
    "server-side",
];

pub static SUB_HUB_DEFINITIONS: Lazy<HashMap<&'static str, HubDefinition>> = Lazy::new(|| {
    let mut hubs = HashMap::new();

    // code-quality Hub
    let mut code_quality = HashMap::new();

   code_quality.insert(
        "testing-qa",
        SubHubRule {
            keywords: vec![
                "tdd-orchestrator",
                "optimize-performance",
                "optimize-code",
                "web-performance-optimization",
                "core-web-vitals",
                "performance-optimization",
                "performance-tuning",
                "performance-metrics",
                "performance-monitoring",
                "performance-analysis",
                "performance-testing",
                "performance-benchmarking",
                "performance-profiling",
                "performance-optimization-with-llm",
                "debugging",
                "debug",
                "troubleshooting",
                "troubleshoot",
                "error-diagnostics-debug",
                "testing",
                "test",
                "tdd",
                "unit-test",
                "integration-test",
                "cypress",
                "playwright","unit", "jest", "mocha", "pytest", "unittest",
                "e2e", "end-to-end", "cypress", "playwright", "selenium",
                "code-review", "ai-review", "review-local-changes", "review-pr", 
                "code-reviewer", "code-simplifier", "clean-code", "refactoring", 
                "bob-craft", "review-standards", "uncle-bob", "solid-principles"
            ],
            anchor_keywords: vec![
                "tdd-orchestrator","testing", "test", "qa",
                "code-review", "review-pr", "clean-code", "refactoring", "bob-craft"
            ],
            negative_keywords: vec!["marketing", "seo", "business"],
        },
    );
    
    code_quality.insert(
        "version-control",
        SubHubRule {
            keywords: vec![ "git-commit","github-pr-creation", "github-pr-merge", "github-pr-review", "git", "CI/CD integration", "code review", "github", "gitlab", "bitbucket", "version-control", "ci-cd", "version-control"],
            anchor_keywords: vec![ "git-commit","github-pr-creation", "github-pr-merge", "github-pr-review", "git", "github", "gitlab", "bitbucket", "version-control", "version-control"],
            negative_keywords: vec!["ui"],
        },
    );
    
    code_quality.insert(
        "security",
        SubHubRule {
            keywords: vec![
                "data breach", "exfiltration", "credentials", "malicious", "html-injection",
                "auth", "session", "login", "password", "security", "oauth", "jwt",
                "encryption", "pentest", "infrastructure", "firewall", "network", "vpn",
                "waf", "vulnerability", "vulnerabilities", "cve", "scanning", "auditing",
                "red-team", "ddos", "attack", "zero trust", "sqli", "sql injection",
                "threat", "browser isolation", "injection", "exploit", "xss", "csrf",
                "hardening", "kms", "key management", "cryptography", "cyber-security",
                "cybersecurity", "cyber attack", "cyber-attack", "prompt injection",
                "ai-security", "rat", "config-extraction", "malware-analysis", "keylogger",
                "credential-theft",
                // NEW: Offensive security / red team
                "penetration test", "red team", "payload", "shellcode",
                "privilege escalation", "lateral movement", "persistence",
                "command and control", "c2", "beacon", "implant",
                "kill chain", "mitre att", "ttp", "adversary",
                // NEW: Defensive security / blue team  
                "incident response", "soc", "siem", "threat hunting",
                "threat detection", "malware analysis", "forensics",
                "indicators of compromise", "ioc", "threat intelligence",
                "ransomware", "phishing", "credential dumping",
                "detection rule", "sigma rule", "yara rule",
                // NEW: Compliance & governance
                "nist", "cis benchmark", "compliance", "audit",
                "cvss",
            ],
            anchor_keywords: vec![
                "auth", "oauth", "jwt", "security", "pentest", "vulnerability", "vulnerabilities",
                "infrastructure", "firewall", "red-team", "waf", "ddos", "zero trust", "injection",
                "exploit", "xss", "csrf", "encryption", "kms", "key management", "threat",
                // NEW: Added anchor keywords
                "penetration test", "red-team", "red team", "adversary", "kill chain",
                "incident response", "soc", "siem", "threat hunting", "malware",
                "forensics", "ransomware", "cve", "cve-",
            ],
            negative_keywords: vec!["marketing", "seo"],
        },
    );
    hubs.insert(
        "code-quality",
        HubDefinition {
            name: "code-quality",
            sub_hubs: code_quality,
        },
    );



    // Frontend Hub
    let mut fe_sub = HashMap::new();
    fe_sub.insert(
        "web-frameworks",
        SubHubRule {
            keywords: vec!["state", "redux", "context", "zustand", "management", "react-query", "tanstack-query", "mobx", "recoil","wasp", "vue", "vuex", "nuxt", "vue3","react", "nextjs", "jsx", "hooks", "tailwind", "angular", "svelte", "ember"],
            anchor_keywords: vec!["react", "nextjs", "vue", "nuxt"],
            negative_keywords: vec!["sql", "postgres"],
        },
    );
   fe_sub.insert(
       "ui-ux",
       SubHubRule {
           keywords: vec![ "figma", "Designers Kit", "designers", "design-md","tailwind", "styling", "design-systems", "responsive", "design-system", "component-library", "tokens", "storybook", "html", "css", "ui-ux","ux", "usability","ui", "wireframe", "prototype", "user-interface", "user-experience", "stitch", "figma", "ui Kit", "color system", "typography", "accessibility", "a11y", "dark mode", "light mode", "design-tokens", "responsive design", "mobile ui", "ios ui", "android ui"],
           anchor_keywords: vec!["mobile ui", "ios ui", "android ui", "ui", "ux", "design", "css", "tailwind"],
           negative_keywords: vec![
               "backend", "sql",
               // Block Architecture/Backend skills from matching on "design"
               "architect", "architecture", "software-architecture", "ddd",
               "domain-driven", "bounded-context", "database-architect",
               "nosql", "dynamodb", "cassandra", "mongodb",
               "temporal", "workflow-orchestration",
               // Block Security/IAM skills from matching on "design"
               "identity-governance", "incident-response", "playbook",                "iga", "iam", "jml", "provisioning", "privileged-access",
                "access-review", "certification", "recertification",
                "cybersecurity", "cyber-security", "envelope-encryption",
                // Block Business skills
                "team-composition", "hiring", "equity-allocation",
                // Block EDA/hardware design
                "design-space-exploration", "dse", "eda", "parameter-tuning",
            ],
        },
    );
    hubs.insert(
        "frontend",
        HubDefinition {
            name: "frontend",
            sub_hubs: fe_sub,
        },
    );


// server-side Hub 
let mut be_sub = HashMap::new();

   
    // prompting-factory — prompt engineering, templates, and LLM orchestration
    be_sub.insert(
        "prompting-factory",
        SubHubRule {
            keywords: vec![
                "system prompt", "agent prompt", "prompt factory", "prompt-factory",
                "skill-enhancement", "skills-factory", "llm-skill", "agent-skill",
                "prompt-engineering", "context-compression", "meta-prompting", "prompt-optimization",
                "prompt-compression", "skill-authoring", "skill-reviewer",
                "skill-creator", "skill-writer", "skill-optimizer", "prompt-patterns",
                "ai-native", "llm-application", "llm", "memory-systems",
                "machine-learning-ops", "ml-pipeline", "ml-ops"
            ],
            anchor_keywords: vec![
                "prompt-engineering", "prompt", "skills-factory",
                "tool-builder", "ml-pipeline", "ml-ops"
            ],
            negative_keywords: vec!["MCP", "deep-research", "article-writing","postgres","n8n", "WhatsApp", "Generate images", "google", "github", "DevOps","jira", "atlassian", "git", "stitch", "tdd-orchestrator", "testing-qa", "ai-seo", "SEO and AEO", "Optimize content","ui", "css", "html", "tailwind", "figma", "code-quality", "code review", "security", "javascript", "typescript", "rust", "golang"],
        },
    );


// architect 
be_sub.insert(
    "architect",
    SubHubRule {
        keywords: vec![ "python-sdk", "sdk",
            "rest", "websocket", "restful", "graphql", "grpc", "trpc",
            "openapi", "swagger", "api spec", "api contract",
            "api gateway", "api versioning", "api documentation",
            "api design", "api best practices", "api standards",
            "endpoint", "rate limiting", "throttling",
            "api authentication", "api authorization",
            "api error handling", "api pagination",
            "webhook", "sse", "server-sent events", "api caching", 
            "architecture", "system design", "ddd", "bounded contexts",
            "adr", "architectural decisions", "software architecture",
            "system architecture", "microservices", "monolith",
            "event-driven architecture",
            "hexagonal architecture", "clean architecture",
            "onion architecture", "modular monolith",
            "domain-driven design", "strategic design",
            "express", "koa", "hapi", "fastify", "hono", "elysia",
            "nestjs", "adonisjs",
            "django", "flask", "fast api", "litestar", "tornado",
            "spring", "spring-boot", "quarkus", "micronaut",
            "rails", "laravel", "phoenix", "gin", "fiber", "echo",
            "server framework", "backend framework", "server",
            "middleware", "routing", "dependency injection"
            
        ],
        anchor_keywords: vec![
            "rest", "graphql", "openapi", "swagger", "api design",
            "api gateway", "webhook", "websocket", "grpc", "trpc",
            "architecture", "system design", "ddd",
            "clean architecture", "microservices", "software architecture",
        ],
        negative_keywords: vec![
            "html", "css", "react"
        ],
    },
);

// 3. DATABASES 
be_sub.insert(
    "databases",
    SubHubRule {
        keywords: vec![
            "sql", "postgres", "postgresql", "mysql", "mariadb",
            "sqlite", "oracle", "mssql",
            "mongodb", "nosql", "dynamodb", "firestore", "firebase",
            "cassandra", "couchdb", "documentdb",
            "orm", "prisma", "drizzle", "typeorm", "sequelize",
            "sqlalchemy", "hibernate", "activerecord",
            "knex", "sqlx", "diesel", "sqlc",
            "database migration", "schema design", "database modeling",
            "transaction", "acid", "indexing", "query optimization",
            "database replication", "sharding", "partitioning",
        ],
        anchor_keywords: vec![
            "sql", "postgres", "mongodb", "orm", "prisma", "drizzle",
            "migration", "schema", "database",
        ],
        negative_keywords: vec![
            "frontend", "ui", "css", "react",
            "redis", "memcached", "cache",  // → caching
            "kafka", "rabbitmq",            // → messaging
        ],
    },
);

// 4. CACHING 
be_sub.insert(
    "caching",
    SubHubRule {
        keywords: vec![
            "redis", "memcached", "valkey",
            "cache", "caching", "cache invalidation",
            "cache strategy", "cache patterns", "cache-aside",
            "write-through", "write-behind", "read-through",
            "ttl", "eviction policy", "lru", "lfu",
            "distributed cache", "cache warming", "cache stampede",
            "cdn cache", "http cache", "etag", "cache-control",
            "browser cache", "service worker cache",
            "in-memory cache", "query cache",
        ],
        anchor_keywords: vec![
            "redis", "memcached", "cache", "caching",
            "cache invalidation", "ttl", "eviction",
        ],
        negative_keywords: vec![
            "frontend", "ui", "sql", "database", "orm",
            "kafka", "rabbitmq", "docker", "kubernetes",
        ],
    },
);

// 5. MESSAGING 
be_sub.insert(
    "messaging",
    SubHubRule {
        keywords: vec![
            "kafka", "rabbitmq", "sqs", "sns", "pub-sub",
            "event-driven", "event sourcing", "cqrs",
            "message queue", "message broker", "message bus",
            "nats", "activemq", "zeromq", "pulsar",
            "event streaming", "stream processing",
            "dead letter queue", "dlq",
            "saga pattern", "outbox pattern",
            "async communication", "background jobs",
            "celery", "bull", "bullmq", "sidekiq", "resque",
        ],
        anchor_keywords: vec![
            "kafka", "rabbitmq", "pub-sub", "event-driven",
            "message queue", "event sourcing", "cqrs",
        ],
        negative_keywords: vec![
            "ui", "frontend", "css", "sql", "database",
            "docker", "kubernetes",  // → containers
        ],
    },
);

// 6. CONTAINERS 
be_sub.insert(
    "containers",
    SubHubRule {
        keywords: vec![
            "docker", "docker-compose", "dockerfile",
            "kubernetes", "k8s", "helm", "kubectl",
            "container", "containerization", "orchestration",
            "service mesh", "istio", "envoy", "linkerd",
            "microservice", "microservices architecture",
            "distributed systems", "service discovery",
            "load balancer", "ingress", "sidecar",
            "pod", "deployment", "statefulset", "configmap",
            "terraform", "ansible", "infrastructure as code",
        ],
        anchor_keywords: vec![
            "docker", "kubernetes", "k8s", "container",
            "microservices", "service mesh", "helm",
        ],
        negative_keywords: vec![
            "ui", "frontend", "css", "serverless",
            "cloudflare workers", "lambda",  // → serverless-edge
        ],
    },
);

// 7. SERVERLESS-EDGE
be_sub.insert(
    "serverless-edge",
    SubHubRule {
        keywords: vec![
            "cloudflare", "cloudflare workers", "cloudflare pages",
            "edge computing", "edge functions",
            "lambda", "aws lambda", "azure functions",
            "google cloud functions", "faas",
            "serverless", "serverless architecture",
            "serverless framework", "sst", "arc",
            "vercel edge", "netlify functions", "deno deploy",
            "wasm", "webassembly at edge",
            "worker", "hono on workers", "itty-router",
        ],
        anchor_keywords: vec![
            "cloudflare", "serverless", "lambda", "edge computing",
            "cloudflare workers", "faas", "edge functions", "netlify",
        ],
        negative_keywords: vec![
            "ui", "frontend", "security", "waf", "ddos",
            "injection", "vulnerability", "pentest",
            "sql", "database", "docker", "kubernetes",
        ],
    },
);

// 8. OBSERVABILITY 
be_sub.insert(
    "observability",
    SubHubRule {
        keywords: vec![
            "logging", "log management", "structured logging",
            "tracing", "distributed tracing", "opentelemetry",
            "metrics", "monitoring", "alerting",
            "prometheus", "grafana", "datadog", "newrelic",
            "sentry", "jaeger", "zipkin", "tempo",
            "apm", "application performance monitoring",
            "error tracking", "uptime monitoring",
            "health check", "sla", "slo", "sli",
            "observability", "three pillars",
            "elk", "elasticsearch", "kibana", "logstash",
            "loki", "fluentd", "splunk",
        ],
        anchor_keywords: vec![
            "prometheus", "grafana", "opentelemetry", "tracing",
            "logging", "monitoring", "apm", "observability",
            "distributed tracing", "metrics",
        ],
        negative_keywords: vec![
            "ui", "frontend", "sql", "database",
            "kafka", "docker", "serverless",
        ],
    },
);

hubs.insert(
    "server-side",
    HubDefinition {
        name: "server-side",
        sub_hubs: be_sub,
    },
);



// ── BUSINESS HUB 
let mut bus_sub = HashMap::new();

// 1. STRATEGY — product, GTM, analytics, business model
bus_sub.insert(
    "business-strategy",
    SubHubRule {
        keywords: vec![
            // Product Management
            "product management", "prd", "roadmap", "feature roadmap",
            "user story", "backlog", "mvp", "product discovery",
            "customer discovery", "north star metric", "epic",
            "jobs to be done", "jtbd", "feature prioritization",
            "requirements", "acceptance criteria", "sprint", "okr",
            "product-strategy", "product strategy",
            "case-study", "risk-assessment",
            // Go-to-Market & Business Model
            "go-to-market", "gtm", "market analysis", "competitive analysis",
            "positioning", "business model", "value proposition",
            "pricing strategy", "beachhead", "tam sam som",
            "porters five forces", "swot", "pestel", "lean canvas",
            "business case", "stakeholder alignment", "growth strategy",
            "market segmentation", "icp", "ideal customer profile",
            "b2b", "b2c", "saas strategy",
            // Analytics & Metrics
            "google analytics", "ga4", "mixpanel", "amplitude",
            "utm", "attribution", "conversion rate", "funnel analytics",
            "cohort analysis", "a/b testing", "split testing",
            "kpi", "metrics", "dashboard", "reporting",
            "data driven", "product analytics", "user analytics",
            "revenue analytics", "ltv", "cac", "arpu", "mrr", "arr",
            "retention rate", "churn rate", "nps", "tracking",
            "measurement", "analytics-tracking", "insights-dashboard",
            // Operations (migrated)
            "sop", "standard operating procedure", "process improvement",
            "workflow optimization", "runbook", "playbook",
            "operational excellence", "business operations", "service operations",
            "cost optimization", "capacity planning", "process documentation",
            "knowledge base", "wiki", "project management", "okr tracking",
            "resource planning", "capacity management", "incident management",
            "change management", "onboarding", "offboarding", "hiring",
            "hr ops", "invoicing", "billing", "accounting", "bookkeeping",
            "budgeting", "financial reporting", "sred", "tax", "odoo", "erp",
            "notion", "airtable", "monday",
        ],
        anchor_keywords: vec![
            "go-to-market", "product strategy", "product-strategy",
            "business model", "value proposition", "competitive analysis",
            "prd", "roadmap", "mvp", "product discovery",
            "kpi", "mrr", "ltv", "cac", "analytics", "metrics", "dashboard",
            "google analytics", "attribution", "conversion rate",
            // operations anchors
            "sop", "process improvement", "runbook", "project management",
            "okr tracking", "odoo", "erp", "workflow optimization", "process documentation",
        ],
        negative_keywords: vec![
            "react", "nextjs", "sdk", "python", "rust",
            "golang", "java", "kubernetes", "docker", "sql",
            "vulnerability", "injection", "auth", "oauth", "jwt",
            "seo", "copywriting", "social media", "brand",
            "cold email", "sales funnel", "lead generation",
        ],
    },
);

// 2. MARKETING — SEO/AEO, content, social media, brand, paid ads
bus_sub.insert(
    "marketing",
    SubHubRule {
        keywords: vec![
            // SEO & AEO
            "seo", "aeo", "ai-seo", "aeo-optimization",
            "search engine optimization", "keyword research",
            "technical seo", "on-page seo", "off-page seo",
            "serp", "backlinks", "link building", "crawlability",
            "indexability", "core-web-vitals", "schema markup",
            "featured snippet", "voice search",
            // Content Marketing
            "content marketing", "copywriting", "blog", "editorial",
            "content strategy", "thought leadership", "landing page copy",
            "content design", "microcopy", "video script",
            "content calendar", "content creation", "ghostwriting",
            "newsletter content", "email copywriting",
            // Social Media & Brand
            "social media", "twitter", "facebook", "instagram",
            "whatsapp", "telegram", "discord", "reddit", "X",
            "linkedin", "tiktok", "youtube", "brand strategy",
            "brand identity", "tone of voice", "messaging",
            "audience", "community management", "influencer",
            // Paid & Campaign
            "campaign strategy", "ad copy", "paid ads", "ppc",
            "google ads", "meta ads", "facebook ads", "marketing strategy",
            "growth marketing", "demand generation", "pr", "press release",
            // Operations & Automation (migrated)
            "n8n", "zapier", "make", "integromat", "automate",
            "ai automation", "workflow automation", "no-code automation",
            "productivity", "automation-ai",
            "email campaign", "email marketing", "newsletter",
            "mailchimp", "sendgrid", "klaviyo", "active campaign",
            "drip campaign", "transactional email", "email automation",
            "notion", "airtable", "monday", "Google Workspace MCP"
        ],
        anchor_keywords: vec![
            "seo", "aeo", "ai-seo", "copywriting", "content marketing",
            "social media", "brand strategy", "campaign strategy",
            "keyword research", "backlinks", "paid ads", "ppc",
            // operations anchors
            "email campaign", "email marketing", "mailchimp", "n8n", "zapier", "automation-ai",
        ],
        negative_keywords: vec![
            "backend", 
            "vulnerability",
            "crm", "sales funnel", "cold email", "lead generation",
            "go-to-market", "product strategy", "business model",
        ],
    },
);

// 3. SALES — lead gen, CRM, funnel, outreach, customer success
bus_sub.insert(
    "sales",
    SubHubRule {
        keywords: vec![
            // Lead Generation & Outreach
            "lead generation", "lead nurturing", "lead scoring",
            "lead qualification", "cold email", "cold outreach",
            "cold calling", "prospecting", "outbound", "inbound",
            "email sequence", "drip sequence", "follow-up",
            "sales engagement", "sales automation", "sales outreach",
            // Sales Process
            "sales funnel", "sales pipeline", "closing", "deal",
            "pitch", "pitch deck", "proposal", "contract",
            "objection handling", "discovery call", "demo",
            "negotiation", "sales strategy", "sales-strategy",
            "b2b sales", "enterprise sales", "smb sales",
            "sales enablement", "sales collateral", "sales playbook",
            // CRM & Account Management
            "crm", "salesforce", "hubspot", "pipedrive", "zoho",
            "account management", "upsell", "cross-sell",
            "customer success", "churn", "retention", "renewal",
            "nrr", "expansion revenue", "customer onboarding",
            "customer support", "helpdesk", "ticketing system",
            // Revenue
            "revenue operations", "revops", "sales ops",
            "quota", "commission", "incentive", "forecast",
            // Operations-related
            "transactional email", "invoicing", "billing", "onboarding", "customer onboarding",
        ],
        anchor_keywords: vec![
            "lead generation", "sales funnel", "cold email", "cold outreach",
            "crm", "closing", "pitch", "sales pipeline",
            "customer success", "b2b sales", "revenue operations",
            "sales strategy", "sales-strategy",
        ],
        negative_keywords: vec![
            "backend", "kubernetes", "python", "docker",
            "vulnerability", "seo", "content marketing", "copywriting",
            "social media", "product strategy", "business model", "go-to-market",
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

    // Mobile Hub decommissioned

    hubs
});

pub static DEFAULT_EXCLUSION_PATTERNS: &[&str] = &[
    "news",
    "legal",
    "medical",
    "hospital",
    "patient",
    "clinical",
    "sexual-health",
    "sexual",
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
    "history",
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
    "physics",
    "blender",
    "threejs",
    "three.js",
    "chinese",
    "japanese",
    "korean",
    "spanish",
    "french",
    "german",
    "italian",
    "portuguese",
    "russian",
    "hindi",
    "bengali",
    "punjabi",
    "telugu",
    "marathi",
    "tamil",
    "urdu",
    "zh-cn",
    "zh-tw",
    "simplified-chinese",
    "resume",
    "cv",
    "cover letter",
    "ats",
    "cv builder",
    "resume",
    "cover-letter",
    "interview",
    "interview-preparation",
    "job-search",
    "hiring",
    "recruitment",
    "talent acquisition",
    "job application",
    "unreal-engine",
    "unity-3d",
    "unity-developer",
    "unity-ecs-patterns",
    "unreal-engine-cpp-pro",
    "game-engine",
];

static ENV_EXCLUSION_PATTERNS: Lazy<Vec<String>> = Lazy::new(|| {
    let mut out = Vec::new();

    // 1. Add all hardcoded default patterns
    for p in DEFAULT_EXCLUSION_PATTERNS {
        out.push(p.to_lowercase());
    }

    // 2. Merge patterns from the environment variable (if any)
    if let Ok(raw) = env::var("SKILL_MANAGE_EXCLUSIONS") {
        for p in raw.split(|c| c == ';' || c == ',') {
            let val = p.trim().to_lowercase();
            if !val.is_empty() && !out.contains(&val) {
                out.push(val);
            }
        }
    }

    out
});

static CANONICAL_SUBHUB_ALIASES: &[(&str, &str, &str)] = &[
    // AI -> Server-side
    ("prompt-engineering", "server-side", "prompting-factory"),
    ("skills-factory",    "server-side", "prompting-factory"),

    // server-side aliases
    ("backend",               "server-side", "architect"), // Generic backend → architect
    ("architect",            "server-side", "architect"),
    ("api-rest-design",       "server-side", "architect"),
    ("server-side-frameworks","server-side", "architect"),
    ("backend-frameworks",    "server-side", "architect"),
    ("databases",             "server-side", "databases"),
    ("caching",               "server-side", "caching"),
    ("message-queues",        "server-side", "messaging"),
    ("event-driven",          "server-side", "messaging"),
    ("containerization",      "server-side", "containers"),
    ("microservices",         "server-side", "containers"),
    ("serverless-edge",       "server-side", "serverless-edge"),
    ("monitoring",            "server-side", "observability"),
    ("logging",               "server-side", "observability"),
    
  // business hub
("product-strategy",      "business", "business-strategy"),
("product-management",    "business", "business-strategy"),
("strategy",              "business", "business-strategy"),
("go-to-market",          "business", "business-strategy"),
("gtm",                   "business", "business-strategy"),
("business-strategy",     "business", "business-strategy"),
("analytics",             "business", "business-strategy"),
("product-analytics",     "business", "business-strategy"),
("case-study",            "business", "business-strategy"),
("risk-assessment",       "business", "business-strategy"),

("marketing",             "business", "marketing"),
("content",               "business", "marketing"),
("seo",                   "business", "marketing"),
("aeo",                   "business", "marketing"),
("ai-seo",                "business", "marketing"),
("social-media",          "business", "marketing"),
("brand",                 "business", "marketing"),
("copywriting",           "business", "marketing"),
("campaign",              "business", "marketing"),

("sales",                 "business", "sales"),
("lead-generation",       "business", "sales"),
("crm",                   "business", "sales"),
("cold-email",            "business", "sales"),
("customer-success",      "business", "sales"),
("revenue-operations",    "business", "sales"),
("revops",                "business", "sales"),

("operations",            "business", "business-strategy"),
("business-operations",   "business", "business-strategy"),
("sop",                   "business", "business-strategy"),
("automation-ai",         "business", "marketing"),
("email-campaign",        "business", "marketing"),
("email-marketing",       "business", "marketing"),
("project-management",    "business", "business-strategy"),
("workflow-optimization", "business", "business-strategy"),
("tactical",              "business", "business-strategy"),
("email",                 "business", "marketing"),

    // frontend hub
    ("ui-ux",             "frontend", "ui-ux"),
    ("ux",                "frontend", "ui-ux"),
    ("react-nextjs",      "frontend", "web-frameworks"),
    ("web-basics",        "frontend", "web-frameworks"),
    ("web-frameworks",    "frontend", "web-frameworks"),
    ("state-management",  "frontend", "web-frameworks"),
    ("css",               "frontend", "ui-ux"),
    ("tailwind",          "frontend", "ui-ux"),

    // mobile hub (mapped to frontend/web-frameworks)
    ("mobile",            "frontend", "web-frameworks"),
    ("cross-platform",    "frontend", "web-frameworks"),
    ("ios",               "frontend", "web-frameworks"),
    ("android",           "frontend", "web-frameworks"),

    // code-quality hub
    ("security",          "code-quality", "security"),
    ("testing",           "code-quality", "testing-qa"),
    ("testing-qa",        "code-quality", "testing-qa"),
    ("automation-testing","code-quality", "testing-qa"),
    ("unit-testing",      "code-quality", "testing-qa"),
    ("e2e-testing",       "code-quality", "testing-qa"),
    ("code-review",       "code-quality", "testing-qa"),
    ("ci-cd",             "code-quality", "version-control"),
    ("performance-code",      "code-quality", "code-quality"),
    ("git",               "code-quality", "version-control"),
    ("github",            "code-quality", "version-control"),
    ("gitlab",            "code-quality", "version-control"),
    ("bitbucket",         "code-quality", "version-control"),
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
        "server-side" => Some("architect"),  // الافتراضي الأوسع انتشاراً
        "business" => Some("business-strategy"),  // الافتراضي الأوسع
        "frontend" => Some("web-frameworks"),
        "code-quality" => Some("testing-qa"),
        _ => None,
    }
}

fn canonicalize_assignment(hub: &str, sub_hub: &str) -> Option<(String, String)> {
    let hub_norm = normalize_slug(hub);
    let sub_norm = normalize_slug(sub_hub);

    // 1. Check for explicit aliases (Sub-hub is the primary key for aliases)
    if !sub_norm.is_empty() {
        for (alias, canon_hub, canon_sub) in CANONICAL_SUBHUB_ALIASES {
            if sub_norm == *alias {
                return Some(((*canon_hub).to_string(), (*canon_sub).to_string()));
            }
        }
    }

    // 2. Check if the hub its self is a known sub-hub alias (e.g. "ios" given as hub)
    if !hub_norm.is_empty() {
        for (alias, canon_hub, canon_sub) in CANONICAL_SUBHUB_ALIASES {
            if hub_norm == *alias {
                return Some(((*canon_hub).to_string(), (*canon_sub).to_string()));
            }
        }
    }

    // 3. Fallback to definitions with default sub-hub
    if !hub_norm.is_empty() {
        if let Some(hub_def) = SUB_HUB_DEFINITIONS.get(hub_norm.as_str()) {
            if sub_norm.is_empty() || !hub_def.sub_hubs.contains_key(sub_norm.as_str()) {
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
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .filter(|s| !s.starts_with('.') && !s.ends_with(".git") && s != "git")
        .map(|s| normalize_slug(&s))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Conflict resolution table: when a skill matches multiple sub-hubs simultaneously,
/// this defines which sub-hub should take precedence.
/// Format: (losing_hub, losing_sub_hub, winning_hub, winning_sub_hub)
static CONFLICT_RESOLUTION: &[(&str, &str, &str, &str)] = &[
    // Domain specialists always win over language hubs
    ("code-quality", "python", "code-quality", "security"),
    ("code-quality", "javascript", "code-quality", "security"),
    ("code-quality", "typescript", "code-quality", "security"),
    ("code-quality", "rust", "code-quality", "security"),
    ("code-quality", "golang", "code-quality", "security"),
    ("code-quality", "java", "code-quality", "security"),
    
    ("code-quality", "python", "code-quality", "testing-qa"),
    ("code-quality", "javascript", "code-quality", "testing-qa"),
    ("code-quality", "typescript", "code-quality", "testing-qa"),
    ("code-quality", "rust", "code-quality", "testing-qa"),
    
    // Sales overrides marketing and business strategy
    ("business", "marketing", "business", "sales"),
    ("business", "business-strategy", "business", "sales"),
];

/// Extracts the repository directory name from a skill path (the segment
/// immediately following `lib/` or `src/`) and maps known domain signals
/// to a hub/sub-hub pair. This is intentionally a substring check so that
/// repo names like "mukul975-anthropic-cybersecurity-skills" are matched.
fn infer_hub_from_repo_name(path: &std::path::Path) -> Option<(String, String)> {
    let components: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();

    // Find the repo name: the segment right after `lib`, `src`, or `skills`
    let repo_name = components
        .windows(2)
        .find_map(|w| {
            if w[0] == "lib" || w[0] == "src" || w[0] == "skills" {
                Some(w[1].clone())
            } else {
                None
            }
        })?;

    // Specialized Azure/Microsoft SDK classification
    let path_str = path.to_string_lossy().to_lowercase();
    let skill_name = components
        .windows(2)
        .find_map(|w| {
            if w[1] == "skill.md" || w[1] == "readme.md" {
                Some(w[0].clone())
            } else {
                None
            }
        })
        .unwrap_or_default();

    if path_str.contains("microsoft-skills") || path_str.contains("azure-sdk") || skill_name.starts_with("azure-") {
        if skill_name.contains("cosmos") || skill_name.contains("database") || skill_name.contains("sql") || skill_name.contains("storage") {
            return Some(("server-side".to_string(), "databases".to_string()));
        }
        if skill_name.contains("security") || skill_name.contains("keyvault") || skill_name.contains("vault") || skill_name.contains("identity") || skill_name.contains("auth") {
            return Some(("code-quality".to_string(), "security".to_string()));
        }
        if skill_name.contains("ai") || skill_name.contains("openai") || skill_name.contains("cognitive") || skill_name.contains("agent") || skill_name.contains("search") {
            return Some(("server-side".to_string(), "agents".to_string()));
        }
        // Fallback for Azure SDKs
        return Some(("server-side".to_string(), "serverless-edge".to_string()));
    }

    // Specialized repository path mappings for GSAP, DuckDB, Netlify, and Odoo front-end views
    if path_str.contains("greensock-gsap") || path_str.contains("gsap") || skill_name.contains("gsap") {
        return Some(("frontend".to_string(), "ui-ux".to_string()));
    }
    if path_str.contains("duckdb") || skill_name.contains("duckdb") {
        return Some(("server-side".to_string(), "databases".to_string()));
    }
    if path_str.contains("netlify") || skill_name.contains("netlify") {
        if skill_name.contains("database") || skill_name.contains("db") || skill_name.contains("sql") {
            return Some(("server-side".to_string(), "databases".to_string()));
        }
        return Some(("server-side".to_string(), "serverless-edge".to_string()));
    }
    if skill_name.contains("odoo-qweb") || skill_name.contains("odoo-xml-views") { 
        return Some(("frontend".to_string(), "ui-ux".to_string()));
    }

    // Specialized platform mappings for React Native, Expo, Flutter, iOS, and Android
    if skill_name.contains("react-native") || skill_name.contains("expo-") || skill_name.contains("flutter") {
        return Some(("frontend".to_string(), "web-frameworks".to_string()));
    }
    if skill_name.contains("android-") || skill_name.contains("android_") {
        return Some(("frontend".to_string(), "web-frameworks".to_string()));
    }
    if skill_name.contains("ios-") || skill_name.contains("swiftui-") {
        if !skill_name.contains("security") && !skill_name.contains("reverse-engineering") {
            return Some(("frontend".to_string(), "web-frameworks".to_string()));
        }
    }
    
    // Specialized sales mappings for cold email, sales enablement, RevOps, and CRM Salesforce
    if skill_name.contains("cold-email") 
        || skill_name.contains("sales-enablement") 
        || skill_name.contains("revops") 
        || skill_name.contains("battlecard") 
        || skill_name.contains("salesforce") 
    {
        return Some(("business".to_string(), "sales".to_string()));
    }

    // Security domain — must be checked BEFORE generic agents or language hubs
    if repo_name.contains("security")
        || repo_name.contains("cybersecurity")
        || repo_name.contains("pentest")
        || repo_name.contains("vulnerability")
        || repo_name.contains("vibesec")
        || repo_name.contains("bluebook")
    {
        return Some(("code-quality".to_string(), "security".to_string()));
    }

    // BMAD / WDS / CIS Framework skills (local agent skills)
    if repo_name.starts_with("bmad-testarch") || repo_name.contains("qa") {
        return Some(("code-quality".to_string(), "testing-qa".to_string()));
    }
    if repo_name.starts_with("bmad-agent-dev") || repo_name.starts_with("bmad-quick-dev") || repo_name.starts_with("bmad-code-review") {
        return Some(("code-quality".to_string(), "testing-qa".to_string()));
    }
    if repo_name.starts_with("wds-") || repo_name.contains("ux") || repo_name.contains("design") {
        return Some(("frontend".to_string(), "ui-ux".to_string()));
    }
    if repo_name.starts_with("bmad-cis") || repo_name.starts_with("bmad-agent-pm") || repo_name.contains("business") || repo_name.contains("strategy") {
        return Some(("business".to_string(), "business-strategy".to_string()));
    }
    if repo_name.starts_with("bmad-") {
        return Some(("business".to_string(), "operations".to_string())); // Default for other generic bmad workflow skills
    }
    
    if repo_name.contains("test") || repo_name.contains("playwright") || repo_name.contains("cypress") || repo_name.contains("jest") {
        return Some(("code-quality".to_string(), "testing-qa".to_string()));
    }
    
    // Generic provider/curated repositories (like a clean "openai", "anthropic" or "gemini" folder)
    // should NOT blindly hijack all generic engineering/marketing/sales/design skills into ai/agents.
    let is_generic_provider_repo = repo_name == "openai" 
        || repo_name == "anthropic" 
        || repo_name == "gemini" 
        || repo_name == "microsoft-skills";

    // AI / prompt-engineering domain (high precedence)
    if repo_name.contains("prompt")
        || repo_name.contains("agent-skill")
        || repo_name.contains("ai-skills")
        || repo_name.contains("skill-authoring")
        || repo_name.contains("skill-creation")
        || repo_name.contains("skills-factory")
        || repo_name.contains("llm")
    {
        return Some(("server-side".to_string(), "prompting-factory".to_string()));
    }

    if !is_generic_provider_repo && (repo_name.contains("agent") 
        || repo_name.contains("toolkit") 
        || repo_name.contains("bot") 
        || repo_name.contains("gemini") 
        || repo_name.contains("openai") 
        || repo_name.contains("anthropic"))
    {
        return Some(("server-side".to_string(), "agents".to_string()));
    }

    if repo_name.contains("tool") || repo_name.contains("util") || repo_name.contains("script") {
        return Some(("business".to_string(), "operations".to_string()));
    }

    // Mobile — iOS
    if repo_name.contains("swiftui")
        || repo_name.contains("ios-")
        || repo_name.contains("-ios")
        || repo_name.contains("swift-patterns")
        || repo_name.contains("apple-hig")
        || repo_name.contains("app-store")
    {
        return Some(("frontend".to_string(), "web-frameworks".to_string()));
    }

    // Mobile — Android
    if repo_name.contains("android") || repo_name.contains("kotlin") {
        return Some(("frontend".to_string(), "web-frameworks".to_string()));
    }

    // Frontend / UI
    if repo_name.contains("ui-ux") || repo_name.contains("ui-skills") {
        return Some(("frontend".to_string(), "ui-ux".to_string()));
    }

    // Testing / QA
    if repo_name.contains("playwright") || repo_name.contains("testdino") || repo_name.contains("testing") || repo_name.contains("jest") || repo_name.contains("cypress") {
        return Some(("code-quality".to_string(), "testing-qa".to_string()));
    }

    // Cloudflare / Serverless
    if repo_name.contains("cloudflare") || repo_name.contains("workers-") || repo_name.contains("serverless") {
        return Some(("server-side".to_string(), "serverless-edge".to_string()));
    }

    // Docker / Kubernetes / Containers
    if repo_name.contains("docker") || repo_name.contains("kubernetes") || repo_name.contains("k8s") || repo_name.contains("helm") {
        return Some(("server-side".to_string(), "containers".to_string()));
    }

    // Python
    if repo_name.contains("python") || repo_name.contains("django") || repo_name.contains("flask") || repo_name.contains("fastapi") {
        return Some(("code-quality".to_string(), "python".to_string()));
    }

    // Java / Spring
    if repo_name.contains("java") && !repo_name.contains("javascript") || repo_name.contains("spring-boot") {
        return Some(("code-quality".to_string(), "java".to_string()));
    }

    // React / Frontend
    if repo_name.contains("react") || repo_name.contains("nextjs") || repo_name.contains("vue") || repo_name.contains("angular") {
        return Some(("frontend".to_string(), "web-frameworks".to_string()));
    }

    // SEO / Marketing / Ads
    if repo_name.contains("seo") || repo_name.contains("marketing") || repo_name.contains("copywriting") || repo_name.contains("ads") {
        return Some(("business".to_string(), "marketing".to_string()));
    }

    // Database
    if repo_name.contains("postgres") || repo_name.contains("mysql") || repo_name.contains("mongodb") || repo_name.contains("database") {
        return Some(("server-side".to_string(), "databases".to_string()));
    }

    // Code review
    if repo_name.contains("code-review") || repo_name.contains("clean-code") || repo_name.contains("refactor") {
        return Some(("code-quality".to_string(), "testing-qa".to_string()));
    }

    // Git / Version controlversion-control
    if repo_name.contains("github") || repo_name.contains("gitlab") || repo_name.contains("ci-cd") {
        return Some(("code-quality".to_string(), "version-control".to_string()));
    }

    None
}

fn infer_from_path(meta: &SkillMetadata) -> Option<(String, String)> {
    let components = path_components(meta);
    if components.is_empty() {
        return None;
    }

    // Guard: pm-go-to-market paths are business/strategy, not golang
    let is_gtm_path = components.iter().any(|c| c.contains("go-to-market"));
    if is_gtm_path {
        return Some(("business".to_string(), "strategy".to_string()));
    }

    // Antigravity's `skill-*` and `writing-skills` packs are meta AI skill
    // authoring workflows and should be grouped under server-side/prompting-factory.
    let is_antigravity_repo = components
        .iter()
        .any(|c| c == "antigravity-awesome-skills");
    let is_skill_factory_pack = components
        .iter()
        .any(|c| (c.starts_with("skill-") && c != "skill-md") || c == "writing-skills");
    if is_antigravity_repo && is_skill_factory_pack {
        return Some(("server-side".to_string(), "prompting-factory".to_string()));
    }

    // ── repo-name substring signal (highest confidence) ──
    if let Some(result) = infer_hub_from_repo_name(&meta.path) {
        return Some(result);
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

pub fn keyword_matches(normalized_text: &str, tokens: &HashSet<String>, keyword: &str) -> bool {
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

pub fn rule_matches(normalized_text: &str, tokens: &HashSet<String>, rule: &SubHubRule) -> bool {
    let is_netlify_or_cloudflare = tokens.contains("netlify") || tokens.contains("cloudflare") || normalized_text.contains("netlify") || normalized_text.contains("cloudflare");

    if rule
        .negative_keywords
        .iter()
        .any(|neg| {
            if is_netlify_or_cloudflare && (*neg == "sql" || *neg == "database" || *neg == "ui" || *neg == "frontend" || *neg == "security") {
                return false;
            }
            keyword_matches(normalized_text, tokens, neg)
        })
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

pub fn matched_keyword_hits(normalized_text: &str, tokens: &HashSet<String>, rule: &SubHubRule) -> usize {
    rule
        .keywords
        .iter()
        .filter(|kw| keyword_matches(normalized_text, tokens, kw))
        .count()
}

pub fn hub_match_priority(hub: &str) -> usize {
    match hub {
        // Prefer technical hubs first to avoid generic business matches.
        "code-quality" => 0,
        "server-side" => 1,
        "frontend" => 2,
        "business" => 3,
        _ => 99,
    }
}

/// When a skill matches multiple sub-hubs simultaneously, this resolves
/// the conflict by defining which sub-hub should take precedence.
fn resolve_conflict<'a>(
    _normalized_text: &str,
    _tokens: &HashSet<String>,
    candidates: &[(&'a str, &'a str)],
) -> Option<(String, String)> {
    if candidates.len() <= 1 {
        return candidates.first().map(|(h, s)| (h.to_string(), s.to_string()));
    }

    for (lose_hub, lose_sub, win_hub, win_sub) in CONFLICT_RESOLUTION {
        let has_loser = candidates.iter().any(|(h, s)| h == lose_hub && s == lose_sub);
        let has_winner = candidates.iter().any(|(h, s)| h == win_hub && s == win_sub);
        if has_loser && has_winner {
            return Some((win_hub.to_string(), win_sub.to_string()));
        }
    }

    // Fallback: return the one with highest hub priority
    candidates
        .iter()
        .min_by_key(|(h, _)| hub_match_priority(h))
        .map(|(h, s)| (h.to_string(), s.to_string()))
}

pub fn infer_from_rules_ranked_with_min(
    normalized_text: &str,
    tokens: &HashSet<String>,
    min_rule_score: i32,
    excluded_hubs: &[&str],
) -> Option<(String, String)> {

    let mut all_matches: Vec<(i32, usize, usize, String, String)> = Vec::new();

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
                    all_matches.push((score, hits, priority, hub.to_string(), sub.to_string()));
                }
            }
        }
    }

    if all_matches.is_empty() {
        return None;
    }

    // Sort by score (desc), hits (desc), priority (asc)
    all_matches.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.2.cmp(&b.2))
    });

    // Apply conflict resolution when multiple sub-hubs match the top score
    let top_score = all_matches[0].0;
    let candidates: Vec<(&str, &str)> = all_matches
        .iter()
        .filter(|(s, ..)| *s >= top_score - 5) // include near-ties
        .map(|(_, _, _, h, s)| (h.as_str(), s.as_str()))
        .collect();

    let resolved = resolve_conflict(normalized_text, tokens, &candidates)?;

    if let Some((canon_hub, canon_sub)) = canonicalize_assignment(&resolved.0, &resolved.1) {
        return Some((canon_hub, canon_sub));
    }

    Some(resolved)
}

pub fn infer_from_rules_ranked(
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
            score += 5;
        } else {
            score -= 15; // Harder penalty for missing anchors in specific technical hubs
        }
    }

    score
}

pub fn is_excluded(normalized_text: &str, tokens: &HashSet<String>) -> bool {
    /*
        Exclusion Matcher Branching
        --------------------------
        1. Substring: pattern.contains(' ') -> check normalized_text.contains(pattern)
        2. Strict: length < 4 -> check tokens.contains(pattern)
        3. Mixed: default -> check normalized_text.contains(pattern) || tokens.contains(pattern)
    */

    // 1. Keyword-based exclusions (Integrated Defaults + Config)
    for pattern in ENV_EXCLUSION_PATTERNS.iter() {
        let mut matched = false;
        // Multi-word patterns (e.g. "Cover Letter", "Unity 3D") must match as substrings
        if pattern.contains(' ') || pattern.contains('.') {
            if normalized_text.contains(pattern) {
                matched = true;
            }
        } else if pattern.len() < 4 {
            // Short patterns (e.g. "CV", "ATS") must match as exact tokens to avoid substrings
            // like "ats" matching "stats" or "cats".
            if tokens.contains(pattern) {
                matched = true;
            }
        } else {
            // Default case: match anywhere
            if tokens.contains(pattern) || normalized_text.contains(pattern) {
                matched = true;
            }
        }

        if matched {
            // Only print for known "problem" skills to avoid flood
            if normalized_text.contains("resume") || normalized_text.contains("game") {
                if std::env::var("SKILL_MANAGE_DEBUG").is_ok() {
                    println!("DEBUG: Excluding skill because it matches pattern: '{}' (text start: {:.50})", pattern, normalized_text);
                }
            }
            return true;
        }
    }

    // 2. Strict Language/Script filtering (Allowlist approach)
    // We strictly allow ONLY Basic Latin (English), Arabic characters, and common punctuation/numbers.
    // This blocks CJK, Cyrillic, Spanish (accented), French (accented), etc.
    for c in normalized_text.chars() {
        // Allow whitespace, digits, and standard ASCII punctuation
        if c.is_ascii_whitespace() || c.is_ascii_punctuation() || c.is_ascii_digit() {
            continue;
        }

        let u = c as u32;

        // Basic Latin (A-Z, a-z)
        if (u >= 0x0041 && u <= 0x005A) || (u >= 0x0061 && u <= 0x007A) {
            continue;
        }
        
        // Latin-1 Supplement and Latin Extended (Accents, special Latin letters)
        if u >= 0x00A0 && u <= 0x024F {
            continue;
        }

        // General Punctuation, Currency, Letterlike Symbols, Number Forms, Arrows, Math Operators, Misc Technical
        if u >= 0x2000 && u <= 0x2BFF {
            continue;
        }
        
        // Emojis and Misc Symbols and Pictographs
        if (u >= 0x2600 && u <= 0x27BF) || (u >= 0x1F000 && u <= 0x1FAFF) {
            continue;
        }

        // Arabic block (\u0600-\u06FF)
        if u >= 0x0600 && u <= 0x06FF {
            continue;
        }

        // Arabic Supplement and Extended blocks
        if (u >= 0x0750 && u <= 0x077F) || (u >= 0x08A0 && u <= 0x08FF) {
            continue;
        }

        // If the character is NOT in any of the above allowed ranges, it's an UNWANTED script/language.
        return true;
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

    // Exclusions are now handled exclusively by `native_pipeline.rs` 
    // to allow for hybrid LLM exclusion + tracking before final drop.

    // Force Cloudflare-related skills into backend/serverless-edge only when
    // the Cloudflare signal is explicit in name/tags/path and there is no
    // obvious security context that should route to testing/security.
    let cloudflare_signal_text = format!(
        "{} {}",
        meta.name,
        meta.triggers.clone().unwrap_or_default()
    );
    let (cloudflare_norm, cloudflare_tokens) = normalize_text(&cloudflare_signal_text);
    
    let path_str = meta.path.to_string_lossy().replace('\\', "/");
    let is_in_skills_repo = path_str.contains("lib/skills/") || path_str.contains("lib/cloudflare");

    let found_cloudflare_signal = is_in_skills_repo 
        || cloudflare_tokens.contains("cloudflare")
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
        meta.hub = "server-side".to_string();
        meta.sub_hub = "serverless-edge".to_string();
        // If it was explicitly forced by being in the skills repo, give it 100
        meta.match_score = Some(100);
    } else if is_in_skills_repo && security_context && meta.name == "cloudflare" {
        // Special case: The master "cloudflare" skill mentions WAF/DDoS, so it usually trips
        // security_context, but we strictly want it in serverless-edge.
        meta.hub = "server-side".to_string();
        meta.sub_hub = "serverless-edge".to_string();
        meta.match_score = Some(100);
    } else if let Some((hub, sub_hub)) = canonicalize_assignment(&meta.hub, &meta.sub_hub) {
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        meta.match_score = Some(100);
    } else if let Some((hub, sub_hub)) = infer_from_path(meta) {
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        // Repo-name inference is treated as high confidence but still allows LLM refinement (90 < 95)
        meta.match_score = Some(90);
    } else if let Some((hub, sub_hub)) = infer_from_rules_ranked(&normalized, &tokens) {
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        // Keyword rule match (85 < 95)
        meta.match_score = Some(85);
    } else if let Some((hub, sub_hub)) =
        infer_from_rules_ranked_with_min(&normalized, &tokens, 4, &["business", "marketing"])
    {
        // Low-confidence salvage pass
        meta.hub = hub;
        meta.sub_hub = sub_hub;
        meta.match_score = Some(80);
    } else {
        meta.hub = "business".to_string();
        meta.sub_hub = "business-strategy".to_string();
        meta.match_score = Some(70);
    }

    // Security content override: if name/description are strongly security-related,
    // override path-based classification (e.g. "exploiting-sql-injection" should
    // be security even if repo name matched "anthropic" → ai/agents).
    let security_override_signals = [
        "pentest", "penetration test", "vulnerability", "exploit",
        "red team", "red-team", "incident response", "soc ",
        "threat hunting", "threat detection", "malware",
        "forensic", "ransomware", "phishing", "credential dumping",
        "lateral movement", "privilege escalation", "brute force",
        "owasp", "mitre att", "cve-", "nist", "siem",
        "intrusion detection", "security assessment", "security audit",
        "cybersecurity", "cyber-security", "zero-day", "zero day",
        "attack surface", "kill chain", "indicators of compromise",
        "data exfiltration", "command and control", "c2 ",
    ];

    if meta.hub != "code-quality" || meta.sub_hub != "security" {
        let name_desc = format!("{} {}", meta.name, meta.description).to_lowercase();
        let strong_hits = security_override_signals
            .iter()
            .filter(|sig| name_desc.contains(**sig))
            .count();
        if strong_hits >= 2 {
            meta.hub = "code-quality".to_string();
            meta.sub_hub = "security".to_string();
            meta.match_score = Some(95); 
        }
    }

    // AI skill / prompt authoring content override
    let prompting_override_signals = [
        "skill creation", "creating skills", "write skill.md",
        "writing skills", "prompt engineering", "system prompt",
        "agent prompt", "meta-prompting", "prompt optimization",
        "skill authoring", "skill-authoring", "skill enhancement",
        "prompt factory", "prompt-factory", "skills factory",
        "skills-factory", "skills-bank", "skills bank",
    ];

    if meta.hub != "server-side" || meta.sub_hub != "prompting-factory" {
        let name_desc = format!("{} {}", meta.name, meta.description).to_lowercase();
        let prompt_hits = prompting_override_signals
            .iter()
            .filter(|sig| name_desc.contains(**sig))
            .count();
        if prompt_hits >= 1 {
            meta.hub = "server-side".to_string();
            meta.sub_hub = "prompting-factory".to_string();
            meta.match_score = Some(95);
        }
    }

    if meta.triggers.is_none() || meta.triggers.as_ref().unwrap().is_empty() {
        meta.triggers = Some(generate_triggers(&meta.name));
    }

    // Ensure strict hub/sub-hub validation and normalization
    let (valid_hub, valid_sub) = ensure_valid_hub_subhub(&meta.hub, &meta.sub_hub);
    meta.hub = valid_hub;
    meta.sub_hub = valid_sub;

    meta.phase = Some(match meta.hub.as_str() {
        "code-quality" => 1,
        "frontend" => 1,
        "server-side" => 2,
        "business" => 3,
        _ => 4,
    });

    true
}

pub fn ensure_valid_hub_subhub(hub: &str, sub_hub: &str) -> (String, String) {
    let mut hub_norm = hub.trim().to_lowercase();
    let mut sub_hub_norm = sub_hub.trim().to_lowercase();

    // Intercept decommissioned mobile and code-review to prevent business-strategy fallback
    if hub_norm == "mobile" || sub_hub_norm == "ios" || sub_hub_norm == "android" || sub_hub_norm == "cross-platform" {
        hub_norm = "frontend".to_string();
        sub_hub_norm = "web-frameworks".to_string();
    }
    if sub_hub_norm == "code-review" {
        hub_norm = "code-quality".to_string();
        sub_hub_norm = "testing-qa".to_string();
    }

    // 1. Normalize the hub first
    let resolved_hub = if VALID_HUBS.contains(&hub_norm.as_str()) {
        hub_norm
    } else if hub_norm == "ai" {
        "server-side".to_string()
    } else {
        // Fallback to "business" if entirely unrecognized
        "business".to_string()
    };

    // 2. Validate/normalize the sub_hub under the resolved hub
    if let Some(hub_def) = SUB_HUB_DEFINITIONS.get(resolved_hub.as_str()) {
        // Handle common invalid sub_hub cases
        if resolved_hub == "server-side" && sub_hub_norm == "frameworks" {
            sub_hub_norm = "architect".to_string();
        } else if resolved_hub == "business" && (sub_hub_norm == "operations" || sub_hub_norm == "strategy") {
            sub_hub_norm = "business-strategy".to_string();
        }

        if hub_def.sub_hubs.contains_key(sub_hub_norm.as_str()) {
            (resolved_hub, sub_hub_norm)
        } else {
            // Fallback to default sub_hub for that hub
            let default_sub = default_subhub_for_hub(&resolved_hub).unwrap_or("business-strategy").to_string();
            (resolved_hub, default_sub)
        }
    } else {
        // Fallback safety
        ("business".to_string(), "business-strategy".to_string())
    }
}

