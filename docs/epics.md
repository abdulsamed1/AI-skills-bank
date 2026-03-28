---
stepsCompleted:
  - step-01-extract-requirements
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

{{requirements_coverage_map}}

## Epic List

{{epics_list}}
