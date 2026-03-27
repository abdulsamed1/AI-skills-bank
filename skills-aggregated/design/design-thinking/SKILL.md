---
name: design-thinking
description: |
  Auto-generated router for design/design-thinking.

  Skills: 2
  Required gates: 0
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# design / design-thinking Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=design and sub_hub=design-thinking.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 2
- Required skills: 0
- Phase distribution: P1=0, P2=0, P3=2, P4=0
- Top triggers: bmad, cis, design, thinking, agent

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| bmad + cis | bmad-cis-agent-design-thinking-coach | 10 | 3 | false | none | [A1] |
| bmad + cis | bmad-cis-design-thinking | 10 | 3 | false | none | [A2] |

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
