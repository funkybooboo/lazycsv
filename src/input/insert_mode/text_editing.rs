//! Text editing operations for Insert Mode
//!
//! Handles basic text editing operations:
//!
//! - **Character input**: Insert typed characters at cursor position
//! - **Backspace**: Delete character before cursor
//! - **Delete**: Delete character at cursor position
//!
//! ## Unicode Support
//!
//! All operations correctly handle multi-byte UTF-8 characters:
//! - Cursor position is tracked in characters (not bytes)
//! - String mutations use byte offsets (converted from char positions)
//! - Single backspace deletes one character regardless of byte size
//!
//! This ensures correct behavior with emoji (🚀 = 1 char, 4 bytes),
//! CJK characters (こんにちは = 5 chars, 15 bytes), and accented
//! characters with combining marks.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

/// Handle text editing operations in Insert mode
pub fn handle_text_editing(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // Text editing: Type character
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                // Convert char cursor position to byte position for insert
                let byte_pos = buffer
                    .content
                    .char_indices()
                    .nth(buffer.cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(buffer.content.len());
                buffer.content.insert(byte_pos, c);
                buffer.cursor += 1;
            }
        }

        // Text editing: Backspace
        (KeyCode::Backspace, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                if buffer.cursor > 0 {
                    buffer.cursor -= 1;
                    // Convert char cursor position to byte position for remove
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

        // Text editing: Delete
        (KeyCode::Delete, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                let char_count = buffer.content.chars().count();
                if buffer.cursor < char_count {
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

        _ => {}
    }
}
