<div align="center">

# skill-manage

Unified, visual, multi-tool skill routing platform for AI workflows.

[![Node](https://img.shields.io/badge/node-%3E%3D18-2f7d32?style=for-the-badge)](https://nodejs.org/)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-ce422b?style=for-the-badge)](https://www.rust-lang.org/)
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
```

---

## 📚 Documentation

- **[Agent Skill Loading Guide](./AGENTS.md)** — How agents discover and load skills (routing, token budget, anti-hallucination gates)
- **[Agent Architecture](./docs/agent-skill-loading-architecture.md)** — Full technical specification (routing strategy, file roles, examples)
- **[Project Context](./docs/project-context.md)** — Project structure and conventions
- **[Epics & Roadmap](./docs/epics.md)** — Backlog and planned work

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
Collects and routes skills from configured repositories to `skills-aggregated/`.

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

**Validate**
```bash
cargo run --release -- doctor
cargo run --release -- release-gate
```

---

## ⚙️ Configuration Files

Generated during aggregation:

- **`skills-aggregated/routing.csv`** — Skill routing rules (hub, sub-hub, src_path)
- **`skills-aggregated/subhub-index.json`** — Hub and sub-hub registry
- **`skills-aggregated/.skill-lock.json`** — Aggregation metadata and lock (timestamps, repo state)
- **Per-subhub `skills-manifest.json`** — Skill metadata and triggers

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

## � License

MIT — See [LICENSE](./LICENSE) or [cli/package.json](./cli/package.json)
