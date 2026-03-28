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
node ./src/index.mjs doctor --project ..\\..
node ./src/index.mjs security --project ..\\..
node ./src/index.mjs init --project ..\\.. --src-repo-mode changed-only --skip-sync --dry-run
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
