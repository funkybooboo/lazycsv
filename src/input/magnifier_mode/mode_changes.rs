//! Mode transition handling for magnifier mode
//!
//! Handles transitions between Normal, Insert, Visual, and Command modes.

use crate::magnifier::MagnifierState;
use crossterm::event::KeyCode;

/// Handle mode transition commands (i, a, o, v, V, :)
pub fn handle_mode_change_command(mag: &mut MagnifierState, key: KeyCode) -> bool {
    match key {
        // Enter insert mode
        KeyCode::Char('i') => {
            mag.insert_before();
            true
        }
        KeyCode::Char('a') => {
            mag.insert_after();
            true
        }
        KeyCode::Char('A') => {
            mag.move_to_line_end();
            mag.insert_after();
            true
        }
        KeyCode::Char('I') => {
            mag.move_to_first_non_blank();
            mag.insert_before();
            true
        }
        KeyCode::Char('o') => {
            mag.insert_line_below();
            true
        }
        KeyCode::Char('O') => {
            mag.insert_line_above();
            true
        }
        KeyCode::Char('s') => {
            mag.substitute_char();
            true
        }
        KeyCode::Char('C') => {
            mag.change_to_eol();
            true
        }

        // Enter visual mode
        KeyCode::Char('v') => {
            mag.enter_visual_mode();
            true
        }
        KeyCode::Char('V') => {
            mag.enter_visual_line_mode();
            true
        }

        // Enter command mode
        KeyCode::Char(':') => {
            mag.enter_command_mode();
            true
        }

        _ => false,
    }
}
