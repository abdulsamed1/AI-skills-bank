# AI Skills Bank

Unified operating guide for AI agents to use this repository on any machine and in any project.

## Core Design

- Keep `SKILL.md` tiny (router only).
- Put execution logic in `workflow.md`.
- Keep large data outside `SKILL.md` in index/catalog files.
- Mirror the same generated structure to all three tools.

This protects context windows and reduces hallucination risk.

## Repository Layout

```text
AI-skills-bank/
├─ aggregate-skills-to-subhubs.ps1
├─ sync-hubs.ps1
├─ skills-aggregated/
│  ├─ subhub-index.json
│  ├─ backend/
│  ├─ frontend/
│  ├─ programming/
│  ├─ devops/
│  └─ general/
└─ source/
```

Each sub-hub typically includes:

```text
{main}/{sub}/
├─ SKILL.md
├─ workflow.md
├─ skills-manifest.json
├─ skills-index.json
└─ skills-catalog.ndjson
```

## Requirements

- Git
- PowerShell 7+ (`pwsh`) recommended
- Read/write permission in workspace

## Quick Start

Run these commands (now with dynamic path support):

```powershell
# Step 1: Aggregate skills into sub-hubs
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/aggregate-skills-to-subhubs.ps1"

# Optional: scan only the latest repository under source (default behavior)
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/aggregate-skills-to-subhubs.ps1" -SourceRepoMode latest

# Optional: scan all repositories under source (legacy behavior)
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/aggregate-skills-to-subhubs.ps1" -SourceRepoMode all

# Optional: scan specific repositories only
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/aggregate-skills-to-subhubs.ps1" -SourceRepoMode selected -SourceRepoNames antigravity-awesome-skills

# Optional: scan only repositories changed since last lock
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/aggregate-skills-to-subhubs.ps1" -SourceRepoMode changed-only

# Step 2: Sync to all supported AI tools
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/sync-hubs.ps1" -Force

# Optional: avoid duplicate hub files by syncing as links (default is Auto)
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/sync-hubs.ps1" -SyncMode Auto -Force

# Optional: include workspace-local targets too (not recommended by default)
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/sync-hubs.ps1" -SyncMode Auto -IncludeWorkspaceTargets -Force
```

Default policy:
- Sync to global locations only to avoid duplicate skill indexing conflicts.
- Workspace-local target folders are NOT deleted automatically.

`-SyncMode` options:
- `Auto`: tries `Junction` first, then falls back to `Copy` if linking fails
- `Junction`: creates directory junctions (best for Windows local workflow)
- `SymbolicLink`: creates directory symlinks
- `Copy`: legacy full copy mode

Global targets (default):
- `~/.gemini/antigravity/skills`
- `~/.gemini/skills`
- `~/.copilot/skills`

`-IncludeWorkspaceTargets` adds these workspace-local targets:
- `.agent/skills`
- `.gemini/skills`
- `.github/skills`

`-PruneWorkspaceTargets` explicitly deletes workspace-local target folders.

## Add a New Source Repository (From the Internet)

Yes, this workflow is supported.

If you find an external skills repository you like, add it under `source/`, then regenerate and sync. The skills will be distributed to all supported tools.

### Steps

1. Go to `AI-skills-bank/source`.
2. Clone any repository (example):

```powershell
cd AI-skills-bank/source
git clone https://github.com/example/awesome-skills.git
```

3. Return to workspace root and run:

```powershell
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/aggregate-skills-to-subhubs.ps1"
pwsh -ExecutionPolicy Bypass -File "AI-skills-bank/sync-hubs.ps1" -Force
```

4. Verify sync results:

```powershell
Get-ChildItem -Recurse "AI-skills-bank/skills-aggregated" -Filter "SKILL.md"
Get-ChildItem -Recurse ".github/skills" -Filter "SKILL.md"
Get-ChildItem -Recurse ".gemini/skills" -Filter "SKILL.md"
Get-ChildItem -Recurse ".agent/skills" -Filter "SKILL.md"
```

### Source Compatibility Requirements

- The current pipeline discovers skills from `SKILL.md` files.
- Each `SKILL.md` should ideally include frontmatter with at least:
  - `name`
  - `description`

If a repository does not use `SKILL.md`, convert it to this format before aggregation.

## Agent Loading Strategy

1. Read `skills-aggregated/subhub-index.json`.
2. Select only 2-4 relevant sub-hubs.
3. For each selected sub-hub, read:
   - `SKILL.md`
   - `workflow.md`
   - `skills-manifest.json`
   - `skills-index.json`
4. Read `skills-catalog.ndjson` selectively (only needed records).

Never fully load large catalogs unless explicitly required.

## Tool Compatibility

Generated output is mirrored to:

- `~/.copilot/skills` (GitHub Copilot)
- `~/.gemini/skills` (Gemini CLI)
- `~/.gemini/antigravity/skills` (Antigravity)

## Project Integration Template

Add this to any project `project-context.md`:

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

## Maintenance

When classification needs tuning:

1. Edit rules in `AI-skills-bank/aggregate-skills-to-subhubs.ps1`.
2. Update `keywords` and `negative_keywords`.
3. Regenerate and sync again.

## Source Scan Policy

Aggregation now supports selective scan for repositories under `AI-skills-bank/source`:

- `-SourceRepoMode latest` (default): scans only the newest repository inside `source`
- `-SourceRepoMode all`: scans all repositories inside `source` (legacy behavior)
- `-SourceRepoMode selected -SourceRepoNames <names>`: scans only named repositories
- `-SourceRepoMode changed-only`: scans only repositories changed since last `skills-aggregated/.skill-lock.json`

Notes for `changed-only`:
- The lock file is written after each non-dry aggregation run at `AI-skills-bank/skills-aggregated/.skill-lock.json`.
- If no previous lock exists, `changed-only` falls back to `latest` for bootstrap.

## Operational Checks

Count all skills in generated sub-hubs:

```powershell
$m = Get-ChildItem -Recurse "AI-skills-bank/skills-aggregated" -Filter "skills-manifest.json"
$sum = 0
foreach ($f in $m) {
  $j = Get-Content $f.FullName -Raw | ConvertFrom-Json
  $sum += [int]$j.skill_count
}
"manifests=$($m.Count) totalSkillsInSubhubs=$sum"
```

Validate `SKILL.md` files remain tiny:

```powershell
Get-ChildItem -Recurse "AI-skills-bank/skills-aggregated" -Filter "SKILL.md" |
ForEach-Object { "{0} | {1}" -f $_.FullName, $_.Length }
```
