---
name: product-strategy
description: |
  Auto-generated router for business/product-strategy.

  Skills: 131
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# business / product-strategy Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 131
- Phase distribution: P1=18, P2=6, P3=37, P4=70
- Top triggers: product, prd, strategy, advisor, brainstorm, user, app, optimization, to, workshop

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| acquisition + channel | acquisition-channel-advisor | 100 | 1 | true | none | [A1] |
| app + analytics | app-analytics | 100 | 1 | true | none | [A2] |
| app + icon | app-icon-optimization | 100 | 1 | true | none | [A3] |
| brainstorm + experiments | brainstorm-experiments-existing | 100 | 1 | true | none | [A4] |
| dummy + dataset | dummy-dataset | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 131 skills with src_path resolution.
