---
name: server-side
description: |
  Router for server-side skills (211 skills across 8 sub-hubs).
  DO NOT execute from this file. Follow the steps below to load the real skill.
---

# server-side Hub

## Sub-Hubs

| Sub-Hub | Skills | Routing |
|---------|--------|---------|
| architect | 84 | architect/routing.csv |
| caching | 14 | caching/routing.csv |
| containers | 17 | containers/routing.csv |
| databases | 35 | databases/routing.csv |
| frameworks | 1 | frameworks/routing.csv |
| messaging | 5 | messaging/routing.csv |
| observability | 45 | observability/routing.csv |
| serverless-edge | 10 | serverless-edge/routing.csv |

## How To Use

1. Match user request to a sub-hub from the table above.
2. Open `<sub_hub>/routing.csv` in this directory.
3. Find the `skill_id` row whose `description` best matches the task.
4. Read the full skill from the `src_path` column (the SKILL.md in `lib/`).
5. Follow that SKILL.md as the source of truth.

## Anti-Hallucination

- NEVER guess skill behavior from description alone.
- ALWAYS load the actual SKILL.md from src_path before acting.
- If ambiguous, present top 3 candidates to the user.
