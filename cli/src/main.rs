use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use clap::Parser;
use skill_manage::cli::{Cli, Commands};
use skill_manage::components::manifest::RepoManifest;
use skill_manage::components::fetcher::Fetcher;
use skill_manage::components::syncer::Syncer;
use skill_manage::components::aggregator::Aggregator;
use skill_manage::components::diagnostics::Diagnostics;
use skill_manage::components::CommandResult;
use skill_manage::utils::progress::ProgressManager;
use skill_manage::utils::log::Logger;
use skill_manage::utils::theme::Theme;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize Theme, Logger and ProgressManager
    let theme = Arc::new(Theme::new());
    let logger = Logger::new(cli.json, Arc::clone(&theme));
    let progress = Arc::new(ProgressManager::new(!cli.json, Arc::clone(&theme)));
    
    let result: Result<CommandResult, skill_manage::error::SkillManageError> = match cli.command {
        Commands::Fetch { dry_run } => {
            logger.step(theme.fetch_emoji, "Starting repository fetch...");
            let manifest_path = Path::new("repos.json");
            if !manifest_path.exists() {
                Err(skill_manage::error::SkillManageError::ConfigError("Manifest file not found: repos.json".to_string()))
            } else {
                let manifest = RepoManifest::load(manifest_path)?;
                let fetcher = Fetcher::with_manifest(manifest, Arc::clone(&progress));
                fetcher.fetch(dry_run).await
            }
        },
        Commands::Sync { destination, link, dry_run } => {
            logger.step(theme.sync_emoji, "Starting skill synchronization...");
            let syncer = Syncer::new(Arc::clone(&progress));
            syncer.sync(destination, link, dry_run).await
        },
        Commands::Aggregate { force } => {
            logger.step(theme.aggregate_emoji, "Starting skill aggregation...");
            let aggregator = Aggregator::new(Arc::clone(&progress));
            let res = aggregator.aggregate(force).await?;
            if let CommandResult::Aggregate { ref skills } = res {
                aggregator.generate_csv(skills.clone()).await?;
            }
            Ok(res)
        },
        Commands::Doctor => {
            logger.step(theme.doctor_emoji, "Running system diagnostics...");
            let diagnostics = Diagnostics::new();
            diagnostics.run_all()
        },
    };

    match result {
        Ok(cmd_res) => {
            if cli.json {
                let json = serde_json::to_string_pretty(&cmd_res).unwrap();
                logger.result(&json);
            } else {
                match cmd_res {
                    CommandResult::Fetch { cloned, updated } => {
                        logger.success(&format!("Fetch complete: {} cloned, {} updated.", cloned.len(), updated.len()));
                    },
                    CommandResult::Sync { synced, target } => {
                        logger.success(&format!("Sync complete: {} skills synchronized to {}.", synced.len(), target));
                    },
                    CommandResult::Aggregate { skills } => {
                        logger.success(&format!("Aggregation complete: {} skills processed into hub-manifests.csv.", skills.len()));
                    },
                    CommandResult::Doctor { health_score, .. } => {
                        if health_score == 100 {
                            logger.success("Diagnostic complete. Skills bank is in optimal health!");
                        } else {
                            logger.info(&format!("Diagnostic complete. Health Score: {}%", health_score));
                        }
                    }
                }
            }
        },
        Err(e) => {
            if cli.json {
                let json = serde_json::to_string_pretty(&e.to_json()).unwrap();
                eprintln!("{}", json);
            } else {
                logger.error(&e.to_string());
            }
            std::process::exit(1);
        }
    }
    
    Ok(())
}
