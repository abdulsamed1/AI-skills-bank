---
name: core
description: |
  Auto-generated router for security/core.

  Skills: 253
  Generated: 2026-03-29 22:25:26+02:00
metadata:
    version: '2.0'
---

# security / core Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 253
- Phase distribution: P1=22, P2=2, P3=65, P4=164
- Top triggers: security, with, performing, implementing, malware, analyzing, assessment, for, forensics, testing

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| active + directory | active-directory-attacks | 100 | 1 | true | none | [A1] |
| attack + tree | attack-tree-construction | 100 | 1 | true | none | [A2] |
| aws + penetration | aws-penetration-testing | 100 | 1 | true | none | [A3] |
| codebase + audit | codebase-audit-pre-push | 100 | 1 | true | none | [A4] |
| cred + omega | cred-omega | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 253 skills with src_path resolution.
