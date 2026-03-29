use crate::utils::theme::Theme;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

pub struct ProgressManager {
    multi: Option<MultiProgress>,
    theme: Arc<Theme>,
    pub silent: bool,
}

impl ProgressManager {
    /// Create a new ProgressManager. If enabled is false or silent is true, all methods will return dummy bars.
    pub fn new(enabled: bool, silent: bool, theme: Arc<Theme>) -> Self {
        let multi = if enabled && !silent {
            Some(MultiProgress::with_draw_target(ProgressDrawTarget::stderr()))
        } else {
            None
        };

        Self {
            multi,
            theme,
            silent,
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
