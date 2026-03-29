---
name: social
description: |
  Auto-generated router for marketing/social.

  Skills: 19
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# marketing / social Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 19
- Phase distribution: P1=2, P2=2, P3=5, P4=10
- Top triggers: apify, social, x, analysis, article, content, publisher, ads, analytics, audience

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| instagram | instagram | 100 | 1 | true | none | [A1] |
| paid + ads | paid-ads | 100 | 1 | true | none | [A2] |
| social + card | social-card-gen | 22 | 2 | false | none | [A3] |
| ckm:design | ckm:design | 21 | 2 | false | none | [A4] |
| download + video | download-video | 17 | 3 | false | none | [A5] |

## Full Catalog

See routing.csv for all 19 skills with src_path resolution.
