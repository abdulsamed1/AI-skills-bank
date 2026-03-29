---
name: llm-agents
description: |
  Auto-generated router for ai/llm-agents.

  Skills: 342
  Generated: 2026-03-29 22:25:24+02:00
metadata:
    version: '2.0'
---

# ai / llm-agents Skill Router

## Critical Instructions

1. Read routing.csv in this directory.
2. Match task or sub-task against triggers column.
3. For simple tasks: pick the highest score match.
4. For story/epic tasks: pick the top 2-3 non-overlapping skills that cover different sub-problems.
5. Use src_path to load each selected skill.
6. Never invent a skill_id or guess a path.

## Hub Snapshot

- Total skills: 342
- Phase distribution: P1=14, P2=1, P3=94, P4=233
- Top triggers: agent, ai, customaize, skill, claude, code, llm, mcp, agents, face

## Quick Intent Matcher (top 5 by score)

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| ai + agent | ai-agent-development | 100 | 1 | true | none | [A1] |
| amazon + alexa | amazon-alexa | 100 | 1 | true | none | [A2] |
| autonomous + agent | autonomous-agent-patterns | 100 | 1 | true | none | [A3] |
| claude + api | claude-api | 100 | 1 | true | none | [A4] |
| claude + code | claude-code-expert | 100 | 1 | true | none | [A5] |

## Full Catalog

See routing.csv for all 342 skills with src_path resolution.
