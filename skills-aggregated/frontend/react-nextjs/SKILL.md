---
name: react-nextjs
description: |
  Auto-generated router for frontend/react-nextjs.

  Skills: 24
  Required gates: 4
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# frontend / react-nextjs Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=frontend and sub_hub=react-nextjs.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 24
- Required skills: 4
- Phase distribution: P1=4, P2=0, P3=3, P4=17
- Top triggers: react, nextjs, best, practices, auth, expert, fp, patterns, 3d, ai

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| clerk + auth | clerk-auth | 100 | 1 | true | none | [A1] |
| nextjs + supabase | nextjs-supabase-auth | 100 | 1 | true | none | [A2] |
| sveltekit | sveltekit | 100 | 1 | true | none | [A3] |
| web + artifacts | web-artifacts-builder | 100 | 1 | true | none | [A4] |
| netlify + frameworks | netlify-frameworks | 14 | 3 | false | none | [A5] |

## Selection Rules

1. Select candidates with score >= 10.
2. If multiple candidates remain, sort by score descending.
3. If the best candidate has required=true and unmet dependency (after), block and explain the prerequisite.
4. If user intent is ambiguous, present top 3 candidates and ask user to choose.

## Dependency Rules

- after: prerequisite skill IDs that must be completed first.
- before: reverse dependency links for planning only.
- required=true: blocking gate for progression.

## Output Format

When proposing a skill, respond with:
- skill_id
- reason (trigger + score)
- phase
- required
- dependencies (after)
- next step

## Verification Checklist

- Skill exists in hub-manifests.csv
- Trigger overlap is explicit
- Score is >= 10
- Dependency gates are respected
- No hallucinated IDs
