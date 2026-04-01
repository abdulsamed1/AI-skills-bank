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
    "ai",
    "business",
    "code-quality",
    "frontend",
    "mobile",
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
                "unit-test",
                "integration-test",
                "cypress",
                "playwright","unit", "jest", "mocha", "pytest", "unittest",
           "e2e", "end-to-end", "cypress", "playwright", "selenium" ],
            anchor_keywords: vec!["testing", "test", "qa"],
            negative_keywords: vec!["marketing", "seo", "business", "odoo"],
        },
    );
    
    code_quality.insert(
        "ci-cd",
        SubHubRule {
            keywords: vec!["ci", "cd", "github-actions", "jenkins", "pipeline"],
            anchor_keywords: vec!["ci", "cd", "pipeline", "github-actions", "jenkins", "github actions", "jenkins pipeline", "github actions pipeline"],
            negative_keywords: vec!["ui"],
        },
    );
    code_quality.insert(
        "code-review",
        SubHubRule {
            keywords: vec!["code-review", "code-review-ai-ai-review", "code-review-excellence", "code-review:review-local-changes", "code-review:review-pr", "code-reviewer", "code-simplifier"],
            anchor_keywords: vec!["code-review", "code-review-ai-ai-review", "code-review-excellence", "code-review:review-local-changes", "code-review:review-pr", "code-reviewer", "code-simplifier"],
            negative_keywords: vec!["ui"],
        },
    );
    
    code_quality.insert(
        "security",
        SubHubRule {
            keywords: vec!["malicious","html-injection","auth", "session", "login", "password", "security", "oauth", "jwt", "encryption", "pentest", "infrastructure", "firewall", "network", "vpn", "waf", "vulnerability", "vulnerabilities", "cve", "scanning", "auditing", "red-team", "ddos", "attack", "zero trust", "sqli", "sql injection", "threat", "browser isolation", "injection", "exploit", "xss", "csrf", "hardening", "kms", "key management", "cryptography", "cyber-security", "cybersecurity", "cyber attack", "cyber-attack"],
            anchor_keywords: vec!["auth", "oauth", "jwt", "security", "pentest", "vulnerability", "vulnerabilities", "infrastructure", "firewall", "red-team", "waf", "ddos", "zero trust", "injection", "exploit", "xss", "csrf", "encryption", "kms", "key management", "threat"],
            negative_keywords: vec!["marketing", "seo"],
        },
    );
    code_quality.insert(
        "javascript",
        SubHubRule {
            keywords: vec!["javascript", "js", "es6", "node", "npm", "v8"],
            anchor_keywords: vec!["javascript", "js", "es6"],
            negative_keywords: vec![
                "code-quality concepts", "data structures",
                "seo", "crawl", "crawlability", "indexability",
                "core-web-vitals",
            ],
        },
    );
    code_quality.insert(
        "typescript",
        SubHubRule {
            keywords: vec!["typescript", "tsconfig", "tsx", "type-system", "ts"],
            anchor_keywords: vec!["typescript", "tsconfig"],
            negative_keywords: vec!["python", "rust", "golang"],
        },
    );
    code_quality.insert(
        "python",
        SubHubRule {
            keywords: vec!["python", "py", "django", "fastapi", "pandas", "numpy"],
            anchor_keywords: vec!["python", "fastapi"],
            negative_keywords: vec!["typescript", "rust"],
        },
    );
    code_quality.insert(
        "rust",
        SubHubRule {
            keywords: vec!["rust", "cargo", "ownership", "lifetimes"],
            anchor_keywords: vec!["rust", "cargo"],
            negative_keywords: vec!["python", "typescript"],
        },
    );
    code_quality.insert(
        "golang",
        SubHubRule {
            keywords: vec!["golang", "go", "goroutine", "channels"],
            anchor_keywords: vec!["golang", "go"],
            negative_keywords: vec![
                "python", "rust",
                "go-to-market", "gtm", "market", "marketing",
                "beachhead", "growth-loop", "customer-profile",
                "product-discovery", "product-launch", "saas",
                "stock", "equity", "sred",
                "brainstorm", "ideation",
                "robot", "robotics", "embodied",
            ],
        },
    );
    code_quality.insert(
        "java",
        SubHubRule {
            keywords: vec!["java", "spring", "maven", "jvm"],
            anchor_keywords: vec!["java", "spring"],
            negative_keywords: vec!["python", "rust"],
        },
    );
    hubs.insert(
        "code-quality",
        HubDefinition {
            name: "code-quality",
            sub_hubs: code_quality,
        },
    );

    // AI Hub
    let mut ai_sub = HashMap::new();
    ai_sub.insert(
        "prompting-factory",
        SubHubRule {
            keywords: vec!["skill-enhancement",
                "skills-factory",
                "llm-skill",
                "prompt-engineering", "context-compression", "meta-prompting", "prompt-optimization", "prompt-compression"],
            anchor_keywords: vec![   "skill-enhancement",
                "skills-factory",
                "agent-skill",
                "llm-skill",
                "prompt-engineering"],
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
            keywords: vec!["html", "css", "tailwind", "styling", "design-systems", "responsive", "design-system", "component-library", "tokens", "storybook", "html", "css", "tailwind", "styling", "ui-ux", "responsive","ux", "user-experience", "usability","ui", "design", "wireframe", "prototype", "user-interface", "user-experience", "stitch", "figma"],
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


// server-side Hub — 8 sub-hubs متخصصة
let mut be_sub = HashMap::new();

// 1. FRAMEWORKS — اختيار الفريموورك واستخدامه
be_sub.insert(
    "frameworks",
    SubHubRule {
        keywords: vec![
            "express", "koa", "hapi", "fastify", "hono", "elysia",
            "nestjs", "adonisjs",
            "django", "flask", "fastapi", "litestar", "tornado",
            "spring", "spring-boot", "quarkus", "micronaut",
            "rails", "laravel", "phoenix", "gin", "fiber", "echo",
            "server framework", "backend framework", "web server",
            "middleware", "routing", "dependency injection",
        ],
        anchor_keywords: vec![
            "express", "fastapi", "django", "flask", "spring", "nestjs",
            "hono", "gin", "rails", "laravel", "phoenix",
        ],
        negative_keywords: vec![
            "react", "vue", "angular", "nextjs", "html", "css",
            "sql", "postgres", "database", "docker", "kubernetes",
            "kafka", "rabbitmq", "serverless", "cloudflare",
        ],
    },
);

// 2. API-DESIGN — تصميم الـ API وعقوده وأنماطه
be_sub.insert(
    "api-design",
    SubHubRule {
        keywords: vec![
            "rest", "restful", "graphql", "grpc", "trpc",
            "openapi", "swagger", "api spec", "api contract",
            "api gateway", "api versioning", "api documentation",
            "api design", "api best practices", "api standards",
            "endpoint", "rate limiting", "throttling",
            "api authentication", "api authorization",
            "api error handling", "api pagination",
            "webhook", "websocket", "sse", "server-sent events",
            "api security", "api caching", "api monitoring",
        ],
        anchor_keywords: vec![
            "rest", "graphql", "openapi", "swagger", "api design",
            "api gateway", "webhook", "websocket", "grpc", "trpc",
        ],
        negative_keywords: vec![
            "html", "css", "react", "vue", "sql", "postgres",
            "docker", "kubernetes", "kafka", "serverless",
        ],
    },
);

// 3. DATABASES — قواعد البيانات والـ ORM والمخططات
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

// 4. CACHING — استراتيجيات التخزين المؤقت
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

// 5. MESSAGING — الأنظمة القائمة على الأحداث والرسائل
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

// 6. CONTAINERS — الحاويات والتنسيق والبنية التحتية
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

// 7. SERVERLESS-EDGE — اللاخادمية وحوسبة الحافة
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
            "cloudflare workers", "faas", "edge functions",
        ],
        negative_keywords: vec![
            "ui", "frontend", "security", "waf", "ddos",
            "injection", "vulnerability", "pentest",
            "sql", "database", "docker", "kubernetes",
        ],
    },
);

// 8. OBSERVABILITY — المراقبة والتتبع والتسجيل
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


    
    // Business Hub - 6 sub-hubs متخصصة
let mut bus_sub = HashMap::new();

// 1. PRODUCT — إدارة المنتج، roadmap، user stories
bus_sub.insert(
    "product",
    SubHubRule {
        keywords: vec![
            "product management", "prd", "roadmap", "feature roadmap",
            "user story", "backlog", "mvp", "product discovery",
            "customer discovery", "north star metric", "epic",
            "jobs to be done", "jtbd", "feature prioritization",
            "requirements", "acceptance criteria", "sprint", "okr",
        ],
        anchor_keywords: vec![
            "prd", "roadmap", "mvp", "backlog", "product discovery",
        ],
        negative_keywords: vec![
            "react", "api", "sql", "docker", "python", "rust",
            "vulnerability", "auth",
        ],
    },
);

// 2. STRATEGY — go-to-market، تحليل السوق، نموذج الأعمال
bus_sub.insert(
    "strategy",
    SubHubRule {
        keywords: vec![
            "go-to-market", "gtm", "market analysis", "competitive analysis",
            "positioning", "business model", "value proposition",
            "pricing strategy", "beachhead", "tam sam som",
            "porters five forces", "swot", "pestel", "lean canvas",
            "business case", "stakeholder alignment", "growth strategy",
            "market segmentation", "icp", "ideal customer profile",
            "b2b", "b2c", "saas strategy",
        ],
        anchor_keywords: vec![
            "go-to-market", "positioning", "business model",
            "value proposition", "competitive analysis",
        ],
        negative_keywords: vec![
            "react", "nextjs", "api", "sdk", "python", "rust",
            "golang", "java", "kubernetes", "docker", "sql",
            "vulnerability", "injection", "auth", "oauth", "jwt",
            "backlog", "sprint", "prd",
        ],
    },
);

// 3. MARKETING
bus_sub.insert(
    "marketing",
    SubHubRule {
        keywords: vec![
            "seo", "search engine optimization", "keyword research",
            "technical seo", "on-page seo", "serp", "backlinks",
            "content marketing", "copywriting", "blog", "editorial",
            "content strategy", "thought leadership", "landing page copy",
            "content design", "microcopy", "video script",
            "social media", "twitter", "facebook", "instagram",
            "linkedin", "tiktok", "brand strategy", "brand identity",
            "tone of voice", "messaging", "audience", "campaign strategy",
            "ad copy", "paid ads", "ppc", "marketing strategy",
        ],
        anchor_keywords: vec![
            "seo", "copywriting", "content marketing", "social media",
            "brand strategy", "campaign strategy",
        ],
        negative_keywords: vec![
            "backend", "sql", "api", "python", "kubernetes",
            "vulnerability", "email automation", "crm",
        ],
    },
);

// 4. SALES 
bus_sub.insert(
    "sales",
    SubHubRule {
        keywords: vec![
            "sales", "lead generation", "lead nurturing", "lead scoring",
            "lead qualification", "cold email", "cold outreach",
            "sales funnel", "closing", "deal", "pitch", "proposal",
            "crm", "pipeline", "objection handling", "discovery call",
            "demo", "negotiation", "account management", "upsell",
            "cross-sell", "churn", "retention", "customer success",
            "b2b sales", "enterprise sales", "inbound", "outbound",
        ],
        anchor_keywords: vec![
            "lead generation", "sales funnel", "cold email",
            "crm", "closing", "pitch",
        ],
        negative_keywords: vec![
            "backend", "api", "sql", "kubernetes", "python",
            "vulnerability", "seo", "content",
        ],
    },
);

// 5. ANALYTICS — قياس الأداء، attribution، تحليل البيانات
bus_sub.insert(
    "analytics",
    SubHubRule {
        keywords: vec![
            "google analytics", "ga4", "mixpanel", "amplitude",
            "utm", "attribution", "conversion rate", "funnel analytics",
            "cohort analysis", "a/b testing", "split testing",
            "kpi", "metrics", "dashboard", "reporting",
            "data driven", "product analytics", "user analytics",
            "revenue analytics", "ltv", "cac", "arpu", "mrr", "arr",
            "retention rate", "churn rate", "nps",
        ],
        anchor_keywords: vec![
            "google analytics", "attribution", "conversion rate",
            "funnel analytics", "kpi", "mrr", "ltv", "cac",
        ],
        negative_keywords: vec![
            "backend", "sql", "database", "api", "python",
            "machine learning", "vulnerability",
        ],
    },
);

// 6. OPERATIONS
bus_sub.insert(
    "operations",
    SubHubRule {
        keywords: vec![
            "sop", "standard operating procedure", "process improvement",
            "workflow optimization", "runbook", "incident management",
            "operational excellence", "business operations",
            "service operations", "cost optimization", "capacity planning",
            "n8n", "zapier", "make", "ai automation",
            "productivity automation-ai", "project management",
            "okr tracking", "team management", "hiring", "onboarding",
            "email", "newsletter", "email marketing", "mailchimp",
            "sendgrid", "email campaign", "drip campaign",
        ],
        anchor_keywords: vec![
            "sop", "process improvement", "runbook",
            "n8n", "zapier", "automation-ai", "email campaign",
        ],
        negative_keywords: vec![
            "backend", "api", "sql", "kubernetes", "python",
            "rust", "vulnerability", "injection",
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
    "job",
    "jobs",
    "career",
    "resume",
    "cv",
    "cover-letter",
    "interview",
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
    "game",
    "games",
    "game-development",
    "gaming",
    "unreal-engine",
    "unity-3d",
    "blender",
    "3d",
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
    // AI hub
    ("prompt-engineering", "ai", "prompting-factory"),

  // server-side aliases
("api-design",            "server-side", "api-design"),
("api-rest-design",       "server-side", "api-design"),
("server-side-frameworks","server-side", "frameworks"),
("backend-frameworks",    "server-side", "frameworks"),
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
("product-strategy", "business", "product"),
("product", "business", "product"),
("product-management", "business", "product"),
("strategy", "business", "strategy"),
("go-to-market", "business", "strategy"),
("gtm", "business", "strategy"),
("marketing", "business", "marketing"),
("content", "business", "marketing"),
("seo", "business", "marketing"),
("social", "business", "marketing"),
("social-media", "business", "marketing"),
("brand", "business", "marketing"),
("sales", "business", "sales"),
("lead-generation", "business", "sales"),
("crm", "business", "sales"),
("analytics", "business", "analytics"),
("email", "business", "operations"),
("operations", "business", "operations"),
("automation", "business", "operations"),
    // frontend hub
    ("ui-ux", "frontend", "ui-ux"),
    ("ux", "frontend", "ui-ux"),
    ("react-nextjs", "frontend", "web-frameworks"),
    ("web-basics", "frontend", "web-frameworks"),
    ("web-frameworks", "frontend", "web-frameworks"),
    ("state-management", "frontend", "state-management"),
    // mobile hub
    ("cross-platform", "mobile", "cross-platform"),
    ("ios", "mobile", "ios"),
    ("android", "mobile", "android"),
    // code-quality hub
    ("typescript", "code-quality", "typescript"),
    ("python", "code-quality", "python"),
    ("rust", "code-quality", "rust"),
    ("golang", "code-quality", "golang"),
    ("java", "code-quality", "java"),
    ("javascript", "code-quality", "javascript"),
    // Legacy testing/security → code-quality
    ("automation", "code-quality", "testing-qa"),
    ("automation-testing", "code-quality", "testing-qa"),
    ("unit-testing", "code-quality", "testing-qa"),
    ("e2e-testing", "code-quality", "testing-qa"),
    ("performance-testing", "code-quality", "testing-qa"),
    ("security", "code-quality", "security"),
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
        "server-side" => Some("api-design"),  // الافتراضي الأوسع انتشاراً
        "business" => Some("strategy"),  // الافتراضي الأوسع
        "frontend" => Some("web-frameworks"),
        "mobile" => Some("cross-platform"),
        "code-quality" => Some("testing-qa"),
        _ => None,
    }
}

fn canonicalize_assignment(hub: &str, sub_hub: &str) -> Option<(String, String)> {
    let hub_norm = normalize_slug(hub);
    let sub_norm = normalize_slug(sub_hub);

    // Legacy "security" hub → code-quality/security
    if hub_norm == "security" {
        return Some(("code-quality".to_string(), "security".to_string()));
    }
    // Legacy "testing" hub → code-quality/testing-qa
    if hub_norm == "testing" {
        return Some(("code-quality".to_string(), "testing-qa".to_string()));
    }
    // Legacy "marketing" hub → business/tactical
    if hub_norm == "marketing" {
        let canonical_sub = match sub_norm.as_str() {
            "strategy" => "strategy",
            "seo" | "content" | "social" | "social-media" => "tactical",
            "email" | "analytics" | "sales" => "operations",
            _ => "tactical",
        };
        return Some(("business".to_string(), canonical_sub.to_string()));
    }
    // Legacy "backend" hub → server-side
    if hub_norm == "backend" {
        let canonical_sub = match sub_norm.as_str() {
            "api-design" => "core",
            "" => "core",
            other => other,
        };
        return Some(("server-side".to_string(), canonical_sub.to_string()));
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

    // Guard: pm-go-to-market paths are business/strategy, not golang
    let is_gtm_path = components.iter().any(|c| c.contains("go-to-market"));
    if is_gtm_path {
        return Some(("business".to_string(), "strategy".to_string()));
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
        "ai" => 3,
        "mobile" => 4,
        "business" => 5,
        _ => 99,
    }
}

pub fn infer_from_rules_ranked_with_min(
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
            score += 3;
        } else {
            score -= 3;
        }
    }

    score
}

pub fn is_excluded(normalized_text: &str, tokens: &HashSet<String>) -> bool {
    // 1. Keyword-based exclusions (Blacklist)
    for pattern in ENV_EXCLUSION_PATTERNS.iter() {
        if tokens.contains(pattern.as_str()) || normalized_text.contains(pattern) {
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
        "code-quality" => 1,
        "frontend" => 1,
        "server-side" => 2,
        "ai" => 3,
        "business" => 4,
        "mobile" => 4,
        _ => 5,
    });

    true
}

