//! Help overlay rendering with keybinding reference.
//!
//! Displays a modal help overlay showing all available keybindings and
//! navigation commands when triggered by '?'.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Width percentage for help overlay (60% of terminal width)
const HELP_OVERLAY_WIDTH_PERCENT: u16 = 60;

/// Height percentage for help overlay (70% of terminal height)
const HELP_OVERLAY_HEIGHT_PERCENT: u16 = 70;

/// Render the help overlay with keybinding reference.
///
/// Displays a centered modal window showing all available keybindings
/// for navigation, editing, and other commands. The overlay covers
/// 60% of terminal width and 70% of height.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
pub fn render_help_overlay(frame: &mut Frame) {
    // Create centered area (60% width, 70% height)
    let area = centered_rect(
        HELP_OVERLAY_WIDTH_PERCENT,
        HELP_OVERLAY_HEIGHT_PERCENT,
        frame.area(),
    );

    let help_text = vec![
        Line::from(Span::styled(
            "LazyCSV v0.3.0 - Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "BASIC NAVIGATION",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  hjkl / arrows      Move cursor (with optional count: 5j, 10h)"),
        Line::from("  Enter              Move down one row"),
        Line::from("  w / b / e          Next/prev/last non-empty cell in row"),
        Line::from("  Ctrl+d / Ctrl+u    Page down/up"),
        Line::from(""),
        Line::from(Span::styled(
            "JUMPING",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  gg                 Jump to first row"),
        Line::from("  G / <n>G           Jump to last row / line n (e.g., 15G)"),
        Line::from("  0 / $              Jump to first/last column"),
        Line::from("  ga / gB / gBC      Jump to column A/B/BC (Excel-style)"),
        Line::from(""),
        Line::from(Span::styled(
            "COMMAND MODE",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  :                  Enter command mode"),
        Line::from("  :15                Jump to line 15"),
        Line::from("  :B / :BC           Jump to column B/BC"),
        Line::from("  Esc                Cancel command"),
        Line::from(""),
        Line::from(Span::styled(
            "VIEWPORT CONTROL",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  zt                 Position current row at top"),
        Line::from("  zz                 Position current row at center"),
        Line::from("  zb                 Position current row at bottom"),
        Line::from(""),
        Line::from(Span::styled(
            "FILE MANAGEMENT",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  [ / ]              Previous/next file"),
        Line::from(""),
        Line::from(Span::styled(
            "GLOBAL",
            Style::default().add_modifier(Modifier::BOLD),
        )),
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
