use anyhow::{bail, Context, Result};
// Interactive UI archived. Keep CLI non-interactive by default.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use skills_bank::components::native_pipeline::{
    aggregate_to_output, sync_output_to_targets, NativeSyncMode,
};
use skills_bank::components::diagnostics::Diagnostics;
use skills_bank::components::fetcher::Fetcher;
use skills_bank::components::manifest::{RepoManifest, Repository};
use skills_bank::components::CommandResult;
use skills_bank::utils::progress::ProgressManager;
use skills_bank::utils::theme::Theme;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use walkdir::WalkDir;

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Clone)]
struct ToolDef {
    key: &'static str,
    global_rels: &'static [&'static str],
    local_rels: &'static [&'static str],
}

const TOOL_DEFS: &[ToolDef] = &[
    ToolDef {
        key: "claude",
        global_rels: &[".claude/skills"],
        local_rels: &[".claude/skills"],
    },
    ToolDef {
        key: "hermes",
        global_rels: &[".hermes/skills"],
        local_rels: &[
            ".hermes/skills",
        ],
    },
    ToolDef {
        key: "code",
        global_rels: &[".agents/skills"],
        local_rels: &[".agents/skills"],
    },
    ToolDef {
        key: "cursor",
        global_rels: &[".cursor/skills"],
        local_rels: &[".cursor/skills"],
    },
    ToolDef {
        key: "antigravity",
        global_rels: &[".agent/skills"],
        local_rels: &[".agent/skills"],
    },
    ToolDef {
        key: "copilot",
        global_rels: &[".github/skills"],
        local_rels: &[".github/skills"],
    },
    ToolDef {
        key: "windsurf",
        global_rels: &[".codeium/windsurf/skills"],
        local_rels: &[".windsurf/skills"],
    },
    ToolDef {
        key: "opencode",
        // ponytail: minimal mapping — opencode discovers ~/.config/opencode/skills/*/SKILL.md and ~/.agents/skills; keep both global/local standard
        global_rels: &[".config/opencode/skills"],
        local_rels: &[".opencode/skills"],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncScope {
    Global,
    Local,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_sync_mode")]
    pub mode: String,
    #[serde(default)]
    pub targets: HashMap<String, String>,
}

fn default_sync_mode() -> String {
    "local".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub repo_root: String,
    #[serde(default)]
    pub workspace_root: String,
    #[serde(default = "default_sync_scope")]
    pub sync_scope: SyncScope,
    #[serde(default)]
    pub tools: Vec<String>,
    pub repositories: Vec<Repository>,
    #[serde(default = "default_category_exclusions")]
    pub category_exclusions: Vec<String>,
    #[serde(default)]
    pub sync: Option<SyncConfig>,
}

fn default_sync_scope() -> SyncScope {
    SyncScope::Both
}

impl SetupConfig {
    fn repo_root_path(&self) -> PathBuf {
        PathBuf::from(&self.repo_root)
    }

    fn workspace_root_path(&self) -> PathBuf {
        PathBuf::from(&self.workspace_root)
    }
}

fn default_category_exclusions() -> Vec<String> {
    vec!["medicine".to_string(), "law".to_string()]
}

fn normalize_exclusion_category(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in raw.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

fn apply_exclusion_env(config: Option<&SetupConfig>) {
    let categories = config
        .map(|cfg| {
            if cfg.category_exclusions.is_empty() {
                default_category_exclusions()
            } else {
                cfg.category_exclusions.clone()
            }
        })
        .unwrap_or_else(default_category_exclusions);

    let payload = categories
        .iter()
        .map(|c| normalize_exclusion_category(c))
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>()
        .join(";");

    std::env::set_var("SKILL_MANAGE_EXCLUSIONS", payload);
}

struct DirGuard {
    original: PathBuf,
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    if let Err(err) = run().await {
        eprintln!("\n[ERROR] {err:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let repo_root = discover_repo_root()?;
    let config_path = config_path(&repo_root);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // Default to non-interactive 'run' behavior when no args provided.
        let config = ensure_config(&repo_root, &config_path)?;
        apply_exclusion_env(Some(&config));
        run_full_pipeline(&config).await?;
        return Ok(());
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_help();
        }
        "--version" | "-v" => {
            println!("skills-bank v0.1.0");
        }
        "setup" | "init" => {
            let config = run_setup_wizard(&repo_root)?;
            save_config(&config_path, &config)?;
            apply_exclusion_env(Some(&config));
            run_full_pipeline(&config).await?;
        }
        "run" => {
            let config = ensure_config(&repo_root, &config_path)?;
            apply_exclusion_env(Some(&config));
            run_full_pipeline(&config).await?;
        }
        "fetch" => {
            let config = ensure_config(&repo_root, &config_path)?;
            if let Some(manifest) = prepare_manifest(&repo_root, &config.repositories)? {
                run_fetch(&repo_root, manifest).await?;
            } else {
                bail!("No repositories configured. Run setup or provideconfig.json.");
            }
        }
        "aggregate" => {
            let loaded = load_config(&config_path)?;
            apply_exclusion_env(loaded.as_ref());
            let output_dir = repo_root.join("skills-aggregated");
            if let Err(native_err) = run_aggregate_native(&repo_root, &output_dir, None, false).await {
                eprintln!("[ERROR] Native aggregation failed (no archive fallback): {}", native_err);
                return Err(native_err);
            }
        }
        "sync" => {
            let config = ensure_config(&repo_root, &config_path)?;
            let targets = resolve_sync_targets(&repo_root, &config)?;
            let output_dir = repo_root.join("skills-aggregated");
            let dry_run = args.iter().any(|arg| arg == "--dry-run") || env::var("SKILL_MANAGE_DRY_RUN").is_ok();
            
            if dry_run {
                println!("\n[DRY-RUN MODE] Preview of aggregated skills that will be synced:\n");
                run_sync_dry_run(&output_dir, &targets)?;
                println!("\n[DRY-RUN] No changes made. Run without --dry-run to execute the sync.");
            } else {
                println!("\nDetected {} sync targets.", targets.len());
                for t in &targets {
                    println!("  - {}", t.display());
                }

                // Non-interactive mode: auto-approve unless explicitly disabled.
                let auto_approve = std::env::var("SKILL_MANAGE_YES").map(|v| v == "true" || v == "1").unwrap_or(true);
                if !auto_approve {
                    println!("Sync cancelled by SKILL_MANAGE_YES=false");
                    return Ok(());
                }

                if let Err(native_err) = run_sync_native(&output_dir, &targets) {
                    eprintln!("[ERROR] Native sync failed (no archive fallback): {}", native_err);
                    return Err(native_err);
                }
            }
        }
        "cleanup-legacy-duplicates" | "cleanup-legacy" | "cleanup-src-duplicates" | "cleanup-src" => {
            run_cleanup_legacy_duplicates(&repo_root)?;
        }
        "add-repo" => {
            let mut config = ensure_config(&repo_root, &config_path)?;
            apply_exclusion_env(Some(&config));
            let repo_url = if let Some(url) = args.get(2) {
                url.clone()
            } else {
                bail!("Usage: skills-bank add-repo <URL>");
            };

            run_add_repo_pipeline(&repo_root, &config_path, &mut config, &repo_url).await?;
        }
        "doctor" => {
            let _ = run_doctor(&repo_root)?;
        }
        "release-gate" | "gate" => {
            run_release_gate(&repo_root)?;
        }
        "tui" => {
            bail!("Interactive TUI has been archived. Use non-interactive commands instead.");
        }
        "list" | "ls" => {
            let output_dir = repo_root.join("skills-aggregated");
            if !output_dir.exists() {
                eprintln!("Skills not aggregated yet. Run 'skills-bank aggregate' first.");
                return Ok(());
            }
            
            let filter = args.get(2).map(|s| s.as_str());
            run_list_skills(&output_dir, filter)?;
        }
        "search" => {
            let output_dir = repo_root.join("skills-aggregated");
            if !output_dir.exists() {
                eprintln!("Skills not aggregated yet. Run 'skills-bank aggregate' first.");
                return Ok(());
            }
            
            let query = if let Some(q) = args.get(2) {
                q.clone()
            } else {
                bail!("Usage: skills-bank search <query>");
            };
            run_search_skills(&output_dir, &query)?;
        }
        _ => {
            print_help();
            bail!("Unknown command: {}", args[1]);
        }
    }

    Ok(())
}

fn print_help() {
    println!("skills-bank v0.1.0");
    println!("Non-interactive automation for clone -> aggregate -> sync");
    println!();
    println!("USAGE:");
    println!("    skills-bank                    # run full pipeline from saved config");
    println!("    skills-bank setup              # initialize or reinitialize config");
    println!("    skills-bank run                # run full pipeline from saved config");
    println!("    skills-bank fetch              # fetch configured repositories only");
    println!("    skills-bank aggregate          # aggregate only");
    println!("    skills-bank sync               # sync to all targets (auto-approved)");
    println!("    skills-bank sync --dry-run     # preview sync without making changes");
    println!("    skills-bank list               # list all aggregated skills");
    println!("    skills-bank list <hub>         # list skills in a specific hub (ai, business, etc)");
    println!("    skills-bank search <query>     # search for skills by name or description");
    println!("    skills-bank cleanup-legacy     # one-time cleanup of legacy repo caches into lib/");
    println!("    skills-bank add-repo <URL>     # add repo then run targeted pipeline");
    println!("    skills-bank doctor             # run diagnostics");
    println!("    skills-bank release-gate       # enforce production readiness checks");
    println!("    skills-bank tui                # (archived - use non-interactive commands instead)");
    println!("    skills-bank --help");
    println!("    skills-bank --version");
}

fn run_setup_wizard(_repo_root: &Path) -> Result<SetupConfig> {
    bail!("Interactive setup wizard archived. Create a config file or use non-interactive commands.");
}

async fn run_full_pipeline(config: &SetupConfig) -> Result<()> {
    apply_exclusion_env(Some(config));
    let repo_root = config.repo_root_path();
    println!("\n=== skills-bank full automation ===");

    if let Some(manifest) = prepare_manifest(&repo_root, &config.repositories)? {
        println!("[1/3] Clone/update repositories (shallow)...");
        run_fetch(&repo_root, manifest).await?;
    } else {
        println!("[1/3] No repository inputs configured; skipping clone/update.");
    }

    println!("[2/3] Aggregate skills...");
    let full_output = repo_root.join("skills-aggregated");
            if let Err(native_err) = run_aggregate_native(&repo_root, &full_output, None, false).await {
                eprintln!("[ERROR] Native aggregation failed (no archive fallback): {}", native_err);
                return Err(native_err);
            }

    println!("[3/3] Sync to selected tools...");
    let targets = resolve_sync_targets(&repo_root, config)?;
    if let Err(native_err) = run_sync_native(&full_output, &targets) {
        eprintln!("[ERROR] Native sync failed (no archive fallback): {}", native_err);
        return Err(native_err);
    }

    println!("\n[OK] Automation complete.");
    Ok(())
}

async fn run_add_repo_pipeline(
    repo_root: &Path,
    config_path: &Path,
    config: &mut SetupConfig,
    repo_url: &str,
) -> Result<()> {
    add_repo(config, repo_url)?;
    save_config(config_path, config)?;

    // Clone only the newly added repository.
    let single_manifest = build_manifest_from_urls(&[repo_url.to_string()]);
    println!("[1/3] Cloning new repository (shallow)...");
    run_fetch(repo_root, single_manifest).await?;

    let repo_name = repo_name_from_url(repo_url);
    let temp_output = repo_root.join("skills-aggregated-temp").join(&repo_name);

    println!("[2/3] Aggregating new repository only: {}...", repo_name);
    let selected = HashSet::from([repo_name.clone()]);
    if let Err(native_err) = run_aggregate_native(
        &repo_root,
        &temp_output,
        Some(&selected),
        false,
    )
    .await
    {
        eprintln!("[ERROR] Native targeted aggregation failed (no archive fallback): {}", native_err);
        return Err(native_err);
    }

    println!("[3/3] Syncing newly aggregated output for {}...", repo_name);
    let targets = resolve_sync_targets(repo_root, config)?;
    if let Err(native_err) = run_sync_native(&temp_output, &targets) {
        eprintln!("[ERROR] Native targeted sync failed (no archive fallback): {}", native_err);
        return Err(native_err);
    }

    println!("[OK] Added and synced repository: {}", repo_url);
    Ok(())
}

async fn run_aggregate_native(
    repo_root: &Path,
    output_dir: &Path,
    selected_repos: Option<&HashSet<String>>,
    write_global_csv: bool,
) -> Result<()> {
    let skills = aggregate_to_output(
        repo_root,
        output_dir,
        selected_repos,
        write_global_csv,
        false,
    )
    .await
    .context("Native aggregation failed")?;

    println!("  native aggregation output: {} skills", skills.len());
    Ok(())
}

fn run_sync_native(source_root: &Path, targets: &[PathBuf]) -> Result<()> {
    // Sync targets one-by-one so we can surface which target fails
    for target in targets {
        println!("  syncing target: {}", target.display());
        sync_output_to_targets(source_root, std::slice::from_ref(target), NativeSyncMode::Auto)
            .with_context(|| format!("Native sync failed for target: {}", target.display()))?;
    }
    println!("  native sync targets: {}", targets.len());
    Ok(())
}

fn run_sync_dry_run(source_root: &Path, targets: &[PathBuf]) -> Result<()> {
    use std::collections::HashSet;
    
    if !source_root.exists() {
        return Err(anyhow::anyhow!("Source directory not found: {}", source_root.display()));
    }

    for target in targets {
        println!("[DRY-RUN] Target: {}", target.display());
        println!("  Source: {}", source_root.display());
        
        // Walk the source and collect all files
        let mut files_to_sync = Vec::new();
        for entry in WalkDir::new(source_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel_path = entry.path()
                .strip_prefix(source_root)
                .unwrap_or_else(|_| entry.path())
                .to_string_lossy()
                .replace('\\', "/");
            files_to_sync.push(rel_path);
        }
        
        if files_to_sync.is_empty() {
            println!("  → No files to sync (source is empty)");
        } else {
            files_to_sync.sort();
            println!("  Files to be synced ({} total):", files_to_sync.len());
            for file_rel in &files_to_sync {
                let dest_file = target.join(file_rel);
                let status = if dest_file.exists() {
                    "UPDATE"
                } else {
                    "NEW"
                };
                println!("    - {} [{}]", file_rel, status);
            }
        }
        
        // Show existing BMAD/custom files that will be preserved
        if target.exists() {
            let synced_files: HashSet<String> = files_to_sync.iter().cloned().collect();
            let mut existing_preserved = Vec::new();
            
            for entry in WalkDir::new(target)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let rel_path = entry.path()
                    .strip_prefix(target)
                    .unwrap_or_else(|_| entry.path())
                    .to_string_lossy()
                    .replace('\\', "/");
                
                if !synced_files.contains(&rel_path) {
                    existing_preserved.push(rel_path);
                }
            }
            
            if !existing_preserved.is_empty() {
                existing_preserved.sort();
                println!("  Files to be PRESERVED ({} existing BMAD/custom files):", existing_preserved.len());
                for (idx, file_rel) in existing_preserved.iter().enumerate() {
                    if idx >= 5 {
                        println!("    ... and {} more files", existing_preserved.len() - 5);
                        break;
                    }
                    println!("    - {} [PRESERVED]", file_rel);
                }
            }
        }
        
        println!();
    }
    
    Ok(())
}

/// List all aggregated skills, optionally filtered by hub.
fn run_list_skills(output_dir: &Path, hub_filter: Option<&str>) -> Result<()> {
    let mut all_skills = Vec::new();
    let mut hub_totals: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Scan all hubs
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if !path.is_dir() {
            continue;
        }
        
        let hub_name = path.file_name().unwrap().to_string_lossy().to_string();
        
        // Skip hidden files and non-hub directories
        if hub_name.starts_with('.') {
            continue;
        }
        
        // If hub_filter is specified, only process matching hub
        if let Some(filter) = hub_filter {
            if !hub_name.eq_ignore_ascii_case(filter) {
                continue;
            }
        }
        
        // Scan sub-hubs within this hub
        if let Ok(sub_entries) = fs::read_dir(&path) {
            for sub_entry in sub_entries.flatten() {
                let sub_path = sub_entry.path();
                if !sub_path.is_dir() {
                    continue;
                }
                
                let sub_hub_name = sub_path.file_name().unwrap().to_string_lossy().to_string();
                
                // Read skills-catalog.csv from this sub-hub
                let catalog_path = sub_path.join("skills-catalog.csv");
                if catalog_path.exists() {
                    if let Ok(content) = fs::read_to_string(&catalog_path) {
                        for line in content.lines().skip(1) { // Skip header
                            let parts: Vec<&str> = line.split(',').collect();
                            if parts.len() >= 2 {
                                let skill_name = parts[0].trim();
                                if !skill_name.is_empty() {
                                    all_skills.push((
                                        hub_name.clone(),
                                        sub_hub_name.clone(),
                                        skill_name.to_string(),
                                    ));
                                    *hub_totals.entry(hub_name.clone()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if all_skills.is_empty() {
        if hub_filter.is_some() {
            println!("No skills found in hub '{}'", hub_filter.unwrap());
        } else {
            println!("No skills aggregated yet. Run 'skills-bank aggregate' first.");
        }
        return Ok(());
    }

    // Display results
    if let Some(filter) = hub_filter {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║  SKILLS IN HUB: {}                          ║", pad_string(filter, 28));
        println!("╚════════════════════════════════════════════════════════════╝");
    } else {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║  ALL AGGREGATED SKILLS                                    ║");
        println!("╚════════════════════════════════════════════════════════════╝");
    }
    
    let mut current_hub = String::new();
    for (hub, sub_hub, skill_name) in &all_skills {
        if hub != &current_hub {
            current_hub = hub.clone();
            println!("\n📦 {}", hub);
            println!("  {}", "─".repeat(52));
        }
        
        println!("  ├─ {} ({})", skill_name, sub_hub);
    }
    
    println!("\n\n📊 Summary:");
    let mut sorted_hubs: Vec<_> = hub_totals.iter().collect();
    sorted_hubs.sort_by(|a, b| a.0.cmp(b.0));
    
    for (hub, count) in sorted_hubs {
        println!("  {} {} skills", hub, count);
    }
    println!("  Total {} skills", all_skills.len());
    
    println!("\n💡 Tip: Use 'skills-bank search <query>' to find skills by name.");
    println!("        Use 'skills-bank list <hub-name>' to filter by hub.\n");
    
    Ok(())
}

/// Search for skills by name or hub.
fn run_search_skills(output_dir: &Path, query: &str) -> Result<()> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    // Scan all hubs
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if !path.is_dir() {
            continue;
        }
        
        let hub_name = path.file_name().unwrap().to_string_lossy().to_string();
        
        if hub_name.starts_with('.') {
            continue;
        }
        
        // Scan sub-hubs
        if let Ok(sub_entries) = fs::read_dir(&path) {
            for sub_entry in sub_entries.flatten() {
                let sub_path = sub_entry.path();
                if !sub_path.is_dir() {
                    continue;
                }
                
                let sub_hub_name = sub_path.file_name().unwrap().to_string_lossy().to_string();
                
                // Read skills-catalog.csv to search for matching skills
                let catalog_path = sub_path.join("skills-catalog.csv");
                if catalog_path.exists() {
                    if let Ok(content) = fs::read_to_string(&catalog_path) {
                        for line in content.lines().skip(1) { // Skip header
                            let parts: Vec<&str> = line.split(',').collect();
                            if parts.len() >= 2 {
                                let skill_name = parts[0].trim().to_lowercase();
                                
                                // Match if skill name, hub name, or sub_hub contains query
                                if skill_name.contains(&query_lower) || 
                                   hub_name.to_lowercase().contains(&query_lower) ||
                                   sub_hub_name.to_lowercase().contains(&query_lower) {
                                    results.push((
                                        hub_name.clone(),
                                        sub_hub_name.clone(),
                                        parts[0].trim().to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if results.is_empty() {
        println!("\n❌ No skills found matching '{}'", query);
        return Ok(());
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  SEARCH RESULTS FOR: {}                 ║", pad_string(query, 25));
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    for (hub, sub_hub, skill) in &results {
        println!("  📌 {}", skill);
        println!("     └─ Hub: {} → {}", hub, sub_hub);
    }
    
    println!("\n✓ Found {} skill(s)\n", results.len());
    
    Ok(())
}

fn pad_string(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

async fn run_fetch(repo_root: &Path, manifest: RepoManifest) -> Result<()> {
    let _guard = pushd(repo_root)?;

    let theme = Arc::new(Theme::new());
    let progress = Arc::new(ProgressManager::new(true, false, Arc::clone(&theme), None));
    let fetcher = Fetcher::with_manifest(manifest, progress);
    let result = fetcher
        .fetch(false)
        .await
        .context("Failed to fetch repositories")?;

    if let CommandResult::Fetch { cloned, updated } = result {
        println!("  cloned: {}", cloned.len());
        println!("  updated: {}", updated.len());
    }

    Ok(())
}

fn run_cleanup_legacy_duplicates(repo_root: &Path) -> Result<()> {
    let lib_root = repo_root.join("lib");
    if !lib_root.is_dir() {
        println!("  cleanup: lib/ directory not found; nothing to clean.");
        return Ok(());
    }

    let legacy_roots = [repo_root.join("src"), repo_root.join("repos")];
    let has_legacy_root = legacy_roots.iter().any(|root| root.is_dir());
    if !has_legacy_root {
        println!("  cleanup: no legacy repo cache directories found (src/ or repos/).");
        return Ok(());
    }

    let mut removed = Vec::new();
    let mut skipped_no_lib = Vec::new();
    let mut skipped_remote_mismatch = Vec::new();
    let mut errors = Vec::new();

    for legacy_root in legacy_roots {
        if !legacy_root.is_dir() {
            continue;
        }

        let legacy_label = legacy_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("legacy")
            .to_string();

        let entries = fs::read_dir(&legacy_root)
            .with_context(|| format!("Failed to read {}", legacy_root.display()))?;

        for entry_result in entries {
            let entry = match entry_result {
                Ok(v) => v,
                Err(err) => {
                    errors.push(format!("{}: {}", legacy_label, err));
                    continue;
                }
            };

            let legacy_repo_dir = entry.path();
            if !legacy_repo_dir.is_dir() {
                continue;
            }

            let repo_name = entry.file_name().to_string_lossy().to_string();
            let repo_ref = format!("{}/{}", legacy_label, repo_name);
            let lib_repo_dir = lib_root.join(&repo_name);

            if !lib_repo_dir.is_dir() {
                skipped_no_lib.push(repo_ref);
                continue;
            }

            let legacy_origin = git_origin_url(&legacy_repo_dir);
            let lib_origin = git_origin_url(&lib_repo_dir);

            if let (Some(legacy_url), Some(lib_url)) = (legacy_origin.as_ref(), lib_origin.as_ref()) {
                let legacy_id = normalize_git_remote_identity(legacy_url);
                let lib_id = normalize_git_remote_identity(lib_url);
                if legacy_id != lib_id {
                    skipped_remote_mismatch.push(repo_ref);
                    continue;
                }
            }

            match fs::remove_dir_all(&legacy_repo_dir) {
                Ok(_) => removed.push(repo_ref),
                Err(err) => errors.push(format!("{}: {}", repo_ref, err)),
            }
        }
    }

    removed.sort_unstable();
    skipped_no_lib.sort_unstable();
    skipped_remote_mismatch.sort_unstable();

    println!("\n=== cleanup legacy duplicates ===");
    println!("  removed: {}", removed.len());
    println!("  skipped (no lib match): {}", skipped_no_lib.len());
    println!("  skipped (remote mismatch): {}", skipped_remote_mismatch.len());
    println!("  errors: {}", errors.len());

    if !removed.is_empty() {
        println!("  removed repos: {}", removed.join(", "));
    }

    if !skipped_remote_mismatch.is_empty() {
        eprintln!(
            "[WARN] Skipped due to remote mismatch (same folder name, different origin): {}",
            skipped_remote_mismatch.join(", ")
        );
    }

    if !errors.is_empty() {
        eprintln!("[WARN] Cleanup encountered {} filesystem errors:", errors.len());
        for err in errors {
            eprintln!("  - {}", err);
        }
    }

    Ok(())
}

fn git_origin_url(repo_dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn normalize_git_remote_identity(url: &str) -> String {
    let mut normalized = url.trim().to_ascii_lowercase();

    if let Some(rest) = normalized.strip_prefix("git@") {
        normalized = rest.to_string();
    } else {
        for prefix in ["ssh://", "https://", "http://", "git://"] {
            if let Some(rest) = normalized.strip_prefix(prefix) {
                normalized = rest.to_string();
                break;
            }
        }
    }

    normalized = normalized.replace(':', "/");
    normalized = normalized.trim_end_matches('/').trim_end_matches(".git").to_string();
    normalized
}



fn run_doctor(repo_root: &Path) -> Result<CommandResult> {
    let _guard = pushd(repo_root)?;
    let diagnostics = Diagnostics::new();
    let result = diagnostics.run_all().context("Diagnostics failed")?;
    Ok(result)
}

fn run_release_gate(repo_root: &Path) -> Result<()> {
    println!("\n=== skills-bank release gate ===");

    let doctor = run_doctor(repo_root)?;
    let health_score = match doctor {
        CommandResult::Doctor { health_score, .. } => health_score,
        _ => 0,
    };

    if health_score < 100 {
        bail!(
            "Release gate failed: doctor health score is {}% (must be 100%)",
            health_score
        );
    }

    let checklist_path = repo_root.join("docs").join("cli-parity-checklist.md");
    if !checklist_path.exists() {
        bail!(
            "Release gate failed: missing parity checklist at {}",
            checklist_path.display()
        );
    }

    let checklist = fs::read_to_string(&checklist_path)
        .with_context(|| format!("Failed to read {}", checklist_path.display()))?;
    if checklist.contains("- [ ]") {
        bail!(
            "Release gate failed: parity checklist has unchecked items ({})",
            checklist_path.display()
        );
    }

    let cli_dir = if repo_root.join("Cargo.toml").exists() {
        repo_root.to_path_buf()
    } else {
        repo_root.join("cli")
    };

    if !cli_dir.join("Cargo.toml").exists() {
        bail!(
            "Release gate failed: could not find Cargo.toml in {}",
            cli_dir.display()
        );
    }

    let status = Command::new("cargo")
        .current_dir(&cli_dir)
        .arg("test")
        .status()
        .context("Failed to execute cargo test for release gate")?;

    if !status.success() {
        bail!("Release gate failed: cargo test returned non-zero status");
    }

    println!("[OK] Release gate passed. CLI is ready for production rollout.");
    Ok(())
}



fn resolve_sync_targets(repo_root: &Path, config: &SetupConfig) -> Result<Vec<PathBuf>> {
    let home_dir = home::home_dir().context("Could not resolve user home directory")?;
    let workspace_root = config.workspace_root_path();
    let repo_root_canonical = repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf());
    let mut targets = Vec::new();

    // 1. Load from explicit sync.targets in config (if present)
    if let Some(sync_cfg) = &config.sync {
        for (_name, path_str) in &sync_cfg.targets {
            let path = if path_str.starts_with("./") {
                workspace_root.join(&path_str[2..])
            } else if path_str.starts_with("~/") {
                home_dir.join(&path_str[2..])
            } else {
                PathBuf::from(path_str)
            };
            targets.push(path);
        }
    }

    // 2. Load from tool-based defaults
    for key in &config.tools {
        if let Some(tool) = tool_by_key(key) {
            match config.sync_scope {
                SyncScope::Global => {
                    for rel in tool.global_rels {
                        targets.push(home_dir.join(std::path::Path::new(rel)));
                    }
                }
                SyncScope::Local => {
                    for rel in tool.local_rels {
                        targets.push(workspace_root.join(std::path::Path::new(rel)));
                    }
                }
                SyncScope::Both => {
                    for rel in tool.global_rels {
                        targets.push(home_dir.join(std::path::Path::new(rel)));
                    }
                    for rel in tool.local_rels {
                        targets.push(workspace_root.join(std::path::Path::new(rel)));
                    }
                }
            }
        }
    }

    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    
    for p in targets {
        // 1. Determine absolute path for comparison
        let p_abs = if p.is_absolute() { p.clone() } else { repo_root.join(&p) };
        
        // 2. Normalize paths for string comparison (lowercase, forward slashes, strip UNC prefix)
        let normalize = |path: &Path| -> String {
            path.to_string_lossy()
                .to_lowercase()
                .replace('\\', "/")
                .replace("//?/", "")
        };

        let p_norm = normalize(&p_abs);
        let r_norm = normalize(repo_root);
        let rc_norm = normalize(&repo_root_canonical);

        // 3. Check if target is inside repo_root
        let is_inside = p_norm.starts_with(&r_norm) || p_norm.starts_with(&rc_norm);

        if is_inside {
            // ponytail: allow .opencode/skills inside repo — native discovery needs it (4 hubs, 120 tokens)
            // other inside paths remain excluded to avoid recursive copy into self
            let allowed_inside = p_norm.ends_with(".opencode/skills")
                || p_norm.ends_with(".opencode/skill")
                || p_norm.ends_with("/.opencode/skills")
                || p_norm.ends_with("/.opencode/skill");
            if !allowed_inside {
                continue;
            }
        }

        // 4. Use normalized absolute path as key for deduplication
        if seen.insert(p_norm) {
            deduped.push(p);
        }
    }

    if deduped.is_empty() {
        bail!("No external sync targets were selected. Note that targets inside the skills-bank repository are automatically excluded. Please update your config or rerun setup.");
    }

    Ok(deduped)
}

fn add_repo(config: &mut SetupConfig, repo_url: &str) -> Result<()> {
    let trimmed = repo_url.trim();
    if !looks_like_repo_url(trimmed) {
        bail!("Invalid repository URL: {}", trimmed);
    }

    let new_key = normalized_repo_key(trimmed);
    let exists = config
        .repositories
        .iter()
        .any(|r| normalized_repo_key(&r.url) == new_key);

    if !exists {
        let manifest = build_manifest_from_urls(&[trimmed.to_string()]);
        if let Some(new_repo) = manifest.repositories.first() {
            config.repositories.push(new_repo.clone());
            println!("Added repository: {}", trimmed);
        }
    } else {
        println!("Repository already exists in setup: {}", trimmed);
    }

    Ok(())
}

fn prepare_manifest(repo_root: &Path, repositories: &[Repository]) -> Result<Option<RepoManifest>> {
    let manifest_path = repo_root.join("config.json");

    if !repositories.is_empty() {
        let manifest = RepoManifest {
            repositories: repositories.to_vec(),
        };
        write_manifest_file(&manifest_path, &manifest)?;
        return Ok(Some(manifest));
    }

    if manifest_path.exists() {
        let manifest = RepoManifest::load(&manifest_path)
            .with_context(|| format!("Failed to load {}", manifest_path.display()))?;
        if manifest.repositories.is_empty() {
            return Ok(None);
        }
        return Ok(Some(manifest));
    }

    Ok(None)
}

fn write_manifest_file(path: &Path, manifest: &RepoManifest) -> Result<()> {
    let mut current_val: Value = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    if let Some(obj) = current_val.as_object_mut() {
        obj.insert("repositories".to_string(), serde_json::to_value(&manifest.repositories)?);
    }

    let json = serde_json::to_string_pretty(&current_val)?;
    fs::write(path, json)
        .with_context(|| format!("Failed to write manifest file: {}", path.display()))?;
    Ok(())
}

use serde_json::json;

fn build_manifest_from_urls(urls: &[String]) -> RepoManifest {
    let mut used_names = HashSet::new();
    let mut repositories = Vec::new();

    for url in urls {
        let base_name = repo_name_from_url(url);
        let mut candidate = base_name.clone();
        let mut idx = 2;

        while !used_names.insert(candidate.to_lowercase()) {
            candidate = format!("{}-{}", base_name, idx);
            idx += 1;
        }

        repositories.push(Repository {
            name: candidate,
            url: url.trim().to_string(),
            branch: None,
        });
    }

    RepoManifest { repositories }
}

fn repo_name_from_url(url: &str) -> String {
    let cleaned = url.trim().trim_end_matches('/').trim_end_matches(".git");

    let (owner, repo) = if cleaned.starts_with("git@") {
        let after_colon = cleaned.rsplit(':').next().unwrap_or(cleaned);
        let parts = after_colon.split('/').collect::<Vec<_>>();
        if parts.len() >= 2 {
            (parts[parts.len() - 2], parts[parts.len() - 1])
        } else {
            ("repo", parts.last().copied().unwrap_or("repo"))
        }
    } else {
        let parts = cleaned.split('/').collect::<Vec<_>>();
        if parts.len() >= 2 {
            (parts[parts.len() - 2], parts[parts.len() - 1])
        } else {
            ("repo", parts.last().copied().unwrap_or("repo"))
        }
    };

    let owner_clean = sanitize_segment(owner);
    let repo_clean = sanitize_segment(repo);
    let combined = format!("{}-{}", owner_clean, repo_clean);
    combined
        .trim_matches('-')
        .to_string()
        .chars()
        .take(80)
        .collect::<String>()
}

fn sanitize_segment(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

fn normalized_repo_key(url: &str) -> String {
    let mut normalized = url.trim().trim_end_matches('/').to_lowercase();
    if normalized.ends_with(".git") {
        normalized.truncate(normalized.len() - 4);
    }
    normalized
}

fn looks_like_repo_url(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.starts_with("https://") || trimmed.starts_with("http://") || trimmed.starts_with("git@")
}

fn discover_repo_root() -> Result<PathBuf> {
    let cwd = env::current_dir().context("Failed to get current directory")?;

    let mut candidates = Vec::new();
    let mut cursor = Some(cwd.as_path());
    while let Some(path) = cursor {
        candidates.push(path.to_path_buf());
        candidates.push(path.join("skills-bank"));
        cursor = path.parent();
    }

    let mut seen = HashSet::new();

    for candidate in candidates {
        let key = candidate.to_string_lossy().to_lowercase();
        if !seen.insert(key) {
            continue;
        }

        if is_skill_manage_root(&candidate) {
            return Ok(candidate);
        }
    }

    bail!(
        "Could not locate the skills-bank root. Run this from inside the repo (e.g. skills-bank/cli)."
    )
}

fn is_skill_manage_root(path: &Path) -> bool {
    let has_native_core = path.join("src").is_dir() && cargo_toml_declares_skill_manage(path);

    let has_cli_core = path.join("cli").join("Cargo.toml").exists() && path.join("src").is_dir();

    // Do not rely on the legacy `archive/` scripts to identify the repo root.
    // The CLI should work purely from the Rust code; archived scripts are optional.
    has_native_core || has_cli_core
}

fn cargo_toml_declares_skill_manage(path: &Path) -> bool {
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return false;
    }

    fs::read_to_string(cargo_toml)
        .map(|content| {
            content.contains("name = \"skills-bank\"")
                || content.contains("name=\"skills-bank\"")
        })
        .unwrap_or(false)
}

fn workspace_root_from_repo_root(repo_root: &Path) -> PathBuf {
    repo_root.to_path_buf()
}

fn env_is_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn migrate_legacy_workspace_root(config: &mut SetupConfig) -> bool {
    if env_is_truthy("SKILL_MANAGE_KEEP_PARENT_WORKSPACE") {
        return false;
    }

    let repo_root = PathBuf::from(&config.repo_root);
    let workspace_root = PathBuf::from(&config.workspace_root);
    let repo_name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if !repo_name.eq_ignore_ascii_case("skills-bank") {
        return false;
    }

    let Some(parent) = repo_root.parent() else {
        return false;
    };

    if workspace_root == parent {
        config.workspace_root = repo_root.to_string_lossy().to_string();
        return true;
    }

    false
}

fn config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(CONFIG_FILE_NAME)
}

fn load_config(path: &Path) -> Result<Option<SetupConfig>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let mut config: SetupConfig = serde_json::from_str(&content)
        .with_context(|| format!("Invalid config JSON in {}", path.display()))?;

    // Re-discover root to ensure it's dynamic across devices
    if let Ok(discovered) = discover_repo_root() {
        let discovered_str = discovered.to_string_lossy().to_string();
        // If the saved path doesn't exist or we found a valid one, prefer the discovered one
        if config.repo_root.is_empty() || !Path::new(&config.repo_root).exists() || config.repo_root != discovered_str {
            config.repo_root = discovered_str.clone();
        }
        if config.workspace_root.is_empty() || !Path::new(&config.workspace_root).exists() || config.workspace_root != discovered_str {
            config.workspace_root = discovered_str;
        }
    }

    if migrate_legacy_workspace_root(&mut config) {
        eprintln!(
            "[INFO] Migrated legacy workspace_root from parent directory to repo root. \
Set SKILL_MANAGE_KEEP_PARENT_WORKSPACE=1 to keep the old behavior."
        );
    }

    Ok(Some(config))
}

fn ensure_config(repo_root: &Path, config_path: &Path) -> Result<SetupConfig> {
    match load_config(config_path)? {
        Some(cfg) => Ok(cfg),
        None => {
            // If aconfig.json manifest exists, auto-generate a sensible default config
            if let Some(cfg) = auto_config_from_manifest(repo_root)? {
                println!("No setup file found, butconfig.json detected. Creating default setup and saving it...");
                save_config(config_path, &cfg)?;
                return Ok(cfg);
            }

            println!("No setup file found. Running setup now...");
            let cfg = run_setup_wizard(repo_root)?;
            save_config(config_path, &cfg)?;
            Ok(cfg)
        }
    }
}

fn auto_config_from_manifest(repo_root: &Path) -> Result<Option<SetupConfig>> {
    let manifest_path = repo_root.join("config.json");
    if !manifest_path.exists() {
        return Ok(None);
    }

    let manifest = RepoManifest::load(&manifest_path)
        .with_context(|| format!("Failed to load {}", manifest_path.display()))?;

    if manifest.repositories.is_empty() {
        return Ok(None);
    }

    let workspace_root = workspace_root_from_repo_root(repo_root);
    let tools = TOOL_DEFS.iter().map(|t| t.key.to_string()).collect::<Vec<_>>();

    let cfg = SetupConfig {
        version: 1,
        repo_root: repo_root.to_string_lossy().to_string(),
        workspace_root: workspace_root.to_string_lossy().to_string(),
        sync_scope: SyncScope::Both,
        tools,
        repositories: manifest.repositories,
        category_exclusions: default_category_exclusions(),
        sync: None,
    };

    Ok(Some(cfg))
}

fn save_config(path: &Path, config: &SetupConfig) -> Result<()> {
    let mut config_to_save = config.clone();
    // Save as "." to make it portable across devices
    config_to_save.repo_root = ".".to_string();
    config_to_save.workspace_root = ".".to_string();

    let json = serde_json::to_string_pretty(&config_to_save)?;
    fs::write(path, json).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn pushd(path: &Path) -> Result<DirGuard> {
    let original = env::current_dir().context("Failed to get current directory")?;
    env::set_current_dir(path)
        .with_context(|| format!("Failed to change directory to {}", path.display()))?;
    Ok(DirGuard { original })
}

fn tool_by_key(key: &str) -> Option<&'static ToolDef> {
    TOOL_DEFS.iter().find(|t| t.key == key)
}



