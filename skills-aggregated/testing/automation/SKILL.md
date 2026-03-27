---
name: automation
description: |
  Auto-generated router for testing/automation.

  Skills: 55
  Required gates: 11
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# testing / automation Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=testing and sub_hub=automation.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 55
- Required skills: 11
- Phase distribution: P1=11, P2=3, P3=18, P4=23
- Top triggers: testing, tdd, qa, test, bmad, e2e, workflows, debugging, patterns, playwright

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| content + experimentation | content-experimentation-best-practices | 100 | 1 | true | none | [A1] |
| debugging + strategies | debugging-strategies | 100 | 1 | true | none | [A2] |
| differential + review | differential-review | 100 | 1 | true | none | [A3] |
| find + bugs | find-bugs | 100 | 1 | true | none | [A4] |
| iterate + pr | iterate-pr | 100 | 1 | true | none | [A5] |

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
