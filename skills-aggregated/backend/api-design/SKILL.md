---
name: api-design
description: |
  Auto-generated router for backend/api-design.

  Skills: 136
  Generated: 2026-03-29 22:25:24+02:00
metadata:
    version: '2.0'
---

# backend / api-design Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 136
- Phase distribution: P1=15, P2=0, P3=11, P4=110
- Top triggers: api, odoo, implementing, patterns, with, architecture, performing, database, architect, code

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| api + documentation | api-documentation | 100 | 1 | true | none | [A1] |
| architecture + patterns | architecture-patterns | 100 | 1 | true | none | [A2] |
| comfyui + gateway | comfyui-gateway | 100 | 1 | true | none | [A3] |
| creem | creem | 100 | 1 | true | none | [A4] |
| creem + heartbeat | creem-heartbeat | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 136 skills with src_path resolution.
