pub mod fetcher;
pub mod syncer;
pub mod aggregator;
pub mod diagnostics;
pub mod manifest;

use serde::Serialize;
use crate::components::aggregator::SkillMetadata;
use crate::components::diagnostics::DiagnosticStatus;

pub trait Component {
    // Shared trait for UI/logic components
}

#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "lowercase")]
pub enum CommandResult {
    Fetch {
        cloned: Vec<String>,
        updated: Vec<String>,
    },
    Sync {
        synced: Vec<String>,
        target: String,
    },
    Aggregate {
        skills: Vec<SkillMetadata>,
    },
    Doctor {
        checks: Vec<(String, DiagnosticStatus)>,
        health_score: u32,
    },
}
