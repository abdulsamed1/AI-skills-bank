---
name: frontend
description: |
  Router for frontend skills (2 sub-hubs).
  DO NOT execute from this file. Follow the steps below to load the real skill.
---

# frontend Hub

## Sub-Hubs

| Sub-Hub | Routing |
|---------|---------|
| ui-ux | ui-ux/routing.csv |
| web-frameworks | web-frameworks/routing.csv |

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
