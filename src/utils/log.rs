use crate::utils::theme::Theme;
use crossterm::style::Stylize;
use std::io::{stderr, Write};
use std::sync::Arc;

pub struct Logger {
    pub json: bool,
    pub silent: bool,
    pub theme: Arc<Theme>,
}

impl Logger {
    pub fn new(json: bool, silent: bool, theme: Arc<Theme>) -> Self {
        Self {
            json,
            silent,
            theme,
        }
    }

    /// Log success to stderr.
    pub fn success(&self, msg: &str) {
        if !self.json && !self.silent {
            let line = self.theme.format_log(
                self.theme.success_emoji,
                self.theme.success_color,
                "success",
                msg,
            );
            let _ = writeln!(stderr(), "{}", line);
        }
    }

    /// Log information to stderr.
    pub fn info(&self, msg: &str) {
        if !self.json && !self.silent {
            let line =
                self.theme
                    .format_log(self.theme.info_emoji, self.theme.info_color, "info", msg);
            let _ = writeln!(stderr(), "{}", line);
        }
    }

    /// Log a warning to stderr.
    pub fn warn(&self, msg: &str) {
        if !self.json && !self.silent {
            let line =
                self.theme
                    .format_log(self.theme.warn_emoji, self.theme.warn_color, "warning", msg);
            let _ = writeln!(stderr(), "{}", line);
        }
    }

    /// Log an error to stderr. Errors are shown even in silent mode.
    pub fn error(&self, msg: &str) {
        if self.json {
            let _ = writeln!(stderr(), "error: {}", msg);
        } else {
            let mut line =
                self.theme
                    .format_log(self.theme.error_emoji, self.theme.error_color, "error", msg);
            if self.theme.use_color {
                line = line.bold().to_string();
            }
            let _ = writeln!(stderr(), "{}", line);
        }
    }

    /// Log a step or discovery action.
    pub fn step(&self, emoji: &str, msg: &str) {
        if !self.json && !self.silent {
            let prefix = if self.theme.use_emoji {
                format!("{} ", emoji)
            } else {
                String::new()
            };
            let mut line = format!("{}{}", prefix, msg);
            if self.theme.use_color {
                line = line.with(self.theme.step_color).to_string();
            }
            let _ = writeln!(stderr(), "{}", line);
        }
    }

    /// Print a header or prominent section.
    pub fn header(&self, msg: &str) {
        if !self.json && !self.silent {
            let mut line = msg.to_string();
            if self.theme.use_color {
                line = line
                    .bold()
                    .underlined()
                    .with(self.theme.primary_color)
                    .to_string();
            }
            let _ = writeln!(stderr(), "\n{}", line);
        }
    }

    /// Print primary data to stdout.
    pub fn result(&self, msg: &str) {
        println!("{}", msg);
    }
}
