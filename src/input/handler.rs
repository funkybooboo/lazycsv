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
        Mode::Command => handle_command_mode(app, key),
    }
}

/// Check if pending command has timed out and clear if needed
fn check_pending_timeout(app: &mut App) {
    if let Some(time) = app.input_state.pending_command_time {
        if time.elapsed().as_millis() > MULTI_KEY_TIMEOUT_MS {
            // If we have buffered column letters, execute the jump
            if let Some(PendingCommand::GotoColumn(ref letters)) = app.input_state.pending_command {
                let letters = letters.clone();
                app.input_state.clear_pending_command();
                navigation::commands::goto_column(app, &letters);
            } else {
                app.input_state.clear_pending_command();
                app.status_message = Some(StatusMessage::from(messages::CMD_TIMEOUT));
            }
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
    // Clear transient messages on keypress
    if let Some(ref msg) = app.status_message {
        if msg.should_clear_on_keypress() {
            app.status_message = None;
        }
    }

    // Check if pending command has timed out
    check_pending_timeout(app);

    // Handle pending multi-key sequences
    if let Some(pending) = app.input_state.pending_command.clone() {
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

        // Enter command mode
        KeyCode::Char(':') if is_navigation_allowed(app) => {
            app.mode = Mode::Command;
            app.input_state.clear_command_buffer();
            return Ok(InputResult::Continue);
        }

        // Enter key - move down one row (like j)
        KeyCode::Enter if is_navigation_allowed(app) => {
            navigation::commands::move_down_by(app, 1);
        }

        // Navigation commands
        _ if is_navigation_allowed(app) => {
            navigation::handle_navigation(app, key.code)?;
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Handle multi-key command sequences (gg, zz, zt, zb, g<letters>, etc.)
fn handle_multi_key_command(
    app: &mut App,
    first: PendingCommand,
    second: KeyCode,
) -> Result<InputResult> {
    match (&first, second) {
        // gg - Go to first row
        (PendingCommand::G, KeyCode::Char('g')) => {
            app.input_state.clear_pending_command();
            navigation::goto_first_row(app);
            app.status_message = Some(StatusMessage::from(messages::JUMPED_TO_FIRST_ROW));
        }

        // g + letter - Start column jump (e.g., gA, gB)
        (PendingCommand::G, KeyCode::Char(c)) if c.is_ascii_alphabetic() => {
            let new_pending = first.append_letter(c);
            app.input_state.set_pending_command(new_pending);
            return Ok(InputResult::Continue);
        }

        // g + letter + more letters - Continue buffering (e.g., gB -> gBC)
        (PendingCommand::GotoColumn(_), KeyCode::Char(c)) if c.is_ascii_alphabetic() => {
            let new_pending = first.append_letter(c);
            app.input_state.set_pending_command(new_pending);
            return Ok(InputResult::Continue);
        }

        // g + letter(s) + Enter or non-letter - Execute column jump
        (PendingCommand::GotoColumn(_), KeyCode::Enter)
        | (PendingCommand::GotoColumn(_), KeyCode::Char(_)) => {
            app.input_state.clear_pending_command();
            if let Some(letters) = first.get_column_letters() {
                navigation::commands::goto_column(app, letters);
            }
        }

        // zt - Top of screen
        (PendingCommand::Z, KeyCode::Char('t')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Top;
            app.status_message = Some(StatusMessage::from(messages::VIEW_TOP));
        }

        // zz - Center of screen
        (PendingCommand::Z, KeyCode::Char('z')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Center;
            app.status_message = Some(StatusMessage::from(messages::VIEW_CENTER));
        }

        // zb - Bottom of screen
        (PendingCommand::Z, KeyCode::Char('b')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Bottom;
            app.status_message = Some(StatusMessage::from(messages::VIEW_BOTTOM));
        }

        _ => {
            app.input_state.clear_pending_command();
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

/// Handle keyboard input in Command mode
fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
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
            execute_command(app)?;
            app.mode = Mode::Normal;
            app.input_state.clear_command_buffer();
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

/// Execute command from command buffer
fn execute_command(app: &mut App) -> Result<()> {
    let cmd = app.input_state.command_buffer.trim().to_string();

    if cmd.is_empty() {
        return Ok(());
    }

    // Try to parse as number first (line jump)
    if let Ok(line_num) = cmd.parse::<usize>() {
        navigation::commands::goto_line(app, line_num);
        app.status_message = Some(StatusMessage::from(format!("Jumped to line {}", line_num)));
        return Ok(());
    }

    // Try to parse as column letter
    if cmd.chars().all(|c| c.is_ascii_alphabetic()) {
        navigation::commands::goto_column(app, &cmd);
        return Ok(());
    }

    // Unknown command
    app.status_message = Some(StatusMessage::from(format!("Unknown command: :{}", cmd)));
    Ok(())
}
