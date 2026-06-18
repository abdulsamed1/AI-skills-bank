<div align="center">

# 🏦 skills-bank

**High-performance skill aggregation, classification & routing platform for AI agents.**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-ce422b?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-1f6feb?style=flat-square)](./LICENSE)
[![CI Status](https://img.shields.io/badge/build-passing-16a34a?style=flat-square)]()
[![Agents Supported](https://img.shields.io/badge/agents-12%2B-0066cc?style=flat-square)]()

**Aggregate 100+ skill repositories → Deduplicate 8000+ workflows → Classify semantically → Route to AI agents**

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [FreeLLMAPI Integration](#-freellmapi-integration) • [Troubleshooting](#-troubleshooting)

</div>

---

## 📚 Table of Contents

1. [Overview](#-overview)
2. [Features](#-features)
3. [Prerequisites](#-prerequisites)
4. [Quick Start](#-quick-start)
5. [FreeLLMAPI Integration](#-freellmapi-integration)
6. [Commands Reference](#-commands-reference)
7. [Classification Architecture](#-classification-architecture)
8. [Project Structure](#-project-structure)
9. [Environment Variables](#-environment-variables)
10. [Supported AI Agents](#-supported-ai-agents)
11. [Troubleshooting](#-troubleshooting)
12. [Contributing](#-contributing)
13. [License](#-license)

---

## 🎯 Overview

**skills-bank** is a unified skill aggregation platform designed to solve the "skill discovery bottleneck" for AI agents. Instead of agents searching 100+ distributed GitHub repositories for workflows, skills-bank:

1. **Aggregates** 8000+ `SKILL.md` files from distributed repositories.
2. **Deduplicates** by name and description, catching cross-repo clones.
3. **Classifies** skills into 12 domain hubs and 40+ sub-hubs using a hybrid keyword + LLM approach.
4. **Routes** skills to 12+ AI agent platforms (Claude, Copilot, Cursor, Hermes, etc.).
5. **Maintains** a canonical cache in `lib/` with automatic updates via file watcher.

**Result:** Agents access 3,600+ curated, deduplicated skills through a single unified interface.

### Core Design Principles

- **Source-of-Truth Loading**: Agents load canonical `SKILL.md` files directly from source repositories, not from catalogs (eliminates hallucination, optimizes tokens).
- **Hybrid Classification**: Combines fast keyword rules (Step A) with optional LLM semantic classification (Step B).
- **Smart Deduplication**: Two-key strategy catches both exact collisions and renamed cross-repo clones.
- **Multi-Tool Support**: Syncs to GitHub Copilot, Claude, Cursor, Hermes, Windsurf, Antigravity, and more.
- **Token Efficiency**: Load minimal metadata first, then source files on-demand.

---

## ✨ Features

### Aggregation & Deduplication
- **Parallel Cloning**: Shallow clone optimization (80% faster, lower disk footprint).
- **Smart Deduplication**: Normalization and Levenshtein-based matching of name OR description catches cross-repo clones.
- **Incremental Updates**: Checks for remote updates first via `git fetch --depth 1` and compares HEAD with FETCH_HEAD hashes before executing resets, cleans, or pulls, avoiding disk and network overhead on already up-to-date repositories.
- **Legacy Migration**: Automatic legacy repository directory cleanup.

### Classification & Routing
- **Hybrid Pipeline**: Pre-filter keyword rules (90% confidence) + LLM semantic classification (60-95% confidence).
- **Domain Hubs (12)**: `code-quality`, `server-side`, `frontend`, `business`, and others.
- **Sub-Hubs (40+)**: `security`, `testing-qa`, `performance`, `ux-design`, etc.
- **Conflict Resolution**: Special conflict table ensures domain specialists (e.g., security, testing-qa) always win over generic programming language hubs.
- **Confidence Scoring**: Path-based (98%), keyword-based (90%), LLM-based (60-95%).

### Sync & Distribution
- **Multi-Target Sync**: Automatic distribution to Claude, GitHub Copilot, Hermes, Cursor, Windsurf, Antigravity, and more.
- **Atomic updates**: Sync is all-or-nothing; failures don't corrupt state.
- **Junction management**: Safe symlink and junction manipulation on Windows, macOS, and Linux.

---

## 🔧 Prerequisites

- **Rust 1.70+** ([Install](https://www.rust-lang.org/tools/install))
- **Git** (for repository cloning)
- **~2GB disk space** (for aggregated skills cache)
- **Node.js 20+** (optional, only if running FreeLLMAPI locally)


---

## 🚄 [FreeLLMAPI](https://github.com/tashfeenahmed/freellmapi) Integration

### The Problem: LLM Rate Limits
When classifying 8000+ skills, your LLM provider will hit rate limits. Individual providers (Groq, OpenAI, Anthropic) each have free-tier limits (e.g., Groq: ~120 requests/min), which can cause cascading failures during aggregation.

### The Solution: FreeLLMAPI
**[FreeLLMAPI](https://github.com/tashfeenahmed/freellmapi)** is a unified proxy that aggregates 12+ free LLM providers behind a single OpenAI-compatible endpoint. It automatically falls back when a provider rate-limits, eliminating rate-limit errors during aggregation.

```
skills-bank (classification)
     │
     ▼
FreeLLMAPI (unified proxy)
     │ (automatic fallback/rotation)
     ├─ Google Gemini (1,500 RPM)
     ├─ Groq (120 RPM)
     ├─ Cerebras (200 RPM)
     ├─ SambaNova (100 RPM)
     ├─ Mistral (100 RPM)
     ├─ OpenRouter (20 RPM)
     ├─ GitHub Models (100 RPM)
     ├─ Cloudflare (10,000 req/day)
     ├─ Cohere (100 RPM)
     └─ (Others: Z.ai, NVIDIA, HuggingFace)
```

**Total capacity:** ~1.3 billion tokens/month (free tier combined)

### Setup [FreeLLMAPI](https://github.com/tashfeenahmed/freellmapi)

The entire workspace is consolidated under a **single source of truth** configuration file at `freellmapi/.env`. native Unix symlinks pointing back to this centralized environment file.

To configure the proxy and automatically seed your provider keys:

1. Consolidated environment config is managed centrally at `freellmapi/.env`.
2. A database seeding script automatically reads upstream provider API keys, encrypts them using FreeLLMAPI's native AES-256-GCM, and seeds them into the SQLite database.
3. The unified API key is synchronized automatically to `freellmapi`.

#### Next Steps:
1. Start FreeLLMAPI:
   ```bash
   cd freellmapi && npm run dev
   ```
2. Run `skills-bank` aggregation:
   ```bash
   ./target/release/skills-bank aggregate
   ```


cd skills-bank/
cargo build --release
```

### 2. Run Aggregation
```bash
# First-time interactive setup
./target/release/skills-bank setup

# Execute the aggregation pipeline
./target/release/skills-bank aggregate
```

### 3. Sync to Your Tools
```bash
./target/release/skills-bank sync
```

---

## 🎮 Commands Reference

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `setup` | Interactive configuration wizard | Initial setup or configuration change |
| `aggregate` | Collect, deduplicate, classify, and route skills | First run or when repositories change |
| `sync` | Distribute aggregated skills to configured AI tool directories | After aggregation completes |
| `run` | Execute the full pipeline (`aggregate` → `sync`) in sequence | Automated daily updates |
| `watch` | Auto-trigger aggregation on config changes (file watcher) | Local development / hot-reload |
| `add-repo <URL>` | Add a new skill repository | Onboarding a new source repository |
| `doctor` | Validate installation and report repository state | Troubleshooting or pre-cleanup inspection |
| `release-gate` | Validate aggregation output integrity | Before releases or production sync |
| `cleanup-legacy-duplicates` | Remove legacy repository folders | Migration from older versions |

---

## 🏗️ Classification Architecture

### Multi-Stage Pipeline

```
 8000+ SKILL.md files
        │
        ▼
 ┌──────────────┐
 │  YAML Parse  │  Extract name, description, triggers
 └──────┬───────┘
            │
            ▼
 ┌──────────────┐
 │  Keyword     │  Fast token-based routing to hub/sub-hub
 │  Rules       │  (fallback if LLM unavailable)
 └──────┬───────┘
            │
            ▼
 ┌──────────────┐
 │  Dedup Check │  Name OR Description Levenshtein HashSet
 └──────┬───────┘
            │
            ▼
 ┌──────────────────────────────────────┐
 │  Hybrid Exclusion + LLM Classify     │
 │  Step A: Keyword pre-filter          │
 │  Step B: LLM semantic classify       │
 │         (supports JSON object parser)│
 └──────┬───────────────────────────────┘
            │
            ▼
 ┌──────────────┐
 │  Output      │  routing.csv, per-hub manifests,
 │  Artifacts   │  subhub-index.json
 └──────────────┘
```

### Classification Improvements (v2.0+)

1. **Repository Name Substring Matching**:
   Repository directory names are analyzed using substring matching (e.g., `anthropic-cybersecurity-skills` -> matches `security` -> routes to `code-quality/security`). This runs first with a near-deterministic **98% confidence score**.
2. **Sub-Hub Conflict Resolution**:
   Precedence rules resolve overlap (e.g., `security` wins over programming languages like `python` or `rust`).
3. **Robust Object Response Support**:
   The LLM parser supports both standard JSON strings and direct JSON objects in `choices[0].message.content` responses, avoiding parsing crashes.

---

## 📁 Project Structure

```
skills-bank/
├── src/                              # Rust source code
│   ├── components/
│   │   ├── llm/                     # LLM classification & provider logic
│   │   │   ├── providers/           # Provider implementations (custom, groq, etc.)
│   │   │   │   └── custom.rs        # FreeLLMAPI provider
│   │   ├── aggregator/              # Aggregation engine
│   │   │   ├── mod.rs               # Aggregator lifecycle
│   │   │   └── rules.rs             # Keyword & conflict resolution rules
│   │   ├── native_pipeline.rs       # Core pipeline orchestration
│   │   ├── syncer.rs                # Sync files to AI tools
│   │   └── diagnostics.rs           # Health checks
│   ├── main.rs                      # CLI entry point
│   └── lib.rs                       # Library root
├── Cargo.toml                       # Cargo manifest
├── config.json     # User config (repos, sync targets)
├── skills-aggregated/               # Output directory (generated)
│   ├── routing.csv                 # Master routing table
│   ├── hub-manifests.csv           # Hub manifests registry
│   └── subhub-index.json           # Sub-hub index
└── README.md                        # Project documentation
```

---

## ⚙️ Environment Variables

The project loads environment variables via a symlink `skills-bank/.env` pointing to the centralized configuration at `freellmapi/.env`. Update settings inside `freellmapi/.env` to configure:

### Core Configuration
```bash
PARALLEL_JOBS=8                    # CPU threads for parallel cloning (default: auto)
SKILL_MANAGE_DEBUG=1              # Enable debug logging

# Deduplication thresholds
DEDUP_DESC_THRESHOLD=0.8          # Levenshtein distance for description matching
DEDUP_NAME_THRESHOLD=0.7          # Levenshtein distance for name matching
```

### LLM Classification Configuration
```bash
LLM_PROVIDER=freellmapi           # 'freellmapi', 'groq', 'openai', 'claude', 'gemini'
LLM_API_KEY=freellmapi-...        # API Key (unified key if using freellmapi)
LLM_API_URL=http://localhost:3001/v1 # API Endpoint (default: local proxy)
LLM_MODEL=auto                    # Model identifier ('auto' for proxy routing)

# Batch processing
LLM_BATCH_SIZE=10                 # Number of skills to process in a single batch
LLM_CONCURRENCY=5                 # Concurrent batch request limit
LLM_MAX_RETRIES=3                 # Max retry attempts for failed batches
LLM_INITIAL_BACKOFF_MS=500        # Exponential backoff base duration
```

---

## 🌐 Supported AI Agents

| Agent | Project Local Path | User Global Path |
|-------|--------------------|------------------|
| **Claude** | `.claude/skills/` | `~/.claude/skills/` |
| **Hermes** | `.hermes/skills/` | `~/.hermes/skills/` |
| **GitHub Copilot** | `.agents/skills/` | `~/.agents/skills/` |
| **Cursor** | `.cursor/skills/` | `~/.cursor/skills/` |
| **Windsurf** | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` |
| **Antigravity** | `.agent/skills/` | `~/.agent/skills/` |

---

## 🔧 Troubleshooting

### Issue: Rate Limit Errors (429 Too Many Requests)
- **Solution**: Configure `LLM_PROVIDER=freellmapi` and add multiple provider keys via the FreeLLMAPI dashboard.

### Issue: "Connection refused" to http://localhost:3001
- **Solution**: Verify FreeLLMAPI is running via `npm run dev` and port `3001` is open.

### Issue: Sync Failing with Symlink/Junction Errors
- **Solution**: The tool handles junctions atomically. If conflicting locks occur, manually clean target directories:
  ```bash
  rm -rf ~/.claude/skills/*  # Linux/macOS
  ```

---

## 🤝 Contributing

1. Fork the repository.
2. Create a feature branch: `git checkout -b feature/your-feature`.
3. Add test coverage for new keyword rules or functionality.
4. Run `cargo test` and verify format with `cargo fmt`.
5. Submit a pull request.

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE](./LICENSE) for details.
