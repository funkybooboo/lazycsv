//! Input handling and keyboard event processing

use crate::app::{messages, App, Mode};
use crate::navigation;
use crate::ui::ViewportMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::num::NonZeroUsize;

use super::{InputResult, PendingCommand, StatusMessage};

/// Timeout for multi-key commands (1 second)
pub const MULTI_KEY_TIMEOUT_MS: u128 = 1000;

/// Maximum command count to prevent overflow
pub const MAX_COMMAND_COUNT: usize = 100000;

/// Handle keyboard input events
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
    }
}

/// Check if pending command has timed out and clear if needed
fn check_pending_timeout(app: &mut App) {
    if let Some(time) = app.input_state.pending_command_time {
        if time.elapsed().as_millis() > MULTI_KEY_TIMEOUT_MS {
            app.input_state.pending_command = None;
            app.input_state.pending_command_time = None;
            app.status_message = Some(StatusMessage::from(messages::CMD_TIMEOUT));
        }
    }
}

/// Returns true if navigation commands are allowed (help overlay is closed)
fn is_navigation_allowed(app: &App) -> bool {
    !app.view_state.help_overlay_visible
}

/// Handle quit command with unsaved changes check
fn handle_quit(app: &mut App) {
    if app.document.is_dirty {
        app.status_message = Some(StatusMessage::from(messages::UNSAVED_CHANGES));
    } else {
        app.should_quit = true;
    }
}

/// Toggle help overlay visibility
fn handle_help_toggle(app: &mut App) {
    app.view_state.help_overlay_visible = !app.view_state.help_overlay_visible;
}

/// Handle file switching between next and previous files
fn handle_file_switch(app: &mut App, next: bool) -> InputResult {
    if !app.session.has_multiple_files() {
        return InputResult::Continue;
    }

    let switched = if next {
        app.session.next_file()
    } else {
        app.session.prev_file()
    };

    if switched {
        InputResult::ReloadFile
    } else {
        InputResult::Continue
    }
}

/// Handle keyboard input in Normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Check if pending command has timed out
    check_pending_timeout(app);

    // Handle pending multi-key sequences
    if let Some(pending) = app.input_state.pending_command {
        return handle_multi_key_command(app, pending, key.code);
    }

    // Handle numeric prefixes only when navigation is allowed
    if is_navigation_allowed(app) {
        if let KeyCode::Char(c) = key.code {
            if c.is_numeric() && (c != '0' || app.input_state.command_count.is_some()) {
                return handle_count_prefix(app, c);
            }
        }
    }

    match key.code {
        // Quit command
        KeyCode::Char('q') if is_navigation_allowed(app) => {
            handle_quit(app);
        }

        // Toggle help overlay
        KeyCode::Char('?') => {
            handle_help_toggle(app);
        }

        // Close help overlay with Esc
        KeyCode::Esc if app.view_state.help_overlay_visible => {
            app.view_state.help_overlay_visible = false;
        }

        // Clear pending command with Esc
        KeyCode::Esc if app.input_state.pending_command.is_some() => {
            app.input_state.clear_pending_command();
            app.status_message = Some(StatusMessage::from(messages::CMD_CANCELLED));
        }

        // File switching
        KeyCode::Char('[') if is_navigation_allowed(app) => {
            return Ok(handle_file_switch(app, false));
        }

        KeyCode::Char(']') if is_navigation_allowed(app) => {
            return Ok(handle_file_switch(app, true));
        }

        // Start multi-key sequences
        KeyCode::Char('g') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::G);
            return Ok(InputResult::Continue);
        }

        KeyCode::Char('z') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Z);
            return Ok(InputResult::Continue);
        }

        // Navigation commands
        _ if is_navigation_allowed(app) => {
            navigation::handle_navigation(app, key.code)?;
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Handle multi-key command sequences (gg, zz, zt, zb, etc.)
fn handle_multi_key_command(
    app: &mut App,
    first: PendingCommand,
    second: KeyCode,
) -> Result<InputResult> {
    app.input_state.pending_command = None;
    app.input_state.pending_command_time = None;

    match (first, second) {
        // gg - Go to first row
        (PendingCommand::G, KeyCode::Char('g')) => {
            navigation::goto_first_row(app);
            app.status_message = Some(StatusMessage::from(messages::JUMPED_TO_FIRST_ROW));
        }

        // zt - Top of screen
        (PendingCommand::Z, KeyCode::Char('t')) => {
            app.view_state.viewport_mode = ViewportMode::Top;
            app.status_message = Some(StatusMessage::from(messages::VIEW_TOP));
        }

        // zz - Center of screen
        (PendingCommand::Z, KeyCode::Char('z')) => {
            app.view_state.viewport_mode = ViewportMode::Center;
            app.status_message = Some(StatusMessage::from(messages::VIEW_CENTER));
        }

        // zb - Bottom of screen
        (PendingCommand::Z, KeyCode::Char('b')) => {
            app.view_state.viewport_mode = ViewportMode::Bottom;
            app.status_message = Some(StatusMessage::from(messages::VIEW_BOTTOM));
        }

        _ => {
            app.status_message = Some(StatusMessage::from(messages::unknown_command(
                &format!("{:?}", first),
                &format!("{:?}", second),
            )));
        }
    }

    Ok(InputResult::Continue)
}

/// Handle count prefix (numeric digits for commands like 5j, 10G)
fn handle_count_prefix(app: &mut App, digit: char) -> Result<InputResult> {
    let digit_value = digit.to_digit(10).unwrap() as usize;

    app.input_state.command_count = match app.input_state.command_count.take() {
        None => NonZeroUsize::new(digit_value),
        Some(existing) => {
            let new_value = existing.get() * 10 + digit_value;
            // Limit to reasonable size to prevent overflow
            if new_value < MAX_COMMAND_COUNT {
                NonZeroUsize::new(new_value)
            } else {
                Some(existing)
            }
        }
    };

    Ok(InputResult::Continue)
}
