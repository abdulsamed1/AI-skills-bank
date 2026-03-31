use crate::components::aggregator::{Aggregator, SkillMetadata};
use crate::components::manifest::Repository;
use crate::tui::action::Action;
use crate::utils::progress::ProgressManager;
use crate::utils::theme::Theme;
use crate::utils::progress::ProgressReporter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use tokio::runtime::Runtime;
use std::thread;

pub fn spawn_background_worker(
    _repo_root: &Path,
    _repos: Vec<Repository>,
    tx: Sender<Action>,
    reporter: Option<Arc<dyn ProgressReporter>>,
) {
    
    thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(Action::Error(format!("Failed to start background async runtime: {}", e)));
                return;
            }
        };

        rt.block_on(async move {
            let theme = Arc::new(Theme::new());
            // Create a ProgressManager that uses our TUI reporter
            let progress = Arc::new(ProgressManager::new(false, true, theme, reporter));
            let aggregator = Aggregator::new(progress);

            // Run aggregation (non-forced)
            match aggregator.aggregate(false).await {
                Ok(result) => {
                    if let crate::components::CommandResult::Aggregate { skills } = result {
                        let _ = tx.send(Action::DataLoaded(skills));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(format!("Background aggregation failed: {}", e)));
                }
            }
        });
    });
}
