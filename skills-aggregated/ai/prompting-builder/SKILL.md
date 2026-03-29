---
name: prompting-builder
description: |
  Auto-generated router for ai/prompting-builder.

  Skills: 13
  Generated: 2026-03-29 22:25:24+02:00
metadata:
    version: '2.0'
---

# ai / prompting-builder Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 13
- Phase distribution: P1=12, P2=0, P3=0, P4=1
- Top triggers: prompt, context, engineering, llm, advisor, application, caching, compression, dev, engineer

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| context + compression | context-compression | 100 | 1 | true | none | [A1] |
| context + engineering | context-engineering-advisor | 100 | 1 | true | none | [A2] |
| enhance + prompt | enhance-prompt | 100 | 1 | true | none | [A3] |
| llm + application | llm-application-dev-prompt-optimize | 100 | 1 | true | none | [A4] |
| llm + prompt | llm-prompt-optimizer | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 13 skills with src_path resolution.
