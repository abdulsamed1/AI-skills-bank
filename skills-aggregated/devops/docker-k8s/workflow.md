# devops / docker-k8s

## Purpose

Container orchestration: Docker, Kubernetes, and deployment strategies

This sub-hub is optimized for multi-tool usage (Gemini CLI, Antigravity, GitHub Copilot) with minimal context overhead.

## Loading Strategy

1. Start with `skills-manifest.json` to understand scope and top triggers.
2. Narrow by user intent and trigger keywords first.
3. Load only relevant lines from `skills-catalog.ndjson`.
4. Avoid loading the entire catalog unless explicitly needed.

## Files

- `skills-manifest.json`: Summary, counts, and top triggers.
- `skills-index.json`: Lightweight index for quick filtering before deep reads.
- `skills-catalog.ndjson`: One JSON object per skill (stream-friendly).

## Recommended Use Cases

- Containerizing applications
- Scaling with Kubernetes
- Managing microservices

## Quick Trigger Hints

- orchestration
- agent
- workflow
- patterns
- stack
- ai
- full
- k8s
- kubernetes
- architect
- ml
- linkerd
- manifest
- c4
- generator
- pipeline
- security
- event
- scaffolding
- engineer

## Data Contract

Each index item contains:

```json
{"id":"...","triggers":["..."],"source":"...","primary_hub":"...","is_primary":true,"match_score":8}
```

Each NDJSON item contains:

```json
{"id":"...","description":"...","path":"...","triggers":["..."],"source":"...","primary_hub":"...","assigned_hubs":["..."],"match_score":8,"is_primary":true}
```

## Notes

- Keep this workflow lightweight.
- Prefer selective reads from the catalog.
- This mirrors BMAD's router pattern (`SKILL.md` delegates to `workflow.md`).