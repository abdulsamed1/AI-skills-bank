---
name: docker-k8s
description: |
  Auto-generated router for devops/docker-k8s.

  Skills: 11
  Required gates: 4
  Generated: 2026-03-27 19:10:49+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# devops / docker-k8s Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=devops and sub_hub=docker-k8s.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 11
- Required skills: 4
- Phase distribution: P1=4, P2=1, P3=2, P4=4
- Top triggers: expert, k8s, kubernetes, architect, c4, chart, container, deployment, docker, generator

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| istio + traffic | istio-traffic-management | 100 | 1 | true | none | [A1] |
| linkerd + patterns | linkerd-patterns | 100 | 1 | true | none | [A2] |
| service + mesh | service-mesh-expert | 100 | 1 | true | none | [A3] |
| windows + vm | windows-vm | 100 | 1 | true | none | [A4] |
| kubernetes + deployment | kubernetes-deployment | 20 | 2 | false | none | [A5] |

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
