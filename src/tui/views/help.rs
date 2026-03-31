use crate::tui::app::TuiApp;
use crate::tui::views::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(app: &mut TuiApp, f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(4),    // Help content
        ])
        .split(area);

    let tabs = Paragraph::new(Line::from(vec![
        Span::styled(" [1] Dashboard ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Dashboard { COLORS_ACCENT } else { COLORS_TEXT })),
        Span::raw(" | "),
        Span::styled(" [2] Skills List ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Skills { COLORS_ACCENT } else { COLORS_TEXT })),
        Span::raw(" | "),
        Span::styled(" [3] Help ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Help { COLORS_ACCENT } else { COLORS_TEXT }).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
    .style(Style::default().bg(COLORS_BG));

    f.render_widget(tabs, chunks[0]);

    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD).fg(COLORS_ACCENT))),
        Line::from("  Tab, l, Right : Switch to next tab"),
        Line::from("  Shift+Tab, h, Left: Switch to previous tab"),
        Line::from("  Up, k         : Scroll up"),
        Line::from("  Down, j       : Scroll down"),
        Line::from(""),
        Line::from(Span::styled("Application", Style::default().add_modifier(Modifier::BOLD).fg(COLORS_ACCENT))),
        Line::from("  q, Esc, Ctrl+C: Quit application"),
        Line::from("  r             : Refresh data (reload cache)"),
        Line::from("  Enter         : Select item / drill down"),
        Line::from(""),
        Line::from(Span::styled("About", Style::default().add_modifier(Modifier::BOLD).fg(COLORS_ACCENT))),
        Line::from("  skill-manage v0.1.0"),
        Line::from("  AI Agent skill routing and classification engine"),
    ];

    let content = Paragraph::new(help_text)
        .block(Block::default().title(" Keybindings & Help ").borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
        .style(Style::default().bg(COLORS_BG).fg(COLORS_TEXT));

    f.render_widget(content, chunks[1]);
}
