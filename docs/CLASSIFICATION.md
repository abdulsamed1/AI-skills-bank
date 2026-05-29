# Classification Rules & Methodology

This document details the multi-stage classification and routing architecture of **skills-bank**. The platform processes thousands of distributed `SKILL.md` files and assigns them to a specific **Hub** and **Sub-Hub** with associated confidence weights.

---

## 🏗️ Multi-Stage Pipeline

```
 8000+ SKILL.md files
        │
        ▼
 ┌──────────────┐
 │  YAML Parse  │  Extracts name, description, tags, triggers
 └──────┬───────┘
            │
            ▼
 ┌──────────────┐
 │  Keyword     │  Fast token-based routing with precedence rules
 │  Rules       │  (fallback if LLM unavailable)
 └──────┬───────┘
            │
            ▼
 ┌──────────────┐
 │  Dedup Check │  Levenshtein HashSet of name OR description
 └──────┬───────┘
            │
            ▼
 ┌──────────────────────────────────────┐
 │  Hybrid Exclusion + LLM Classify     │
 │  Step A: Keyword pre-filter          │
 │  Step B: LLM semantic classify       │
 │         (with failover provider loop)│
 └──────┬───────────────────────────────┘
            │
            ▼
 ┌──────────────┐
 │  Output      │  routing.csv, subhub-index.json,
 │  Artifacts   │  per-hub manifests
 └──────────────┘
```

---

## 🎯 Hub & Sub-Hub Registry

The platform defines exactly **4 valid Hubs**, each containing distinct **Sub-Hubs**:

### 1. `code-quality`
- **`testing-qa`**: TDD, unit/integration testing, testing frameworks (Playwright, Cypress, Jest, PyTest), clean code, refactoring.
- **`version-control`**: Git operations, PR creation, PR merging, CI/CD integration, GitHub/GitLab.
- **`security`**: Threat analysis, penetration testing, cryptography, authentication/authorization (JWT, OAuth), red/blue teaming, CVE audits.

### 2. `frontend`
- **`web-frameworks`**: UI frameworks (React, Next.js, Vue, Nuxt, Svelte, Angular), state management (Zustand, Redux).
- **`ui-ux`**: Figma designs, styling (CSS, Tailwind, HSL), responsive design, design systems, accessibility (`a11y`).

### 3. `server-side`
- **`prompting-factory`**: Prompt engineering, prompt templates, system prompts, context compression, LLM application schemas.
- **`architect`**: REST/GraphQL APIs, OpenAPI schemas, system design, microservices, gRPC, dependency injection.
- **`databases`**: Relational and NoSQL schemas, database migrations, SQL, query optimization (Postgres, DynamoDB, MongoDB, Prisma).
- **`caching`**: Redis, Memcached, cache eviction policies, TTL strategies.
- **`messaging`**: Event-driven brokers (Kafka, RabbitMQ, SQS, NATS), background workers (BullMQ, Sidekiq).
- **`containers`**: Docker, Kubernetes (`k8s`), Helm charts, infrastructure as code (Terraform).
- **`serverless-edge`**: Cloudflare Workers, Edge functions, AWS Lambda, WASM compilation.
- **`observability`**: Opentelemetry, Prometheus, Grafana, logging, metrics, error tracing (Sentry).

### 4. `business`
- **`business-strategy`**: Product management, PRDs, roadmaps, sprint planning, GTM execution, TAM analysis, metrics (ARR, MRR, CAC, LTV), Standard Operating Procedures (SOPs).
- **`marketing`**: SEO/AEO optimization, copywriting, social media strategy (X, LinkedIn), paid advertisements, email campaigns.
- **`sales`**: Lead generation, outreach, CRMs (HubSpot, Salesforce), sales pipelines, customer success.

---

## 📊 Confidence Weights & Priorities

Rules are evaluated in order. Higher confidence scores override lower confidence results:

| Match Source | Confidence Score | Description |
|---|---|---|
| **Forced / Explicit Override** | `100` | Hardcoded overrides (e.g. Cloudflare core to `serverless-edge`). |
| **Repository Name Substring** | `98` | Matches repository directory names (e.g. `anthropic-cybersecurity-skills` -> `security`). |
| **Security Override** | `95` | Triggers when name/description matches $\ge 2$ strong security terms. |
| **Prompt Factory Override** | `95` | Triggers when name/description matches $\ge 1$ prompt engineering terms. |
| **Keyword Rules** | `85` | Matches against curated lists of sub-hub keywords. |
| **Salvage Pass** | `80` | Low-threshold keyword match pass for business/marketing. |
| **Default Fallback** | `70` | Fallback to `business/business-strategy`. |

---

## ⚡ Sub-Hub Conflict Resolution

When a skill matches multiple sub-hubs (e.g. mentions both "Python" and "Security"), the conflict resolution matrix determines the winner:

- **Domain Specialist Priority**: `security` and `testing-qa` always override generic programming languages (e.g. `python`, `javascript`, `rust`).
- **Framework Precedence**: Specific frameworks (e.g. `web-frameworks`) win over language anchors.
- **Precedence Order**: 
  $$\text{Security} \succ \text{Testing/QA} \succ \text{Architect} \succ \text{Languages}$$

---

## 🧠 LLM Semantic Classification (Step B)

If the confidence score from initial rules is $< 95\%$, the aggregator delegates classification to the configured LLM provider:

1. **System Prompt**: Enforces strict JSON output formatting.
2. **Batching**: Processes skills concurrently (configurable via `LLM_BATCH_SIZE` and `LLM_CONCURRENCY`).
3. **Resilience**: FreeLLMAPI automatically rotates through 12+ free backup keys and providers if rate limits (429) or timeouts occur.
4. **Parser**: Sanitizes and extracts JSON structures even if returned as raw objects or nested strings.
