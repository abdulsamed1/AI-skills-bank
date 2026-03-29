use crossterm::style::Color;

pub struct Theme {
    pub use_color: bool,
    pub use_emoji: bool,
    
    pub success_color: Color,
    pub error_color: Color,
    pub warn_color: Color,
    pub info_color: Color,
    pub step_color: Color,
    pub primary_color: Color,
    
    pub success_emoji: &'static str,
    pub error_emoji: &'static str,
    pub warn_emoji: &'static str,
    pub info_emoji: &'static str,
    pub step_emoji: &'static str,
    pub fetch_emoji: &'static str,
    pub sync_emoji: &'static str,
    pub aggregate_emoji: &'static str,
    pub doctor_emoji: &'static str,
}

impl Theme {
    pub fn new() -> Self {
        let no_color = std::env::var("NO_COLOR").is_ok();
        let term = std::env::var("TERM").unwrap_or_default();
        let is_dumb = term == "dumb" || term == "";
        
        Self {
            use_color: !no_color && !is_dumb,
            use_emoji: !no_color && !is_dumb && !cfg!(windows), // Default conservative for Windows cmd
            
            success_color: Color::Green,
            error_color: Color::Red,
            warn_color: Color::Yellow,
            info_color: Color::Cyan,
            step_color: Color::Blue,
            primary_color: Color::Magenta,
            
            success_emoji: "✅",
            error_emoji: "❌",
            warn_emoji: "⚠️",
            info_emoji: "ℹ️",
            step_emoji: "🔍",
            fetch_emoji: "📥",
            sync_emoji: "🔄",
            aggregate_emoji: "📊",
            doctor_emoji: "🩺",
        }
    }

    pub fn format_log(&self, emoji: &str, color: Color, label: &str, msg: &str) -> String {
        use crossterm::style::Stylize;
        
        let prefix = if self.use_emoji { format!("{} ", emoji) } else { String::new() };
        let mut line = format!("{}{}: {}", prefix, label, msg);
        
        if self.use_color {
            line = line.with(color).to_string();
        }
        line
    }
}
