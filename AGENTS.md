# Agents

This document provides guidance for AI agents on discovering and loading skills from the skill-manage repository.

## Overview

Skills are organized hierarchically by **hub** (domains) and **sub_hub** (specialties). The canonical source of truth for each skill is its **SKILL.md file** located in the source repository (`lib/`).

The pipeline uses a **hybrid classification system**: fast keyword rules handle obvious routing, while an LLM-powered classifier provides semantic understanding for ambiguous skills. Skills that are irrelevant are excluded by either mechanism.

Agents must **never hallucinate** about skill capabilities—always load the authoritative SKILL.md from the file path provided in the routing manifest.

---

## Skill Discovery

### Step 1: Get Hub/Sub-Hub Routing

Each hub and sub-hub has a routing manifest:

```
skills-aggregated/
├── {hub}/
│   └── {sub_hub}/
│       ├── routing.csv          ← Use this to discover skills
│       ├── skills-index.json    ← Metadata index (optional)
│       └── .skill-lock.json     ← Cache metadata (optional)
```

**Example:** `skills-aggregated/marketing/content/routing.csv`

### Step 2: Parse Routing CSV

Each routing CSV has this schema:

```csv
skill_id,description,src_path
```


---

## Loading Skills (Anti-Hallucination Pattern)

### ✅ DO: Load from Authoritative Source

```
1. Discover skill_id in routing.csv
2. Get src_path from the same row
3. Load SKILL.md from src_path
4. Use the loaded SKILL.md as the source of truth
```

### ❌ DON'T: Assume or Hallucinate

- ❌ Don't infer skill behavior from `description` alone
- ❌ Don't assume skill capabilities without reading SKILL.md
- ❌ Don't make up tool restrictions or parameters
- ❌ Don't use stale cached metadata—always validate against source

---

## Skill Selection Logic (Multi-Match Scenarios)

When discovering and selecting skills from routing.csv, follow these rules:

### Scenario 1: Multiple Skills Match the Same Trigger

**When:** Multiple rows in routing.csv have overlapping keywords/descriptions matching the current tasks.

**Action:**
- Compare all matching skills by relevance to the user's specific goal
- Pick the skill with the **highest contextual relevance** (not just alphabetical)
- If 2+ skills are equally relevant
- **Example:** before write code for "auth.ts"
  - Matches: `api-rest-design`, `api-security`, `typescript-best-practices`
  - If context mentions "REST endpoints" → pick `api-rest-design`
  - If context is unclear → present top 3 or more skills

### Scenario 2: User Request is Ambiguous (Maps to Multiple Sub-Hubs)

**When:** The request could reasonably map to 2+ different hubs/sub-hubs.

**Action:**
1. Identify all candidate sub-hubs
2. Load routing.csv from each candidate
3. Score each sub-hub by relevance (keywords, description match)
4. Present top 3-4 sub-hub candidates to user with explanations
5. **Never guess** — wait for clarification if no clear winner
- **Example:** User says "I need help with analytics"
  - Candidates: `business/strategy` (growth metrics), `server-side/databases` (data systems)
  - Present both options: "Do you mean marketing metrics/tracking or data pipeline?"
  - Load matching sub-hub only

### Scenario 3: Story/Epic or Multi-Part Request

**When:** User asks to implement a full feature, epic, or multi-step workflow (e.g., "build a login system", "create checkout flow", etc.).

**Action:**
1. **Decompose the mission:** Break down into distinct sub-problems (authentication, database, frontend form, error handling, testing, etc.)
2. **Map each sub-problem to a hub/sub-hub:** 
   - Auth → code-quality/security
   - DB schema → server-side/databases
   - UI form → frontend/frameworks
   - Error handling → code-quality/testing-qa
   - Each mapping is independent; use routing.csv from each candidate
3. **Select 1 skill per distinct sub-problem** (recommend +5 skills total, up to 10)
4. **Ensure no skill overlap:** Each skill should address a unique sub-problem (not duplicate effort)
5. **Load all selected skills sequentially:** Use src_path from each routing.csv entry, parse SKILL.md frontmatter
6. **Merge execution:** Combine skills into a unified plan that respects dependencies and hand-offs
- **Example:** "Build a product checkout system"
  - Sub-problems:
    - API design (server-side/core) → pick `rest-api-best-practices`
    - Database schema (server-side/databases) → pick `sql-modeling`
    - Payment handling (server-side/core or security) → pick `pci-compliance` or `payment-api-integration`
    - Frontend form (frontend/web-frameworks + frontend/state-management) → pick 2 skills: `react-form-patterns`, `zustand-state`
    - E2E testing → code-quality/e2e → pick `playwright-automation`
  - Total: 5 skills covering checkout pipeline
  - Load all SKILL.md files, merge their guidance into one coherent plan

---

## Skill Selection Decision Tree

```
User Request
    ↓
Does request map to exactly 1 hub/sub-hub?
    ├─ YES → Load routing.csv, match keywords, pick 1 skill
    │        Load SKILL.md from src_path
    │        Execute
    │
    └─ NO → Multiple candidates or unclear
            ├─ Is request story/epic/multi-part?
            │   ├─ YES → Decompose into sub-problems
            │   │        Map each to hub/sub-hub separately
            │   │        Select 5+ skills (one per sub-problem)
            │   │        Load all SKILL.md files
            │   │        Merge into unified plan
            │   │
            │   └─ NO → Present top 2-3 hub/sub-hub candidates to user
            │            Ask for clarification
            │            Proceed with user's choice
```

---

## Token Budget & Caching Strategy

### Local Caching

Agents may cache skill metadata locally to reduce file I/O:

```json
{
  "skill_id": "quick-dev",
  "cached_at": "2026-03-30T12:00:00Z",
  "ttl_seconds": 3600,
  "frontmatter": {
    "name": "-quick-dev",
    "description": "Implements any user intent...",
    "allowed-tools": ["read", "write", "search"]
  }
}
```

**Cache Invalidation:**
- TTL: 1 hour recommended
- On mismatch: reload and validate against source
- On version change: invalidate entire cache entry

### Hub Summaries (Quick Discovery)

For faster discovery, agents may load `skills-index.json` to get a pre-indexed view:

```json
{
  "hub": "business",
  "sub_hub": "content",
  "total_skills": 42,
  "skills": [
    {
      "skill_id": "-distillator",
      "description": "Lossless LLM-optimized compression...",
      "src_path": "..."
    }
  ]
}
```

This index is read-only and generated during aggregation. Always reconcile with routing.csv for the source of truth.

---

## Finding Skills by Category

### By Hub

All skills under a hub across all sub-hubs:

```
skills-aggregated/{hub}/*/routing.csv
```

Concatenate all routing CSVs under the hub to get the complete skill roster.

### By Sub-Hub

All skills in a specific specialty:

```
skills-aggregated/{hub}/{sub_hub}/routing.csv
```

Example: `skills-aggregated/business/content/routing.csv` → content-focused skills.

### By Hub + Sub-Hub Search

If you need skills matching a keyword:

1. Load relevant routing CSVs
2. Parse `description` for preview matching
3. For full eligibility, load SKILL.md from `src_path`
4. Check frontmatter `allowed-tools`, `patterns`, `triggers`

---

## Frontmatter Schema

Each SKILL.md must have frontmatter. Minimal required fields:

```yaml
---
name: skill-identifier
description: |
  Full description of what the skill does.
allowed-tools:
  - tool-name-1
  - tool-name-2
---
```

**Optional fields:**

```yaml
version: 1.0.0
triggers:
  - keyword1
  - keyword2
patterns:
  - include: "**/*.ts"
    exclude: "**/*.test.ts"
```

Agents must gracefully handle variant/unknown frontmatter fields (use `serde_yaml::Value` coercion).

---

## Error Handling

### Missing SKILL.md

If `src_path` does not exist or is inaccessible:

1. Log the error and skill_id
2. Mark the skill as unavailable
3. **Do not hallucinate** a replacement skill
4. Inform the requestor or fall back to a default

### Malformed Frontmatter

If SKILL.md frontmatter is invalid YAML:

1. Attempt lenient parsing (coerce types, ignore unknown fields)
2. If still fails, log the error
3. Use only trusted fields from the manifest (routing.csv metadata)
4. Mark skill as degraded (metadata loaded, but frontmatter unreliable)

### Out-of-Date Routing

If a skill_id in routing.csv no longer exists in lib/:

1. Check cache TTL
2. Reload and validate
3. If still missing, remove from local cache
4. Mark skill as stale

---

## Best Practices

1. **Always validate source:** Load SKILL.md before assuming behavior
2. **Cache responsibly:** Use TTL and validate on cache hits
3. **Handle gracefully:** Log errors but don't hallucinate
4. **Stay current:** Re-aggregate periodically to catch new skills
5. **Report issues:** Flag malformed SKILL.md to repository maintainers

---

## Files & Artifacts

| File | Scope | Usage |
|------|-------|-------|
| `skills-aggregated/{hub}/{sub_hub}/routing.csv` | Single sub-hub | Primary source for skill discovery |
| `skills-aggregated/{hub}/{sub_hub}/skills-index.json` | Single sub-hub | Fast index for skills discovery (optional cache) |
| `skills-aggregated/hub-manifests.csv` | All hubs+sub-hubs | Complete directory of all skills (master index) |
| `skill-manage/lib/**/SKILL.md` | Source of truth | Authoritative specification per skill |

---

## Glossary

- **Hub:** Top-level domain. One of: `code-quality`, `frontend`, etc.
- **Sub-Hub:** Specialty within a hub (e.g., `business/content`, `server-side/databases`)
- **Skill:** Individual agent capability defined by a SKILL.md file
- **Routing CSV:** Lightweight manifest linking skill_id → src_path
- **SKILL.md:** Authoritative frontmatter + documentation for a skill
- **Source of Truth:** The skill-manage source repo (`lib/`) is canonical; all aggregated files are derived
- **Hybrid Classification:** Dual-stage pipeline (keyword rules + LLM semantic analysis) that routes skills to hubs
- **Excluded:** Skills flagged as irrelevant by either keyword rules (Step A) or LLM classification (Step B) are dropped from output

---

## Classification Pipeline (for Agent Developers)

Understanding how skills are classified helps agents interpret routing confidence:

```
 SKILL.md (8000+ across 100+ repos)
        │
  ┌─────┴─────┐
  │ YAML Parse │  Extract name, description, triggers
  └─────┬─────┘
        │
  ┌─────┴─────┐
  │   Dedup    │  By name OR description (catches cross-repo clones)
  └─────┬─────┘
        │
  ┌─────┴────────────────────────┐
  │ Hybrid Exclusion + Classify  │
  │ Step A: Keyword pre-filter   │  Fast, free, catches obvious junk
  │ Step B: LLM semantic route    │  Can also return hub="excluded"
  └─────┬────────────────────────┘
        │
  ┌─────┴─────┐
  │  Output   │  routing.csv, per-hub manifests
  └───────────┘
```

### Environment Variables (for LLM classification)

| Variable | Description |
|---|---|
| `LLM_ENABLED` | Set `false` to disable LLM and use keyword-only routing |
| `LLM_PROVIDER` | `gemini`, `openai`, or `mock` |
| `LLM_API_KEY` | API key for the configured provider |
| `LLM_CACHE_PATH` | Override path for persistent classification cache |
| `SKILL_MANAGE_EXCLUSIONS` | Semicolon-separated category exclusion overrides |
