---
name: golang
description: |
  Auto-generated router for programming/golang.

  Skills: 5
  Required gates: 0
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# programming / golang Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=programming and sub_hub=golang.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 5
- Required skills: 0
- Phase distribution: P1=0, P2=3, P3=2, P4=0
- Top triggers: golang, pro, concurrency, dbos, go, grpc, patterns, temporal

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| dbos + golang | dbos-golang | 20 | 2 | false | none | [A1] |
| golang + pro | golang-pro | 20 | 2 | false | none | [A2] |
| grpc + golang | grpc-golang | 20 | 2 | false | none | [A3] |
| temporal + golang | temporal-golang-pro | 16 | 3 | false | none | [A4] |
| go + concurrency | go-concurrency-patterns | 15 | 3 | false | none | [A5] |

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
