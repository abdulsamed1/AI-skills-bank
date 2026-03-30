<div align="center">

# skill-manage

Unified, visual, multi-tool skill routing platform for AI workflows.

[![Node](https://img.shields.io/badge/node-%3E%3D18-2f7d32?style=for-the-badge)](https://nodejs.org/)
[![PowerShell](https://img.shields.io/badge/powershell-7%2B-4b9cd3?style=for-the-badge)](https://learn.microsoft.com/powershell/)
[![License](https://img.shields.io/badge/license-MIT-1f6feb?style=for-the-badge)](./cli/package.json)
[![CLI](https://img.shields.io/badge/cli-ready-16a34a?style=for-the-badge)](./cli/README.md)

</div>

---

## 📖 Overview

**skill-manage** aggregates skills (workflows, tasks, specialized agents) from distributed repositories and provides a unified routing system for AI agents to discover, load, and invoke them efficiently.

### Core Design Principles

- **Source-of-Truth Loading**: Agents load canonical `SKILL.md` files directly from source repositories, not from catalogs. This eliminates hallucination risks and optimizes token usage.
- **Smart Routing**: Lightweight routing CSVs enable fast skill discovery by trigger/keyword matching with relevance scoring.
- **Multi-Tool Support**: Skills sync to all major AI tools: GitHub Copilot, Claude Code, Cursor, Gemini CLI, and more.
- **Token Efficiency**: Load minimal metadata first, then source files on-demand—not batch-loading entire catalogs.

---

## 🚀 Quick Start

### 1. Install Dependencies
```bash
cd skill-manage/cli
cargo build --release
```

### 2. Aggregate Skills (Default: Latest Repo)
```powershell
pwsh -ExecutionPolicy Bypass -File "scripts/aggregate-skills-to-subhubs.ps1"
```

### 3. Sync to Tool Directories
```powershell
pwsh -ExecutionPolicy Bypass -File "scripts/sync-hubs.ps1" -SyncMode Auto -Force
```

### 4. Validate Installation
```powershell
cd cli
cargo run -- doctor
cargo run -- release-gate
```

---

## 📚 Documentation

- **[Agent Skill Loading Guide](./AGENTS.md)** — How agents discover and load skills (routing, token budget, anti-hallucination gates)
- **[Agent Architecture](./docs/agent-skill-loading-architecture.md)** — Full technical specification (routing strategy, file roles, examples)
- **[Project Context](./docs/project-context.md)** — Project structure and conventions
- **[Epics & Roadmap](./docs/epics.md)** — Backlog and planned work

---

## 🔧 Script Reference

### Aggregate (Collect Skills from Source Repos)

```powershell
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/aggregate-skills-to-subhubs.ps1"
```

Modes:

```powershell
# Latest repo only (default)
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode latest

# All repos under lib/
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode all

# Specific repos
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode selected -srcRepoNames antigravity-awesome-skills

# Changed since last lock
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode changed-only
```

### Sync (Distribute to Tool Directories)

```powershell
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/sync-hubs.ps1" -SyncMode Auto -Force
```

**Policy:**
- Global-first sync by default (e.g., `~/.copilot/skills/`)
- Workspace targets are optional
- Workspace pruning is explicit only (`-PruneWorkspaceTargets`)

### Generate Routing (Single Entrypoint)

```powershell
# Default: SourceDirect (dynamic repo-relative paths)
powershell -ExecutionPolicy Bypass -File "skill-manage/scripts/generate-routing-csv.ps1"

# SourceDirect: dynamic repo-relative paths (recommended)
powershell -ExecutionPolicy Bypass -File "skill-manage/scripts/generate-routing-csv.ps1" -ToolProfile SourceDirect

# HubLocal: local junctions in each sub-hub
powershell -ExecutionPolicy Bypass -File "skill-manage/scripts/generate-routing-csv.ps1" -ToolProfile HubLocal
```

**Profiles:**
- `Auto`: Dynamic repo-relative `lib/.../SKILL.md` paths
- `SourceDirect`: Same as Auto; portable, no hub-local mount dependency
- `HubLocal`: Local `skills/<skill-id>/SKILL.md` (requires junctions)
- `SourceDirectStatic`: Absolute paths (machine-specific; legacy)

---

## 🎯 Tool Integration Targets

Sync skills to any of these destinations:

| Tool | Project | Global | Docs |
|---|---|---|---|
| **GitHub Copilot** | `.github/skills/` | `~/.copilot/skills/` | [Copilot Skills](https://github.com/features/copilot) |
| **Cursor** | `.cursor/skills/` | `~/.cursor/skills/` | [Cursor Skills](https://docs.cursor.sh/) |
| **Claude Code** (Windsurf) | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` | [Cascade Skills](https://docs.codeium.com/windsurf) |
| **VS Code Gemini** | `.gemini/skills/` | `~/.gemini/skills/` | [Gemini CLI Skills](https://ai.google.dev) |
| **Antigravity** | `.agent/skills/` | `~/.gemini/antigravity/skills/` | [Antigravity Skills](https://google.ai/antigravity) |
| **OpenCode** | `.opencode/skills/` | `~/.config/opencode/skills/` | OpenCode Skills |
| **Codex** | `.agents/skills/` | `~/.agents/skills/` | Codex Skills |

---

## 🛠️ Troubleshooting

### Skill Conflicts
If using multiple tools and you see `Skill conflict detected`, remove the conflicting global directory:
```powershell
Remove-Item -Recurse -Force ~/.gemini/skills/
```

### Broken Sub-hubs (Windows Junctions)
If agents can't read skills from specific sub-hubs, junctions may be stale:
1. **Check**: `fsutil reparsepoint query "path/to/sub-hub"`
2. **Fix**: Re-run sync with `-Force`:
   ```powershell
   pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/sync-hubs.ps1" -SyncMode Auto -Force
   ```

---

## 📦 Project Structure

```
skill-manage/
├── cli/                          # Rust CLI for aggregation
│   ├── Cargo.toml
│   └── lib/
│       ├── components/           # Core pipeline logic
│       │   ├── native_pipeline.rs     # CSV generation
│       │   ├── aggregator.rs          # SKILL.md parsing
│       │   └── commands/              # CLI commands
│       └── main.rs
├── scripts/                      # PowerShell utilities
│   ├── aggregate-skills-to-subhubs.ps1
│   ├── sync-hubs.ps1
│   ├── generate-routing-csv.ps1
│   └── ...
├── lib/                          # Source repositories (via git clone)
│   └── [repo-name]/
│       └── ...skills/            # Distributed skill definitions
├── skills-aggregated/            # Generated output (artifacts)
│   ├── {hub}/
│   │   └── {sub_hub}/
│   │       ├── routing.csv       # Agent routing layer
│   │       ├── routing.csv # Metadata snapshot
│   │       └── skills-manifest.json
│   └── .md                 # Agent loading guide
├── docs/
│   ├── agent-skill-loading-architecture.md
│   ├── project-context.md
│   ├── epics.md
│   └── ...
└── readme.md                      # This file
```

---

## 🧬 CSV File Formats

### `routing.csv` (Agent Routing Layer)
Lightweight file loaded by agents to discover skills.

```csv
skill_id,description,src_path
agent-builder,Builds AI agent skills through conversational discovery,lib/bmad/agent-builder/SKILL.md
quick-dev,Rapid implementation of stories and feature changes,lib/bmad/quick-dev/SKILL.md
```

### `skills-catalog.csv` (Optional Metadata)
Read-only reference; not used by agents for routing. Contains optional scoring and phase metadata for UI previews or downstream ranking.

```csv
skill_id,description,score,phase
agent-builder,Builds AI agent skills through conversational discovery.,100,stable
quick-dev,Rapid implementation of stories and feature changes.,95,stable
```

### `skills-manifest.json` (Full Export)
Complete schema for global imports/exports.

```json
[
  {
    "skill_id": "agent-builder",
    "triggers": "agent;creation;skill",
    "score": 100,
    "src_path": "lib/bmad/agent-builder/SKILL.md",
    "description": "Builds AI agent skills...",
    "phase": "stable"
  }
]
```

---

## 🔨 Development

### Build
```bash
cd cli
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Run Doctor (Validation)
```bash
cargo run -- doctor
```

### Run Release Gate (Production Check)
```bash
cargo run -- release-gate
```

---

## 📝 License

MIT — See [cli/package.json](./cli/package.json)
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/aggregate-skills-to-subhubs.ps1" -srcRepoMode changed-only
pwsh -ExecutionPolicy Bypass -File "skill-manage/scripts/sync-hubs.ps1" -SyncMode Auto -Force
```

Requirements for imported srcs:

- Skills discoverable via `SKILL.md`
- Frontmatter should include `name` and `description`

---

## Loading Strategy for Agents (v2.0 Protocol)

To minimize token usage (typically <150 tokens) and eliminate hallucinations, AI agents **MUST** follow the 3-Step Flow:

1. **Step 1 (Route):**
   - Understand the request and map to `{hub}/{sub_hub}`.
2. **Step 2 (Lookup):** Read `skills-aggregated/{hub}/{sub_hub}/routing.csv`
   - Match user intent against the `triggers` column.
   - Extract `skill_id` and `src_path` from the row with the highest score.
3. **Step 3 (Invoke):** Read `{hub_mount_path}/{src_path}`
   - Load the exact file referenced by the routing layer locally from the junction.

> **Note:** Agents should NEVER read `hub-manifests.csv` (too large) or guess file paths. Always use `routing.csv` for exact resolution. Reference `./AGENTS.md` for full implementation rules.

---

## Project Integration Snippet

```yaml
ai_skills_routing:
  protocol: v2.0
  entrypoint: skill-manage/skills-aggregated/quick-index.json
  rules:
    - Step 1: Route via quick-index.json
    - Step 2: Extract src_path from {hub}/{sub_hub}/routing.csv
    - Step 3: Load exact contents of {src_path}
  anti_hallucination:
    - never invent skill_ids
    - never guess path locations
    - never read hub-manifests.csv
```

---

## Operational Checks

```powershell
# count skills from manifests
$m = Get-ChildItem -Recurse "skill-manage/skills-aggregated" -Filter "skills-manifest.json"
$sum = 0
foreach ($f in $m) {
  $j = Get-Content $f.FullName -Raw | ConvertFrom-Json
  $sum += [int]$j.skill_count
}
"manifests=$($m.Count) totalSkillsInSubhubs=$sum"
```

```powershell
# ensure generated SKILL files are lightweight
Get-ChildItem -Recurse "skill-manage/skills-aggregated" -Filter "SKILL.md" |
ForEach-Object { "{0} | {1}" -f $_.FullName, $_.Length }
```
---
# skill-manage CLI

[![npm version](https://img.shields.io/npm/v/skill-manage-cli.svg)](https://www.npmjs.com/package/skill-manage-cli)
[![npm downloads](https://img.shields.io/npm/dm/skill-manage-cli.svg)](https://www.npmjs.com/package/skill-manage-cli)
[![license](https://img.shields.io/npm/l/skill-manage-cli.svg)](./package.json)

Visual CLI for skill-manage with guided flows, changed-only aggregation, and global-first sync.

## Why This Tool

- Zero-friction onboarding for skills aggregation and sync
- Safer updates with `changed-only` repository scanning
- Better discoverability with clear terminal UX and interactive mode

## Install (Local Dev)

```powershell
cd skill-manage/cli
npm install
npm link
```

Then run:

```powershell
skills-bank --help
```

Readable output modes:

```powershell
# default: concise enterprise summary
skills-bank run --src-repo-mode changed-only

# verbose: full detailed logs
skills-bank run --src-repo-mode changed-only --verbose

# raw-output: exact script output without filtering
skills-bank run --src-repo-mode changed-only --raw-output
```

## Visual Demo (Terminal)

```text
╭───────────────────────────────────────────────╮
│                                               │
│   skill-manage CLI                          │
│   Aggregate, Sync, and Manage AI skill hubs   │
│                                               │
╰───────────────────────────────────────────────╯

? Choose what to do
❯ Run full pipeline (aggregate + sync)
  Aggregate only
  Sync only
  Add src repository
  Doctor
```

## Commands

### Interactive Mode (Recommended)

```powershell
skills-bank interactive
```

### Init (One-Step Setup)

```powershell
skills-bank init --src-repo-mode changed-only
```

With optional src repository bootstrap:

```powershell
skills-bank init --repo-url https://github.com/owner/repo.git --name awesome-skills --src-repo-mode changed-only
```

### Aggregate

```powershell
skills-bank aggregate --src-repo-mode changed-only
```

Custom exclusion policy per project/run:

```powershell
# Exclude only specific categories
skills-bank aggregate --src-repo-mode all --exclude-categories games,law-legal

# Disable exclusions entirely for this run
skills-bank aggregate --src-repo-mode all --no-category-exclusions
```

Optional review band (AI/manual review queue for medium-confidence routing):

```powershell
# Route medium-confidence skills to review-candidates.ndjson
skills-bank aggregate --src-repo-mode all --enable-review-band --review-min-score 4 --auto-accept-min-score 8
```

### Sync

```powershell
skills-bank sync --sync-mode Auto --force
```

### Run All

```powershell
skills-bank run --src-repo-mode changed-only --sync-mode Auto
```

With custom exclusions:

```powershell
skills-bank run --src-repo-mode all --exclude-categories games,law-legal,medicine-medical
```

With review band enabled:

```powershell
skills-bank run --src-repo-mode all --enable-review-band --review-min-score 4 --auto-accept-min-score 8
```

### Add src Repo

```powershell
skills-bank add-src https://github.com/owner/repo.git
```

### Doctor

```powershell
skills-bank doctor
```

### Security Audit

```powershell
skills-bank security
```

## Publish To npm (Open src)

```powershell
cd skill-manage/cli
npm run release:check
npm run pack:preview
npm login
npm publish --access public
```

## Release Checklist (Best Practice)

1. Update `version` in `package.json` using semver.
2. Replace placeholder GitHub links in `package.json` (`homepage`, `repository`, `bugs`).
3. Run quality checks:

```powershell
npm run release:check
npm run pack:preview
```

4. Verify CLI behavior:

```powershell
node ./lib/index.mjs doctor --project ..\\..
node ./lib/index.mjs security --project ..\\..
node ./lib/index.mjs init --project ..\\.. --src-repo-mode changed-only --skip-sync --dry-run
```

5. Publish package:

```powershell
npm publish --access public
```

6. Create GitHub release with notes (template below).

## GitHub Release Notes Template

```markdown
## skill-manage CLI vX.Y.Z

### Highlights
- Added/Improved: <feature summary>
- UX: <terminal experience or interactive updates>
- Reliability: <stability / compatibility improvements>

### Commands
- `skills-bank init`
- `skills-bank interactive`
- `skills-bank run --src-repo-mode changed-only`

### Breaking Changes
- None / <describe clearly>

### Migration Notes
1. Update package: `npm i -g skill-manage-cli@latest` or use `npx skill-manage-cli@latest`.
2. Re-run init once: `skills-bank init --src-repo-mode changed-only`.

### Verification
- `npm run release:check`
- `npm run pack:preview`
- End-to-end test: aggregate + sync passed.
```

## Suggested Launch Checklist

- Add real GitHub URLs in `package.json` (`homepage`, `repository`, `bugs`)
- Add GIF screenshots in your root project README
- Create GitHub release notes for each minor version
- Share `npx skill-manage-cli run` quick-start snippet

## Suggested Demo Assets

- `docs/media/cli-run.gif` showing `skills-bank init`
- `docs/media/cli-interactive.gif` showing guided prompts
- `docs/media/cli-sync.gif` showing final sync summary

## Notes

- This CLI wraps scripts from `skill-manage/scripts`.
- Use `--project <path>` if you run outside project root.
- Tool sync supports Antigravity, Claude Code, Codex, Cursor, Gemini CLI, Copilot, OpenCode, and Windsurf.
