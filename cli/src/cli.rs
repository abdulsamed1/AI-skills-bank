use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skill-manage")]
#[command(about = "skill-manage aggregation and sync workflows", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Set the project root directory
    #[arg(short, long, global = true)]
    pub project: Option<String>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output machine-readable JSON to stdout
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fetch remote repositories from repos.json
    Fetch {
        /// Only check for updates without downloading
        #[arg(long)]
        dry_run: bool,
    },
    /// Synchronize discovered skills to destination
    Sync {
        /// Target destination path
        #[arg(short, long)]
        destination: Option<String>,
        
        /// Perform a dry run without modifying files
        #[arg(long)]
        dry_run: bool,
    },
    /// Aggregate all skills into a central routing manifest
    Aggregate {
        /// Force re-aggregation of all files
        #[arg(short, long)]
        force: bool,
    },
    /// Run diagnostic checks on the skills bank
    Doctor,
}
