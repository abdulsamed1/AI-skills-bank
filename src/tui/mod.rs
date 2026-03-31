pub mod action;
pub mod app;
pub mod background;
pub mod event;
pub mod terminal;
pub mod views;

use crate::components::manifest::Repository;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use crate::utils::progress::ProgressReporter;

struct TuiReporter {
    sender: std::sync::mpsc::Sender<action::Action>,
}

impl ProgressReporter for TuiReporter {
    fn report(&self, value: u64, total: u64, msg: String) {
        let _ = self.sender.send(action::Action::ProgressUpdate { value, total, msg });
    }
}

pub async fn run_tui(repo_root: &Path, repos: Vec<Repository>) -> Result<()> {
    let mut terminal = terminal::init()?;
    
    let root = repo_root.to_path_buf();
    let repositories = repos.clone();

    let result = (|| -> Result<()> {
        // Initialize app state
        let mut app = app::TuiApp::new();
        app.load_data(repo_root, repos)?;
        
        // Setup events
        let mut events = event::EventHandler::new(250);
        let sender = events.sender();
        
        // Create reporter
        let reporter: Arc<dyn ProgressReporter> = Arc::new(TuiReporter { sender: sender.clone() });

        // Spawn background task for data processing
        background::spawn_background_worker(&root, repositories, sender, Some(reporter));

        loop {
            // Render
            terminal.draw(|f| app.render(f))?;

            // Handle events
            let action = events.next()?;
            app.update(action);

            if app.should_quit {
                break;
            }
        }
        Ok(())
    })();

    terminal::restore()?;
    result
}
