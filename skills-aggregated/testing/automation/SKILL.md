---
name: automation
description: |
  Auto-generated router for testing/automation.

  Skills: 141
  Generated: 2026-03-29 22:25:26+02:00
metadata:
    version: '2.0'
---

# testing / automation Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 141
- Phase distribution: P1=11, P2=4, P3=30, P4=96
- Top triggers: testing, performing, test, for, penetration, tdd, api, playwright, vulnerabilities, with

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| content + experimentation | content-experimentation-best-practices | 100 | 1 | true | none | [A1] |
| debugging + strategies | debugging-strategies | 100 | 1 | true | none | [A2] |
| differential + review | differential-review | 100 | 1 | true | none | [A3] |
| find + bugs | find-bugs | 100 | 1 | true | none | [A4] |
| iterate + pr | iterate-pr | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 141 skills with src_path resolution.
