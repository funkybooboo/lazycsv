//! Vim operator handling for magnifier mode
//!
//! Handles operators like x, dd, yy, p, J, u, etc.

use crate::magnifier::MagnifierState;
use crossterm::event::KeyCode;

/// Handle operator commands (x, p, P, J, u)
pub fn handle_operator_command(mag: &mut MagnifierState, key: KeyCode) -> bool {
    match key {
        // Delete character
        KeyCode::Char('x') => {
            mag.push_undo();
            mag.delete_char();
            true
        }

        // Paste
        KeyCode::Char('p') => {
            mag.paste_below();
            true
        }
        KeyCode::Char('P') => {
            mag.paste_above();
            true
        }

        // Join lines
        KeyCode::Char('J') => {
            mag.join_lines();
            true
        }

        // Undo
        KeyCode::Char('u') => {
            mag.undo();
            true
        }

        _ => false,
    }
}
