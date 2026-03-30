---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
  - step-12-complete
inputDocuments:
  - readme.md
workflowType: 'prd'
classification:
  projectType: CLI Tool
  domain: Developer Tools / AI Agent Workflows
  complexity: Medium
  projectContext: brownfield
---

# Product Requirements Document - skill-manage

**Author:** ABDULSAMED
**Date:** 2026-03-28

## Executive Summary

skill-manage is an ultra-lightweight, zero-latency local routing layer for AI agent workflows. It enables developers and autonomous agents to instantly locate and invoke capabilities from a curated library of over 1,400 skills. By migrating from a Node.js/PowerShell architecture to a natively compiled Rust binary distributed via NPX, the tool provides immediate, friction-free tool access while completely eliminating host system dependencies, PowerShell execution policies, and Node runtime overhead.

### What Makes This Special

The core differentiator is the intersection of massive skill scale, BMAD-pattern governance, and instantaneous execution. Compiling the routing logic into a single native Rust binary eliminates interpretation delays and environment mismatches. Distributing this blazing-fast executable via NPM/NPX leverages the most ubiquitous package manager for a true "zero-install" user experience. Agents and developers can execute complex skill aggregation and invocation seamlessly, focusing on their tasks rather than fighting environment setups.

## Project Classification

* **Project Type:** CLI Tool (Native Binary distributed via NPM/NPX)
* **Domain:** Developer Tools / AI Agent Workflows
* **Complexity:** Medium (Requires cross-platform binary compilation, NPM wrapping, and high-performance text parsing)
* **Project Context:** Brownfield (Migrating existing functional Node.js/PowerShell logic to Rust for maximum performance)

## Success Criteria

### User Success
* **Frictionless Installation:** Users can execute the tool via a simple `npx skill-manage` command (or equivalent) without ever needing to pre-install specific versions of Node.js, PowerShell, or environment variables.
* **Instantaneous Feedback:** Actions feel instantaneous. Users don't experience the typical startup lag associated with interpreted scripting languages.
* **Reliability:** Commands execute predictably across Windows, macOS, and Linux without platform-specific bugginess or weird terminal behavior.

### Business Success
* **Adoption:** Increased usage and contribution from the developer community due to the dramatically lowered barrier to entry.
* **Zero Support Tickets for Setup:** Complete elimination of "How do I install this?" or "My execution policy is blocking the script" issues.
* **Modernized Ecosystem:** Successfully transitioning the project from a set of individual scripts into a cohesive, professional-grade product.

### Technical Success
* **Cross-Platform Compilation:** The Rust codebase successfully builds native standalone executables for Windows (`.exe`), macOS, and Linux.
* **Performance:** CLI startup time is consistently under 50ms, and heavy parsing operations (like aggregation) execute in a fraction of the time compared to the previous PowerShell implementation.
* **Seamless NPM Wrapper:** The NPM package successfully detects the user's OS and architecture, downloading and executing the correct native Rust binary invisibly in the background.

### Measurable Outcomes
* 100% reduction in required system dependencies (Node/PS1 no longer needed by the core binary).
* 80%+ reduction in command execution time for heavy operations.
* Successful execution on all 3 major operating systems out-of-the-box.

## User Journeys

### 1. The Autonomous Agent Developer (Primary Success Path)
**Persona:** Sarah, building a complex multi-agent system that requires dynamic tool loading.
* **Opening Scene:** Sarah is configuring her agents and realizes they need access to the `bmad-create-prd` skill, but she doesn't want to manually copy-paste hundreds of markdown files from GitHub into her project.
* **Rising Action:** She opens her terminal and simply runs `npx skill-manage fetch`. 
* **Climax:** Because the CLI is now a native Rust binary wrapped in NPM, there is *zero* installation delay or Node environment setup. The tool instantly reads her local manifest and downloads exactly the skills she requested.
* **Resolution:** Within 2 seconds, her `.agent/skills` folder is populated. She's thrilled by the pure, frictionless speed and can immediately get back to coding her agent logic.

### 2. The Windows Corporate Developer (Edge Case / Pain Relief)
**Persona:** Alex, working on a strictly managed corporate Windows machine.
* **Opening Scene:** Alex tries to run a legacy powershell tool, but the corporate `ExecutionPolicy` blocks the `.ps1` script entirely, halting his workflow. He’s extremely frustrated.
* **Rising Action:** He checks the docs and sees the new NPX command. He runs `npx skill-manage sync`.
* **Climax:** The NPM wrapper detects he's on Windows and seamlessly executes the pre-compiled `skill-manage-win.exe` binary in the background. It completely bypasses the PowerShell execution restrictions because it's a native executable.
* **Resolution:** The sync succeeds on the first try without any security pop-ups or errors. Alex feels immense relief that it "just works."

### 3. The Skills Bank Maintainer (Admin/Operations)
**Persona:** Omar, the repository owner who curates the 1,400+ skills library.
* **Opening Scene:** Omar has just added 50 new repository URLs to the `repos.json` manifest. Now he needs to regenerate the massive `hub-manifests.csv` and all sub-hub index files.
* **Rising Action:** Previously, this PowerShell script took nearly 45 seconds to scan thousands of files, slowing down his daily workflow. He runs the new `skill-manage aggregate` native command.
* **Climax:** The Rust engine's blazing-fast multi-threaded I/O rips through the 1,400 files, applying the BMAD phase/dependency rules and generating the CSV.
* **Resolution:** The entire aggregation finishes in under 2 seconds. Omar smiles, realizing that maintaining the project at scale is no longer a chore.

### 4. The Troubled Integrator (Support/Investigation)
**Persona:** Maya, trying to debug why a specific skill isn't showing up in her IDE.
* **Opening Scene:** Maya synced the skills, but her Copilot still isn't routing correctly to the `backend/databases` skill. She doesn't know where the failure happened.
* **Rising Action:** She runs `npx skill-manage doctor`. 
* **Climax:** The Rust binary instantly runs a rapid diagnostic check validating the structural integrity of the CSV manifest against the local file system, highlighting that her target path is missing a `.md` extension.
* **Resolution:** She receives a clear, color-coded terminal error telling her exactly what to fix. She fixes the path, re-runs sync, and the skill appears. Her frustration turns into appreciation for the tool's helpfulness.

## Product Scope & Strategy

**Approach:** The primary goal is to immediately relieve the pain of PowerShell execution policies and slow parsing speeds by delivering the core file-system operations (`fetch` and `sync`) alongside aggregation capabilities via a native binary.
**Resource Requirements:** Rust development + CI/CD DevOps setup.

### Core Architecture Capabilities
* **NPM OS-Detection Wrapper:** JavaScript wrapper to detect os (`win32`, `darwin`, `linux`) and architecture to dynamically invoke the correct binary.
* **No-Dependency Core:** The native Rust binary must not spawn `pwsh` internally or rely on external script interpreters.
* **Multi-threaded Aggregation:** The Rust file scanner must utilize parallel processing frameworks (like `rayon`) to achieve the 2-second aggregation target.

### Supported Feature Set
* Rust binary implementing `fetch` (reading `repos.json` and cloning missing skill repositories into `lib/`).
* Rust binary implementing `sync` (safe file copying/junctioning to global tool directories).
* Rust binary implementing the heavy `aggregate` engine (keyword scoring, semantic matching, sub-hub routing).
* The `doctor` diagnostic command for schema validation.
* A rich Interactive Terminal UI (TUI) replacing standard prompts (using libraries like `ratatui` or `inquire`).
* GitHub Actions CI/CD pipeline for cross-compiling the Rust code (Mac, Linux, Windows) and seamlessly publishing to NPM.

### Risk Mitigation Plan
**Technical Risks:** Managing a cross-platform compilation matrix and NPM binary downloads is notoriously tricky. *Mitigation:* Rely on trusted, battle-tested publishing tools like `cargo-dist` or `napi-rs` to automate the release process rather than writing custom deployment shell scripts.
**Market Risks:** Users rejecting the tool due to unfamiliarity. *Mitigation:* Ensure the NPX command perfectly mimics standard POSIX CLI behavior with robust `--help` documentation.
**Resource Risks:** Rust's steep learning curve slowing down development. *Mitigation:* Keep the initial engineering strictly focused on basic file I/O operations (`fetch`/`sync`) before moving on to the complex string-parsing required for the aggregation engine.

## Technical & Domain Constraints

### Domain Patterns (CLI Standards)
* **Argument Parsing:** Use a robust Rust argument parser (like `clap`) to handle complex subcommands (`fetch`, `sync`, `aggregate`, `doctor`), POSIX-compliant long/short flags (`--dry-run`, `--verbose`, `-v`), and rich inline documentation (`--help`).
* **POSIX Compliance:** The CLI must follow standard developer expectations: utilizing proper exit codes (`0` for success, non-zero for specific failure states e.g., `1` for general error, `2` for missing config, `126` for permission denied) so it can be reliably used inside automated CI/CD pipelines or Agent automated scripts.
* **I/O Streams & Logging:** Strict separation of streams: Structured output goes to `stdout`, while critical errors and warnings go to `stderr`. Support for the standard `NO_COLOR` environment variable to ensure clean output when piped to dumb terminals.

### Configuration & File System Integrity
* **Atomic Operations:** Because the tool's primary purpose is fetching and synchronizing thousands of files, it must use atomic file operations. If a `sync` or `fetch` is interrupted (e.g., user hits `Ctrl+C`), it must not leave the target directory in a corrupted or half-written state.
* **Local State:** Rely on `repos.json` and the `lib/` directory relative to the current working directory for local project execution.
* **Global Configuration:** Support resolving a global config directory (e.g., `~/.config/skill-manage/`) for user-wide settings, authentication tokens, or cached skill data.

### Security Constraints
* **Binary Integrity:** Since the NPX (JavaScript) wrapper will download or execute a native binary, it must perform checksum validation to ensure the binary hasn't been tampered with to prevent supply-chain attacks.
* **Permissions:** The tool must strictly request and utilize only local file system permissions required for the `lib/` and target config directories. It must never inadvertently expose local environment variables or private keys.
* **Gatekeeper/Defender Readiness:** macOS binaries may require Apple Notarization, and Windows binaries may require signing to prevent OS-level security warnings from blocking execution.

## Functional Requirements

### Repository Management
* **FR1:** The system can read a local manifest file (e.g., `repos.json`) containing a list of remote repository URLs.
* **FR2:** The CLI can download (clone) remote repositories from the manifest into a local `lib/` directory.
* **FR3:** The CLI can detect if a repository has already been downloaded to prevent redundant fetching operations.
* **FR4:** The CLI can pull the latest updates for previously downloaded repositories.

### Skill Synchronization
* **FR5:** The CLI can locate and identify valid skill directories within the local `lib/` folder.
* **FR6:** The CLI can synchronize (copy or link) discovered skills into a targeted destination directory (e.g., `~/.agent/skills/`).
* **FR7:** The CLI can safely overwrite or update existing skills in the target directory using atomic file operations to prevent data corruption.
* **FR8:** The user can specify a custom absolute or relative destination path for the synchronization process via command parameters.

### Hub Aggregation
* **FR9:** The system can parse skill manifest files (`SKILL.md`, `skills-index.json`, etc.) across multiple sub-hub directories.
* **FR10:** The CLI can generate a unified, centralized routing manifest (e.g., `hub-manifests.csv`) containing all parsed skill metadata.
* **FR11:** The system can apply workflow progression rules (phases, dependencies) to the skills during manifest generation.
* **FR12:** The system can generate semantic routing triggers and assign matching score weights for each skill.

### Diagnostics & Validation
* **FR13:** The CLI can validate the structural integrity and syntax of the `repos.json` manifest.
* **FR14:** The CLI can verify that all generated CSV and JSON schemas precisely match the required system specification.
* **FR15:** The system can detect and report broken file paths, missing extensions, or orphaned skills in the routing manifests.
* **FR16:** The CLI can output clear, actionable error messages pointing the user directly to the schema violation or missing file.

### Core System Execution
* **FR17:** The user can invoke the CLI from any directory using the NPX platform.
* **FR18:** The system can automatically detect the host operating system architecture and seamlessly execute the appropriate native binary.
* **FR19:** The user can access built-in help and command documentation via standard CLI flags (e.g., `--help`).
* **FR20:** The system can cleanly separate its outputs, sending structured data to `stdout` and diagnostic/error logs to `stderr`.

## Non-Functional Requirements

### Performance
* **Startup Time:** The time from triggering the command (`skill-manage <cmd>`) to the first `stdout` log must be under **50ms**.
* **Aggregation Speed:** The `aggregate` command must parse up to 1,500 markdown files and generate the complete CSV manifest in under **2.0 seconds** on standard hardware (e.g., Apple M1 / Intel i5 equivalent).
* **Memory Efficiency:** Peak RAM utilization during the heaviest multi-threaded file aggregation pipeline must never exceed **500MB**, ensuring the tool can safely run inside constrained CI/CD environments (like standard GitHub Action runners) without crashing.

### Security
* **Supply Chain Validation:** Any native binaries downloaded dynamically by the NPM/JS wrapper must undergo a **SHA-256 checksum verification** against a static manifest before execution, ensuring users aren't exposed to MITM injection hacks.
* **File System Boundaries:** The executable must implicitly restrict file I/O modification operations purely to the explicitly targeted directories (`.agent/skills`, `lib/`) and never attempt to modify system directories.
* **Execution Privileges:** The process must never demand or require elevated permissions (`sudo` or Windows Run as Administrator) to function correctly.

### Portability & Reliability
* **Zero System Dependencies:** The pre-compiled Rust binaries must execute successfully on clean installations of Windows 10/11, macOS (12+), and Ubuntu (20.04+) without requiring the developer to install missing C-runtime libraries, `Node-Gyp`, or specific python distributions via `brew` / `apt-get`.
* **Network Resilience:** If the `fetch` command loses network connectivity midway through pulling 100 repositories, it must gracefully fail without corrupting existing downloads, and the next run must resume from the failure point.
