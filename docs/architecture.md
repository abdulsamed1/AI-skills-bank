---
stepsCompleted:
  - step-01-init
  - step-02-context
  - step-03-starter
  - step-04-decisions
  - step-05-patterns
  - step-06-structure
  - step-07-validation
  - step-08-complete
lastStep: 8
status: 'complete'
completedAt: '2026-03-28'
inputDocuments:
  - prd.md
  - AGENT-PROTOCOL.md
  - FINAL-ROADMAP-BMAD-HYBRID.md
  - SKILL.md
workflowType: 'architecture'
project_name: 'production'
user_name: 'ABDULSAMED'
date: '2026-03-28'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
The system must handle the full lifecycle of AI skill management: from fetching repositories (FR1-4) to synchronizing them into agent-accessible paths (FR5-8), and eventually aggregating 1,400+ skills into a performant routing manifest (FR9-12). The architecture must support both interactive TUI usage and strict non-interactive Agent execution (FR17-20).

**Non-Functional Requirements:**
Extremely aggressive performance targets (50ms startup, 2s aggregation) drive the choice of a multi-threaded Rust core. Security is managed through SHA-256 binary validation and strict file-system boundaries. Portability requires zero-dependency native binaries for Windows, macOS, and Linux.

**Scale & Complexity:**
The project handles large-scale file processing (>1,500 files) with a requirement for sub-second latency. The distribution model (NPM wrapper) adds significant DevOps complexity.

- Primary domain: CLI / Developer Tooling
- Complexity level: Medium
- Estimated architectural components: 4 (NPM Wrapper, CLI Engine, Sync/Fetch Module, Aggregation Engine)

### Technical Constraints & Dependencies

The primary constraint is the elimination of the Node/PowerShell runtime for core logic. The tool must be a standalone native binary. The distribution depends on the NPM registry and GitHub Actions for cross-platform compilation.

### Cross-Cutting Concerns Identified

- **Binary Integrity:** Automatic verification of native payloads.
- **File System Atomicity:** Ensuring `sync`/`fetch` operations are transactional.
- **Error Reliability:** Standardized POSIX exit codes and stream separation (stdout/stderr) for machine-readability.

## Starter Template Evaluation

### Primary Technology Domain
**CLI Tool / Systems Tooling** based on project requirements analysis focusing on native performance and zero-dependency distribution.

### Starter Options Considered
*   **Custom Binary Build (Manual):** Lowest overhead, but requires manually building the TUI/Event loop.
*   **Ratatui Simple Template:** Good for quick TUI apps, but less extensible for multi-threaded background workers.
*   **Ratatui Component Template (Selected):** Provides advanced state management and a modular component system that cleanly separates CLI parsing from heavy I/O operations.

### Selected Starter: `ratatui/templates` (Component Architecture)

**Rationale for Selection:**
The Component template offers the most modular architecture for our phased roadmap. It future-proofs the transition from a pure CLI to a rich Terminal UI by separating rendering from the core event loop. This ensures the 1,500-file aggregation logic won't block the UI thread.

**Initialization Command:**

```bash
# Required: cargo install cargo-generate
cargo generate ratatui/templates --name skill manage --template component
```

**Architectural Decisions Provided by Starter:**

**Language & Runtime:**
Rust 1.90.0 (released 2025-09-18) with asynchronous event handling provided by `tokio`, ensuring high concurrency for network operations during `fetch`.

**CLI Setup:**
Uses `clap` (derive) with a dedicated `src/cli.rs` module, allowing for complex subcommands like `sync` and `aggregate` to be added with minimal friction.

**Build Tooling & Optimization:**
Standard Rust `cargo` build system. Includes pre-configured `dist` profile for release builds.

**Error Handling:**
Utilizes `anyhow` for top-level binary error wrapping and `thiserror` for library-level modules, ensuring descriptive error reporting (FR16).

**Code Organization:**
Separates core "Components" (like the Synchronizer and Aggregator) into modular units that can be tested independently.

**Development Experience:**
Includes pre-configured testing infrastructure and logging via `tracing` to a `debug.log`, preventing stdout corruption during interactive usage.

**Note:** Project initialization using this command should be the primary story for the implementation phase.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- **Architecture Base:** Rust 2024 Edition with `tokio` for async I/O.
- **Distribution:** NPM wrapper with `optionalDependencies` for platform-specific binary orchestration.
- **State Storage:** Simple flat-file (JSON/CSV) architecture to maximize portability and hackability.

**Important Decisions (Shape Architecture):**
- **Concurrency:** `rayon` for data-parallel task execution in the aggregation engine.
- **CLI Protocol:** Hybrid output (Human-friendly text by default, `--json` for machine-readability).
- **Security:** Binary checksum verification (SHA-256) performed by the JS wrapper before execution.

**Deferred Decisions (Post-MVP):**
- **TUI Implementation:** Deferred to Phase 3; the modular Component architecture ensures this won't require a rewrite of core logic.
- **Enhanced Caching:** Postponed until users report performance bottlenecks with large numbers of remote repositories.

### Data Architecture
We will utilize **Flat File state management** using standard Rust crates like `serde_json` and `csv`.
- **Primary data:** `repos.json` (Source of truth for skill repositories).
- **Output data:** `hub-manifests.csv` (The central routing table).
- **Rationale:** This ensures the tool remains zero-config and human-editable. Performance is maintained through parallel file scanning via `rayon`.

### Authentication & Security
- **Binary Integrity:** The NPM JavaScript shim will perform a **SHA-256 checksum verification** of the native binary.
- **Verification Gate:** Verification happens at the "invoker" level, ensuring that if a binary is corrupted during download or tampered with locally, it never executes.

### API & Communication Patterns
- **User Interface:** Standard CLI using `clap` v4.6.0.
- **Machine Interface:** Every command supports a `--json` flag to emit structured, schema-validated JSON to `stdout`.
- **Error Handling:** Standardized POSIX exit codes (0 for success, non-zero for specific failure states) with all error logs directed to `stderr`.

### Infrastructure & Deployment
- **Build Matrix:** GitHub Actions will target the following platforms:
  - `x86_64-pc-windows-msvc`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-unknown-linux-musl` (Statically linked for maximum Linux portability).
- **Release Tooling:** **`cargo-dist`** v0.31.0 for automated artifact generation and GitHub Release creation.
- **Registry:** Public NPM registry under the `skill manage` namespace.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:**
6 primary areas where AI agents could make different choices have been standardized for consistency.

### Naming Patterns
- **Rust Code:** Strictly follow `rustfmt` standard: `snake_case` for functions/variables, `PascalCase` for Types/Traits/Structs.
- **File Naming:** `snake_case.rs`.
- **CLI Arguments:** `kebab-case` for flags and subcommands (e.g., `skill manage sync --dry-run`).
- **Data Exchange:** `camelCase` for JSON field keys to maintain compatibility with the NPM/NodeJS ecosystem.

### Structure Patterns
- **Project Organization:**
  - `src/cli.rs`: Command-line argument parsing and routing.
  - `src/app.rs`: Main application state management.
  - `src/components/`: Modular logic blocks (e.g., `fetcher.rs`, `syncer.rs`, `aggregator.rs`).
  - `tests/`: Integration tests that exercise the CLI as a whole.
  - In-file `#[cfg(test)]` blocks for unit testing specific logic.

### Format Patterns
- **API Response Formats:**
  - All machine-readable output must follow a tagged-envelope structure:
    ```json
    {
      "type": "Success | Error | Progress",
      "command": "fetch",
      "payload": { ... }
    }
    ```
- **Data Exchange Formats:**
  - Use ISO 8601 strings for date/time representations.
  - Use `null` for missing values; do not use sentinel values or empty strings.

### Communication Patterns
- **Event System Patterns:**
  - Use `tokio::sync::mpsc` channels for communication between long-running workers (fetching/aggregation) and the TUI/CLI reporting layer.
- **State Management Patterns:**
  - The `App` struct is the source of truth, modified through explicit update methods to maintain thread safety.

### Process Patterns
- **Error Handling Patterns:**
  - **Enforcement:** All fallible I/O operations MUST use `.context()` from the `anyhow` crate to provide meaningful error traces.
  - Use `thiserror` to define domain-specific error types (e.g., `StorageError`, `NetworkError`).
- **Loading State Patterns:**
  - CLI usage should output a bounded progress bar for `fetch` operations and a spinner for `aggregate` processing.

### Enforcement Guidelines
**All AI Agents MUST:**
- Run `cargo clippy` and `cargo fmt` before proposing any changes.
- Ensure all logic is isolated from platform-specific shell spawning (no `pwsh` calls).
- Use the **Atomic Write Pattern**: Write to `.tmp` files and perform a `std::fs::rename` for all local state updates to prevent corruption.

### Pattern Examples

**Good Examples:**
`anyhow::bail!("Failed to clone repository: {}", repo_url);`

**Anti-Patterns:**
`panic!("Repo missing");` or `std::process::Command::new("pwsh").arg("git clone...");`

## Project Structure & Boundaries

### Complete Project Directory Structure

```text
skill manage/
├── Cargo.toml                  # Rust workspace configuration
├── package.json               # NPM Wrapper (entrypoint for npx)
├── bin/                       # NPM executable scripts
│   └── skill manage.js      # JS shim for binary detection & execution
├── .github/
│   └── workflows/
│       └── release.yml        # GitHub Actions (cargo-dist) release pipeline
├── src/                       # Core Rust Source
│   ├── main.rs                # CLI Entrypoint & Async Event Loop
│   ├── cli.rs                 # Subcommand definitions (clap derive)
│   ├── app.rs                 # Central Application State
│   ├── error.rs               # Custom error Enums (thiserror)
│   ├── components/            # Logic Modules (Feature Boundaries)
│   │   ├── mod.rs
│   │   ├── fetcher.rs         # FR1-4: Repository Cloning logic
│   │   ├── syncer.rs          # FR5-8: Skill linking & junction logic
│   │   ├── aggregator.rs      # FR9-12: High-speed Skill parsing (rayon)
│   │   └── diagnostics.rs     # FR13-16: 'doctor' command validation
│   └── utils/                 # Shared Cross-Cutting Utilities
│       ├── mod.rs
│       ├── atomicity.rs       # Atomic Write Pattern implementation
│       └── crypto.rs          # SHA-256 Payload validation
├── tests/                     # Integration & Acceptance Tests
│   ├── integration_test.rs    # Core workflow verification
│   └── mock_repos/            # Test data for local filesystem testing
├── repos.json                 # Input Manifest (Source of Truth)
└── hub-manifests.csv          # Generated Manifest (Central Routing)
```

### Architectural Boundaries

**The NPM Barrier:**
- The `package.json` and `bin/*.js` act as the **Distribution Boundary**. No domain logic exists in Javascript; its only job is to locate, verify, and spawn the native Rust binary.

**The IO Boundary:**
- All platform-specific filesystem quirks (Windows Junctions vs. Unix Symlinks) are encapsulated within `src/utils/atomicity.rs`. The rest of the app interacts with a high-level `AtomicWrite` trait.

**The Data Boundary:**
- The `Fetcher` (Network) and `Aggregator` (Local FS) are completely decoupled. They communicate via the `App` state in `src/app.rs`, ensuring that network failures don't impact local diagnostic runs.

### Requirements to Structure Mapping

| Feature Set | Primary Location | Key Pattern |
| :--- | :--- | :--- |
| **Repo Management** | `src/components/fetcher.rs` | Async `tokio` streams |
| **Skill Aggregation** | `src/components/aggregator.rs` | Parallel `rayon` iterators |
| **Diagnostics** | `src/components/diagnostics.rs` | Strict schema validation |
| **Atomic Updates** | `src/utils/atomicity.rs` | Temp-file rename pattern |
| **CLI / TUI Surface** | `src/cli.rs` & `src/main.rs` | `clap` + internal event loop |

### Integration Points

**Internal Communication:**
The CLI entrypoint routes subcommands from `cli.rs` to the `App` in `app.rs`. The `App` spawns component tasks using an `mpsc` channel for unified reporting.

**External Integrations:**
The only external dependency is the GitHub API / Git protocol for repository fetching.

**Data Flow:**
`repos.json` → `Fetcher` → `src/` directory → `Aggregator` → `hub-manifests.csv`.

## Architecture Validation Results

### Coherence Validation ✅
- **Decision Compatibility:** All core technologies (Rust 1.90, Tokio 1.50, Rayon 1.11) are fully compatible and represent the leading edge of systems engineering in 2025/2026.
- **Pattern Consistency:** Established a "best-of-all-worlds" naming convention (Snake for Code, Camel for JSON, Kebab for CLI).
- **Structure Alignment:** The `src/components/` modularity directly mirrors the FR categories in the PRD, facilitating parallel development by multiple AI agents.

### Requirements Coverage Validation ✅
- **Functional Requirements Coverage:** Every FR group has a dedicated implementation module in the proposed project tree.
- **Non-Functional Requirements Coverage:**
  - **Performance:** Addressed by `rayon` and native compilation.
  - **Security:** Addressed by the JS-side SHA-256 verification gate.
  - **Reliability:** Addressed by the mandatory Atomic Write Pattern.

### Implementation Readiness Validation ✅
- **Decision Completeness:** All critical and important decisions are documented with specific versions and rationale.
- **Structure Completeness:** A complete project tree has been defined, removing any ambiguity about where code should live.
- **Pattern Completeness:** All 6 major conflict points for AI agents have been addressed for consistency.

### Gap Analysis Results
- **Priority (Minor):** Final JSON schemas for each command payload will be refined during implementation.
- **Confidence Level:** **HIGH** - The architecture is specific, versioned, and explicitly anti-fragile.
- **Key Strengths:** Zero-dependency distribution via NPX and ultra-high-performance multi-threaded aggregation.

### Architecture Completeness Checklist
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped
- [x] Critical decisions documented with versions
- [x] Technology stack fully specified
- [x] Naming conventions established
- [x] Complete directory structure defined
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment
**Overall Status:** READY FOR IMPLEMENTATION

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented.
- Use implementation patterns consistently across all components.
- Respect project structure and architectural boundaries.
- **First Implementation Priority:** Initialize the workspace using `cargo generate ratatui/templates --name skill manage --template component`.
