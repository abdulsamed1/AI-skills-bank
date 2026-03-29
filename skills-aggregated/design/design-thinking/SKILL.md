---
name: design-thinking
description: |
  Auto-generated router for design/design-thinking.

  Skills: 2
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# design / design-thinking Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 2
- Phase distribution: P1=0, P2=0, P3=2, P4=0
- Top triggers: bmad, cis, design, thinking, agent

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| bmad + cis | bmad-cis-agent-design-thinking-coach | 10 | 3 | false | none | [A1] |
| bmad + cis | bmad-cis-design-thinking | 10 | 3 | false | none | [A2] |

## Full Catalog

See routing.csv for all 2 skills with src_path resolution.
