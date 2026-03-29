---
name: strategy
description: |
  Auto-generated router for marketing/strategy.

  Skills: 19
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# marketing / strategy Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 19
- Phase distribution: P1=8, P2=0, P3=7, P4=4
- Top triggers: competitor, marketing, positioning, strategy, competitive, context, gtm, ideas, analysis, app

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| competitor + tracking | competitor-tracking | 100 | 1 | true | none | [A1] |
| gtm + motions | gtm-motions | 100 | 1 | true | none | [A2] |
| gtm + strategy | gtm-strategy | 100 | 1 | true | none | [A3] |
| marketing + ideas | marketing-ideas | 100 | 1 | true | none | [A4] |
| market + movers | market-movers | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 19 skills with src_path resolution.
