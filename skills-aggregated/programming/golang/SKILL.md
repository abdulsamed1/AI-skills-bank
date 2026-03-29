---
name: golang
description: |
  Auto-generated router for programming/golang.

  Skills: 8
  Generated: 2026-03-29 22:25:26+02:00
metadata:
    version: '2.0'
---

# programming / golang Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 8
- Phase distribution: P1=0, P2=3, P3=3, P4=2
- Top triggers: golang, pro, analyzing, concurrency, database, dbos, gene, ghidra, go, grpc

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| dbos + golang | dbos-golang | 20 | 2 | false | none | [A1] |
| golang + pro | golang-pro | 20 | 2 | false | none | [A2] |
| grpc + golang | grpc-golang | 20 | 2 | false | none | [A3] |
| analyzing + golang | analyzing-golang-malware-with-ghidra | 16 | 3 | false | none | [A4] |
| temporal + golang | temporal-golang-pro | 16 | 3 | false | none | [A5] |

## Full Catalog

See routing.csv for all 8 skills with src_path resolution.
