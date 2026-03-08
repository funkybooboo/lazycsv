//! Vim motion handling for magnifier mode
//!
//! Handles all cursor movement commands: hjkl, w/b/e, 0/$, gg/G, etc.

use crate::magnifier::MagnifierState;
use crossterm::event::KeyCode;

/// Handle basic motion commands (hjkl, arrows, 0/$, w/b/e, gg/G)
pub fn handle_motion_command(mag: &mut MagnifierState, key: KeyCode) -> bool {
    match key {
        // Basic motions
        KeyCode::Char('h') | KeyCode::Left => {
            mag.move_left();
            true
        }
        KeyCode::Char('j') | KeyCode::Down => {
            mag.move_down();
            true
        }
        KeyCode::Char('k') | KeyCode::Up => {
            mag.move_up();
            true
        }
        KeyCode::Char('l') | KeyCode::Right => {
            mag.move_right();
            true
        }

        // Line motions
        KeyCode::Char('0') => {
            mag.move_to_line_start();
            true
        }
        KeyCode::Char('$') => {
            mag.move_to_line_end();
            true
        }
        KeyCode::Char('^') => {
            mag.move_to_first_non_blank();
            true
        }

        // Word motions
        KeyCode::Char('w') => {
            mag.move_next_word();
            true
        }
        KeyCode::Char('b') => {
            mag.move_prev_word();
            true
        }
        KeyCode::Char('e') => {
            mag.move_end_word();
            true
        }

        // Buffer motions
        KeyCode::Char('G') => {
            mag.move_to_last_line();
            true
        }

        // Repeat find
        KeyCode::Char(';') => {
            mag.repeat_find();
            true
        }
        KeyCode::Char(',') => {
            mag.repeat_find_reverse();
            true
        }

        _ => false,
    }
}
