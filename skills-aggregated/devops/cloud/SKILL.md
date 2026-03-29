---
name: cloud
description: |
  Auto-generated router for devops/cloud.

  Skills: 248
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# devops / cloud Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 248
- Phase distribution: P1=16, P2=11, P3=35, P4=186
- Top triggers: azure, for, cloud, implementing, with, aws, detecting, analyzing, hunting, performing

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| azure + keyvault | azure-keyvault-keys-ts | 100 | 1 | true | none | [A1] |
| azure + keyvault | azure-keyvault-secrets-ts | 100 | 1 | true | none | [A2] |
| changelog + automation | changelog-automation | 100 | 1 | true | none | [A3] |
| cicd + automation | cicd-automation-workflow-automate | 100 | 1 | true | none | [A4] |
| cloudformation + best | cloudformation-best-practices | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 248 skills with src_path resolution.
