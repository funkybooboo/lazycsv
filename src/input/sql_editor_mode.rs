//! SQL Editor mode input handling
//!
//! This module handles keyboard input when the user is editing SQL queries (after pressing 'q' in Normal mode).

use crate::app::{App, Mode};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard input in SQL editor mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Get the vim editor, or return early if not initialized
    let vim_editor = match app.sql_vim_editor.as_mut() {
        Some(editor) => editor,
        None => {
            // Fallback: exit SQL editor mode if vim editor is not initialized
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
    };

    // Special handling for Ctrl+Enter to execute query
    if matches!(key.code, KeyCode::Enter) && key.modifiers.contains(KeyModifiers::CONTROL) {
        return execute_query(app);
    }

    // Special handling for Esc in Normal mode to exit SQL editor
    if matches!(key.code, KeyCode::Esc) && vim_editor.mode() == crate::vim_editor::VimMode::Normal {
        app.mode = Mode::Normal;
        app.sql_vim_editor = None;
        return Ok(InputResult::Continue);
    }

    // Route all other keys to the vim editor
    vim_editor.handle_key(key);

    // Update sql_buffer from vim editor content
    app.sql_buffer = vim_editor.content();

    Ok(InputResult::Continue)
}

/// Execute SQL query from buffer
fn execute_query(app: &mut App) -> Result<InputResult> {
    let query = app.sql_buffer.trim().to_string();
    if query.is_empty() {
        app.status_message = Some(StatusMessage::new_owned("Empty query".to_string()));
        app.mode = Mode::Normal;
        app.sql_vim_editor = None;
        return Ok(InputResult::Continue);
    }
    Ok(InputResult::ExecuteQuery { query })
}
