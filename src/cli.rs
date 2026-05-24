use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skills-bank")]
#[command(about = "skills-bank aggregation and sync workflows", long_about = None)]
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

    /// Suppress all progress and informational output to stderr
    #[arg(short, long, global = true)]
    pub silent: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fetch remote repositories from.skills-bank-cli-config.json
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

        /// Use junctions (Windows) or symlinks (Unix) instead of copying
        #[arg(short, long)]
        link: bool,

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
    /// Launch the interactive Terminal User Interface (TUI)
    Tui,
}
