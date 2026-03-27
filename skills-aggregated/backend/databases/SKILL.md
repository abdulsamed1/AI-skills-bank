---
name: databases
description: |
  Auto-generated router for backend/databases.

  Skills: 46
  Required gates: 10
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# backend / databases Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=backend and sub_hub=databases.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 46
- Required skills: 10
- Phase distribution: P1=10, P2=1, P3=11, P4=24
- Top triggers: database, data, postgres, sql, netlify, optimization, patterns, postgresql, best, db

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| attach + db | attach-db | 100 | 1 | true | none | [A1] |
| database + admin | database-admin | 100 | 1 | true | none | [A2] |
| data + engineering | data-engineering-data-driven-feature | 100 | 1 | true | none | [A3] |
| data + quality | data-quality-frameworks | 100 | 1 | true | none | [A4] |
| dbt + transformation | dbt-transformation-patterns | 100 | 1 | true | none | [A5] |

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
