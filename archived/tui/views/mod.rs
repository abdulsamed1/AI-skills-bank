pub mod dashboard;
pub mod help;
pub mod skills;

use ratatui::style::Color;

// A premium color palette
pub const COLORS_BG: Color = Color::Rgb(15, 15, 18);
pub const COLORS_PANEL: Color = Color::Rgb(30, 30, 35);
pub const COLORS_BORDER: Color = Color::Rgb(80, 80, 90);
pub const COLORS_TEXT: Color = Color::Rgb(220, 220, 230);
pub const COLORS_ACCENT: Color = Color::Rgb(80, 160, 255); // A nice blue
pub const COLORS_SUCCESS: Color = Color::Rgb(50, 200, 120);
pub const COLORS_ERROR: Color = Color::Rgb(255, 80, 80);
