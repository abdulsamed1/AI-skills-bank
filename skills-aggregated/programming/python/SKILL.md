---
name: python
description: |
  Auto-generated router for programming/python.

  Skills: 76
  Required gates: 5
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# programming / python Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=programming and sub_hub=python.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 76
- Required skills: 5
- Phase distribution: P1=5, P2=5, P3=40, P4=26
- Top triggers: py, azure, python, ai, pro, fastapi, mgmt, monitor, storage, agents

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| dbos + python | dbos-python | 100 | 1 | true | none | [A1] |
| julia + pro | julia-pro | 100 | 1 | true | none | [A2] |
| n8n + code | n8n-code-python | 100 | 1 | true | none | [A3] |
| polars | polars | 100 | 1 | true | none | [A4] |
| uv + package | uv-package-manager | 100 | 1 | true | none | [A5] |

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
