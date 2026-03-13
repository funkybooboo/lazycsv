//! Input handling and keyboard event processing

use crate::app::{App, EditBuffer, Mode};
use crate::domain::position::RowIndex;
use anyhow::Result;
use crossterm::event::KeyEvent;

use super::InputResult;

/// Timeout for multi-key commands (no longer used in handler, but still exported for state)
pub const MULTI_KEY_TIMEOUT_MS: u128 = 1000;

/// Maximum command count to prevent overflow
pub const MAX_COMMAND_COUNT: usize = 100000;

/// Handle keyboard input events
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match app.mode {
        Mode::Normal => crate::input::normal_mode::handle(app, key),
        Mode::Command => crate::input::command_mode::handle(app, key),
        Mode::Insert => super::insert_mode::handle_insert_mode(app, key),
        Mode::FileList => crate::input::file_list_mode::handle(app, key),
        Mode::SqlEditor => crate::input::sql_editor_mode::handle(app, key),
        Mode::Magnifier => crate::input::magnifier_mode::handle(app, key),
        Mode::Search => crate::input::search_mode::handle(app, key),
        Mode::FileOperationPrompt => crate::input::file_operation_mode::handle(app, key),
        Mode::VisualBlock | Mode::VisualLine | Mode::VisualColumn => {
            super::visual_mode::handle(app, key)
        }
    }
}

/// Handle file switching between next and previous files
pub fn handle_file_switch(app: &mut App, next: bool) -> InputResult {
    if !app.session.has_multiple_files() {
        return InputResult::Continue;
    }

    // Cache current document before switching if it's a query output or dirty
    // (query results don't exist on disk, so they must be cached to switch back)
    let current_path = app.current_file().clone();
    if app.session.is_query_output(&current_path) || app.document.is_dirty {
        app.session
            .cache_document(current_path.clone(), app.document.clone());
        if app.document.is_dirty {
            app.session.mark_dirty(&current_path);
        }
    }

    let switched = if next {
        app.session.next_file()
    } else {
        app.session.prev_file()
    };

    if switched {
        app.search_state = None;
        InputResult::ReloadFile
    } else {
        InputResult::Continue
    }
}

/// Cursor position when entering insert mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorPosition {
    /// Cursor at start of cell content
    Start,
    /// Cursor at end of cell content
    End,
}

/// Initial content when entering insert mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialContent {
    /// Keep existing cell content
    Keep,
    /// Clear cell content (for substitute command)
    Clear,
}

/// Enter Insert mode for cell editing
///
/// # Arguments
/// * `cursor` - Where to place the cursor (start or end)
/// * `content` - Whether to keep or clear existing content
pub fn enter_insert_mode(app: &mut App, cursor: CursorPosition, content: InitialContent) {
    let row_idx = app.selected_row().unwrap_or(RowIndex::new(0));
    let col_idx = app.view_state.selected_column;

    let current_value = app.document.cell(row_idx, col_idx).to_string();

    let (buffer_content, cursor_pos) = match (content, cursor) {
        (InitialContent::Clear, _) => (String::new(), 0),
        (InitialContent::Keep, CursorPosition::Start) => (current_value.clone(), 0),
        (InitialContent::Keep, CursorPosition::End) => {
            // Use character count, not byte length, for cursor position
            let char_count = current_value.chars().count();
            (current_value.clone(), char_count)
        }
    };

    app.edit_buffer = Some(EditBuffer {
        content: buffer_content,
        cursor: cursor_pos,
        original: current_value,
    });
    app.mode = Mode::Insert;
}
