---
name: skills-factory
description: |
  Auto-generated router for ai/skills-factory.

  Skills: 23
  Generated: 2026-03-29 22:25:24+02:00
metadata:
    version: '2.0'
---

# ai / skills-factory Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 23
- Phase distribution: P1=20, P2=0, P3=1, P4=2
- Top triggers: skill, creator, skills, agent, agents, and, antigravity, audit, authoring, before

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| agents + md | agents-md | 100 | 1 | true | none | [A1] |
| antigravity + skill | antigravity-skill-orchestrator | 100 | 1 | true | none | [A2] |
| audit + skills | audit-skills | 100 | 1 | true | none | [A3] |
| documentation + generation | documentation-generation-doc-generate | 100 | 1 | true | none | [A4] |
| hierarchical + agent | hierarchical-agent-memory | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 23 skills with src_path resolution.
