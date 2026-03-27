---
name: ui-ux
description: |
  Auto-generated router for design/ui-ux.

  Skills: 93
  Required gates: 25
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# design / ui-ux Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=design and sub_hub=ui-ux.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 93
- Required skills: 25
- Phase distribution: P1=25, P2=2, P3=26, P4=40
- Top triggers: design, ui, hig, threejs, components, figma, ux, patterns, system, audit

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| animejs + animation | animejs-animation | 100 | 1 | true | none | [A1] |
| c4 + architecture | c4-architecture-c4-architecture | 100 | 1 | true | none | [A2] |
| c4 + component | c4-component | 100 | 1 | true | none | [A3] |
| canvas + design | canvas-design | 100 | 1 | true | none | [A4] |
| claude + d3js | claude-d3js-skill | 100 | 1 | true | none | [A5] |

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
