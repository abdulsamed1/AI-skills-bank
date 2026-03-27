<div align="center">

# AI Skills Bank

Unified, visual, multi-tool skill routing platform for AI workflows.

[![Node](https://img.shields.io/badge/node-%3E%3D18-2f7d32?style=for-the-badge)](https://nodejs.org/)
[![PowerShell](https://img.shields.io/badge/powershell-7%2B-4b9cd3?style=for-the-badge)](https://learn.microsoft.com/powershell/)
[![License](https://img.shields.io/badge/license-MIT-1f6feb?style=for-the-badge)](./cli/package.json)
[![CLI](https://img.shields.io/badge/cli-ready-16a34a?style=for-the-badge)](./cli/README.md)

</div>

---

## Dashboard

| Area | Status | Notes |
|---|---:|---|
| Aggregation Engine | Active | Sub-hub classification + metadata enrichment |
| Sync Engine | Active | Global-first distribution to AI tool targets |
| CLI | Active | `init`, `interactive`, `run`, `aggregate`, `sync` |
| src Filtering | Active | `latest`, `all`, `selected`, `changed-only` |

---

## Architecture

```text
AI-skills-bank/
├─ scripts/
│  ├─ aggregate-skills-to-subhubs.ps1
│  └─ sync-hubs.ps1
├─ cli/
│  ├─ package.json
│  └─ src/index.mjs
├─ skills-aggregated/
│  ├─ subhub-index.json
│  ├─ <main>/<sub>/
│  │  ├─ SKILL.md
│  │  ├─ workflow.md
│  │  ├─ skills-manifest.json
│  │  ├─ skills-index.json
│  │  └─ skills-catalog.ndjson
└─ src/
```

---

## CLI Quick Launch

```powershell
cd AI-skills-bank/cli
npm install
node ./src/index.mjs init --project ..\\.. --src-repo-mode changed-only
```

Interactive mode:

```powershell
node ./src/index.mjs interactive --project ..\\..
```

Visual flow:

```text
╭───────────────────────────────────────────────╮
│   AI Skills Bank CLI                          │
│   Aggregate, Sync, and Manage AI skill hubs   │
╰───────────────────────────────────────────────╯
? Choose what to do
❯ Initialize project (doctor + aggregate + sync)
  Run full pipeline (aggregate + sync)
  Aggregate only
```

---

## Script Deck

### Aggregate

```powershell
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/aggregate-skills-to-subhubs.ps1"
```

Modes:

```powershell
# newest repo only (default)
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode latest

# all repos under src
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode all

# explicit repos
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode selected -srcRepoNames antigravity-awesome-skills

# changed since last lock
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode changed-only
```

### Sync

```powershell
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/sync-hubs.ps1" -SyncMode Auto -Force
```

Policy:

- Global-first sync by default
- Workspace targets are optional
- Workspace pruning is explicit only (`-PruneWorkspaceTargets`)

---

## Tool Targets

| Tool | Project Path | Global Path | Official Docs |
|---|---|---|---|
| Antigravity | `.agent/skills/` | `~/.gemini/antigravity/skills/` | Antigravity Skills |
| Claude Code | `.claude/skills/` | `~/.claude/skills/` | Claude Code Skills |
| Codex | `.agents/skills/` | `~/.agents/skills/` | Codex Skills |
| Cursor | `.cursor/skills/` | `~/.cursor/skills/` | Cursor Skills |
| Gemini CLI | `.gemini/skills/` | ~/.agents/skills/ | Gemini CLI Skills |
| GitHub Copilot | `.github/skills/` | `~/.copilot/skills/` | Copilot Skills |
| OpenCode | `.opencode/skills/` | `~/.config/opencode/skills/` | OpenCode Skills |
| Windsurf | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` | Windsurf Cascade Skills |

### Troubleshooting Skill Conflicts

If you use multiple CLI tools or agents (e.g., Gemini CLI and Antigravity), you might encounter `Skill conflict detected` warnings. This typically happens when the bank syncs skills to multiple global directories (e.g., `~/.agents/skills/` and `~/.gemini/skills/`), causing one tool to read from multiple overlapping folders.

**Fix:** Remove the conflicting global directory and rely on a single primary destination. For example:
```powershell
Remove-Item -Recurse -Force ~/.gemini/skills/
```

---

## src Onboarding

```powershell
cd AI-skills-bank/src
git clone https://github.com/example/awesome-skills.git

cd ../..
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode changed-only
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/scripts/sync-hubs.ps1" -SyncMode Auto -Force
```

Requirements for imported srcs:

- Skills discoverable via `SKILL.md`
- Frontmatter should include `name` and `description`

---

## Loading Strategy for Agents

1. Read `skills-aggregated/subhub-index.json`
2. Pick 2-4 relevant sub-hubs
3. Read per-hub: `SKILL.md`, `workflow.md`, `skills-manifest.json`, `skills-index.json`
4. Read only needed records from `skills-catalog.ndjson`

---

## Project Integration Snippet

```yaml
ai_skills_routing:
  default_subhubs:
    - programming/typescript
    - backend/api-design
    - backend/databases
  optional_subhubs:
    - frontend/react-nextjs
    - devops/cloud
  loading_rule:
    - read SKILL.md
    - read workflow.md
    - filter via skills-index.json
    - selective read from skills-catalog.ndjson
```

---

## Operational Checks

```powershell
# count skills from manifests
$m = Get-ChildItem -Recurse "AI-skills-bank/skills-aggregated" -Filter "skills-manifest.json"
$sum = 0
foreach ($f in $m) {
  $j = Get-Content $f.FullName -Raw | ConvertFrom-Json
  $sum += [int]$j.skill_count
}
"manifests=$($m.Count) totalSkillsInSubhubs=$sum"
```

```powershell
# ensure generated SKILL files are lightweight
Get-ChildItem -Recurse "AI-skills-bank/skills-aggregated" -Filter "SKILL.md" |
ForEach-Object { "{0} | {1}" -f $_.FullName, $_.Length }
```
