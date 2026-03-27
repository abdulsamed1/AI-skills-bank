---
name: email
description: |
  Auto-generated router for marketing/email.

  Skills: 8
  Required gates: 2
  Generated: 2026-03-27 19:10:50+02:00
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# marketing / email Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=marketing and sub_hub=email.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: 8
- Required skills: 2
- Phase distribution: P1=2, P2=0, P3=2, P4=4
- Top triggers: email, automation, bible, churn, cold, comms, creation, curation, internal, mailchimp

## Quick Intent Matcher

| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |
|---|---|---:|---:|---|---|---|
| churn + prevention | churn-prevention | 100 | 1 | true | none | [A1] |
| email + sequence | email-sequence | 100 | 1 | true | none | [A2] |
| email + marketing | email-marketing-bible | 14 | 3 | false | none | [A3] |
| mailchimp + automation | mailchimp-automation | 13 | 3 | false | none | [A4] |
| newsletter + creation | newsletter-creation-curation | 7 | 4 | false | none | [A5] |

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
