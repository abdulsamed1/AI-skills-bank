use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use clap::Parser;
use skill_manage::cli::{Cli, Commands};
use skill_manage::components::manifest::RepoManifest;
use skill_manage::components::fetcher::Fetcher;
use skill_manage::components::syncer::Syncer;
use skill_manage::components::aggregator::Aggregator;
use skill_manage::utils::progress::ProgressManager;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize ProgressManager based on machine-readable flag
    let progress = Arc::new(ProgressManager::new(!cli.json));
    
    match cli.command {
        Commands::Fetch { dry_run } => {
            let manifest_path = Path::new("repos.json");
            if !manifest_path.exists() {
                anyhow::bail!("Manifest file not found: repos.json");
            }
            let manifest = RepoManifest::load(manifest_path)?;
            let fetcher = Fetcher::with_manifest(manifest, Arc::clone(&progress));
            fetcher.fetch(dry_run).await?;
        },
        Commands::Sync { destination, link, dry_run } => {
            let syncer = Syncer::new(Arc::clone(&progress));
            syncer.sync(destination, link, dry_run).await?;
        },
        Commands::Aggregate { force } => {
            let aggregator = Aggregator::new(Arc::clone(&progress));
            let skills = aggregator.aggregate(force).await?;
            aggregator.generate_csv(skills).await?;
        },
        _ => {
            if !cli.json {
                println!("skill-manage v{}", env!("CARGO_PKG_VERSION"));
                println!("Command: {:?}", cli.command);
            }
        }
    }
    
    Ok(())
}
