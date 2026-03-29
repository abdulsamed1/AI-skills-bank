---
name: docker-k8s
description: |
  Auto-generated router for devops/docker-k8s.

  Skills: 26
  Generated: 2026-03-29 22:25:25+02:00
metadata:
    version: '2.0'
---

# devops / docker-k8s Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 26
- Phase distribution: P1=4, P2=1, P3=2, P4=19
- Top triggers: container, implementing, kubernetes, for, policies, security, with, analyzing, docker, expert

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| istio + traffic | istio-traffic-management | 100 | 1 | true | none | [A1] |
| linkerd + patterns | linkerd-patterns | 100 | 1 | true | none | [A2] |
| service + mesh | service-mesh-expert | 100 | 1 | true | none | [A3] |
| windows + vm | windows-vm | 100 | 1 | true | none | [A4] |
| kubernetes + deployment | kubernetes-deployment | 20 | 2 | false | none | [A5] |

## Full Catalog

See routing.csv for all 26 skills with src_path resolution.
