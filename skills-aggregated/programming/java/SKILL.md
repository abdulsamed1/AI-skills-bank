---
name: java
description: |
  Auto-generated router for programming/java.

  Skills: 64
  Required gates: 1
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# programming / java Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=programming and sub_hub=java.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 64
- Required skills: 1
- Phase distribution: P1=1, P2=0, P3=27, P4=36
- Top triggers: azure, java, dotnet, ai, pro, communication, manager, mgmt, resource, agents

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| csharp + pro | csharp-pro | 100 | 1 | true | none | [A1] |
| java + pro | java-pro | 16 | 3 | false | none | [A2] |
| azure + ai | azure-ai-agents-persistent-java | 12 | 3 | false | none | [A3] |
| azure + ai | azure-ai-anomalydetector-java | 12 | 3 | false | none | [A4] |
| azure + ai | azure-ai-contentsafety-java | 12 | 3 | false | none | [A5] |

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
