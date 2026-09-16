# Contributing to skills-bank

Thank you for your interest in contributing to `skills-bank`! This project provides a high-performance, ultra-lightweight skill aggregation, semantic classification, and routing layer for AI agents, written in Rust.

---

## Code of Conduct

All contributors are expected to uphold a professional, collaborative, and harassment-free environment.

---

## Development Setup

### Prerequisites

- **Rust 1.70+** & Cargo ([Install via rustup](https://www.rust-lang.org/tools/install))
- **Git**
- **Node.js 18+** (optional, for the npm distribution wrapper)

### Getting Started

1. **Clone the repository**:
   ```bash
   git clone https://github.com/abdulsamed1/AI-skills-bank.git
   cd AI-skills-bank
   ```

2. **Verify the environment**:
   ```bash
   cargo check --tests
   ```

3. **Run the test suite**:
   ```bash
   cargo test
   ```

4. **Build the release binary**:
   ```bash
   cargo build --release
   ```
   The resulting binary will be located at `target/release/skills-bank`.

---

## Architectural Invariants & Guardrails

When working on `skills-bank`, keep the following rules in mind:

1. **Guardrail on `lib/` (Non-Negotiable)**:
   - The `lib/` directory contains raw clone caches (2GB+, 10,000+ files).
   - **NEVER** run recursive globs (like `**/SKILL.md`) against the repository root or scan `lib/` directly.
   - Always follow the canonical 3-hop routing model:
     `skills-aggregated/AGENTS.md` ➔ `skills-aggregated/<hub>/SKILL.md` ➔ `<sub-hub>/routing.csv`.

2. **Atomic File and Symlink Operations**:
   - All disk mutations (manifest updates, directory syncs, symlink creation) must use the helpers in `src/utils/atomicity.rs`. Operations must be crash-resilient and cross-platform (Windows junctions, macOS/Linux symlinks).

3. **Error Handling**:
   - Prefer structured domain errors using `thiserror` for library components and `anyhow` for top-level CLI handlers. Avoid `unwrap()` on production paths.

4. **Test Coverage**:
   - Any new routing logic, normalization rules, or sync mechanics must include unit tests in `tests/`.

---

## Pull Request Process

1. **Create a topic branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Implement your changes** and add relevant unit/integration tests.

3. **Verify tests and formatting**:
   ```bash
   cargo test
   cargo fmt -- --check
   ```

4. **Commit with descriptive messages** and push to your fork.

5. **Open a Pull Request** against `master`, detailing your implementation and test evidence.
