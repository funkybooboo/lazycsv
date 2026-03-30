//! Visual mode input handling
//!
//! This module handles keyboard input for visual modes (Block, Line, Column).

use crate::app::{App, Mode, VisualMode};
use crate::clipboard::copy_text_to_system_clipboard;
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::visual_mode::{handle_visual_delete, handle_visual_paste, handle_visual_yank};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard input in Visual mode (Block, Line, Column)
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Get current visual selection or initialize if missing
    if app.visual_selection.is_none() {
        // Should not happen, but handle gracefully
        app.mode = Mode::Normal;
        return Ok(InputResult::Continue);
    }

    match key.code {
        // Exit visual mode
        KeyCode::Esc => {
            // Save selection for gv
            app.last_visual_selection = app.visual_selection.take();
            app.mode = Mode::Normal;
        }

        // Movement keys - extend selection
        KeyCode::Char('h') | KeyCode::Left => {
            let current_col = app.view_state.selected_column;
            let current_row = app.selected_row();
            if let Some(sel) = &mut app.visual_selection {
                if current_col.get() > 0 {
                    let new_col = ColIndex::new(current_col.get() - 1);
                    app.view_state.selected_column = new_col;
                    if let Some(row) = current_row {
                        sel.update_cursor(row, new_col);
                    }
                }
            }
        }

        KeyCode::Char('j') | KeyCode::Down => {
            let current_row = app.selected_row();
            let row_count = app.document.row_count();
            let selected_col = app.view_state.selected_column;
            if let Some(sel) = &mut app.visual_selection {
                if let Some(current_row) = current_row {
                    if current_row.get() + 1 < row_count {
                        let new_row = RowIndex::new(current_row.get() + 1);
                        app.view_state.table_state.select(Some(new_row.get()));
                        sel.update_cursor(new_row, selected_col);
                    }
                }
            }
        }

        KeyCode::Char('k') | KeyCode::Up => {
            let current_row = app.selected_row();
            let selected_col = app.view_state.selected_column;
            if let Some(sel) = &mut app.visual_selection {
                if let Some(current_row) = current_row {
                    if current_row.get() > 0 {
                        let new_row = RowIndex::new(current_row.get() - 1);
                        app.view_state.table_state.select(Some(new_row.get()));
                        sel.update_cursor(new_row, selected_col);
                    }
                }
            }
        }

        KeyCode::Char('l') | KeyCode::Right => {
            let current_col = app.view_state.selected_column;
            let col_count = app.document.column_count();
            let current_row = app.selected_row();
            if let Some(sel) = &mut app.visual_selection {
                if current_col.get() + 1 < col_count {
                    let new_col = ColIndex::new(current_col.get() + 1);
                    app.view_state.selected_column = new_col;
                    if let Some(row) = current_row {
                        sel.update_cursor(row, new_col);
                    }
                }
            }
        }

        // Delete operation
        KeyCode::Char('d') => {
            handle_visual_delete(app)?;
        }

        // Yank operation (internal clipboard)
        KeyCode::Char('y') => {
            handle_visual_yank(app)?;
        }

        // Yank to system clipboard as CSV
        KeyCode::Char('Y') => {
            yank_to_system_clipboard(app);
        }

        // Paste operation
        KeyCode::Char('p') => {
            handle_visual_paste(app)?;
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Yank the visual selection to the system clipboard as CSV text.
fn yank_to_system_clipboard(app: &mut App) {
    let selection = match app.visual_selection {
        Some(sel) => sel,
        None => return,
    };

    let (start_row, end_row, start_col, end_col) = selection.bounds();

    // Determine row/col ranges based on visual mode
    let (r_start, r_end, c_start, c_end) = match selection.mode {
        VisualMode::Block => (
            start_row.get(),
            end_row.get(),
            start_col.get(),
            end_col.get(),
        ),
        VisualMode::Line => (
            start_row.get(),
            end_row.get(),
            0,
            app.document.column_count().saturating_sub(1),
        ),
        VisualMode::Column => (
            1, // skip header row
            app.document.row_count().saturating_sub(1),
            start_col.get(),
            end_col.get(),
        ),
    };

    // Build CSV text from selected cells
    let mut csv_lines = Vec::new();
    for row_idx in r_start..=r_end {
        let mut cells = Vec::new();
        for col_idx in c_start..=c_end {
            let value = app
                .document
                .cell(RowIndex::new(row_idx), ColIndex::new(col_idx));
            // Quote values containing commas, quotes, or newlines
            if value.contains(',') || value.contains('"') || value.contains('\n') {
                cells.push(format!("\"{}\"", value.replace('"', "\"\"")));
            } else {
                cells.push(value);
            }
        }
        csv_lines.push(cells.join(","));
    }
    let csv_text = csv_lines.join("\n");

    let rows = r_end - r_start + 1;
    let cols = c_end - c_start + 1;

    match copy_text_to_system_clipboard(&csv_text) {
        Ok(()) => {
            app.status_message = Some(StatusMessage::from(format!(
                "Copied {}x{} cells to system clipboard",
                rows, cols
            )));
        }
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!("Clipboard error: {}", e)));
        }
    }

    // Move cursor to selection start and exit visual mode
    app.view_state.table_state.select(Some(start_row.get()));
    app.view_state.selected_column = start_col;
    app.last_visual_selection = app.visual_selection.take();
    app.mode = Mode::Normal;
}
