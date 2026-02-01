use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render centered help overlay
pub fn render_help_overlay(frame: &mut Frame) {
    // Create centered area (60% width, 70% height)
    let area = centered_rect(60, 70, frame.area());

    let help_text = vec![
        Line::from(Span::styled(
            "LazyCSV - Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  hjkl / arrows      Move cursor"),
        Line::from("  gg / Home          First row"),
        Line::from("  G / End            Last row"),
        Line::from("  w / b              Next/previous column"),
        Line::from("  0 / $              First/last column"),
        Line::from("  PageUp / PageDown  Page up/down"),
        Line::from("  [ / ]              Previous/next file"),
        Line::from(""),
        Line::from("Future - Cell Editing (Phase 2):"),
        Line::from("  i / Enter          Edit cell (not yet implemented)"),
        Line::from("  Esc                Cancel edit"),
        Line::from("  Ctrl+S             Save file"),
        Line::from(""),
        Line::from("Future - Rows/Columns (Phase 3):"),
        Line::from("  o / O              Add row below/above"),
        Line::from("  dd                 Delete row"),
        Line::from("  yy / p             Copy/paste row"),
        Line::from(""),
        Line::from("Future - Search (Phase 4):"),
        Line::from("  /                  Fuzzy search"),
        Line::from("  s                  Sort by column"),
        Line::from(""),
        Line::from("Other:"),
        Line::from("  ?                  Toggle this help"),
        Line::from("  q                  Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? or Esc to close",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];

    let help =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title(" Help "));

    // Clear background
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

/// Helper to create centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
