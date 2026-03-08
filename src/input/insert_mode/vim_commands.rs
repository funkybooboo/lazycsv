//! Vim-style editing commands for Insert Mode
//!
//! Provides power-user editing commands inspired by vim:
//!
//! - **Ctrl+h**: Vim-style backspace (delete previous character)
//! - **Ctrl+w**: Delete word backward (vim word deletion)
//! - **Ctrl+u**: Delete from cursor to start of line
//!
//! ## Ctrl+w Behavior
//!
//! Word deletion follows vim semantics:
//! 1. Delete trailing spaces first
//! 2. Then delete non-space characters (the "word")
//! 3. Stop at next space boundary
//!
//! Example: `"hello world"` with cursor at end:
//! - First Ctrl+w: `"hello "` (deletes "world")
//! - Second Ctrl+w: `"hello"` (deletes trailing space)
//! - Third Ctrl+w: `""` (deletes "hello")
//!
//! ## Ctrl+u Behavior
//!
//! Deletes all content from start to cursor position:
//! - `"hello|world"` → `"world"` (cursor was at |)
//! - Cursor moves to position 0 after deletion
//! - Common pattern: Ctrl+u to clear and retype

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

/// Handle Vim-style commands in Insert mode
pub fn handle_vim_commands(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // Ctrl+h (vim-style backspace)
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                if buffer.cursor > 0 {
                    buffer.cursor -= 1;
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
            }
        }

        // Ctrl+w - delete word backward
        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                // Delete trailing spaces first
                while buffer.cursor > 0
                    && buffer.content.chars().nth(buffer.cursor - 1) == Some(' ')
                {
                    buffer.cursor -= 1;
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
                // Delete word characters
                while buffer.cursor > 0
                    && buffer.content.chars().nth(buffer.cursor - 1) != Some(' ')
                {
                    buffer.cursor -= 1;
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
            }
        }

        // Ctrl+u - delete to start of line
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                // Convert char cursor position to byte position for slicing
                let byte_pos = buffer
                    .content
                    .char_indices()
                    .nth(buffer.cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(buffer.content.len());
                buffer.content = buffer.content[byte_pos..].to_string();
                buffer.cursor = 0;
            }
        }

        _ => {}
    }
}
