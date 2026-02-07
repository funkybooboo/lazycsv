//! Help overlay rendering with keybinding reference.
//!
//! Displays a modal help overlay showing all available keybindings and
//! navigation commands when triggered by '?'. Supports scrolling on small
//! screens.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Width percentage for help overlay (70% of terminal width)
const HELP_OVERLAY_WIDTH_PERCENT: u16 = 70;

/// Height percentage for help overlay (80% of terminal height)
const HELP_OVERLAY_HEIGHT_PERCENT: u16 = 80;

/// Build the help text lines
fn build_help_text() -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            "LazyCSV v0.4.0 - Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "NAVIGATION",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  hjkl / arrows      Move cursor (with count: 5j, 10h)"),
        Line::from("  w / b / e          Next/prev/last non-empty cell"),
        Line::from("  gg                 First row"),
        Line::from("  G / <n>G           Last row / row n (e.g., 15G)"),
        Line::from("  0 / $              First/last column"),
        Line::from("  Ctrl+d / Ctrl+u    Page down/up"),
        Line::from(""),
        Line::from(Span::styled(
            "COMMAND MODE",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  :                  Enter command mode"),
        Line::from("  :15                Jump to row 15"),
        Line::from("  :c A / :c BC       Jump to column A/BC"),
        Line::from("  :q                 Quit"),
        Line::from("  Esc                Cancel command"),
        Line::from(""),
        Line::from(Span::styled(
            "INSERT MODE",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  i / a              Edit cell (cursor at end)"),
        Line::from("  I                  Edit cell (cursor at start)"),
        Line::from("  A                  Edit cell (cursor at end)"),
        Line::from("  s                  Replace cell (clear + edit)"),
        Line::from("  F2                 Edit cell"),
        Line::from("  Delete             Clear cell (stay in Normal)"),
        Line::from(""),
        Line::from(Span::styled(
            "INSERT MODE EDITING",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Enter              Commit, move down"),
        Line::from("  Shift+Enter        Commit, move up"),
        Line::from("  Tab                Commit, move right"),
        Line::from("  Shift+Tab          Commit, move left"),
        Line::from("  Esc                Cancel edit"),
        Line::from("  Backspace          Delete char before cursor"),
        Line::from("  Ctrl+w             Delete word backward"),
        Line::from("  Ctrl+u             Delete to start"),
        Line::from(""),
        Line::from(Span::styled(
            "ROW OPERATIONS",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  o                  Insert row below, enter Insert"),
        Line::from("  O                  Insert row above, enter Insert"),
        Line::from("  dd                 Delete row"),
        Line::from("  yy                 Yank (copy) row"),
        Line::from("  p                  Paste row below"),
        Line::from(""),
        Line::from(Span::styled(
            "VIEWPORT & FILES",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  zt / zz / zb       Row at top/center/bottom"),
        Line::from("  [ / ]              Previous/next file"),
        Line::from(""),
        Line::from(Span::styled(
            "GLOBAL",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ?                  Toggle this help (j/k to scroll)"),
        Line::from("  :q                 Quit"),
        Line::from(""),
    ]
}

/// Render the help overlay with keybinding reference.
///
/// Displays a centered modal window showing all available keybindings
/// for navigation, editing, and other commands. The overlay covers
/// 70% of terminal width and 80% of height. Supports scrolling with
/// j/k keys on small screens.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `scroll_offset` - Vertical scroll offset for content
pub fn render_help_overlay(frame: &mut Frame, scroll_offset: u16) {
    // Create centered area
    let area = centered_rect(
        HELP_OVERLAY_WIDTH_PERCENT,
        HELP_OVERLAY_HEIGHT_PERCENT,
        frame.area(),
    );

    let help_text = build_help_text();

    // Calculate if scrolling is needed
    let content_height = help_text.len() as u16;
    let visible_height = area.height.saturating_sub(2); // -2 for borders
    let needs_scroll = content_height > visible_height;

    // Build title with scroll indicator
    let title = if needs_scroll {
        let max_scroll = content_height.saturating_sub(visible_height);
        if scroll_offset >= max_scroll {
            " Help (END) ".to_string()
        } else if scroll_offset > 0 {
            format!(" Help ({}/{}) ", scroll_offset + 1, max_scroll + 1)
        } else {
            " Help (j/k to scroll) ".to_string()
        }
    } else {
        " Help ".to_string()
    };

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .scroll((scroll_offset, 0));

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
