---
name: brand-guidelines
description: |
  Auto-generated router for design/brand-guidelines.

  Skills: 3
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# design / brand-guidelines Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 3
- Phase distribution: P1=3, P2=0, P3=0, P4=0
- Top triggers: brand, guidelines, anthropic, community

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| brand + guidelines | brand-guidelines | 100 | 1 | true | none | [A1] |
| brand + guidelines | brand-guidelines-anthropic | 100 | 1 | true | none | [A2] |
| brand + guidelines | brand-guidelines-community | 100 | 1 | true | none | [A3] |

## Full Catalog

See routing.csv for all 3 skills with src_path resolution.
