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
            Constraint::Length(3), // Header tabs
            Constraint::Length(4), // Stats summary
            Constraint::Min(4),    // Main content (Hub breakdown, LLM caching)
            Constraint::Length(1), // Footer
        ])
        .split(area);

    let tabs = Paragraph::new(Line::from(vec![
        Span::styled(" [1] Dashboard ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Dashboard { COLORS_ACCENT } else { COLORS_TEXT }).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(" [2] Skills List ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Skills { COLORS_ACCENT } else { COLORS_TEXT })),
        Span::raw(" | "),
        Span::styled(" [3] Help ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Help { COLORS_ACCENT } else { COLORS_TEXT })),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
    .style(Style::default().bg(COLORS_BG));

    f.render_widget(tabs, chunks[0]);

    let stats = Paragraph::new(vec![
        Line::from(vec![Span::raw(format!("Repositories configured: {}", app.repos.len()))]),
        Line::from(vec![Span::raw(format!("LLM Cache: {} Hits / {} Misses", app.cache_hits, app.cache_misses))]),
    ])
    .block(Block::default().title(" System Stats ").borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
    .style(Style::default().bg(COLORS_BG).fg(COLORS_TEXT));

    f.render_widget(stats, chunks[1]);
    
    // Background task loading bar overlay
    if app.is_loading {
        let pct = if app.loading_total > 0 { (app.loading_value as f64 / app.loading_total as f64) * 100.0 } else { 0.0 };
        let load_info = Paragraph::new(format!("Loading... {:.1}% | {}", pct, app.loading_msg))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLORS_ACCENT)))
            .style(Style::default().fg(COLORS_ACCENT));
        // simple overlay on chunks 2
        f.render_widget(load_info, chunks[2]);
    } else {
        let mut stats_lines = Vec::new();
        if app.hub_stats.is_empty() {
            stats_lines.push(Line::from("No aggregated data found. Run 'aggregate' to generate stats."));
        } else {
            let mut sorted_hubs: Vec<_> = app.hub_stats.iter().collect();
            sorted_hubs.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
            
            for (hub, count) in sorted_hubs {
                stats_lines.push(Line::from(vec![
                    Span::styled(format!(" {:width$} ", hub, width=25), Style::default().fg(COLORS_ACCENT)),
                    Span::raw(format!(": {}", count)),
                ]));
            }
        }

        let content = Paragraph::new(stats_lines)
            .block(Block::default().title(" Hub Breakdown ").borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
            .style(Style::default().bg(COLORS_BG).fg(COLORS_TEXT));
        f.render_widget(content, chunks[2]);
    }

    let footer = Paragraph::new(" Press 'q' or 'esc' to quit | 'Tab' to switch views")
        .style(Style::default().bg(COLORS_PANEL).fg(COLORS_TEXT));
    f.render_widget(footer, chunks[3]);
}
