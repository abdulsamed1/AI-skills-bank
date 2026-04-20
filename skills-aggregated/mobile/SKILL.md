---
name: mobile
description: |
  Router for mobile skills (50 skills across 3 sub-hubs).
  DO NOT execute from this file. Follow the steps below to load the real skill.
---

# mobile Hub

## Sub-Hubs

| Sub-Hub | Skills | Routing |
|---------|--------|---------|
| android | 1 | android/routing.csv |
| cross-platform | 3 | cross-platform/routing.csv |
| ios | 46 | ios/routing.csv |

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
