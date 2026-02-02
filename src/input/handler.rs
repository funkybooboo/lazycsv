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

/// Handle keyboard input events
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        // v0.4.0: Mode::Edit => handle_edit_mode(app, key),
    }
}

/// Handle keyboard input in Normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Check if pending key has timed out
    if let Some(time) = app.input_state.pending_command_time {
        if time.elapsed().as_millis() > MULTI_KEY_TIMEOUT_MS {
            app.input_state.pending_command = None;
            app.input_state.pending_command_time = None;
            app.status_message = Some(StatusMessage::from(messages::CMD_TIMEOUT));
        }
    }

    // Handle pending multi-key sequences
    if let Some(pending) = app.input_state.pending_command {
        return handle_multi_key_command(app, pending, key.code);
    }

    // Handle numeric prefixes (5, 10, 25, etc.) - only when help is closed
    // Special case: '0' alone means "go to first column", not start of count
    if !app.view_state.help_overlay_visible {
        if let KeyCode::Char(c) = key.code {
            if c.is_numeric() {
                // If '0' is pressed without existing count, treat it as "first column" command
                if c == '0' && app.input_state.command_count.is_none() {
                    // Let it fall through to navigation handling
                } else {
                    return handle_count_prefix(app, c);
                }
            }
        }
    }

    match key.code {
        // Quit - vim-style (warns if unsaved in Phase 2)
        KeyCode::Char('q') if !app.view_state.help_overlay_visible => {
            if app.document.is_dirty {
                app.status_message = Some(StatusMessage::from(messages::UNSAVED_CHANGES));
            } else {
                app.should_quit = true;
            }
        }

        // Toggle help overlay
        KeyCode::Char('?') => {
            app.view_state.help_overlay_visible = !app.view_state.help_overlay_visible;
        }

        // Close help overlay
        KeyCode::Esc if app.view_state.help_overlay_visible => {
            app.view_state.help_overlay_visible = false;
        }

        // Clear pending command on Esc
        KeyCode::Esc if app.input_state.pending_command.is_some() => {
            app.input_state.pending_command = None;
            app.input_state.pending_command_time = None;
            app.status_message = Some(StatusMessage::from(messages::CMD_CANCELLED));
        }

        // File switching - Previous file
        KeyCode::Char('[')
            if !app.view_state.help_overlay_visible && app.session.has_multiple_files() =>
        {
            if app.session.prev_file() {
                return Ok(InputResult::ReloadFile);
            }
        }

        // File switching - Next file
        KeyCode::Char(']')
            if !app.view_state.help_overlay_visible && app.session.has_multiple_files() =>
        {
            if app.session.next_file() {
                return Ok(InputResult::ReloadFile);
            }
        }

        // Start multi-key sequences (only when help is closed)
        KeyCode::Char('g') if !app.view_state.help_overlay_visible => {
            app.input_state.pending_command = Some(PendingCommand::G);
            app.input_state.pending_command_time = Some(std::time::Instant::now());
            return Ok(InputResult::Continue);
        }

        KeyCode::Char('z') if !app.view_state.help_overlay_visible => {
            app.input_state.pending_command = Some(PendingCommand::Z);
            app.input_state.pending_command_time = Some(std::time::Instant::now());
            return Ok(InputResult::Continue);
        }

        // Navigation (only when help is closed)
        _ if !app.view_state.help_overlay_visible => navigation::handle_navigation(app, key.code)?,

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
            if new_value < 100000 {
                NonZeroUsize::new(new_value)
            } else {
                Some(existing)
            }
        }
    };

    Ok(InputResult::Continue)
}
