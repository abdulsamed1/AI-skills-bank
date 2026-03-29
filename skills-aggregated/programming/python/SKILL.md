---
name: python
description: |
  Auto-generated router for programming/python.

  Skills: 102
  Generated: 2026-03-29 22:25:26+02:00
metadata:
    version: '2.0'
---

# programming / python Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 102
- Phase distribution: P1=5, P2=5, P3=53, P4=39
- Top triggers: py, azure, python, ai, pro, storage, fastapi, mgmt, monitor, agents

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| dbos + python | dbos-python | 100 | 1 | true | none | [A1] |
| julia + pro | julia-pro | 100 | 1 | true | none | [A2] |
| n8n + code | n8n-code-python | 100 | 1 | true | none | [A3] |
| polars | polars | 100 | 1 | true | none | [A4] |
| uv + package | uv-package-manager | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 102 skills with src_path resolution.
