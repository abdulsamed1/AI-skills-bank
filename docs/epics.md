---
stepsCompleted:
  - step-01-extract-requirements
  - step-02-decomposition-complete
inputDocuments:
  - prd.md
  - architecture.md
---

# skill manage - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for skill manage, decomposing the requirements from the PRD and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Read local manifest `repos.json` containing remote repository URLs.
FR2: Download (clone) remote repositories from manifest into local `src/`.
FR3: Detect already downloaded repositories to prevent redundant fetching.
FR4: Pull latest updates for previously downloaded repositories.
FR5: Locate and identify valid skill directories within `src/`.
FR6: Synchronize discovered skills into targeted destination (e.g., `~/.agent/skills/`).
FR7: Safely overwrite/update skills using atomic file operations.
FR8: Support custom destination paths via command parameters.
FR9: Parse skill manifest files across multiple sub-hub directories.
FR10: Generate central routing manifest `hub-manifests.csv`.
FR11: Apply workflow progression rules (phases, dependencies) during generation.
FR12: Generate semantic routing triggers and matching score weights.
FR13: Validate structural integrity/syntax of `repos.json`.
FR14: Verify CSV/JSON schemas against system specification.
FR15: Detect/report broken paths, missing extensions, or orphaned skills.
FR16: Output clear, actionable error messages with schema violations.
FR17: Invoke CLI from any directory via NPX platform.
FR18: Detect host OS/architecture and execute correct native binary.
FR19: Access built-in help/documentation via `--help`.
FR20: Separate binary outputs (stdout for data, stderr for logs).

### NonFunctional Requirements

NFR1: Startup time must be under 50ms.
NFR2: Aggregation of 1,500 files must complete in under 2.0 seconds.
NFR3: Peak RAM utilization during aggregation must not exceed 500MB.
NFR4: SHA-256 checksum verification for native binaries in the NPM shim.
NFR5: Strict file I/O boundaries (only explicitly targeted directories).
NFR6: No elevated permissions (sudo/Admin) required for execution.
NFR7: Zero system dependencies (statically linked/standalone binaries).
NFR8: Network resilience (graceful failure/resume without corruption).

### Additional Requirements

- **Starter:** Initialize using `ratatui/templates` (Component Architecture).
- **Core:** Implement using Rust 1.90.0.
- **Async/IO:** Use `tokio` for network operations and `rayon` for parallel aggregation.
- **Deployment:** Configure `cargo-dist` v0.31.0 for cross-compilation release matrix.
- **Reliability:** Mandatory Atomic Write Pattern (temp file + rename).
- **Standardization:** Kebab-case CLI flags and Camel-case JSON field names.

### UX Design Requirements

UX-DR1: Color-coded terminal error messages for actionable diagnostics.
UX-DR2: Bounded progress bars for repository `fetch` operations.
UX-DR3: Animated spinners for heavy `aggregate` processing cycles.
UX-DR4: Interactive Terminal UI (TUI) surface for future phases.
UX-DR5: Stream separation: Clean structured stdout vs verbose stderr logs.

### FR Coverage Map

| FR | Description | Epic | Story ID |
| :--- | :--- | :--- | :--- |
| FR1 | Read local manifest `repos.json` | Epic 2 | story-02-manifest-parsing |
| FR2 | Download (clone) remote repositories | Epic 2 | story-02-fetch-implementation |
| FR3 | Detect already downloaded repositories | Epic 2 | story-02-fetch-implementation |
| FR4 | Pull latest updates for repositories | Epic 2 | story-02-fetch-implementation |
| FR5 | Identify valid skill directories | Epic 3 | story-03-sync-implementation |
| FR6 | Synchronize skills to destination | Epic 3 | story-03-sync-implementation |
| FR7 | Safe overwrite using atomic operations | Epic 3 | story-03-atomic-file-ops |
| FR8 | Support custom destination paths | Epic 3 | story-03-sync-implementation |
| FR9 | Parse skill manifest files | Epic 4 | story-04-aggregate-implementation |
| FR10 | Generate `hub-manifests.csv` | Epic 4 | story-04-aggregate-implementation |
| FR11 | Apply workflow progression rules | Epic 4 | story-04-aggregate-rules |
| FR12 | Generate triggers and matching scores | Epic 4 | story-04-aggregate-rules |
| FR13 | Validate `repos.json` integrity | Epic 5 | story-05-doctor-implementation |
| FR14 | Verify CSV/JSON schemas | Epic 5 | story-05-schema-validation |
| FR15 | Detect broken paths/orphaned skills | Epic 5 | story-05-doctor-implementation |
| FR16 | Clear actionable error messages | Epic 5 | story-05-doctor-implementation |
| FR17 | Invoke via NPX platform | Epic 1 | story-01-npm-wrapper |
| FR18 | Detect host OS/architecture | Epic 1 | story-01-npm-wrapper |
| FR19 | Built-in help via `--help` | Epic 6 | story-06-clap-integration |
| FR20 | Separate stdout/stderr streams | Epic 6 | story-06-clap-integration |

## Epic List

### Epic 1: Project Core & Infrastructure
**Goal:** Establish the foundational Rust architecture, distribution mechanism, and cross-platform build pipeline.

- **story-01-rust-init:** Initialize Rust workspace using Ratatui component template.
- **story-01-npm-wrapper:** Create NPM package with OS/architecture detection shim to spawn the native binary.
- **story-01-cargo-dist:** Configure `cargo-dist` and GitHub Actions for cross-compilation release matrix.

### Epic 2: Repository Management (Fetch)
**Goal:** Implement the ability to parse a manifest and pull/update skill repositories from GitHub.

- **story-02-manifest-parsing:** Implement `repos.json` parsing with schema validation.
- **story-02-fetch-implementation:** Implement the `fetch` subcommand using `tokio` for async git operations.
- **story-02-fetch-progress:** Add bounded progress bars for repository download operations.

### Epic 3: Skill Synchronization (Sync)
**Goal:** Provide a safe mechanism to link or copy discovered skills into target agent directories.

- **story-03-sync-implementation:** Implement the `sync` subcommand with support for custom destination paths.
- **story-03-atomic-file-ops:** Implement the Atomic Write Pattern (temp + rename) for all file system modifications.
- **story-03-junction-logic:** Support Windows Junctions and Unix Symlinks for efficient synchronization.

### Epic 4: Hub Aggregation (Aggregate)
**Goal:** High-performance parsing of 1,500+ skills into a centralized routing manifest.

- **story-04-aggregate-implementation:** Implement the `aggregate` subcommand using `rayon` for parallel file processing.
- **story-04-csv-generation:** Build the CSV generator for `hub-manifests.csv` with 2.0s performance target.
- **story-04-aggregate-rules:** Implement BMAD phase rules, semantic trigger generation, and score weighting.

### Epic 5: Diagnostics & Validation (Doctor)
**Goal:** Provide automated integrity checks and helpful troubleshooting for the skills bank.

- **story-05-doctor-implementation:** Implement the `doctor` subcommand for general integrity checks.
- **story-05-schema-validation:** Implement strict JSON/CSV schema validation against system specs.
- **story-05-error-reporting:** Develop color-coded, actionable error diagnostic output.

### Epic 6: CLI Surface & Polishing
**Goal:** Refine the user experience with proper argument parsing, styling, and terminal interaction.

- **story-06-clap-integration:** Implement full `clap` derive structure for subcommands and flags.
- **story-06-terminal-styling:** Apply consistent terminal colors, spinners, and theme.
- **story-06-stream-separation:** Ensure strict stdout (data) and stderr (logs) separation.

