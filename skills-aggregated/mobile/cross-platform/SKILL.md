---
name: cross-platform
description: |
  Auto-generated router for mobile/cross-platform.

  Skills: 27
  Required gates: 5
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# mobile / cross-platform Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=mobile and sub_hub=cross-platform.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 27
- Required skills: 5
- Phase distribution: P1=5, P2=0, P3=7, P4=15
- Top triggers: expo, app, expert, mobile, android, developer, development, macos, native, store

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| app + clips | app-clips | 100 | 1 | true | none | [A1] |
| app + store | app-store-changelog | 100 | 1 | true | none | [A2] |
| crash + analytics | crash-analytics | 100 | 1 | true | none | [A3] |
| kotlin + coroutines | kotlin-coroutines-expert | 100 | 1 | true | none | [A4] |
| native + data | native-data-fetching | 100 | 1 | true | none | [A5] |

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
