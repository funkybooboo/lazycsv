//! Search operations for Normal mode

use crate::app::App;
use crate::domain::position::RowIndex;
use crate::input::StatusMessage;

/// Jump to next search match (n)
pub fn next_match(app: &mut App) {
    let cursor_row = app.selected_row().unwrap_or(RowIndex::new(0));
    let cursor_col = app.view_state.selected_column;
    if let Some(ref mut state) = app.search_state {
        if let Some(((row, col), wrapped)) = state.jump_to_next(cursor_row, cursor_col) {
            app.view_state.table_state.select(Some(row.get()));
            app.view_state.selected_column = col;
            let pos = state.display_position();
            let pattern = state.pattern.clone();
            if wrapped {
                app.status_message = Some(StatusMessage::new_owned(format!(
                    "search hit BOTTOM, continuing at TOP  /{} {}",
                    pattern, pos
                )));
            } else {
                app.status_message =
                    Some(StatusMessage::new_owned(format!("/{} {}", pattern, pos)));
            }
        }
    }
}

/// Jump to previous search match (N)
pub fn prev_match(app: &mut App) {
    let cursor_row = app.selected_row().unwrap_or(RowIndex::new(0));
    let cursor_col = app.view_state.selected_column;
    if let Some(ref mut state) = app.search_state {
        if let Some(((row, col), wrapped)) = state.jump_to_prev(cursor_row, cursor_col) {
            app.view_state.table_state.select(Some(row.get()));
            app.view_state.selected_column = col;
            let pos = state.display_position();
            let pattern = state.pattern.clone();
            if wrapped {
                app.status_message = Some(StatusMessage::new_owned(format!(
                    "search hit TOP, continuing at BOTTOM  /{} {}",
                    pattern, pos
                )));
            } else {
                app.status_message =
                    Some(StatusMessage::new_owned(format!("/{} {}", pattern, pos)));
            }
        }
    }
}

/// Search for current cell content (vim *)
pub fn search_current_cell(app: &mut App) {
    let cursor_row = app.selected_row().unwrap_or(RowIndex::new(0));
    let cursor_col = app.view_state.selected_column;
    let cell_content = app.document.cell(cursor_row, cursor_col).to_string();

    if !cell_content.is_empty() {
        let matches = crate::search::find_matches(&app.document, &cell_content);
        if !matches.is_empty() {
            let mut state = crate::search::SearchState::new(cell_content.clone(), matches);
            // Jump to next match (skips the current cell)
            if let Some(((row, col), _wrapped)) = state.jump_to_next(cursor_row, cursor_col) {
                app.view_state.table_state.select(Some(row.get()));
                app.view_state.selected_column = col;
                app.status_message = Some(StatusMessage::new_owned(format!(
                    "/{} {}",
                    cell_content,
                    state.display_position()
                )));
            }
            app.search_state = Some(state);
        } else {
            app.status_message = Some(StatusMessage::new_owned(format!(
                "Pattern not found: {}",
                cell_content
            )));
        }
    }
}

/// Clear search highlighting (Esc)
pub fn clear_search(app: &mut App) {
    app.search_state = None;
}
