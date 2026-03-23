//! Mode transition operations for Normal mode

use crate::app::{App, Mode};
use crate::input::handler::{enter_insert_mode, CursorPosition, InitialContent};

/// Enter SQL query editor mode
pub fn enter_sql_editor(app: &mut App) {
    app.sql_cursor = app.sql_buffer.chars().count();
    app.mode = Mode::SqlEditor;
    let mut editor = crate::vim_editor::VimEditor::new(app.sql_buffer.clone());
    editor.set_undo_limit(app.config.defaults.undo_limit);
    // Auto-enter INSERT mode when there's no existing query
    if app.sql_buffer.trim().is_empty() {
        editor.enter_insert_mode();
    }
    app.sql_vim_editor = Some(editor);
}

/// Enter command mode (:)
pub fn enter_command_mode(app: &mut App) {
    app.mode = Mode::Command;
    app.input_state.clear_command_buffer();
}

/// Enter search mode (/)
pub fn enter_search_mode(app: &mut App) {
    app.search_buffer.clear();
    app.mode = Mode::Search;
}

/// Enter insert mode: 'i' - edit cell, cursor at end
pub fn insert_at_end(app: &mut App) {
    enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
}

/// Enter insert mode: 'a' - edit cell, cursor at end (same as 'i' for cells)
pub fn append_at_end(app: &mut App) {
    enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
}

/// Enter insert mode: 'I' - edit cell, cursor at start
pub fn insert_at_start(app: &mut App) {
    enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
}

/// Enter insert mode: 'A' - edit cell, cursor at end (same as 'i')
pub fn append_at_line_end(app: &mut App) {
    enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
}

/// Enter insert mode: 's' - replace cell (clear + edit)
pub fn substitute_cell(app: &mut App) {
    enter_insert_mode(app, CursorPosition::Start, InitialContent::Clear);
}

/// Enter insert mode: F2 - edit cell (same as 'i')
pub fn edit_with_f2(app: &mut App) {
    enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
}

/// Enter magnifier mode: 'm' - open magnifier for complex cell editing
pub fn enter_magnifier(app: &mut App) {
    app.open_magnifier();
}
