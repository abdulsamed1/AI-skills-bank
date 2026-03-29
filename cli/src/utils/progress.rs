use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;
use std::sync::Arc;
use crate::utils::theme::Theme;

pub struct ProgressManager {
    multi: Option<MultiProgress>,
    theme: Arc<Theme>,
}

impl ProgressManager {
    /// Create a new ProgressManager. If enabled is false, all methods will return dummy bars.
    pub fn new(enabled: bool, theme: Arc<Theme>) -> Self {
        Self {
            multi: if enabled { Some(MultiProgress::new()) } else { None },
            theme,
        }
    }

    /// Create a global progress bar for a sequence of tasks
    pub fn create_main_bar(&self, total: u64, message: &str) -> ProgressBar {
        match &self.multi {
            Some(multi) => {
                let pb = multi.add(ProgressBar::new(total));
                pb.set_style(
                    ProgressStyle::with_template(
                        "{span:.bold.blue} {bar:40.cyan/blue} {pos}/{len} ({percent}%) {msg}",
                    )
                    .unwrap()
                    .progress_chars("━━╴"),
                );
                pb.set_message(message.to_string());
                pb
            }
            None => ProgressBar::hidden(),
        }
    }

    /// Create a spinner for a long-running individual task
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        match &self.multi {
            Some(multi) => {
                let pb = multi.add(ProgressBar::new_spinner());
                pb.enable_steady_tick(Duration::from_millis(120));
                
                let template = if self.theme.use_emoji {
                    "{spinner} {msg}"
                } else {
                    "{spinner:.green} {msg}"
                };

                let tick_strings: &[&str] = if self.theme.use_emoji {
                    &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
                } else {
                    &["-", "\\", "|", "/"]
                };

                pb.set_style(
                    ProgressStyle::with_template(template)
                        .unwrap()
                        .tick_strings(tick_strings),
                );
                pb.set_message(message.to_string());
                pb
            }
            None => ProgressBar::hidden(),
        }
    }
}
