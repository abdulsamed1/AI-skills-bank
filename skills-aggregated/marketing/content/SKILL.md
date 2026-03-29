---
name: content
description: |
  Auto-generated router for marketing/content.

  Skills: 65
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# marketing / content Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 65
- Phase distribution: P1=22, P2=0, P3=22, P4=21
- Top triggers: seo, content, ai, cro, audit, writing, competitor, fundamentals, geo, images

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| ad + creative | ad-creative | 100 | 1 | true | none | [A1] |
| ai + discoverability | ai-discoverability-audit | 100 | 1 | true | none | [A2] |
| beautiful + prose | beautiful-prose | 100 | 1 | true | none | [A3] |
| competitor + alternatives | competitor-alternatives | 100 | 1 | true | none | [A4] |
| content + strategy | content-strategy | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 65 skills with src_path resolution.
