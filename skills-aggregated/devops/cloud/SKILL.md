---
name: cloud
description: |
  Auto-generated router for devops/cloud.

  Skills: 93
  Required gates: 16
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# devops / cloud Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=devops and sub_hub=cloud.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 93
- Required skills: 16
- Phase distribution: P1=16, P2=7, P3=19, P4=51
- Top triggers: azure, ts, error, aws, cloud, deployment, observability, patterns, terraform, automation

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| azure + keyvault | azure-keyvault-keys-ts | 100 | 1 | true | none | [A1] |
| azure + keyvault | azure-keyvault-secrets-ts | 100 | 1 | true | none | [A2] |
| changelog + automation | changelog-automation | 100 | 1 | true | none | [A3] |
| cicd + automation | cicd-automation-workflow-automate | 100 | 1 | true | none | [A4] |
| cloudformation + best | cloudformation-best-practices | 100 | 1 | true | none | [A5] |

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
