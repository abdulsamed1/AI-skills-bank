---
name: workflow-automation
description: |
  Auto-generated router for productivity/workflow-automation.

  Skills: 428
  Required gates: 19
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# productivity / workflow-automation Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=productivity and sub_hub=workflow-automation.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 428
- Required skills: 19
- Phase distribution: P1=19, P2=1, P3=121, P4=287
- Top triggers: automation, bmad, review, code, ai, context, builder, development, git, workflow

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| airflow + dag | airflow-dag-patterns | 100 | 1 | true | none | [A1] |
| apify + ultimate | apify-ultimate-scraper | 100 | 1 | true | none | [A2] |
| brainstorming | brainstorming | 100 | 1 | true | none | [A3] |
| claude + monitor | claude-monitor | 100 | 1 | true | none | [A4] |
| code + documentation | code-documentation-code-explain | 100 | 1 | true | none | [A5] |

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
