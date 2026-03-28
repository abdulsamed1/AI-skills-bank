# Agent Skill Invocation Protocol v2.0

## MANDATORY 3-STEP FLOW — NEVER SKIP

### STEP 1: keyword → hub (cost: ~50 tokens)

Read `quick-index.json` ONLY.
Extract 1–3 keywords from user request.
Map to hub/sub_hub.

- Single object = unique match → proceed to Step 2
- Array = multiple candidates → pick first unless user context suggests otherwise
- Not found → FALLBACK (see below)

### STEP 2: skill lookup from routing.tsv (cost: ~30-80 tokens)

Read `skills-aggregated/{hub}/{sub_hub}/routing.tsv`.
This file contains ONLY: skill_id, triggers, score, src_path.

Match user intent against `triggers` column.
Pick highest `score` match.

- If multiple triggers match → pick highest score
- If ambiguous → present top 3 candidates and ask user to choose
- If no match → try next candidate from Step 1 (if array)

Result: skill_id + src_path.

### STEP 3: load skill content

Read `{project-root}/skill manage/{src_path}` from the routing.tsv row.

- If src_path is empty → HALT, report to user
- NEVER guess a file path — use ONLY what routing.tsv provides
- If file does not exist → HALT, report to user

## ANTI-HALLUCINATION GATES

- **Gate 1:** skill_id must exist in routing.tsv before invocation
- **Gate 2:** hub must exist in quick-index.json or subhub-index.json
- **Gate 3:** match_score must be >= 10
- **Gate 4:** Never combine skills from different hubs in one request
- **Gate 5:** Never invent skill_id values — use ONLY what routing.tsv contains
- **Gate 6:** Never guess src_path — use ONLY what routing.tsv contains

## FALLBACK (when quick-index has no match)

1. Read `subhub-index.json` for fuzzy trigger matching
2. If still no match → ask user for clarification
3. NEVER read full SKILL.md or full CSV as a fallback

## TOKEN BUDGET PER INVOCATION

| Step | Source | Cost |
|------|--------|------|
| 1 | quick-index.json | ~50 tokens |
| 2 | routing.tsv (filtered) | ~30-80 tokens |
| 3 | src_path resolution | ~20 tokens |
| **Total** | | **< 150 tokens** |

## FILE READING ORDER

```
✅ ALWAYS START:  quick-index.json (10KB)
✅ THEN READ:     {hub}/{sub_hub}/routing.tsv (1-15KB)
✅ THEN LOAD:     src_path from routing.tsv row
✅ FALLBACK:      subhub-index.json (17KB)
❌ NEVER READ:    hub-manifests.csv (572KB) — build source only
❌ NEVER FIRST:   SKILL.md router (14KB full router)
```

## EXAMPLES

### Example 1: Simple routing
```
User: "I need an API for my backend"
Step 1: keyword "api" → quick-index → {"hub":"backend","sub_hub":"api-design"}
Step 2: read backend/api-design/routing.tsv → match "api" trigger
        → skill_id="api-documentation", score=100
        → src_path="src/antigravity-awesome-skills/skills/api-documentation/SKILL.md"
Step 3: read {project-root}/skill manage/src/antigravity-awesome-skills/skills/api-documentation/SKILL.md
```

### Example 2: Ambiguous keyword
```
User: "I need help with agents"
Step 1: keyword "agents" → quick-index → [ai/llm-agents, programming/python, programming/java]
        Context says "agents" likely means AI → pick ai/llm-agents
Step 2: read ai/llm-agents/routing.tsv → match "agent" trigger
        → skill_id="ai-agent-development", score=100
        → src_path="src/antigravity-awesome-skills/skills/ai-agent-development/SKILL.md"
Step 3: read the resolved src_path
```

### Example 3: No match
```
User: "I need help with blockchain"
Step 1: keyword "blockchain" → not in quick-index
Fallback: read subhub-index.json → no trigger match
Action: ask user for clarification. DO NOT hallucinate a skill.
```
