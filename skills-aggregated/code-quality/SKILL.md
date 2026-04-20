---
name: code-quality
description: |
  Router for code-quality skills (1335 skills across 9 sub-hubs).
  DO NOT execute from this file. Follow the steps below to load the real skill.
---

# code-quality Hub

## Sub-Hubs

| Sub-Hub | Skills | Routing |
|---------|--------|---------|
| code-review | 25 | code-review/routing.csv |
| golang | 13 | golang/routing.csv |
| javascript | 46 | javascript/routing.csv |
| python | 1 | python/routing.csv |
| rust | 9 | rust/routing.csv |
| security | 888 | security/routing.csv |
| testing-qa | 107 | testing-qa/routing.csv |
| typescript | 20 | typescript/routing.csv |
| virsion-control | 226 | virsion-control/routing.csv |

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
