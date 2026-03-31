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
            Constraint::Min(4),    // Skills Table
            Constraint::Length(10), // Detail Pane
        ])
        .split(area);

    let tabs = Paragraph::new(Line::from(vec![
        Span::styled(" [1] Dashboard ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Dashboard { COLORS_ACCENT } else { COLORS_TEXT })),
        Span::raw(" | "),
        Span::styled(" [2] Skills List ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Skills { COLORS_ACCENT } else { COLORS_TEXT }).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(" [3] Help ", Style::default().fg(if app.active_tab == crate::tui::app::Tab::Help { COLORS_ACCENT } else { COLORS_TEXT })),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
    .style(Style::default().bg(COLORS_BG));

    f.render_widget(tabs, chunks[0]);

    if app.skills.is_empty() {
        let empty = Paragraph::new("No skills loaded. Try running aggregation to prime the LLM cache first.")
            .block(Block::default().title(" Skills ").borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
            .style(Style::default().bg(COLORS_BG).fg(COLORS_TEXT));
        f.render_widget(empty, chunks[1]);
    } else {
        // Handle scrolling
        let height = (chunks[1].height.saturating_sub(2)) as usize;
        if app.table_selected_index < app.table_scroll_index {
            app.table_scroll_index = app.table_selected_index;
        } else if app.table_selected_index >= app.table_scroll_index + height {
            app.table_scroll_index = app.table_selected_index - height + 1;
        }

        let mut lines = Vec::new();
        for (i, skill) in app.skills.iter().enumerate().skip(app.table_scroll_index).take(height) {
            let actual_idx = i + app.table_scroll_index;
            let style = if actual_idx == app.table_selected_index {
                Style::default().bg(COLORS_ACCENT).fg(COLORS_BG)
            } else {
                Style::default().fg(COLORS_TEXT)
            };
            
            lines.push(Line::from(vec![
                Span::styled(format!(" {:width$} ", skill.hub, width=15), style.add_modifier(Modifier::BOLD)),
                Span::styled(format!(" | {:width$} ", skill.name, width=30), style),
                Span::styled(format!(" | {}", skill.sub_hub), style.add_modifier(Modifier::ITALIC)),
            ]));
        }
        
        let list = Paragraph::new(lines)
            .block(Block::default()
                .title(format!(" Skills Explorer ({}/{}) - ↑/↓ to scroll ", app.table_selected_index + 1, app.skills.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLORS_BORDER)))
            .style(Style::default().bg(COLORS_BG).fg(COLORS_TEXT));
        f.render_widget(list, chunks[1]);
        
        // Detail panel
        if let Some(selected) = app.skills.get(app.table_selected_index) {
            let color = match selected.hub.to_lowercase().as_str() {
                "system" => ratatui::style::Color::Cyan,
                "workflow" => ratatui::style::Color::Green,
                "tool" => ratatui::style::Color::Yellow,
                _ => COLORS_ACCENT,
            };

            let details = Paragraph::new(vec![
                Line::from(vec![Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&selected.name)]),
                Line::from(vec![Span::styled("Hub:  ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(&selected.hub, Style::default().fg(color))]),
                Line::from(vec![Span::styled("Sub:  ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&selected.sub_hub)]),
                Line::from(vec![Span::styled("Prob: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(format!("{}%", selected.match_score.unwrap_or(0)))]),
                Line::from(vec![Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(selected.path.to_string_lossy())]),
                Line::from(vec![Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&selected.description)]),
            ])
            .block(Block::default().title(" Detailed Metadata ").borders(Borders::ALL).border_style(Style::default().fg(COLORS_BORDER)))
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(Style::default().bg(COLORS_PANEL).fg(COLORS_TEXT));
            f.render_widget(details, chunks[2]);
        }
    }
}
