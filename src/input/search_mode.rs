//! Search mode input handling
//!
//! This module handles keyboard input when the user is typing a search pattern (after pressing '/' in Normal mode).

use crate::app::{App, Mode};
use crate::domain::position::RowIndex;
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard input in Search mode (with keymap pre-pass).
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    if let Some(result) = crate::input::keymap_dispatch::try_keymap(
        app,
        key,
        crate::config::keys::KeymapScope::Search,
        handle_raw,
    )? {
        return Ok(result);
    }
    handle_raw(app, key)
}

/// Legacy match-based search-mode handler.
pub fn handle_raw(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => {
            execute_search(app);
        }
        KeyCode::Backspace => {
            if app.search_buffer.is_empty() {
                app.mode = Mode::Normal;
            } else {
                app.search_buffer.pop();
            }
        }
        KeyCode::Char(c) => {
            app.search_buffer.push(c);
        }
        _ => {}
    }
    Ok(InputResult::Continue)
}

/// Execute search and jump to first match
fn execute_search(app: &mut App) {
    let buffer = app.search_buffer.clone();
    app.mode = Mode::Normal;

    if buffer.is_empty() {
        return;
    }

    let matches = crate::search::find_matches(&app.document, &buffer);
    if matches.is_empty() {
        app.search_state = None;
        app.status_message = Some(StatusMessage::new_owned(format!(
            "Pattern not found: {}",
            buffer
        )));
        return;
    }

    let mut state = crate::search::SearchState::new(buffer.clone(), matches);
    let cursor_row = app.selected_row().unwrap_or(RowIndex::new(0));
    let cursor_col = app.view_state.selected_column;

    if let Some(((row, col), _wrapped)) = state.jump_to_next(cursor_row, cursor_col) {
        app.view_state.table_state.select(Some(row.get()));
        app.view_state.selected_column = col;
        app.status_message = Some(StatusMessage::new_owned(format!(
            "/{} {}",
            buffer,
            state.display_position()
        )));
    }

    app.search_state = Some(state);
}
