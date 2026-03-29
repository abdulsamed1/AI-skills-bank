use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect, Select};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
}

impl SetupConfig {
    fn repo_root_path(&self) -> PathBuf {
        PathBuf::from(&self.repo_root)
    }

    fn workspace_root_path(&self) -> PathBuf {
        PathBuf::from(&self.workspace_root)
    }
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
            run_full_pipeline(&config).await?;
        }
        "run" => {
            let config = ensure_config(&repo_root, &config_path)?;
            run_full_pipeline(&config).await?;
        }
        "add-repo" => {
            let mut config = ensure_config(&repo_root, &config_path)?;
            let repo_url = if let Some(url) = args.get(2) {
                url.clone()
            } else {
                Input::<String>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Repository URL")
                    .interact_text()
                    .context("Failed to read repository URL")?
            };
            add_repo(&mut config, &repo_url)?;
            save_config(&config_path, &config)?;
            run_full_pipeline(&config).await?;
        }
        "doctor" => {
            run_doctor(&repo_root)?;
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
    println!("    skill-manage add-repo <URL>     # add repo then run full pipeline");
    println!("    skill-manage doctor             # run diagnostics");
    println!("    skill-manage --help");
    println!("    skill-manage --version");
}

async fn run_interactive(repo_root: &Path, config_path: &Path) -> Result<()> {
    let mut config = match load_config(config_path)? {
        Some(cfg) => cfg,
        None => {
            println!("No setup found. Starting first-time setup...");
            let cfg = run_setup_wizard(repo_root)?;
            save_config(config_path, &cfg)?;
            run_full_pipeline(&cfg).await?;
            cfg
        }
    };

    let theme = ColorfulTheme::default();
    let options = vec![
        "Run full pipeline now",
        "Add new repository URL and run",
        "Show current setup",
        "Reconfigure setup",
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
                let repo_url: String = Input::with_theme(&theme)
                    .with_prompt("Repository URL")
                    .interact_text()
                    .context("Failed to read repository URL")?;
                add_repo(&mut config, &repo_url)?;
                save_config(config_path, &config)?;
                run_full_pipeline(&config).await?;
            }
            2 => {
                print_config_summary(&config);
            }
            3 => {
                let new_cfg = run_setup_wizard(repo_root)?;
                save_config(config_path, &new_cfg)?;
                config = new_cfg;
                run_full_pipeline(&config).await?;
            }
            4 => break,
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

    let workspace_root = workspace_root_from_repo_root(repo_root);
    let config = SetupConfig {
        version: 1,
        repo_root: repo_root.to_string_lossy().to_string(),
        workspace_root: workspace_root.to_string_lossy().to_string(),
        sync_scope,
        tools,
        repositories,
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
    let repo_root = config.repo_root_path();
    println!("\n=== skill-manage full automation ===");

    if let Some(manifest) = prepare_manifest(&repo_root, &config.repositories)? {
        println!("[1/3] Clone/update repositories (shallow)...");
        run_fetch(&repo_root, manifest).await?;
    } else {
        println!("[1/3] No repository inputs configured; skipping clone/update.");
    }

    println!("[2/3] Aggregate skills...");
    run_aggregate_script(&repo_root)?;

    println!("[3/3] Sync to selected tools...");
    let targets = resolve_sync_targets(config)?;
    run_sync_script(&repo_root, &targets)?;

    println!("\n[OK] Automation complete.");
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

fn run_aggregate_script(repo_root: &Path) -> Result<()> {
    let script = repo_root
        .join("scripts")
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

fn run_sync_script(repo_root: &Path, targets: &[PathBuf]) -> Result<()> {
    if targets.is_empty() {
        bail!("No sync targets were resolved from setup");
    }

    let script = repo_root.join("scripts").join("sync-hubs.ps1");
    if !script.exists() {
        bail!("Sync script not found: {}", script.display());
    }

    let target_list = targets
        .iter()
        .map(|p| format!("'{}'", escape_ps_single(&p.to_string_lossy())))
        .collect::<Vec<_>>()
        .join(", ");

    let command = format!(
        "& '{}' -NoPrompt -Force -SyncMode Auto -TargetTools @({})",
        escape_ps_single(&script.to_string_lossy()),
        target_list
    );

    run_powershell_command(repo_root, &command).context("Failed while running sync script")
}

fn run_doctor(repo_root: &Path) -> Result<()> {
    let _guard = pushd(repo_root)?;
    let diagnostics = Diagnostics::new();
    let _ = diagnostics.run_all().context("Diagnostics failed")?;
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
                SyncScope::Global => targets.push(home_dir.join(tool.global_rel)),
                SyncScope::Local => targets.push(workspace_root.join(tool.local_rel)),
                SyncScope::Both => {
                    targets.push(home_dir.join(tool.global_rel));
                    targets.push(workspace_root.join(tool.local_rel));
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

            for key in ["repositories", "repos", "links", "items"] {
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
    candidates.push(cwd.clone());
    candidates.push(cwd.join("skill-manage"));
    if let Some(parent) = cwd.parent() {
        candidates.push(parent.to_path_buf());
        candidates.push(parent.join("skill-manage"));
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.to_path_buf());
            candidates.push(grandparent.join("skill-manage"));
        }
    }

    for candidate in candidates {
        if is_skill_manage_root(&candidate) {
            return Ok(candidate);
        }
    }

    bail!(
        "Could not locate the skill-manage root. Run this from inside the repo (e.g. skill-manage/cli)."
    )
}

fn is_skill_manage_root(path: &Path) -> bool {
    path.join("scripts").join("sync-hubs.ps1").exists()
        && path
            .join("scripts")
            .join("aggregate-skills-to-subhubs.ps1")
            .exists()
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
            println!("No setup file found. Running setup now...");
            let cfg = run_setup_wizard(repo_root)?;
            save_config(config_path, &cfg)?;
            Ok(cfg)
        }
    }
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
    let path = PathBuf::from(input.trim());
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
