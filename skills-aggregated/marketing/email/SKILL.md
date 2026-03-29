---
name: email
description: |
  Auto-generated router for marketing/email.

  Skills: 16
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# marketing / email Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 16
- Phase distribution: P1=2, P2=0, P3=2, P4=12
- Top triggers: email, detecting, compromise, phishing, business, with, account, analyzing, attack, automation

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| churn + prevention | churn-prevention | 100 | 1 | true | none | [A1] |
| email + sequence | email-sequence | 100 | 1 | true | none | [A2] |
| email + marketing | email-marketing-bible | 14 | 3 | false | none | [A3] |
| mailchimp + automation | mailchimp-automation | 13 | 3 | false | none | [A4] |
| newsletter + creation | newsletter-creation-curation | 7 | 4 | false | none | [A5] |

## Full Catalog

See routing.csv for all 16 skills with src_path resolution.
