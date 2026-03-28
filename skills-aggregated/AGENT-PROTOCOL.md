# Agent Skill Invocation Protocol v2.0

## MANDATORY 3-STEP FLOW — NEVER SKIP

### STEP 1: keyword → hub (cost: ~50 tokens)


Map to hub/sub_hub.

- Single object = unique match → proceed to Step 2
- Array = multiple candidates → pick first unless user context suggests otherwise
- Not found → FALLBACK (see below)

### STEP 2: skill lookup from routing.csv (cost: ~30-80 tokens)

Read `skills-aggregated/{hub}/{sub_hub}/routing.csv`.
This file contains ONLY: skill_id, triggers, score, src_path.

Match user intent against `triggers` column.
Pick highest `score` match.

- If multiple triggers match → pick highest score
- If ambiguous → present top 3 candidates and ask user to choose
- If no match → try next candidate from Step 1 (if array)

Result: skill_id + src_path.

### STEP 3: load skill content

Read `{hub_mount_path}/{src_path}` (where `{hub_mount_path}` is the directory containing `routing.csv`).

- If src_path is empty → HALT, report to user
- NEVER guess a file path — use ONLY what routing.csv provides
- If file does not exist → HALT, report to user

## ANTI-HALLUCINATION GATES

- **Gate 1:** skill_id must exist in routing.csv before invocation
- **Gate 2:** hub must exist in subhub-index.json
- **Gate 3:** match_score must be >= 10
- **Gate 4:** Never combine skills from different hubs in one request
- **Gate 5:** Never invent skill_id values — use ONLY what routing.csv contains
- **Gate 6:** Never guess src_path — use ONLY what routing.csv contains

## FALLBACK (when subhub-index has no match)

1. Read `subhub-index.json` for fuzzy trigger matching
2. If still no match → ask user for clarification
3. NEVER read full SKILL.md or full CSV as a fallback

## TOKEN BUDGET PER INVOCATION

| Step | Source | Cost |
|------|--------|------|
| 1 | subhub-index.json | ~50 tokens |
| 2 | routing.csv (filtered) | ~30-80 tokens |
| 3 | src_path resolution | ~20 tokens |
| **Total** | | **< 150 tokens** |

## FILE READING ORDER

```
✅ ALWAYS START:  subhub-index.json (10KB)
        {hub}/{sub_hub}/routing.csv (1-15KB)
✅ THEN LOAD:     src_path from routing.csv row
✅ FALLBACK:      subhub-index.json (17KB)
❌ NEVER READ:    hub-manifests.csv (572KB) — build source only
❌ NEVER FIRST:   SKILL.md router (14KB full router)
```

## EXAMPLES

### Example 1: Simple routing
```
User: "I need an API for my backend"
Step 1: keyword "api" → subhub-index → {"hub":"backend","sub_hub":"api-design"}
Step 2: read backend/api-design/routing.csv → match "api" trigger
        → skill_id="api-documentation", score=100
        → src_path="skills/api-documentation/SKILL.md"
Step 3: read backend/api-design/skills/api-documentation/SKILL.md
```

### Example 2: Ambiguous keyword
```
User: "I need help with agents"
Step 1: keyword "agents" → subhub-index → [ai/llm-agents, programming/python, programming/java]
        Context says "agents" likely means AI → pick ai/llm-agents
Step 2: read ai/llm-agents/routing.csv → match "agent" trigger
        → skill_id="ai-agent-development", score=100
        → src_path="skills/ai-agent-development/SKILL.md"
Step 3: read the resolved src_path
```

### Example 3: No match
```
User: "I need help with blockchain"
Step 1: keyword "blockchain" → not in subhub-index.json
Fallback: read hub-manifests.csv → no trigger match
Action: ask user for clarification. DO NOT hallucinate a skill.
```
