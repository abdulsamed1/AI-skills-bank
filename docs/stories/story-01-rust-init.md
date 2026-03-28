# Story: Initialize Rust Workspace

**ID:** story-01-rust-init
**Epic:** Epic 1: Project Core & Infrastructure
**Status:** TODO

## Description
Initialize the Rust project using the Ratatui component template as specified in the Architecture Decision Document. This provides the modular structure needed for the application's components (Fetcher, Syncer, Aggregator).

## Acceptance Criteria
- [ ] Project initialized using `cargo generate ratatui/templates --name skill manage --template component`.
- [ ] `Cargo.toml` includes dependencies: `tokio`, `clap`, `anyhow`, `thiserror`, `serde`, `serde_json`, `csv`, `rayon`.
- [ ] Basic directory structure (`src/cli.rs`, `src/app.rs`, `src/components/`) exists.
- [ ] `cargo build` and `cargo test` pass on the initial boilerplate.

## Implementation Details
- Use `cargo generate` (requires installation if not present).
- Name the project `skill manage`.
- Ensure the template version is consistent with Ratatui's current best practices.

## Verification Plan
- Run `cargo check` to ensure all dependencies resolve.
- Run `cargo run -- --help` to verify the initial CLI surface.
