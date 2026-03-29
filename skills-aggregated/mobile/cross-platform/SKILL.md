---
name: cross-platform
description: |
  Auto-generated router for mobile/cross-platform.

  Skills: 64
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# mobile / cross-platform Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 64
- Phase distribution: P1=5, P2=0, P3=11, P4=48
- Top triggers: app, mobile, with, performing, android, expo, expert, swift, asc, ios

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| app + clips | app-clips | 100 | 1 | true | none | [A1] |
| app + store | app-store-changelog | 100 | 1 | true | none | [A2] |
| crash + analytics | crash-analytics | 100 | 1 | true | none | [A3] |
| kotlin + coroutines | kotlin-coroutines-expert | 100 | 1 | true | none | [A4] |
| native + data | native-data-fetching | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 64 skills with src_path resolution.
