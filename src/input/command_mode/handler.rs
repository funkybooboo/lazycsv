//! Command mode input handling
//!
//! This module handles keyboard input when the user is in command mode (after pressing ':' in Normal mode).

use crate::app::{messages, App, Mode};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard input in Command mode.
///
/// Tries the keymap pre-pass first; falls through to the legacy line
/// editor for character input and unbound keys.
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    if let Some(ref msg) = app.status_message {
        if msg.should_clear_on_keypress() {
            app.status_message = None;
        }
    }
    if let Some(result) = crate::input::keymap_dispatch::try_keymap(
        app,
        key,
        crate::config::keys::KeymapScope::Command,
        handle_raw,
    )? {
        return Ok(result);
    }
    handle_raw(app, key)
}

/// Legacy match-based command-mode handler.
pub fn handle_raw(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.input_state.clear_command_buffer();
            app.status_message = Some(StatusMessage::from(messages::CMD_CANCELLED));
        }

        KeyCode::Enter => {
            let cmd = app.input_state.command_buffer.trim().to_string();
            let result = super::executor::execute(app)?;
            // Only return to Normal if command didn't change mode
            // (Some commands like :files switch to a different mode)
            if app.mode == Mode::Command {
                app.mode = Mode::Normal;
            }
            app.input_state.clear_command_buffer();
            app.command_history_index = None;
            app.command_history_pending = None;
            app.push_command_history(cmd);
            if !matches!(result, InputResult::Continue) {
                return Ok(result);
            }
        }

        KeyCode::Backspace => {
            app.input_state.pop_command_char();
        }

        KeyCode::Delete => {
            app.input_state.delete_command_char();
        }

        KeyCode::Left => {
            app.input_state.command_cursor_left();
        }

        KeyCode::Right => {
            app.input_state.command_cursor_right();
        }

        KeyCode::Home => {
            app.input_state.command_cursor_home();
        }

        KeyCode::End => {
            app.input_state.command_cursor_end();
        }

        KeyCode::Up => history_prev(app),

        KeyCode::Down => history_next(app),

        KeyCode::Char(c) => {
            // Typing invalidates history navigation — keep current buffer.
            app.command_history_index = None;
            app.command_history_pending = None;
            app.input_state.push_command_char(c);
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Walk older into command history (Up arrow).
/// Index 0 = most recent. None = at the live prompt.
fn history_prev(app: &mut App) {
    if app.command_history.is_empty() {
        return;
    }
    let new_index = match app.command_history_index {
        None => {
            // Save what's currently typed so Down can restore it.
            app.command_history_pending = Some(app.input_state.command_buffer.clone());
            0
        }
        Some(i) if i + 1 < app.command_history.len() => i + 1,
        Some(i) => i, // already at oldest
    };
    let entry = app.command_history[new_index].clone();
    app.command_history_index = Some(new_index);
    set_command_buffer(app, entry);
}

/// Walk newer in command history (Down arrow). Past the newest, restore the live prompt.
fn history_next(app: &mut App) {
    let Some(i) = app.command_history_index else {
        return;
    };
    if i == 0 {
        // Past the most recent — return to whatever the user had typed.
        let pending = app.command_history_pending.take().unwrap_or_default();
        app.command_history_index = None;
        set_command_buffer(app, pending);
    } else {
        let new_index = i - 1;
        let entry = app.command_history[new_index].clone();
        app.command_history_index = Some(new_index);
        set_command_buffer(app, entry);
    }
}

fn set_command_buffer(app: &mut App, value: String) {
    app.input_state.command_buffer = value;
    app.input_state.command_cursor = app.input_state.command_buffer.chars().count();
}
