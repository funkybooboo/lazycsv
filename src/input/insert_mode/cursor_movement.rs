//! Cursor movement operations for Insert Mode
//!
//! Handles cursor navigation within the edit buffer:
//!
//! - **Left/Right arrows**: Move cursor one character at a time
//! - **Home**: Jump to start of content
//! - **End**: Jump to end of content
//!
//! ## Boundary Behavior
//!
//! Cursor movement uses saturation at boundaries:
//! - Moving left at position 0 stays at 0 (no wrapping, no panic)
//! - Moving right at end stays at end (no wrapping, no panic)
//!
//! This provides intuitive UX and prevents crashes from repeated navigation.

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

/// Handle cursor movement operations in Insert mode
pub fn handle_cursor_movement(app: &mut App, key: KeyEvent) {
    match key.code {
        // Cursor movement: Left
        KeyCode::Left => {
            if let Some(ref mut buffer) = app.edit_buffer {
                buffer.cursor = buffer.cursor.saturating_sub(1);
            }
        }

        // Cursor movement: Right
        KeyCode::Right => {
            if let Some(ref mut buffer) = app.edit_buffer {
                let char_count = buffer.content.chars().count();
                buffer.cursor = (buffer.cursor + 1).min(char_count);
            }
        }

        // Cursor movement: Home
        KeyCode::Home => {
            if let Some(ref mut buffer) = app.edit_buffer {
                buffer.cursor = 0;
            }
        }

        // Cursor movement: End
        KeyCode::End => {
            if let Some(ref mut buffer) = app.edit_buffer {
                buffer.cursor = buffer.content.chars().count();
            }
        }

        _ => {}
    }
}
