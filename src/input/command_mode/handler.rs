//! Command mode input handling
//!
//! This module handles keyboard input when the user is in command mode (after pressing ':' in Normal mode).

use crate::app::{messages, App, Mode};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard input in Command mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Clear transient messages on keypress
    if let Some(ref msg) = app.status_message {
        if msg.should_clear_on_keypress() {
            app.status_message = None;
        }
    }

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.input_state.clear_command_buffer();
            app.status_message = Some(StatusMessage::from(messages::CMD_CANCELLED));
        }

        KeyCode::Enter => {
            let result = super::executor::execute(app)?;
            // Only return to Normal if command didn't change mode
            // (Some commands like :files switch to a different mode)
            if app.mode == Mode::Command {
                app.mode = Mode::Normal;
            }
            app.input_state.clear_command_buffer();
            if !matches!(result, InputResult::Continue) {
                return Ok(result);
            }
        }

        KeyCode::Backspace => {
            app.input_state.pop_command_char();
        }

        KeyCode::Char(c) => {
            app.input_state.push_command_char(c);
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}
