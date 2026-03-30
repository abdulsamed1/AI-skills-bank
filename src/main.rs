use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect, Select};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use skill_manage::components::native_pipeline::{
    aggregate_to_output, sync_output_to_targets, NativeSyncMode,
};
use skill_manage::components::diagnostics::Diagnostics;
use skill_manage::components::fetcher::Fetcher;
use skill_manage::components::manifest::{RepoManifest, Repository};
use skill_manage::components::CommandResult;
use skill_manage::utils::progress::ProgressManager;
use skill_manage::utils::theme::Theme;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

const CONFIG_FILE_NAME: &str = ".skill-manage-cli-config.json";

#[derive(Debug, Clone)]
struct ToolDef {
    key: &'static str,
    label: &'static str,
    global_rel: &'static str,
    local_rel: &'static str,
}

const TOOL_DEFS: &[ToolDef] = &[
    ToolDef {
        key: "claude",
        label: "Claude",
        global_rel: ".claude/skills",
        local_rel: ".claude/skills",
    },
    ToolDef {
        key: "code",
        label: "Code (Codex)",
        global_rel: ".agents/skills",
        local_rel: ".agents/skills",
    },
    ToolDef {
        key: "copilot",
        label: "GitHub Copilot",
        global_rel: ".copilot/skills",
        local_rel: ".github/skills",
    },
    ToolDef {
        key: "cursor",
        label: "Cursor",
        global_rel: ".cursor/skills",
        local_rel: ".cursor/skills",
    },
    ToolDef {
        key: "gemini",
        label: "Gemini",
        global_rel: ".gemini/skills",
        local_rel: ".gemini/skills",
    },
    ToolDef {
        key: "antigravity",
        label: "Antigravity",
        global_rel: ".gemini/antigravity/skills",
        local_rel: ".agent/skills",
    },
    ToolDef {
        key: "opencode",
        label: "OpenCode",
        global_rel: ".config/opencode/skills",
        local_rel: ".opencode/skills",
    },
    ToolDef {
        key: "windsurf",
        label: "Windsurf",
        global_rel: ".codeium/windsurf/skills",
        local_rel: ".windsurf/skills",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SyncScope {
    Global,
    Local,
    Both,
}

impl SyncScope {
    fn label(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Local => "local",
            Self::Both => "both",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetupConfig {
    version: u32,
    repo_root: String,
    workspace_root: String,
    sync_scope: SyncScope,
    tools: Vec<String>,
    repositories: Vec<String>,
    #[serde(default = "default_category_exclusions")]
    category_exclusions: Vec<String>,
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
    vec!["games".to_string(), "medicine".to_string(), "law".to_string()]
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

fn parse_exclusion_categories(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for part in raw.split(|c| c == ',' || c == ';' || c == '\n') {
        let normalized = normalize_exclusion_category(part);
        if normalized.is_empty() {
            continue;
        }

        if seen.insert(normalized.clone()) {
            out.push(normalized);
        }
    }

    out
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
        run_interactive(&repo_root, &config_path).await?;
        return Ok(());
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_help();
        }
        "--version" | "-v" => {
            println!("skill-manage v0.1.0");
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
                bail!("No repositories configured. Run setup or provide.skill-manage-cli-config.json.");
            }
        }
        "aggregate" => {
            let loaded = load_config(&config_path)?;
            apply_exclusion_env(loaded.as_ref());
            let output_dir = repo_root.join("skills-aggregated");
            if let Err(native_err) = run_aggregate_native(&repo_root, &output_dir, None, true).await {
                eprintln!("[ERROR] Native aggregation failed (no archive fallback): {}", native_err);
                return Err(native_err);
            }
        }
        "sync" => {
            let config = ensure_config(&repo_root, &config_path)?;
            let targets = resolve_sync_targets(&config)?;
            let output_dir = repo_root.join("skills-aggregated");
            if let Err(native_err) = run_sync_native(&output_dir, &targets) {
                eprintln!("[ERROR] Native sync failed (no archive fallback): {}", native_err);
                return Err(native_err);
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
                Input::<String>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Repository URL")
                    .interact_text()
                    .context("Failed to read repository URL")?
            };

            run_add_repo_pipeline(&repo_root, &config_path, &mut config, &repo_url).await?;
        }
        "doctor" => {
            let _ = run_doctor(&repo_root)?;
        }
        "release-gate" | "gate" => {
            run_release_gate(&repo_root)?;
        }
        _ => {
            print_help();
            bail!("Unknown command: {}", args[1]);
        }
    }

    Ok(())
}

fn print_help() {
    println!("skill-manage v0.1.0");
    println!("Guided automation for clone -> aggregate -> sync");
    println!();
    println!("USAGE:");
    println!("    skill-manage                    # guided UI (first run asks setup)");
    println!("    skill-manage setup              # rerun first-time setup");
    println!("    skill-manage run                # run full pipeline from saved config");
    println!("    skill-manage fetch              # fetch configured repositories only");
    println!("    skill-manage aggregate          # aggregate only");
    println!("    skill-manage sync               # sync only");
    println!("    skill-manage cleanup-legacy     # one-time cleanup of legacy repo caches into lib/");
    println!("    skill-manage add-repo <URL>     # add repo then run targeted pipeline");
    println!("    skill-manage doctor             # run diagnostics");
    println!("    skill-manage release-gate       # enforce production readiness checks");
    println!("    skill-manage --help");
    println!("    skill-manage --version");
}

async fn run_interactive(repo_root: &Path, config_path: &Path) -> Result<()> {
    let mut config = match load_config(config_path)? {
        Some(cfg) => cfg,
        None => {
            // If.skill-manage-cli-config.json exists, auto-create a default setup and run the pipeline
            if let Some(cfg) = auto_config_from_manifest(repo_root)? {
                println!("No setup found, but.skill-manage-cli-config.json detected. Creating default setup and running automation...");
                save_config(config_path, &cfg)?;
                apply_exclusion_env(Some(&cfg));
                run_full_pipeline(&cfg).await?;
                cfg
            } else {
                println!("No setup found. Starting first-time setup...");
                let cfg = run_setup_wizard(repo_root)?;
                save_config(config_path, &cfg)?;
                apply_exclusion_env(Some(&cfg));
                run_full_pipeline(&cfg).await?;
                cfg
            }
        }
    };

    apply_exclusion_env(Some(&config));

    let theme = ColorfulTheme::default();
    let options = vec![
        "Run full pipeline now",
        "Fetch repositories only",
        "Aggregate only",
        "Sync only",
        "Cleanup legacy duplicate repos",
        "Add new repository URL and run",
        "Show current setup",
        "Reconfigure setup",
        "Run release gate",
        "Exit",
    ];

    loop {
        let selection = Select::with_theme(&theme)
            .with_prompt("Choose an action")
            .items(&options)
            .default(0)
            .interact()
            .context("Interactive menu failed")?;

        match selection {
            0 => {
                run_full_pipeline(&config).await?;
            }
            1 => {
                if let Some(manifest) = prepare_manifest(repo_root, &config.repositories)? {
                    run_fetch(repo_root, manifest).await?;
                } else {
                    eprintln!("[WARN] No repositories configured.");
                }
            }
            2 => {
                apply_exclusion_env(Some(&config));
                let output_dir = repo_root.join("skills-aggregated");
                if let Err(native_err) = run_aggregate_native(&repo_root, &output_dir, None, true).await {
                    eprintln!("[ERROR] Native aggregation failed (no archive fallback): {}", native_err);
                    return Err(native_err);
                }
            }
            3 => {
                let output_dir = repo_root.join("skills-aggregated");
                let targets = resolve_sync_targets(&config)?;
                if let Err(native_err) = run_sync_native(&output_dir, &targets) {
                    eprintln!("[ERROR] Native sync failed (no archive fallback): {}", native_err);
                    return Err(native_err);
                }
            }
            4 => {
                run_cleanup_legacy_duplicates(repo_root)?;
            }
            5 => {
                let repo_url: String = Input::with_theme(&theme)
                    .with_prompt("Repository URL")
                    .interact_text()
                    .context("Failed to read repository URL")?;
                run_add_repo_pipeline(repo_root, config_path, &mut config, &repo_url).await?;
            }
            6 => {
                print_config_summary(&config);
            }
            7 => {
                let new_cfg = run_setup_wizard(repo_root)?;
                save_config(config_path, &new_cfg)?;
                config = new_cfg;
                apply_exclusion_env(Some(&config));
                run_full_pipeline(&config).await?;
            }
            8 => {
                run_release_gate(repo_root)?;
            }
            9 => break,
            _ => {}
        }
    }

    Ok(())
}

fn print_config_summary(config: &SetupConfig) {
    println!("\nCurrent setup");
    println!("  Scope: {}", config.sync_scope.label());
    println!("  Repo root: {}", config.repo_root);
    println!("  Workspace root: {}", config.workspace_root);

    let tool_labels = config
        .tools
        .iter()
        .filter_map(|k| tool_by_key(k).map(|d| d.label))
        .collect::<Vec<_>>();
    println!("  Tools: {}", tool_labels.join(", "));
    println!("  Repositories: {}", config.repositories.len());
    for repo in &config.repositories {
        println!("    - {}", repo);
    }
    println!("  Excluded categories: {}", config.category_exclusions.join(", "));
    println!();
}

fn run_setup_wizard(repo_root: &Path) -> Result<SetupConfig> {
    let theme = ColorfulTheme::default();

    let scope_options = vec![
        "Global (home directories)",
        "Local (workspace directories)",
        "Both global + local",
    ];
    let scope_index = Select::with_theme(&theme)
        .with_prompt("Where should skills be synced?")
        .items(&scope_options)
        .default(0)
        .interact()
        .context("Failed to choose sync scope")?;

    let sync_scope = match scope_index {
        0 => SyncScope::Global,
        1 => SyncScope::Local,
        _ => SyncScope::Both,
    };

    let tool_labels = TOOL_DEFS.iter().map(|t| t.label).collect::<Vec<_>>();
    let defaults = vec![true; tool_labels.len()];
    let selected_tools = MultiSelect::with_theme(&theme)
        .with_prompt("Select your AI tools")
        .items(&tool_labels)
        .defaults(&defaults)
        .interact()
        .context("Failed to select tools")?;

    if selected_tools.is_empty() {
        bail!("Select at least one tool to continue");
    }

    let tools = selected_tools
        .into_iter()
        .map(|idx| TOOL_DEFS[idx].key.to_string())
        .collect::<Vec<_>>();

    let source_options = vec![
        "Paste repository URLs now",
        "Load repository links from JSON file",
        "Skip repository input for now",
    ];
    let source_index = Select::with_theme(&theme)
        .with_prompt("How do you want to provide repositories?")
        .items(&source_options)
        .default(0)
        .interact()
        .context("Failed to choose repository input mode")?;

    let repositories = match source_index {
        0 => collect_repo_urls(&theme)?,
        1 => {
            let path_input: String = Input::with_theme(&theme)
                .with_prompt("Path to JSON file with repository links")
                .interact_text()
                .context("Failed to read JSON path")?;
            let path = resolve_input_path(repo_root, &path_input);
            load_repo_urls_from_json(&path)?
        }
        _ => Vec::new(),
    };

    let repositories = dedupe_urls(repositories);
    let default_exclusions = default_category_exclusions().join(", ");
    let exclusion_input: String = Input::with_theme(&theme)
        .with_prompt("Excluded categories (comma-separated, editable any time)")
        .with_initial_text(default_exclusions)
        .interact_text()
        .context("Failed to read excluded categories")?;

    let mut category_exclusions = parse_exclusion_categories(&exclusion_input);
    if category_exclusions.is_empty() {
        category_exclusions = default_category_exclusions();
    }

    let workspace_root = workspace_root_from_repo_root(repo_root);
    let config = SetupConfig {
        version: 1,
        repo_root: repo_root.to_string_lossy().to_string(),
        workspace_root: workspace_root.to_string_lossy().to_string(),
        sync_scope,
        tools,
        repositories,
        category_exclusions,
    };

    println!("\nSetup saved. Running automation now...");
    Ok(config)
}

fn collect_repo_urls(theme: &ColorfulTheme) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    println!("Enter repository URLs (blank input finishes):");

    loop {
        let input: String = Input::with_theme(theme)
            .with_prompt("Repository URL")
            .allow_empty(true)
            .interact_text()
            .context("Failed to read repository URL")?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            break;
        }

        if !looks_like_repo_url(trimmed) {
            eprintln!("[WARN] Skipping invalid URL: {}", trimmed);
            continue;
        }

        urls.push(trimmed.to_string());
    }

    Ok(urls)
}

async fn run_full_pipeline(config: &SetupConfig) -> Result<()> {
    apply_exclusion_env(Some(config));
    let repo_root = config.repo_root_path();
    println!("\n=== skill-manage full automation ===");

    if let Some(manifest) = prepare_manifest(&repo_root, &config.repositories)? {
        println!("[1/3] Clone/update repositories (shallow)...");
        run_fetch(&repo_root, manifest).await?;
    } else {
        println!("[1/3] No repository inputs configured; skipping clone/update.");
    }

    println!("[2/3] Aggregate skills...");
    let full_output = repo_root.join("skills-aggregated");
            if let Err(native_err) = run_aggregate_native(&repo_root, &full_output, None, true).await {
                eprintln!("[ERROR] Native aggregation failed (no archive fallback): {}", native_err);
                return Err(native_err);
            }

    println!("[3/3] Sync to selected tools...");
    let targets = resolve_sync_targets(config)?;
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
    let targets = resolve_sync_targets(config)?;
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
        true,
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

async fn run_fetch(repo_root: &Path, manifest: RepoManifest) -> Result<()> {
    let _guard = pushd(repo_root)?;

    let theme = Arc::new(Theme::new());
    let progress = Arc::new(ProgressManager::new(true, false, Arc::clone(&theme)));
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

fn run_aggregate_script(repo_root: &Path) -> Result<()> {
    let script = repo_root
        .join("archive")
        .join("aggregate-skills-to-subhubs.ps1");
    if !script.exists() {
        bail!("Aggregate script not found: {}", script.display());
    }

    let mode = if repo_root
        .join("skills-aggregated")
        .join(".skill-lock.json")
        .exists()
    {
        "changed-only"
    } else {
        "all"
    };

    let command = format!(
        "& '{}' -NoPrompt -srcRepoMode {}",
        escape_ps_single(&script.to_string_lossy()),
        mode
    );

    run_powershell_command(repo_root, &command).context("Failed while running aggregate script")
}

fn run_aggregate_selected(repo_root: &Path, repo_names: &[String], output_dir: &Path) -> Result<()> {
    let script = repo_root
        .join("archive")
        .join("aggregate-skills-to-subhubs.ps1");
    if !script.exists() {
        bail!("Aggregate script not found: {}", script.display());
    }

    // Ensure output parent exists
    if let Some(parent) = output_dir.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).with_context(|| format!("Failed to create parent dir: {}", parent.display()))?;
        }
    }

    let names = repo_names
        .iter()
        .map(|n| format!("'{}'", escape_ps_single(n)))
        .collect::<Vec<_>>()
        .join(", ");

    let command = format!(
        "& '{}' -NoPrompt -srcRepoMode selected -srcRepoNames @({}) -OutputDir '{}'",
        escape_ps_single(&script.to_string_lossy()),
        names,
        escape_ps_single(&output_dir.to_string_lossy())
    );

    run_powershell_command(repo_root, &command).context("Failed while running aggregate script for selected repos")
}

fn run_sync_script(repo_root: &Path, targets: &[PathBuf], hubsrc_override: Option<&Path>) -> Result<()> {
    if targets.is_empty() {
        bail!("No sync targets were resolved from setup");
    }

    let script = repo_root.join("archive").join("sync-hubs.ps1");
    if !script.exists() {
        bail!("Sync script not found: {}", script.display());
    }

    let target_list = targets
        .iter()
        .map(|p| format!("'{}'", escape_ps_single(&p.to_string_lossy())))
        .collect::<Vec<_>>()
        .join(", ");

    // Allow overriding the Hubsrc (skills-aggregated) directory to sync only a subset
    let hubsrc_arg = if let Some(override_path) = hubsrc_override {
        format!("-Hubsrc '{}'", escape_ps_single(&override_path.to_string_lossy()))
    } else {
        String::new()
    };

    let command = format!(
        "& '{}' -NoPrompt -Force -SyncMode Auto {} -TargetTools @({})",
        escape_ps_single(&script.to_string_lossy()),
        hubsrc_arg,
        target_list
    );

    run_powershell_command(repo_root, &command).context("Failed while running sync script")
}

fn run_doctor(repo_root: &Path) -> Result<CommandResult> {
    let _guard = pushd(repo_root)?;
    let diagnostics = Diagnostics::new();
    let result = diagnostics.run_all().context("Diagnostics failed")?;
    Ok(result)
}

fn run_release_gate(repo_root: &Path) -> Result<()> {
    println!("\n=== skill-manage release gate ===");

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

fn run_powershell_command(cwd: &Path, command: &str) -> Result<()> {
    for shell in ["pwsh", "powershell"] {
        let result = Command::new(shell)
            .current_dir(cwd)
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(command)
            .status();

        match result {
            Ok(status) => {
                if status.success() {
                    return Ok(());
                }
                bail!("PowerShell command failed with status: {:?}", status.code());
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err).context("Failed to start PowerShell"),
        }
    }

    bail!("Could not find PowerShell runtime (`pwsh` or `powershell`) in PATH")
}

fn resolve_sync_targets(config: &SetupConfig) -> Result<Vec<PathBuf>> {
    let home_dir = home::home_dir().context("Could not resolve user home directory")?;
    let workspace_root = config.workspace_root_path();
    let mut targets = Vec::new();

    for key in &config.tools {
        if let Some(tool) = tool_by_key(key) {
            match config.sync_scope {
                SyncScope::Global => targets.push(home_dir.join(std::path::Path::new(tool.global_rel))),
                SyncScope::Local => targets.push(workspace_root.join(std::path::Path::new(tool.local_rel))),
                SyncScope::Both => {
                    targets.push(home_dir.join(std::path::Path::new(tool.global_rel)));
                    targets.push(workspace_root.join(std::path::Path::new(tool.local_rel)));
                }
            }
        }
    }

    let mut seen = HashSet::new();
    let deduped = targets
        .into_iter()
        .filter(|p| seen.insert(p.to_string_lossy().to_lowercase()))
        .collect::<Vec<_>>();

    

    if deduped.is_empty() {
        bail!("No targets were selected. Please rerun setup.");
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
        .any(|r| normalized_repo_key(r) == new_key);

    if !exists {
        config.repositories.push(trimmed.to_string());
        config.repositories = dedupe_urls(config.repositories.clone());
        println!("Added repository: {}", trimmed);
    } else {
        println!("Repository already exists in setup: {}", trimmed);
    }

    Ok(())
}

fn prepare_manifest(repo_root: &Path, configured_urls: &[String]) -> Result<Option<RepoManifest>> {
    let manifest_path = repo_root.join("repos.json");

    if !configured_urls.is_empty() {
        let urls = dedupe_urls(configured_urls.to_vec());
        let manifest = build_manifest_from_urls(&urls);
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
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(path, json)
        .with_context(|| format!("Failed to write manifest file: {}", path.display()))?;
    Ok(())
}

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

fn load_repo_urls_from_json(path: &Path) -> Result<Vec<String>> {
    // If file doesn't exist, create a stub and guide the user
    if !path.exists() {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let _ = fs::create_dir_all(parent);
        
        let stub = r#"{
  "repositories": [
    "https://github.com/owner/repo-name.git"
  ]
}
"#;
        let _ = fs::write(path, stub);
        
        println!("\n[INFO] repos.json created at: {}", path.display());
        println!("Please edit the file and add your repository URLs, then re-run the setup.");
        println!("Format: Add GitHub URLs to the 'repositories' array.\n");
        
        bail!("repos.json was created. Please add your repositories and re-run setup.");
    }
    
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read repository JSON: {}", path.display()))?;
    let value: Value = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in {}", path.display()))?;

    let mut urls = Vec::new();
    collect_repo_urls_from_value(&value, &mut urls);
    let urls = dedupe_urls(urls);

    if urls.is_empty() {
        bail!(
            "No repository links found in JSON file. Expected URLs in strings, `repositories`, or `repos`"
        );
    }

    Ok(urls)
}

fn collect_repo_urls_from_value(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            if looks_like_repo_url(s) {
                out.push(s.trim().to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_repo_urls_from_value(item, out);
            }
        }
        Value::Object(map) => {
            if let Some(url) = map.get("url").and_then(|v| v.as_str()) {
                if looks_like_repo_url(url) {
                    out.push(url.trim().to_string());
                }
            }

            for key in ["repositories", "repos"] {
                if let Some(nested) = map.get(key) {
                    collect_repo_urls_from_value(nested, out);
                }
            }
        }
        _ => {}
    }
}

fn dedupe_urls(urls: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for raw in urls {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = normalized_repo_key(trimmed);
        if seen.insert(key) {
            deduped.push(trimmed.to_string());
        }
    }

    deduped
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
        candidates.push(path.join("skill-manage"));
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
        "Could not locate the skill-manage root. Run this from inside the repo (e.g. skill-manage/cli)."
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
            content.contains("name = \"skill-manage\"")
                || content.contains("name=\"skill-manage\"")
        })
        .unwrap_or(false)
}

fn workspace_root_from_repo_root(repo_root: &Path) -> PathBuf {
    let name = repo_root.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.eq_ignore_ascii_case("skill-manage") {
        if let Some(parent) = repo_root.parent() {
            return parent.to_path_buf();
        }
    }
    repo_root.to_path_buf()
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
    let config: SetupConfig = serde_json::from_str(&content)
        .with_context(|| format!("Invalid config JSON in {}", path.display()))?;

    Ok(Some(config))
}

fn ensure_config(repo_root: &Path, config_path: &Path) -> Result<SetupConfig> {
    match load_config(config_path)? {
        Some(cfg) => Ok(cfg),
        None => {
            // If a.skill-manage-cli-config.json manifest exists, auto-generate a sensible default config
            if let Some(cfg) = auto_config_from_manifest(repo_root)? {
                println!("No setup file found, but.skill-manage-cli-config.json detected. Creating default setup and saving it...");
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
    let manifest_path = repo_root.join("repos.json");
    if !manifest_path.exists() {
        return Ok(None);
    }

    let manifest = RepoManifest::load(&manifest_path)
        .with_context(|| format!("Failed to load {}", manifest_path.display()))?;

    if manifest.repositories.is_empty() {
        return Ok(None);
    }

    let repositories = manifest
        .repositories
        .iter()
        .map(|r| r.url.clone())
        .collect::<Vec<_>>();

    let workspace_root = workspace_root_from_repo_root(repo_root);
    let tools = TOOL_DEFS.iter().map(|t| t.key.to_string()).collect::<Vec<_>>();

    let cfg = SetupConfig {
        version: 1,
        repo_root: repo_root.to_string_lossy().to_string(),
        workspace_root: workspace_root.to_string_lossy().to_string(),
        sync_scope: SyncScope::Both,
        tools,
        repositories,
        category_exclusions: default_category_exclusions(),
    };

    Ok(Some(cfg))
}

fn save_config(path: &Path, config: &SetupConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
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

fn resolve_input_path(repo_root: &Path, input: &str) -> PathBuf {
    let trimmed = input.trim();
    
    // Auto-detect repos.json if input is empty or looks like a short name
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("repos.json") || trimmed.eq_ignore_ascii_case("repos") {
        let candidates = [repo_root.join("repos.json")];
        
        for candidate in &candidates {
            if candidate.exists() {
                return candidate.clone();
            }
        }
        
        // Also check in current directory
        if let Ok(cwd) = env::current_dir() {
            let cwd_candidate = cwd.join("repos.json");
            if cwd_candidate.exists() {
                return cwd_candidate;
            }
        }
        
        // Default to repo_root/repos.json if none exist
        return repo_root.join("repos.json");
    }
    
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return path;
    }

    let from_cwd = env::current_dir().unwrap_or_else(|_| repo_root.to_path_buf());
    let candidate = from_cwd.join(&path);
    if candidate.exists() {
        return candidate;
    }

    repo_root.join(path)
}

fn escape_ps_single(input: &str) -> String {
    input.replace('\'', "''")
}
