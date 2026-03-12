//! Visual mode input handling
//!
//! This module handles keyboard input for visual modes (Block, Line, Column).

use crate::app::{App, Mode};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::visual_mode::{handle_visual_delete, handle_visual_paste, handle_visual_yank};
use crate::input::InputResult;
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

        // Yank operation
        KeyCode::Char('y') => {
            handle_visual_yank(app)?;
        }

        // Paste operation
        KeyCode::Char('p') => {
            handle_visual_paste(app)?;
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}
