// Archived interactive UI helpers for skills-bank
// Moved out of src/main.rs to keep the non-interactive CLI minimal.
// This file is intentionally NOT compiled; it's kept for reference/recovery.

use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::collections::HashSet;
use std::path::Path;

// The following functions were archived from src/main.rs:

pub async fn run_interactive_archived(repo_root: &Path, config_path: &Path) -> Result<()> {
    // Original run_interactive implementation archived for reference.
    unimplemented!("archived interactive UI");
}

pub fn run_setup_wizard_archived(repo_root: &Path) -> Result<()> {
    unimplemented!("archived setup wizard");
}

pub fn collect_repo_urls_archived(_theme: &ColorfulTheme) -> Result<Vec<String>> {
    unimplemented!("archived collect_repo_urls");
}
