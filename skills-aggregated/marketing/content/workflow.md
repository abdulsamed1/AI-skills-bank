# marketing / content

## Purpose

Content marketing & SEO: copywriting, blog strategy, search optimization, and editorial best practices

This sub-hub is optimized for multi-tool usage (Gemini CLI, Antigravity, GitHub Copilot) with minimal context overhead.

## Loading Strategy

1. Start with `skills-manifest.json` to understand scope and top triggers.
2. Narrow by user intent and trigger keywords first.
3. Load only relevant lines from `skills-catalog.ndjson`.
4. Avoid loading the entire catalog unless explicitly needed.

## Execution Rule (Mandatory)

1. Do not stop at `SKILL.md`, `workflow.md`, or `skills-manifest.json`.
2. After filtering candidate entries from `skills-catalog.ndjson`, open at least one concrete skill file from the `path` field.
3. If multiple candidates exist, open the best match first, then continue with implementation using that skill.
4. If a `path` under `AI-skills-bank/src/` is missing, report it explicitly and request re-aggregation with src repos included.

## Files

- `skills-manifest.json`: Summary, counts, and top triggers.
- `skills-index.json`: Lightweight index for quick filtering before deep reads.
- `skills-catalog.ndjson`: One JSON object per skill (stream-friendly).

## Recommended Use Cases

- Creating SEO-optimized content
- Building blog strategies
- Improving search rankings

## Quick Trigger Hints

- seo
- content
- ai
- competitor
- programmatic
- audit
- strategy
- alternatives
- architect
- ts
- enablement
- ad
- snippet
- geo
- marketer
- copywriting
- auditor
- copy
- magnets
- gen

## Data Contract

Each index item contains:

```json
{"id":"...","triggers":["..."],"src":"...","primary_hub":"...","is_primary":true,"match_score":8}
```

Each NDJSON item contains:

```json
{"id":"...","description":"...","path":"...","triggers":["..."],"src":"...","primary_hub":"...","assigned_hubs":["..."],"match_score":8,"is_primary":true}
```

## Notes

- Keep this workflow lightweight.
- Prefer selective reads from the catalog.
- This mirrors BMAD's router pattern (`SKILL.md` delegates to `workflow.md`).