---
name: workflow-automation
description: |
  Auto-generated router for productivity/workflow-automation.

  Skills: 976
  Generated: 2026-03-29 22:25:26+02:00
metadata:
    version: '2.0'
---

# productivity / workflow-automation Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 976
- Phase distribution: P1=18, P2=2, P3=152, P4=804
- Top triggers: with, automation, implementing, performing, for, detecting, analyzing, review, building, network

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| airflow + dag | airflow-dag-patterns | 100 | 1 | true | none | [A1] |
| apify + ultimate | apify-ultimate-scraper | 100 | 1 | true | none | [A2] |
| brainstorming | brainstorming | 100 | 1 | true | none | [A3] |
| claude + monitor | claude-monitor | 100 | 1 | true | none | [A4] |
| code + documentation | code-documentation-code-explain | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 976 skills with src_path resolution.
