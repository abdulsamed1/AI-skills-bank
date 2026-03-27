---
name: strategy
description: |
  Auto-generated router for marketing/strategy.

  Skills: 19
  Required gates: 8
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# marketing / strategy Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=marketing and sub_hub=strategy.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 19
- Required skills: 8
- Phase distribution: P1=8, P2=0, P3=7, P4=4
- Top triggers: competitor, marketing, positioning, strategy, competitive, context, gtm, ideas, analysis, app

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| competitor + tracking | competitor-tracking | 100 | 1 | true | none | [A1] |
| gtm + motions | gtm-motions | 100 | 1 | true | none | [A2] |
| gtm + strategy | gtm-strategy | 100 | 1 | true | none | [A3] |
| marketing + ideas | marketing-ideas | 100 | 1 | true | none | [A4] |
| market + movers | market-movers | 100 | 1 | true | none | [A5] |

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
