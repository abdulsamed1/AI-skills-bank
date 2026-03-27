---
name: skills-factory
description: |
  Auto-generated router for ai/skills-factory.

  Skills: 21
  Required gates: 20
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# ai / skills-factory Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=ai and sub_hub=skills-factory.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 21
- Required skills: 20
- Phase distribution: P1=20, P2=0, P3=0, P4=1
- Top triggers: skill, creator, skills, agent, agents, and, antigravity, audit, authoring, before

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| agents + md | agents-md | 100 | 1 | true | none | [A1] |
| antigravity + skill | antigravity-skill-orchestrator | 100 | 1 | true | none | [A2] |
| audit + skills | audit-skills | 100 | 1 | true | none | [A3] |
| documentation + generation | documentation-generation-doc-generate | 100 | 1 | true | none | [A4] |
| hierarchical + agent | hierarchical-agent-memory | 100 | 1 | true | none | [A5] |

## Selection Rules

1. Select candidates with score >= 10.
2. If multiple candidates remain, sort by score descending.
3. If the best candidate has required=true and unmet dependency (after), block and explain the prerequisite.
4. If user intent is ambiguous, present top 3 candidates and ask user to choose.

## Dependency Rules

- after: prerequisite skill IDs that must be completed first.
- before: reverse dependency links for planning only.
- required=true: blocking gate for progression.

## Output Format

When proposing a skill, respond with:
- skill_id
- reason (trigger + score)
- phase
- required
- dependencies (after)
- next step

## Verification Checklist

- Skill exists in hub-manifests.csv
- Trigger overlap is explicit
- Score is >= 10
- Dependency gates are respected
- No hallucinated IDs
