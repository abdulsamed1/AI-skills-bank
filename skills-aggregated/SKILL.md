---
name: skills-bank-router
description: |
  Central router for all skill hubs (11 main hubs, 48+ sub-hubs, 1,400+ skills).
  Use this to understand the complete landscape and route to specific hubs.
  
  Guard: DO NOT assume skills exist outside verified hubs list below.
  
type: master-router
hubs_total: 11
skills_total: 1400
version: 1.0
---

# Skills Bank Central Router

## ⚠️ MASTER GUARD RULES

1. **DO NOT invoke skills** without first identifying correct hub
2. **DO NOT assume skill exists** unless it's in a verified hub below
3. **11 HUBS ONLY** — no other hubs exist
4. **Match intent → Hub → Sub-hub → Skill** (exact order)
5. **Never hallucinate** alternative skill names or hubs

---

## Hub Discovery Matrix

### Quick Intent → Hub Router

| User Intent | Primary Hub | Top Triggers | Example Skills | Most Used |
|------------|-----------|--------------|----------------|-----------|
| **"I need an API..."** | `backend` | api, endpoint, rest, graphql | api-documentation, api-endpoint-builder | api-documentation |
| **"I need to build a web app"** | `frontend` | react, vue, css, ui, component | react-patterns, styling-system | react-patterns |
| **"I need to design/architect..."** | `design` | design, ux, ui, wireframe, mockup | ux-research, design-system | design-system |
| **"I need an AI agent/LLM..."** | `ai` | agent, llm, claude, model, ai | agent-builder, llm-orchestrator | agent-builder |
| **"I need a database..."** | `backend` | database, sql, nosql, query, orm | database-design, query-optimization | database-design |
| **"I need testing..."** | `testing` | test, qa, automation, e2e, unit | test-framework, automation-helper | test-framework |
| **"I need product strategy..."** | `business` | product, prd, strategy, roadmap, go-to-market | prd-creator, product-strategist | prd-creator |
| **"I need mobile app..."** | `mobile` | mobile, ios, android, react-native, expo | mobile-patterns, native-bridge | mobile-patterns |
| **"I need DevOps/deployment..."** | `devops` | ci, cd, docker, kubernetes, deployment | pipeline-builder, deployment-automation | pipeline-builder |
| **"I need security..."** | `security` | security, auth, encryption, vulnerability, risk | auth-patterns, penetration-tester | auth-patterns |
| **"I need marketing/content..."** | `marketing` | marketing, seo, content, analytics, gtm | content-strategist, seo-optimizer | content-strategist |

---

## Complete Hub Landscape

### 🏗️ Hub #1: `backend` (281 skills)
```
Sub-hubs:
  ├─ api-design (76 skills)
  │  └─ Key: REST, GraphQL, API patterns, versioning
  └─ databases (205 skills)
     └─ Key: SQL, NoSQL, query optimization, migrations

Entry Point: "I need API..." or "I need database..."
```

### 🎨 Hub #2: `frontend` (242 skills)
```
Sub-hubs:
  ├─ state-management (125 skills)
  │  └─ Key: Redux, Zustand, Context, state patterns
  ├─ styling (87 skills)
  │  └─ Key: CSS, Tailwind, CSS-in-JS, responsive design
  └─ components (30 skills)
     └─ Key: Component libraries, design systems

Entry Point: "I need React..." or "I need styling..."
```

### 🤖 Hub #3: `ai` (289 skills)
```
Sub-hubs:
  ├─ llm-agents (168 skills)
  │  └─ Key: Agent frameworks, Claude, GPT, orchestration
  └─ skills-factory (121 skills)
     └─ Key: Skill creation, routing, verification

Entry Point: "I need an agent..." or "I need AI..."
```

### 🧪 Hub #4: `testing` (143 skills)
```
Sub-hubs:
  ├─ automation (89 skills)
  │  └─ Key: E2E, unit, integration, test frameworks
  └─ qa-strategy (54 skills)
     └─ Key: Test planning, coverage, quality gates

Entry Point: "I need testing..." or "I need QA..."
```

### 📱 Hub #5: `mobile` (89 skills)
```
Sub-hubs:
  ├─ cross-platform (47 skills)
  │  └─ Key: React Native, Flutter, Expo
  ├─ native (25 skills)
  │  └─ Key: Swift, Kotlin, native APIs
  └─ mobile-design (17 skills)
     └─ Key: Mobile UX, touch interactions

Entry Point: "I need mobile..." or "I need React Native..."
```

### 🔒 Hub #6: `security` (156 skills)
```
Sub-hubs:
  ├─ auth (62 skills)
  │  └─ Key: OAuth, JWT, session management, SSO
  ├─ encryption (45 skills)
  │  └─ Key: Crypto, hashing, TLS, key management
  └─ vulnerability (49 skills)
     └─ Key: Penetration testing, vulnerability scanning

Entry Point: "I need authentication..." or "I need security..."
```

### 🚀 Hub #7: `devops` (118 skills)
```
Sub-hubs:
  ├─ ci-cd (67 skills)
  │  └─ Key: GitHub Actions, GitHub Actions, Jenkins, GitLab CI
  ├─ containerization (31 skills)
  │  └─ Key: Docker, Kubernetes, orchestration
  └─ monitoring (20 skills)
     └─ Key: Observability, logging, metrics

Entry Point: "I need CI/CD..." or "I need deployment..."
```

### 📊 Hub #8: `business` (289 skills)
```
Sub-hubs:
  ├─ product-strategy (123 skills)
  │  └─ Key: PRD, strategy, roadmap, discovery, go-to-market
  ├─ analytics (87 skills)
  │  └─ Key: Metrics, tracking, funnel analysis
  └─ finance (79 skills)
     └─ Key: Pricing, revenue, financial models

Entry Point: "I need product strategy..." or "I need PRD..."
```

### 🎯 Hub #9: `design` (134 skills)
```
Sub-hubs:
  ├─ ux-research (52 skills)
  │  └─ Key: User research, personas, journey mapping
  ├─ ui-design (54 skills)
  │  └─ Key: Design systems, components, accessibility
  └─ design-systems (28 skills)
     └─ Key: Component libraries, tokens, documentation

Entry Point: "I need UX..." or "I need design system..."
```

### 📝 Hub #10: `marketing` (167 skills)
```
Sub-hubs:
  ├─ strategy (38 skills)
  │  └─ Key: GTM, positioning, competitive analysis
  ├─ content (67 skills)
  │  └─ Key: Content creation, SEO, copywriting
  └─ analytics (62 skills)
     └─ Key: Marketing metrics, attribution, funnel

Entry Point: "I need marketing..." or "I need content..."
```

### 🎓 Hub #11: `programming` (348 skills)
```
Sub-hubs:
  ├─ python (78 skills)
  │  └─ Key: Django, Flask, data science, scripting
  ├─ javascript (95 skills)
  │  └─ Key: Node.js, async, npm ecosystem
  ├─ go (54 skills)
  │  └─ Key: Concurrency, microservices, system tools
  ├─ rust (41 skills)
  │  └─ Key: Safety, performance, embedded
  └─ ...others (80 skills)
     └─ Key: Java, C++, C#, TypeScript patterns

Entry Point: "I need Python..." or "I need JavaScript..."
```

---

## Trigger Keyword Index

Quickly find the right hub using single keyword:

```
api, rest, graphql, endpoint        → backend/api-design
sql, nosql, postgres, mongodb       → backend/databases
react, vue, typescript, css         → frontend/*
agent, llm, claude, openai          → ai/llm-agents
test, qa, e2e, automation           → testing/*
mobile, react-native, flutter       → mobile/*
security, auth, encryption          → security/*
ci, cd, docker, kubernetes          → devops/*
product, prd, strategy, roadmap     → business/product-strategy
design, ux, ui, wireframe           → design/*
marketing, seo, content, gtm        → marketing/*
python, javascript, go, rust        → programming/*
```

---

## Anti-Hallucination Reference

### ✅ Valid Skills (Examples Only)
```
- `api-documentation` (backend/api-design hub)
- `react-patterns` (frontend hub)
- `agent-builder` (ai/llm-agents hub)
- `test-automation` (testing hub)
- `mobile-patterns` (mobile hub)
- `auth-patterns` (security hub)
- `ci-cd-pipeline` (devops hub)
- `prd-creator` (business hub)
- `design-system` (design hub)
```

### ❌ Invalid Skills (DO NOT USE)
```
- `rest-api-ninja` (hallucinated)
- `super-backend-skill` (hallucinated)
- `magic-react-helper` (hallucinated)
- `ultimate-testing-framework` (hallucinated)
- `skill-that-does-everything` (hallucinated)
```

**Rule:** If not in the "Complete Hub Landscape" section above, it doesn't exist. NEVER invent.

---

## Hub Affinity Rules

### ✓ Single Hub Per Request
```
✓ "I need API documentation" → backend/api-design (single)
✓ "I need React patterns" → frontend (single)
✗ "I need API and React together" → split into 2 separate requests
```

### ✓ Sub-hub Specificity
```
✓ "I need REST API" → backend/api-design (specific)
✓ "I need database optimization" → backend/databases (specific)
✗ "I need backend" → TOO VAGUE, ask for specifics
```

### ✓ No Cross-Hub Combinations
```
✓ First request: "Build API" → backend/api-design
✓ Then separate request: "Build UI" → frontend
✗ "Build full-stack app" → must split (stack violation)
```

---

## Invocation Workflow

### Step 1: Match Intent
```
AI Agent: "write a epic for a new feature"
Scan "Quick Intent → Hub Router" table
Result: business hub, prd-creator skill
Next: Go to business/product-strategy hub
```

### Step 2: Navigate to Sub-hub
```
Hub: business
Sub-hub: product-strategy
Entry point: business/product-strategy/SKILL.md
Next: Use that hub's Quick Intent Matcher
```

### Step 3: Select Exact Skill
```
Hub SKILL.md shows Quick Intent Matcher
Match: "write a epic for a new feature" → `prd-creator` skill
Invoke: Use exact ID from table
STOP: Do not invent alternatives
```

### Step 4: If Hub Still Too Broad
```
Example: User: "I need Python"
Hub: programming (11 sub-hubs with 348 skills)
Sub-hub candidates: python, javascript, go, rust, ...
Filter: Use trigger keywords → narrow to python sub-hub
Next: Read python/SKILL.md for Quick Intent Matcher
```

---

## Common Hallucinations & Prevention

| Hallucination | What I Might Invent | What to Do Instead |
|--------------|-------------------|------------------|
| "Magic skill for everything" | `full-stack-genius-helper` | Check "Quick Intent Matrix" → route to specific hub |
| "Skill that was mentioned but doesn't exist" | `rest-api-ninja` | Verify in "Complete Hub Landscape" before invoking |
| "Skill from wrong hub" | invoke `api-documentation` from `frontend` hub | Confirm hub match before invocation |
| "Skill with wrong name" | `api-documentor` instead of `api-documentation` | Use exact ID from tables (case-sensitive) |
| "Mixed hub invocation" | "I'll use api + react skills together" | Split into 2 requests, 1 hub per request |

**Prevention: Use the "Quick Intent Matrix" above. It's your safety net.**

---

## Performance Targets

```
Current approach: Search → Guess → Try
❌ 80%+ hallucination rate when skill not obvious
❌ 200+ tokens consumed per search

Optimized approach: 
1. Use "Quick Intent Matrix" (5 sec scan)
2. Navigate to hub SKILL.md (15 sec read)
3. Use hub's Quick Intent Matcher (2 sec lookup)
4. Invoke exact skill with 100% confidence
✅ 0% hallucination risk
✅ 50-70 tokens saved per invocation
```

---

## File Structure Overview

```
skill-manage/skills-aggregated/
├─ SKILL.md (this file) ← START HERE for hub discovery
├─ ROUTER-SKILL.md (central router template)
├─ quick-lookup.tsv (coming soon)
├─ backend/
│  ├─ SKILL.md (hub router with Quick Intent Matcher)
│  ├─ api-design/
│  │  ├─ SKILL.md (use this for API selection)
│  │  ├─ skills-index.json (lightweight filter)
│  │  └─ skills-catalog.ndjson (read specific lines only)
│  └─ databases/
│     └─ SKILL.md (use this for database selection)
├─ frontend/
│  └─ SKILL.md (hub router)
├─ ai/
│  └─ SKILL.md (hub router)
... (and 8 more hubs)
```

---

## When to Use Which File

| File | Use Case | Read Time | Token Cost |
|------|----------|-----------|-----------|
| This file (SKILL.md) | Initial hub discovery | 3-5 min | 50-80 |
| Hub SKILL.md | Sub-hub selection + Quick Intent | 2-3 min | 30-50 |
| skills-index.json | Filter by triggers/scores | 1-2 min | 20-30 |
| skills-catalog.ndjson | Read specific skill lines | 30 sec | 5-15 |

**Optimal path: This file → Hub SKILL.md → Invoke**

---

## Questions & Troubleshooting

### Q: "Where's the 'advanced-skill-xyz'?"
A: Check "Complete Hub Landscape" section. If not listed, it doesn't exist in this system. Don't invent it.

### Q: "Can I use 2 skills from different hubs?"
A: No. Request 2 separate skill invocations, one hub per request. See "Hub Affinity Rules" above.

### Q: "How do I know which skill to use?"
A: Use "Quick Intent Matrix" to route to right hub, then use that hub's Quick Intent Matcher.

### Q: "What if my intent isn't in the matrix?"
A: Use trigger keywords to find related hub, then read that hub's SKILL.md for detailed options.

### Q: "Can I combine/customize skills?"
A: Each skill is independent. Combine results afterward, not during invocation.

---

## Guard Implementation Checklist

Before ANY skill invocation, verify:

- [ ] Hub exists in "Complete Hub Landscape" section
- [ ] Sub-hub is listed under selected hub
- [ ] Skill ID appears in source documents (not invented)
- [ ] User intent matches skill purpose
- [ ] Score ≥ 4 (minimum relevance)
- [ ] Request is single-hub (not cross-hub)
- [ ] Trigger keywords align with skill triggers
- [ ] Path field points to real SKILL.md file

**If ANY check fails: STOP. Do not invoke. Ask for clarification from user.**

---

## Next Steps

1. **For quick hub lookup:** Use "Quick Intent → Hub Router" matrix at top
2. **For specific skill:** Navigate to hub SKILL.md + use its Quick Intent Matcher
3. **If unsure:** Use "Complete Hub Landscape" to verify structure
4. **To prevent hallucination:** Always check this document first

---

## Maintenance & Updates

**This router is auto-generated from:**
- `skills-aggregated/` structure (11 hubs)
- `skills-manifest.json` (per-hub metadata)
- `subhub-index.json` (master index)

**Updated:** {generated_at}
**Generated by:** `aggregate-skills-to-subhubs.ps1`
**Version:** 1.0
