---
name: java
description: |
  Auto-generated router for programming/java.

  Skills: 98
  Generated: 2026-03-29 22:25:26+02:00
metadata:
    version: '2.0'
---

# programming / java Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 98
- Phase distribution: P1=1, P2=0, P3=30, P4=67
- Top triggers: azure, dotnet, java, ai, mgmt, manager, pro, resource, communication, escalation

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| csharp + pro | csharp-pro | 100 | 1 | true | none | [A1] |
| java + pro | java-pro | 16 | 3 | false | none | [A2] |
| android + java | android-java | 12 | 3 | false | none | [A3] |
| azure + ai | azure-ai-agents-persistent-java | 12 | 3 | false | none | [A4] |
| azure + ai | azure-ai-anomalydetector-java | 12 | 3 | false | none | [A5] |

## Full Catalog

See routing.csv for all 98 skills with src_path resolution.
