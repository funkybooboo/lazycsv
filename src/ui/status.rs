//! Status bar and file switcher rendering.
//!
//! This module handles rendering the bottom status bar showing current cell
//! position and value, plus the file switcher for multi-file sessions.

use crate::App;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::borrow::Cow;

/// Maximum length for cell value display in status bar
const MAX_STATUS_CELL_LENGTH: usize = 30;

/// Number of characters used for ellipsis truncation
const ELLIPSIS_LENGTH: usize = 3;

/// Render the file switcher showing all open CSV files.
///
/// Displays a list of all CSV files in the current directory with an indicator
/// showing which file is currently active. Only rendered when multiple files exist.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `app` - Application state containing session file list
/// * `area` - The rectangle area to render the switcher within
pub fn render_file_switcher(frame: &mut Frame, app: &App, area: Rect) {
    if app.session.files().is_empty() {
        return;
    }

    let count_info: Cow<'_, str> = if app.session.files().len() > 1 {
        Cow::Owned(format!(
            "Files ({}/{}): ",
            app.session.active_file_index() + 1,
            app.session.files().len()
        ))
    } else {
        Cow::Borrowed("File: ")
    };

    // Build file list with indicator
    let mut file_list = String::new();
    for (idx, path) in app.session.files().iter().enumerate() {
        if idx > 0 {
            file_list.push_str(" | ");
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if idx == app.session.active_file_index() {
            file_list.push_str(&format!("► {}", filename));
        } else {
            file_list.push_str(filename);
        }
    }

    let full_text = format!("{}{}", count_info, file_list);

    // Apply horizontal scrolling if text is too wide
    // Account for borders (2 chars) and padding (2 chars)
    let available_width = area.width.saturating_sub(4) as usize;
    let scroll_offset = app.view_state.file_list_scroll_offset;

    let display_text = if full_text.len() > available_width {
        let start = scroll_offset.min(full_text.len().saturating_sub(available_width));
        let end = (start + available_width).min(full_text.len());
        &full_text[start..end]
    } else {
        &full_text
    };

    let switcher = Paragraph::new(display_text)
        .block(Block::default().borders(Borders::ALL).title(" Files "))
        .style(Style::default());

    frame.render_widget(switcher, area);
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
    let total_rows = app.document.row_count();
    let col_letter = column_to_excel_letter(app.view_state.selected_column.get());
    let col_name = app.document.get_header(app.view_state.selected_column);
    let total_cols = app.document.column_count();

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

    let status_text = match app.mode {
        crate::app::Mode::Command => {
            // Show command input with cursor
            format!(" :{}_", app.input_state.command_buffer)
        }
        crate::app::Mode::Normal => {
            if let Some(ref msg) = app.status_message {
                // Show status message if present
                format!(" {} ", msg.as_str())
            } else {
                // Mode indicator
                let mode_indicator = "-- NORMAL --";

                // Dirty flag
                let dirty_flag = if app.document.is_dirty { " [*]" } else { "" };

                // Build right side: row, col, cell info
                let right_side = format!(
                    "Row {}/{} │ Col {}: {} ({}/{}) │ Cell: {} │ [?] help",
                    selected_row,
                    total_rows,
                    col_letter,
                    col_name,
                    app.view_state.selected_column.to_column_number().get(),
                    total_cols,
                    cell_value
                );

                format!(" {}{} │ {}", mode_indicator, dirty_flag, right_side)
            }
        }
    };

    let status = Paragraph::new(status_text).style(Style::default());

    frame.render_widget(status, area);
}
