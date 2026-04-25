//! Shared modal/overlay UI utilities.
//!
//! This module provides standardized dimensions, layouts, and theme-aware
//! style helpers for all modal overlays (Help, SQL Editor, Magnifier,
//! File Manager, etc.)
//!
//! # Design Philosophy
//!
//! **Single source of truth:** styles are derived from the user's
//! [`crate::config::Theme`]. UI code MUST go through the `*_style(theme)`
//! / `*_block(theme)` helpers — never hardcode colors.
//!
//! **Consistency:** all large modals (Help, SQL, Magnifier, File Manager) use
//! 80% × 80% sizing. All small prompts use 40% × 20% sizing. All modals share
//! the same border, popup background, and status-bar layout.
//!
//! # Quick Start
//!
//! ```ignore
//! use crate::ui::modal;
//!
//! let area = modal::large_modal_rect(frame.area());
//! let block = modal::popup_block(theme, " My Modal ");
//! let inner = block.inner(area);
//! frame.render_widget(block, area);
//!
//! let (content, status) = modal::split_with_status_bar(inner);
//! ```

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders},
};

use crate::config::Theme;

// ============================================================================
// MODAL SIZE CONSTANTS
// ============================================================================

/// Width percentage for large modals (Help, SQL, Magnifier, File Manager).
pub const MODAL_LARGE_WIDTH: u16 = 80;

/// Height percentage for large modals.
pub const MODAL_LARGE_HEIGHT: u16 = 80;

/// Width percentage for small prompts (file operations).
pub const MODAL_SMALL_WIDTH: u16 = 40;

/// Height percentage for small prompts.
pub const MODAL_SMALL_HEIGHT: u16 = 20;

// ============================================================================
// THEME-AWARE STYLES
// ============================================================================

/// Cursor / active-cell style.
pub fn cursor_style(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.table.cursor_bg)
        .fg(theme.table.cursor_fg)
        .add_modifier(Modifier::BOLD)
}

/// Visual-mode (multi-cell) selection style.
pub fn visual_selection_style(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.table.selection_bg)
        .fg(theme.table.selection_fg)
}

/// Search-match highlight style.
pub fn search_match_style(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.table.search_match_bg)
        .fg(theme.table.search_match_fg)
}

/// Zebra-stripe row background.
pub fn zebra_stripe_style(theme: &Theme) -> Style {
    Style::default().bg(theme.table.zebra_bg)
}

/// Mode badge in status bar (e.g., "NORMAL", "INSERT").
pub fn mode_indicator_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.status.mode_fg)
        .bg(theme.status.mode_bg)
}

/// Error message style (status bar, dialogs).
pub fn error_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.status.error_fg)
        .add_modifier(Modifier::BOLD)
}

/// Success message style.
pub fn success_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.status.success_fg)
        .add_modifier(Modifier::BOLD)
}

/// Popup background fill.
pub fn popup_bg_style(theme: &Theme) -> Style {
    Style::default().bg(theme.popup.bg)
}

/// Popup foreground+background (use for body text inside a popup).
pub fn popup_text_style(theme: &Theme) -> Style {
    Style::default().bg(theme.popup.bg).fg(theme.popup.fg)
}

/// Completion menu — selected entry.
pub fn completion_selected_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.popup.completion_sel_fg)
        .bg(theme.popup.completion_sel_bg)
}

/// Completion menu — unselected entry.
pub fn completion_unselected_style(theme: &Theme) -> Style {
    Style::default().fg(theme.popup.fg).bg(theme.popup.bg)
}

/// Bold (theme-agnostic — used for headers, labels).
pub fn bold_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Dim text (previews, hints).
pub fn dim_style() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Row-number bold style.
pub fn row_number_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a centered rectangle with specified width/height percentages.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

/// Standard large modal (80% × 80%).
pub fn large_modal_rect(r: Rect) -> Rect {
    centered_rect(MODAL_LARGE_WIDTH, MODAL_LARGE_HEIGHT, r)
}

/// Standard small prompt (40% × 20%).
pub fn small_modal_rect(r: Rect) -> Rect {
    centered_rect(MODAL_SMALL_WIDTH, MODAL_SMALL_HEIGHT, r)
}

/// Split a modal's inner area into (content, status_bar).
pub fn split_with_status_bar(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    (chunks[0], chunks[1])
}

/// Themed popup block: bordered, titled, with the configured popup background
/// and border color. Replaces the older `standard_block`.
pub fn popup_block<'a>(theme: &Theme, title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(theme.popup.title_fg))
        .border_style(Style::default().fg(theme.popup.border_fg))
        .style(popup_bg_style(theme))
}

/// Build a two-part status line (left + right with padding).
pub fn build_status_line(left: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let padding = width.saturating_sub(left_len).saturating_sub(right_len);
    format!("{}{}{}", left, " ".repeat(padding), right)
}

/// Build a three-part status line (left + center + right).
pub fn build_three_part_status_line(left: &str, center: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let center_len = center.chars().count();
    let right_len = right.chars().count();

    let center_start = (width / 2).saturating_sub(center_len / 2);
    let left_padding = center_start.saturating_sub(left_len);
    let right_start = center_start + center_len;
    let right_padding = width.saturating_sub(right_start).saturating_sub(right_len);

    format!(
        "{}{}{}{}{}",
        left,
        " ".repeat(left_padding),
        center,
        " ".repeat(right_padding),
        right
    )
}

/// Format a mode indicator (" NORMAL", " INSERT", or ":<command>").
pub fn format_mode_indicator(mode: &str, command_buffer: Option<&str>) -> String {
    if let Some(cmd) = command_buffer {
        format!(":{}", cmd)
    } else {
        format!(" {}", mode.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_centered_rect_80_percent() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(80, 80, area);
        assert_eq!(centered.width, 80);
        assert_eq!(centered.height, 80);
        assert_eq!(centered.x, 10);
        assert_eq!(centered.y, 10);
    }

    #[test]
    fn test_large_modal_rect() {
        let area = Rect::new(0, 0, 200, 100);
        let modal = large_modal_rect(area);
        assert_eq!(modal.width, 160);
        assert_eq!(modal.height, 80);
    }

    #[test]
    fn test_small_modal_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let modal = small_modal_rect(area);
        assert_eq!(modal.width, 40);
        assert_eq!(modal.height, 20);
    }

    #[test]
    fn test_split_with_status_bar() {
        let area = Rect::new(0, 0, 80, 20);
        let (content, status) = split_with_status_bar(area);
        assert_eq!(status.height, 1);
        assert_eq!(content.height, 19);
    }

    #[test]
    fn test_build_status_line() {
        let status = build_status_line(" NORMAL", "? for help", 40);
        assert_eq!(status.len(), 40);
        assert!(status.starts_with(" NORMAL"));
        assert!(status.ends_with("? for help"));
    }

    #[test]
    fn test_build_three_part_status_line() {
        let status = build_three_part_status_line(" NORMAL", "file.csv", "A5", 50);
        assert_eq!(status.len(), 50);
        assert!(status.starts_with(" NORMAL"));
        assert!(status.contains("file.csv"));
        assert!(status.ends_with("A5"));
    }

    #[test]
    fn test_format_mode_indicator() {
        assert_eq!(format_mode_indicator("Normal", None), " NORMAL");
        assert_eq!(format_mode_indicator("Command", Some("wq")), ":wq");
        assert_eq!(format_mode_indicator("Command", Some("")), ":");
    }

    #[test]
    fn test_default_theme_styles() {
        let theme = Theme::default();
        assert_eq!(cursor_style(&theme).bg, Some(Color::White));
        assert_eq!(cursor_style(&theme).fg, Some(Color::Black));
        assert_eq!(mode_indicator_style(&theme).bg, Some(Color::Green));
        assert_eq!(mode_indicator_style(&theme).fg, Some(Color::Black));
        assert_eq!(error_style(&theme).fg, Some(Color::Red));
        assert_eq!(popup_bg_style(&theme).bg, Some(Color::DarkGray));
    }
}
