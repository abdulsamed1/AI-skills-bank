---
name: api-design
description: |
  Auto-generated router for backend/api-design.

  Skills: 76
  Required gates: 15
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# backend / api-design Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=backend and sub_hub=api-design.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 76
- Required skills: 15
- Phase distribution: P1=15, P2=0, P3=7, P4=54
- Top triggers: api, odoo, patterns, architect, architecture, code, backend, builder, ddd, design

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| api + documentation | api-documentation | 100 | 1 | true | none | [A1] |
| architecture + patterns | architecture-patterns | 100 | 1 | true | none | [A2] |
| comfyui + gateway | comfyui-gateway | 100 | 1 | true | none | [A3] |
| creem | creem | 100 | 1 | true | none | [A4] |
| creem + heartbeat | creem-heartbeat | 100 | 1 | true | none | [A5] |

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
