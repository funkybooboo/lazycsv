//! Commit and cancel operations for Insert Mode
//!
//! Handles exiting insert mode with or without saving changes:
//!
//! - **Enter**: Save and move down (vertical data entry)
//! - **Shift+Enter**: Save and move up (correction workflow)
//! - **Tab**: Save and move right (horizontal data entry)
//! - **Shift+Tab/BackTab**: Save and move left (backward correction)
//! - **Escape**: Cancel without saving
//!
//! ## Directional Commit Pattern
//!
//! The directional commit pattern minimizes keystrokes for bulk editing:
//! Users can enter a cell, edit it, and immediately move to the next cell
//! in their desired direction without switching modes.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Mode};
use crate::input::handler::{enter_insert_mode, CursorPosition, InitialContent};
use crate::navigation;

/// Handle commit and cancel operations in Insert mode
pub fn handle_commit_cancel(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // Save, move down, and stay in insert mode
        // If on last row, insert a new row below
        (KeyCode::Enter, KeyModifiers::NONE) => {
            let on_last_row = app
                .selected_row()
                .map(|r| r.get() >= app.document.row_count().saturating_sub(1))
                .unwrap_or(false);
            commit_edit(app);
            if on_last_row {
                if let Some(row_idx) = app.selected_row() {
                    let new_row = crate::domain::position::RowIndex::new(row_idx.get() + 1);
                    app.document.insert_row(new_row);
                    app.history
                        .push(crate::history::EditCommand::InsertRow { at: new_row });
                    app.view_state.table_state.select(Some(new_row.get()));
                }
            } else {
                navigation::commands::move_down_by(app, 1);
            }
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Exit: Save and move up
        (KeyCode::Enter, KeyModifiers::SHIFT) => {
            commit_edit(app);
            navigation::commands::move_up_by(app, 1);
        }

        // Save, move right, and stay in insert mode
        // If on last column, insert a new row and move to first column
        (KeyCode::Tab, KeyModifiers::NONE) => {
            let on_last_col = app.view_state.selected_column.get()
                >= app.document.column_count().saturating_sub(1);
            commit_edit(app);
            if on_last_col {
                if let Some(row_idx) = app.selected_row() {
                    let new_row = crate::domain::position::RowIndex::new(row_idx.get() + 1);
                    app.document.insert_row(new_row);
                    app.history
                        .push(crate::history::EditCommand::InsertRow { at: new_row });
                    app.view_state.table_state.select(Some(new_row.get()));
                    app.view_state.selected_column = crate::domain::position::ColIndex::new(0);
                    app.view_state.column_scroll_offset = 0;
                }
            } else {
                navigation::commands::move_right_by(app, 1);
            }
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Save, move left, and stay in insert mode
        (KeyCode::Tab, KeyModifiers::SHIFT) | (KeyCode::BackTab, _) => {
            commit_edit(app);
            navigation::commands::move_left_by(app, 1);
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Save, move up, and stay in insert mode
        (KeyCode::Up, _) => {
            commit_edit(app);
            navigation::commands::move_up_by(app, 1);
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Save, move down, and stay in insert mode
        (KeyCode::Down, _) => {
            commit_edit(app);
            navigation::commands::move_down_by(app, 1);
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Save, move left, and stay in insert mode (Shift+Left)
        (KeyCode::Left, KeyModifiers::SHIFT) => {
            commit_edit(app);
            navigation::commands::move_left_by(app, 1);
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Save, move right, and stay in insert mode (Shift+Right)
        (KeyCode::Right, KeyModifiers::SHIFT) => {
            commit_edit(app);
            navigation::commands::move_right_by(app, 1);
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Exit: Cancel
        (KeyCode::Esc, _) => {
            app.formula_completion = None;
            app.edit_buffer = None;
            app.mode = Mode::Normal;
        }

        _ => {}
    }
}

/// Commit the current edit and return to Normal mode
fn commit_edit(app: &mut App) {
    app.formula_completion = None;
    if let Some(buffer) = app.edit_buffer.take() {
        if let Some(row_idx) = app.selected_row() {
            let col_idx = app.view_state.selected_column;

            // Only mark dirty if content changed
            if buffer.content != buffer.original {
                app.commit_cell_value(row_idx, col_idx, buffer.content);
            }
        }
    }
    app.mode = Mode::Normal;
}
