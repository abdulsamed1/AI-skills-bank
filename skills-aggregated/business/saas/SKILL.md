---
name: saas
description: |
  Auto-generated router for business/saas.

  Skills: 42
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# business / saas Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 42
- Phase distribution: P1=16, P2=2, P3=9, P4=15
- Top triggers: metrics, saas, startup, market, analyst, business, integration, odoo, analysis, finance

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| billing + automation | billing-automation | 100 | 1 | true | none | [A1] |
| carrier + relationship | carrier-relationship-management | 100 | 1 | true | none | [A2] |
| hubspot + integration | hubspot-integration | 100 | 1 | true | none | [A3] |
| logistics + exception | logistics-exception-management | 100 | 1 | true | none | [A4] |
| micro + saas | micro-saas-launcher | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 42 skills with src_path resolution.
