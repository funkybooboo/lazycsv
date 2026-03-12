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
use crate::navigation;

/// Handle commit and cancel operations in Insert mode
pub fn handle_commit_cancel(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // Exit: Save and move down
        (KeyCode::Enter, KeyModifiers::NONE) => {
            commit_edit(app);
            navigation::commands::move_down_by(app, 1);
        }

        // Exit: Save and move up
        (KeyCode::Enter, KeyModifiers::SHIFT) => {
            commit_edit(app);
            navigation::commands::move_up_by(app, 1);
        }

        // Exit: Save and move right
        (KeyCode::Tab, KeyModifiers::NONE) => {
            commit_edit(app);
            navigation::commands::move_right_by(app, 1);
        }

        // Exit: Save and move left
        (KeyCode::Tab, KeyModifiers::SHIFT) | (KeyCode::BackTab, _) => {
            commit_edit(app);
            navigation::commands::move_left_by(app, 1);
        }

        // Exit: Cancel
        (KeyCode::Esc, _) => {
            app.edit_buffer = None;
            app.mode = Mode::Normal;
        }

        _ => {}
    }
}

/// Commit the current edit and return to Normal mode
fn commit_edit(app: &mut App) {
    if let Some(buffer) = app.edit_buffer.take() {
        if let Some(row_idx) = app.selected_row() {
            let col_idx = app.view_state.selected_column;

            // Only mark dirty if content changed
            if buffer.content != buffer.original {
                app.document.set_cell(row_idx, col_idx, buffer.content);
                app.last_edit_position = Some((row_idx, col_idx));
            }
        }
    }
    app.mode = Mode::Normal;
}
