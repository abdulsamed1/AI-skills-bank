---
name: databases
description: |
  Auto-generated router for backend/databases.

  Skills: 80
  Generated: 2026-03-29 22:25:24+02:00
metadata:
    version: '2.0'
---

# backend / databases Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 80
- Phase distribution: P1=10, P2=1, P3=18, P4=51
- Top triggers: database, sql, performing, postgres, injection, azure, data, exploiting, migration, netlify

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| attach + db | attach-db | 100 | 1 | true | none | [A1] |
| database + admin | database-admin | 100 | 1 | true | none | [A2] |
| data + engineering | data-engineering-data-driven-feature | 100 | 1 | true | none | [A3] |
| data + quality | data-quality-frameworks | 100 | 1 | true | none | [A4] |
| dbt + transformation | dbt-transformation-patterns | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 80 skills with src_path resolution.
