//! Status bar and file switcher rendering.
//!
//! This module handles rendering the bottom status bar showing current cell
//! position and value, plus the file switcher for multi-file sessions.

use crate::App;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::borrow::Cow;

/// Maximum length for cell value display in status bar
const MAX_STATUS_CELL_LENGTH: usize = 30;

/// Number of characters used for ellipsis truncation
const ELLIPSIS_LENGTH: usize = 3;

/// Build a status line with left and right content, padding between them
fn build_status_line(left: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let total = left_len + right_len + 2; // +2 for spacing

    if total >= width {
        // If too long, truncate left side
        let available = width.saturating_sub(right_len + 2);
        let truncated_left: String = left.chars().take(available).collect();
        format!(" {} {}", truncated_left, right)
    } else {
        let padding = width - total;
        format!(" {}{}{}", left, " ".repeat(padding), right)
    }
}

/// Render the file switcher showing all open CSV files (minimal single-line format).
///
/// Displays a list of all CSV files in the current directory.
/// Format: "file1.csv | file2.csv | file3.csv [1/3]"
/// Active file is shown first/highlighted.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `app` - Application state containing session file list
/// * `area` - The rectangle area to render the switcher within
pub fn render_file_switcher(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};

    if app.session.files().is_empty() {
        return;
    }

    // Split: horizontal rule + file list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Horizontal rule above file list
    let rule = Paragraph::new("─".repeat(area.width as usize));
    frame.render_widget(rule, chunks[0]);

    let dim_style = Style::default().add_modifier(Modifier::DIM);
    let bold_style = Style::default().add_modifier(Modifier::BOLD);
    let available_width = area.width as usize;

    // File count indicator (shown at end)
    let count_indicator = if app.session.files().len() > 1 {
        format!(
            " [{}/{}]",
            app.session.active_file_index() + 1,
            app.session.files().len()
        )
    } else {
        String::new()
    };
    let count_width = count_indicator.len();

    // Calculate position of each file and find current file's position
    let mut file_positions: Vec<(usize, usize)> = Vec::new(); // (start, end) for each file
    let mut pos = 0usize;

    for (idx, path) in app.session.files().iter().enumerate() {
        if idx > 0 {
            pos += 3; // " | "
        }
        let start = pos;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        pos += filename.len();
        file_positions.push((start, pos));
    }

    let total_len = pos;

    // Calculate scroll offset to keep current file visible
    let active_idx = app.session.active_file_index();
    let (active_start, active_end) = file_positions[active_idx];
    let visible_width = available_width.saturating_sub(count_width + 1);

    // Auto-scroll to keep active file visible
    let scroll_offset = if active_end <= visible_width || active_start < visible_width / 2 {
        0 // File fits without scrolling or is near the start
    } else {
        // Scroll to show active file
        active_start.saturating_sub(visible_width / 4)
    };

    // Build visible portion of file list
    let mut spans: Vec<Span> = Vec::new();

    // Add scroll indicator if scrolled
    if scroll_offset > 0 {
        spans.push(Span::styled("< ", dim_style));
    }

    let mut current_pos = 0usize;
    for (idx, path) in app.session.files().iter().enumerate() {
        let separator = if idx > 0 { " | " } else { "" };
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let sep_start = current_pos;
        let sep_end = sep_start + separator.len();
        let file_start = sep_end;
        let file_end = file_start + filename.len();

        // Check if this segment is visible
        if file_end > scroll_offset && sep_start < scroll_offset + visible_width {
            // Add separator if visible
            if !separator.is_empty() && sep_end > scroll_offset {
                spans.push(Span::styled(separator.to_string(), dim_style));
            }

            // Add filename if visible
            if file_end > scroll_offset {
                let style = if idx == active_idx {
                    bold_style
                } else {
                    dim_style
                };
                spans.push(Span::styled(filename.to_string(), style));
            }
        }

        current_pos = file_end;
    }

    // Add scroll indicator if there's more content
    if total_len > scroll_offset + visible_width {
        spans.push(Span::styled(" >", dim_style));
    }

    // Calculate current display length
    let display_len: usize = spans.iter().map(|s| s.content.len()).sum();

    // Add padding to push count indicator to the right
    let padding_needed = available_width.saturating_sub(display_len + count_width);
    if padding_needed > 0 {
        spans.push(Span::raw(" ".repeat(padding_needed)));
    }

    // Add count indicator
    spans.push(Span::styled(count_indicator, dim_style));

    let line = Line::from(spans);
    let switcher = Paragraph::new(line);
    frame.render_widget(switcher, chunks[1]);
}

/// Render the main status bar showing position and cell information.
///
/// Displays current row/column position, column name, total rows/columns,
/// current cell value (truncated if too long), and help/quit keybinding hints.
/// Also shows any pending status messages.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `app` - Application state containing cursor position and document data
/// * `area` - The rectangle area to render the status bar within
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    use crate::ui::utils::column_to_excel_letter;

    let selected_row = app
        .get_selected_row()
        .map(|r| r.to_line_number().get())
        .unwrap_or(0);
    let col_letter = column_to_excel_letter(app.view_state.selected_column.get());
    let col_name = app.document.get_header(app.view_state.selected_column);

    // Get current cell value
    let cell_value: Cow<'_, str> = if let Some(row_idx) = app.get_selected_row() {
        let value = app
            .document
            .get_cell(row_idx, app.view_state.selected_column);
        if value.is_empty() {
            Cow::Borrowed("<empty>")
        } else if value.len() > MAX_STATUS_CELL_LENGTH {
            let truncate_at = MAX_STATUS_CELL_LENGTH - ELLIPSIS_LENGTH;
            Cow::Owned(format!("\"{}...\"", &value[..truncate_at]))
        } else {
            Cow::Owned(format!("\"{}\"", value))
        }
    } else {
        Cow::Borrowed("<no data>")
    };

    // Vim-like status line format:
    // Left side: mode/notification/command
    // Right side: position and cell preview
    //
    // Examples:
    //   NORMAL                                                    3,C "Mike Johnson"
    //   :sort                                                     3,C "Mike Johnson"
    //   Jumped to column B                                        3,C "Mike Johnson"
    //   g_                                                        3,C "Mike Johnson"

    // Build right side: row,col cell_value (vim-like compact format)
    let right_side = format!("{},{} {}", selected_row, col_letter, cell_value);

    // Build pending/count indicator
    let pending_indicator = match &app.input_state.pending_command {
        Some(crate::input::PendingCommand::G) => "g".to_string(),
        Some(crate::input::PendingCommand::Z) => "z".to_string(),
        Some(crate::input::PendingCommand::GotoColumn(letters)) => format!("g{}", letters),
        Some(crate::input::PendingCommand::D) => "d".to_string(),
        Some(crate::input::PendingCommand::Y) => "y".to_string(),
        None => {
            if let Some(count) = app.input_state.command_count {
                format!("{}", count)
            } else {
                String::new()
            }
        }
    };

    let status_text = match app.mode {
        crate::app::Mode::Command => {
            // Show command input: ":sort_" on left, position on right
            let left = format!(":{}", app.input_state.command_buffer);
            build_status_line(&left, &right_side, area.width as usize)
        }
        crate::app::Mode::Normal => {
            // Show notification or mode indicator
            let left = if let Some(ref msg) = app.status_message {
                msg.as_str().to_string()
            } else if !pending_indicator.is_empty() {
                pending_indicator.clone()
            } else {
                let dirty = if app.document.is_dirty { "*" } else { "" };
                format!("NORMAL{}", dirty)
            };
            build_status_line(&left, &right_side, area.width as usize)
        }
        crate::app::Mode::Insert => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            build_status_line(
                &format!("INSERT{}", dirty),
                &right_side,
                area.width as usize,
            )
        }
        crate::app::Mode::Magnifier => {
            build_status_line("MAGNIFIER", &right_side, area.width as usize)
        }
        crate::app::Mode::HeaderEdit => {
            let left = format!("HEADER EDIT: {}", col_name);
            build_status_line(&left, &right_side, area.width as usize)
        }
        crate::app::Mode::Visual => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            build_status_line(
                &format!("VISUAL{}", dirty),
                &right_side,
                area.width as usize,
            )
        }
    };

    let status = Paragraph::new(status_text).style(Style::default());

    frame.render_widget(status, area);
}
