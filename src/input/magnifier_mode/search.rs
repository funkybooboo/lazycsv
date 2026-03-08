//! Search command handling for magnifier mode
//!
//! Handles search operations: /, n, N, *

use crate::magnifier::MagnifierState;
use crossterm::event::KeyCode;

/// Handle search commands (/, n, N, *)
pub fn handle_search_command(mag: &mut MagnifierState, key: KeyCode) -> bool {
    match key {
        // Start search
        KeyCode::Char('/') => {
            mag.enter_command_mode_with("/");
            true
        }

        // Jump to next match
        KeyCode::Char('n') => {
            mag.jump_to_next_match();
            true
        }

        // Jump to previous match
        KeyCode::Char('N') => {
            mag.jump_to_prev_match();
            true
        }

        // Search word under cursor
        KeyCode::Char('*') => {
            if let Some(word) = mag.get_word_under_cursor() {
                mag.search_forward(word);
            }
            true
        }

        _ => false,
    }
}
