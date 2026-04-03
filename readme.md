<div align="center">

# skill-manage

High-performance skill aggregation, classification & routing platform for AI agents.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-ce422b?style=for-the-badge)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-1f6feb?style=for-the-badge)](./LICENSE)
[![CLI](https://img.shields.io/badge/cli-ready-16a34a?style=for-the-badge)]()
[![TUI](https://img.shields.io/badge/tui-ratatui-6e56cf?style=for-the-badge)]()

</div>

---

## 📖 Overview

**skill-manage** aggregates skills (workflows, tasks, specialized agents) from 100+ distributed repositories and provides a unified routing system for AI agents to discover, load, and invoke them efficiently.

### Core Design Principles

- **Source-of-Truth Loading**: Agents load canonical `SKILL.md` files directly from source repositories, not from catalogs. This eliminates hallucination risks and optimizes token usage.
- **Hybrid Classification**: A dual-stage pipeline combines fast keyword rules (Step A) with LLM-powered semantic classification (Step B) to route skills into 12 domain hubs and 40+ sub-hubs.
- **Smart Deduplication**: Skills are deduplicated by **name OR description** — catching both exact collisions and cross-repo clones with different names but identical content.
- **Multi-Tool Support**: Skills sync to all major AI tools: GitHub Copilot, Claude Code, Cursor, Gemini CLI, Antigravity, OpenCode, Codex, and Windsurf.
- **Token Efficiency**: Load minimal metadata first, then source files on-demand—not batch-loading entire catalogs.
- **Interactive TUI**: A rich terminal UI (powered by Ratatui) provides real-time dashboard, skill explorer, and pipeline monitoring.

---

## 🚀 Quick Start

### 1. Build the CLI
```bash
cd skill-manage/
cargo build --release
```

### 2. Run the Full Pipeline
```bash
# Interactive setup (first run)
cargo run --release

# Or run all steps in sequence
cargo run --release -- run

# Launch the interactive TUI
cargo run --release -- tui
```

### 3. Individual Commands
```bash
# Aggregate skills from configured repositories
cargo run --release -- aggregate

# Sync aggregated skills to AI tool directories
cargo run --release -- sync

# Validate installation
cargo run --release -- doctor
cargo run --release -- release-gate

# Cleanup legacy duplicate repos (legacy locations: src/, repos/)
cargo run --release -- cleanup-legacy-duplicates
```

---
## Core Logic & CLI
- **[src/](./src/)** — Rust source code containing the TUI, fetcher, aggregator, and sync components.
- **[Cargo.toml](./Cargo.toml)** — Rust manifest defining project metadata and dependencies.
- **[.skill-manage-cli-config.json](./.skill-manage-cli-config.json)** — User-specific configuration for sync targets and repository lists.

### Outputs & Aggregation
- **[skills-aggregated/](./skills-aggregated/)** — The generated "Single Source of Truth" containing routed skill hubs and `routing.csv`.
- **[lib/](./lib/)** — Canonical cache directory for cloned external skill repositories.

### Documentation
- **[readme.md](./readme.md)** — Main platform documentation and quick-start guide.
- **[AGENTS.md](./AGENTS.md)** — Instruction manual for AI agents on how to discover and load skills.

### Tooling & Maintenance
- **[tests/](./tests/)** — Integration testing suite for the pipeline and TUI components.
- **[archive/](./archive/)** — Legacy PowerShell scripts from the original PoC phase.
- **[package.json](./package.json)** — Node.js manifest for `npx` distribution support.
- **[.agent/](./.agent/)** — Local agent instructions and project-specific skills.

---

## 🔧 CLI Reference

### Interactive Setup

```bash
cargo run --release -- setup
```

This launches an interactive wizard to configure:
- Where skills should be synced (global, workspace, or both)
- Which AI tools to sync to
- Repository URLs to clone and aggregate
- Excluded categories

### Commands

**Aggregate Skills**
```bash
cargo run --release -- aggregate
```
Collects, deduplicates, classifies and routes skills from configured repositories to `skills-aggregated/`.

**Sync to Tools**
```bash
cargo run --release -- sync
```
Distributes aggregated skills to configured AI tool directories.
- Skips existing junctions/symlinks to avoid recursive errors
- Falls back to direct writes if atomic writes fail
- Updates routing CSVs with absolute paths for global targets

**Add Repository**
```bash
cargo run --release -- add-repo <URL>
```

**Run Full Pipeline**
```bash
cargo run --release -- run
```

**Interactive TUI**
```bash
cargo run --release -- tui
```
Launches a real-time terminal dashboard with skill explorer, hub statistics, and LLM classification progress.

**Validate**
```bash
cargo run --release -- doctor
cargo run --release -- release-gate
```

**Cleanup legacy duplicate repos**
```bash
cargo run --release -- cleanup-legacy-duplicates
```

---

## 📁 Repository Cache & Fetching

- Repositories are cloned into the canonical cache directory `lib/` at the repository root (not `src/`). The fetcher uses shallow clones (`git clone --depth 1 --single-branch --no-tags`) for speed and disk savings.
- Existing repositories inside `lib/` are updated with `git pull` rather than being re-cloned; the fetch pipeline deduplicates manifest entries by normalized remote URL and repository name before operating.
- If you need to remove legacy repository folders left in older locations (`src/`, `repos/`), use the CLI command `cleanup-legacy-duplicates`. This command is destructive: it only deletes a legacy folder when a matching `lib/` repository exists and the Git remote origin identity matches. We recommend running `cargo run --release -- doctor` to inspect repository state before running cleanup.

## ⚙️ Configuration Files

Generated during aggregation:

- **`skills-aggregated/routing.csv`** — Skill routing rules (hub, sub-hub, src_path)
- **`skills-aggregated/subhub-index.json`** — Hub and sub-hub registry
- **`skills-aggregated/.skill-lock.json`** — Aggregation metadata and lock (timestamps, repo state)
- **Per-subhub `skills-manifest.json`** — Skill metadata and triggers
- **`skills-aggregated/hub-manifests.csv`** — Master index of all skills across all hubs

---

## 🌐 Environment Variables

- `skill-manage\.env-example`

| Variable | Default | Description |
|---|---|---|
| `LLM_ENABLED` | `true` | Enable/disable LLM classification (set `false` for keyword-only) |
| `LLM_PROVIDER` | — | LLM provider: `gemini`, `openai`, or `mock` |
| `LLM_API_KEY` | — | API key for the configured LLM provider |
| `LLM_API_URL` | Provider default | Custom API endpoint URL |
| `LLM_MODEL` | `gpt-4o-mini` | Model name (OpenAI provider) |
| `LLM_CACHE_PATH` | `~/.skill-manage/llm-classifications.json` | Persistent cache for classifications |
| `LLM_CA_CERT_PATH` | — | Custom CA certificate for HTTPS pinning |
| `SKILL_MANAGE_EXCLUSIONS` | — | Semicolon-separated category exclusion overrides |

---

## 🎯 Tool Integration Targets

Sync skills to any of these destinations:

| Tool | Project | Global |
|---|---|---|
| **Claude** | `.claude/skills/` | `~/.claude/skills/` |
| **Code (Codex)** | `.agents/skills/` | `~/.agents/skills/` |
| **GitHub Copilot** | `.github/skills/` | `~/.copilot/skills/` |
| **Cursor** | `.cursor/skills/` | `~/.cursor/skills/` |
| **Gemini** | `.gemini/skills/` | `~/.gemini/skills/` |
| **Antigravity** | `.agent/skills/` | `~/.gemini/antigravity/skills/` |
| **OpenCode** | `.opencode/skills/` | `~/.config/opencode/skills/` |
| **Windsurf** | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` |
| **Hermes** | `.hermes/skills/skill-manage/` | `~/.hermes/skills/skill-manage/` |

---

## 🏗️ Classification Architecture

The aggregation pipeline processes 8000+ `SKILL.md` files through a multi-stage classification system:

```
 SKILL.md files (8000+)
        │
        ▼
 ┌──────────────┐
 │  YAML Parse   │  Extract name, description, triggers
 └──────┬───────┘
        │
        ▼
 ┌──────────────┐
 │  Keyword      │  Fast token-based routing to hub/sub-hub
 │  Rules        │  (fallback if LLM unavailable)
 └──────┬───────┘
        │
        ▼
 ┌──────────────┐
 │  Dedup        │  Name OR Description HashSet
 │  (two-key)    │  Catches cross-repo clones
 └──────┬───────┘
        │
        ▼
 ┌──────────────────────────────────┐
 │  Hybrid Exclusion + LLM Classify │
 │  Step A: Keyword pre-filter      │
 │  Step B: LLM semantic classify   │
 │         (can return "excluded")  │
 └──────┬───────────────────────────┘
        │
        ▼
 ┌──────────────┐
 │  Output       │  routing.csv, per-hub manifests,
 │  Artifacts    │  skills-index.json
 └──────────────┘
```

### Hub Taxonomy (12 domains)

`code-quality` · `frontend` · `backend` · `testing` · `ai` · `business` · `marketing` · `mobile` · `design` · `systems` · `data` · `security`

---

## � License

MIT — See [LICENSE](./LICENSE) or [cli/package.json](./cli/package.json)
