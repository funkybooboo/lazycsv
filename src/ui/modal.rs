//! Shared modal/overlay UI utilities and constants.
//!
//! This module provides standardized dimensions, layouts, and helper functions
//! for all modal overlays (Help, SQL Editor, Magnifier, File Manager, etc.)
//!
//! # Design Philosophy
//!
//! **Single Source of Truth:** All UI styling comes from this module. Never hardcode
//! colors or styles in UI code - always use the helpers provided here.
//!
//! **Consistency:** All large modals (Help, SQL, Magnifier, File Manager) use 80% × 80% sizing.
//! All small prompts (file operations) use 40% × 20% sizing. All modals use the same
//! border style, cursor style, and status bar patterns.
//!
//! **Accessibility:** Clear visual hierarchy with bold headers, dim previews, and
//! high-contrast cursor/selection (white on black).
//!
//! # Quick Start
//!
//! ```ignore
//! use crate::ui::modal;
//!
//! // Create a large modal (80% × 80%)
//! let area = modal::large_modal_rect(frame.area());
//!
//! // Add border and title
//! let block = modal::standard_block(" My Modal ");
//! let inner = block.inner(area);
//! frame.render_widget(block, area);
//!
//! // Split for content + status bar
//! let (content, status) = modal::split_with_status_bar(inner);
//!
//! // Use centralized styles
//! let header = Span::styled("SECTION", modal::bold_style());
//! let error = Span::styled("Error!", modal::error_style());
//! let cursor = Span::styled(" ", modal::cursor_style());
//! ```
//!
//! # Available Styles
//!
//! - `cursor_style()` - White bg, black fg, bold (for cursor/selection)
//! - `bold_style()` - Bold text (row numbers)
//! - `dim_style()` - Dimmed text (previews, hints)
//! - `error_style()` - Red, bold (error messages)
//! - `success_style()` - Green, bold (success messages)
//! - `visual_selection_style()` - DarkGray bg, yellow fg (visual mode)
//! - `mode_indicator_style()` - Black on green (mode display)
//! - `search_match_style()` - Yellow bg, black fg (search highlights)
//!
//! # Color Constants
//!
//! All colors are defined as constants. Use these instead of hardcoding `Color::` values:
//!
//! - `COLOR_POPUP_BG` - DarkGray (completion menus)
//! - `COLOR_LINE_NUMBER` - DarkGray (line numbers)
//! - `COLOR_VISUAL_BG/FG` - DarkGray/Yellow (visual selection)
//! - `COLOR_ERROR` - Red (errors)
//! - `COLOR_SUCCESS` - Green (success)
//! - `COLOR_MODE_INDICATOR_BG/FG` - Green/Black (mode indicator)
//!
//! # See Also
//!
//! - `docs/ui-guidelines.md` - Complete UI styling guide
//! - `src/ui/status_bar.rs` - Main table status bar
//! - `src/ui/help.rs` - Example of modal usage
//!
//! This enforces visual consistency and makes the codebase easier to maintain.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};

// ============================================================================
// MODAL SIZE CONSTANTS
// ============================================================================

/// Width percentage for large modals (Help, SQL, Magnifier, File Manager)
pub const MODAL_LARGE_WIDTH: u16 = 80;

/// Height percentage for large modals (Help, SQL, Magnifier, File Manager)
pub const MODAL_LARGE_HEIGHT: u16 = 80;

/// Width percentage for small prompts (File operations)
pub const MODAL_SMALL_WIDTH: u16 = 40;

/// Height percentage for small prompts (File operations)
pub const MODAL_SMALL_HEIGHT: u16 = 20;

// ============================================================================
// STYLE CONSTANTS
// ============================================================================

/// Cursor/selection style (white background, black text, bold)
pub fn cursor_style() -> Style {
    Style::default()
        .bg(Color::White)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

/// Dimmed text style (for inactive/preview content)
pub fn dim_style() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Bold text style (for selected items, headers)
pub fn bold_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Error text style (red, bold)
pub fn error_style() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

/// Search match style (yellow background, black text)
pub fn search_match_style() -> Style {
    Style::default().bg(Color::Yellow).fg(Color::Black)
}

/// Visual selection style (dark gray background, yellow text)
pub fn visual_selection_style() -> Style {
    Style::default().bg(COLOR_VISUAL_BG).fg(COLOR_VISUAL_FG)
}

/// Row number style (bold text for row numbers in table)
pub fn row_number_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Zebra stripe background for alternating rows (subtle dark tint)
pub fn zebra_stripe_style() -> Style {
    Style::default().bg(Color::Rgb(30, 30, 30))
}

/// Success message style (green, bold)
pub fn success_style() -> Style {
    Style::default()
        .fg(COLOR_SUCCESS)
        .add_modifier(Modifier::BOLD)
}

/// Mode indicator style (black text on green background)
pub fn mode_indicator_style() -> Style {
    Style::default()
        .fg(COLOR_MODE_INDICATOR_FG)
        .bg(COLOR_MODE_INDICATOR_BG)
}

/// Completion menu selected item style (white text on blue background)
pub fn completion_selected_style() -> Style {
    Style::default().fg(Color::White).bg(Color::Blue)
}

/// Completion menu unselected item style (white text on dark gray background)
pub fn completion_unselected_style() -> Style {
    Style::default().fg(Color::White).bg(COLOR_POPUP_BG)
}

// ============================================================================
// COLOR CONSTANTS
// ============================================================================

/// Popup background color (for completion menus, etc.)
pub const COLOR_POPUP_BG: Color = Color::DarkGray;

/// Line number color
pub const COLOR_LINE_NUMBER: Color = Color::DarkGray;

/// Visual selection background color
pub const COLOR_VISUAL_BG: Color = Color::DarkGray;

/// Visual selection foreground color
pub const COLOR_VISUAL_FG: Color = Color::Yellow;

/// Error text color
pub const COLOR_ERROR: Color = Color::Red;

/// Success text color
pub const COLOR_SUCCESS: Color = Color::Green;

/// Mode indicator background color
pub const COLOR_MODE_INDICATOR_BG: Color = Color::Green;

/// Mode indicator foreground color
pub const COLOR_MODE_INDICATOR_FG: Color = Color::Black;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a centered rectangle with specified width/height percentages.
///
/// This is the standard way to create modal overlays in lazycsv.
///
/// # Examples
/// ```ignore
/// // Large modal (80% x 80%)
/// let area = modal::centered_rect(MODAL_LARGE_WIDTH, MODAL_LARGE_HEIGHT, frame.area());
///
/// // Small prompt (40% x 20%)
/// let area = modal::centered_rect(MODAL_SMALL_WIDTH, MODAL_SMALL_HEIGHT, frame.area());
/// ```
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

/// Create a standard large modal (80% x 80%)
///
/// This is the most common modal size used in lazycsv.
pub fn large_modal_rect(r: Rect) -> Rect {
    centered_rect(MODAL_LARGE_WIDTH, MODAL_LARGE_HEIGHT, r)
}

/// Create a standard small prompt (40% x 20%)
///
/// Used for file operation prompts (rename, delete, etc.)
pub fn small_modal_rect(r: Rect) -> Rect {
    centered_rect(MODAL_SMALL_WIDTH, MODAL_SMALL_HEIGHT, r)
}

/// Split a modal's inner area into content + status bar.
///
/// Returns (content_area, status_bar_area) where status bar is 1 line high.
///
/// # Examples
/// ```ignore
/// let (content, status) = modal::split_with_status_bar(inner_area);
/// render_content(frame, content);
/// render_status(frame, status);
/// ```
pub fn split_with_status_bar(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Content area
            Constraint::Length(1), // Status bar (1 line)
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create a standard modal block with title.
///
/// Returns a Block with borders and title already configured.
///
/// # Examples
/// ```ignore
/// let block = modal::standard_block(" SQL Query ");
/// let inner = block.inner(area);
/// frame.render_widget(block, area);
/// ```
pub fn standard_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default())
}

/// Build a two-part status line (left + right with padding).
///
/// Common pattern for status bars with mode on left, help on right.
///
/// # Examples
/// ```
/// use lazycsv::ui::modal::build_status_line;
/// let status = build_status_line(" NORMAL", "? for help", 80);
/// // Result: " NORMAL                                           ? for help"
/// ```
pub fn build_status_line(left: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let padding = width.saturating_sub(left_len).saturating_sub(right_len);

    format!("{}{}{}", left, " ".repeat(padding), right)
}

/// Build a three-part status line (left + center + right).
///
/// Used by main table view for mode + filename + position.
///
/// # Examples
/// ```
/// use lazycsv::ui::modal::build_three_part_status_line;
/// let status = build_three_part_status_line(
///     " NORMAL",
///     "sample.csv",
///     "A5 | Row 5/100",
///     100
/// );
/// ```
pub fn build_three_part_status_line(left: &str, center: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let center_len = center.chars().count();
    let right_len = right.chars().count();

    // Calculate center position
    let center_start = (width / 2).saturating_sub(center_len / 2);

    // Calculate padding
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

/// Format a mode indicator with standard uppercase format.
///
/// Returns mode name in uppercase with leading space (e.g., " NORMAL", " INSERT").
/// For command mode, returns the command buffer with colon prefix.
///
/// # Examples
/// ```
/// use lazycsv::ui::modal::format_mode_indicator;
/// let mode = format_mode_indicator("Normal", None);
/// assert_eq!(mode, " NORMAL");
///
/// let cmd = format_mode_indicator("Command", Some("wq"));
/// assert_eq!(cmd, ":wq");
/// ```
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

    #[test]
    fn test_centered_rect_80_percent() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(80, 80, area);

        assert_eq!(centered.width, 80);
        assert_eq!(centered.height, 80);
        assert_eq!(centered.x, 10); // (100 - 80) / 2
        assert_eq!(centered.y, 10);
    }

    #[test]
    fn test_large_modal_rect() {
        let area = Rect::new(0, 0, 200, 100);
        let modal = large_modal_rect(area);

        assert_eq!(modal.width, 160); // 80% of 200
        assert_eq!(modal.height, 80); // 80% of 100
    }

    #[test]
    fn test_small_modal_rect() {
        let area = Rect::new(0, 0, 100, 100);
        let modal = small_modal_rect(area);

        assert_eq!(modal.width, 40); // 40% of 100
        assert_eq!(modal.height, 20); // 20% of 100
    }

    #[test]
    fn test_split_with_status_bar() {
        let area = Rect::new(0, 0, 80, 20);
        let (content, status) = split_with_status_bar(area);

        assert_eq!(status.height, 1);
        assert_eq!(content.height, 19);
        assert_eq!(content.y, 0);
        assert_eq!(status.y, 19);
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
        assert_eq!(format_mode_indicator("Insert", None), " INSERT");
        assert_eq!(format_mode_indicator("Visual", None), " VISUAL");
        assert_eq!(format_mode_indicator("Command", Some("wq")), ":wq");
        assert_eq!(format_mode_indicator("Command", Some("")), ":");
    }

    #[test]
    fn test_cursor_style_colors() {
        let style = cursor_style();
        assert_eq!(style.bg, Some(Color::White));
        assert_eq!(style.fg, Some(Color::Black));
    }

    #[test]
    fn test_visual_selection_style_colors() {
        let style = visual_selection_style();
        assert_eq!(style.bg, Some(COLOR_VISUAL_BG));
        assert_eq!(style.fg, Some(COLOR_VISUAL_FG));
        assert_eq!(style.bg, Some(Color::DarkGray));
        assert_eq!(style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_mode_indicator_style_colors() {
        let style = mode_indicator_style();
        assert_eq!(style.fg, Some(COLOR_MODE_INDICATOR_FG));
        assert_eq!(style.bg, Some(COLOR_MODE_INDICATOR_BG));
        assert_eq!(style.fg, Some(Color::Black));
        assert_eq!(style.bg, Some(Color::Green));
    }

    #[test]
    fn test_error_style_color() {
        let style = error_style();
        assert_eq!(style.fg, Some(COLOR_ERROR));
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn test_success_style_color() {
        let style = success_style();
        assert_eq!(style.fg, Some(COLOR_SUCCESS));
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn test_all_color_constants_defined() {
        // Ensures we don't accidentally remove constants
        let _ = COLOR_POPUP_BG;
        let _ = COLOR_LINE_NUMBER;
        let _ = COLOR_VISUAL_BG;
        let _ = COLOR_VISUAL_FG;
        let _ = COLOR_ERROR;
        let _ = COLOR_SUCCESS;
        let _ = COLOR_MODE_INDICATOR_BG;
        let _ = COLOR_MODE_INDICATOR_FG;
    }

    #[test]
    fn test_completion_styles() {
        let selected = completion_selected_style();
        assert_eq!(selected.bg, Some(Color::Blue));
        assert_eq!(selected.fg, Some(Color::White));

        let unselected = completion_unselected_style();
        assert_eq!(unselected.bg, Some(COLOR_POPUP_BG));
        assert_eq!(unselected.fg, Some(Color::White));
    }

    #[test]
    fn test_zebra_stripe_style() {
        let style = zebra_stripe_style();
        assert_eq!(style.bg, Some(Color::Rgb(30, 30, 30)));
        // No foreground override — inherits default
        assert_eq!(style.fg, None);
    }
}
