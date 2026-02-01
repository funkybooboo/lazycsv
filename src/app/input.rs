use super::{navigation, App, Mode, ViewportMode};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Timeout for multi-key commands (1 second)
const MULTI_KEY_TIMEOUT_MS: u128 = 1000;

/// Handle keyboard input events
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Returns true if we need to reload a different file
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        // v0.4.0: Mode::Edit => handle_edit_mode(app, key),
    }
}

/// Handle keyboard input in Normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Check if pending key has timed out
    if let Some(time) = app.pending_key_time {
        if time.elapsed().as_millis() > MULTI_KEY_TIMEOUT_MS {
            app.pending_key = None;
            app.pending_key_time = None;
            app.status_message = Some("Command timeout".to_string());
        }
    }

    // Handle pending multi-key sequences
    if let Some(pending) = app.pending_key {
        return handle_multi_key_command(app, pending, key.code);
    }

    // Handle numeric prefixes (5, 10, 25, etc.) - only when help is closed
    // Special case: '0' alone means "go to first column", not start of count
    if !app.ui.show_cheatsheet {
        if let KeyCode::Char(c) = key.code {
            if c.is_numeric() {
                // If '0' is pressed without existing count, treat it as "first column" command
                if c == '0' && app.command_count.is_none() {
                    // Let it fall through to navigation handling
                } else {
                    return handle_count_prefix(app, c);
                }
            }
        }
    }

    match key.code {
        // Quit - vim-style (warns if unsaved in Phase 2)
        KeyCode::Char('q') if !app.ui.show_cheatsheet => {
            if app.csv_data.is_dirty {
                app.status_message = Some("Unsaved changes! Use :q! to force quit".to_string());
            } else {
                app.should_quit = true;
            }
        }

        // Toggle help/cheatsheet
        KeyCode::Char('?') => {
            app.ui.show_cheatsheet = !app.ui.show_cheatsheet;
        }

        // Close help overlay
        KeyCode::Esc if app.ui.show_cheatsheet => {
            app.ui.show_cheatsheet = false;
        }

        // Clear pending command on Esc
        KeyCode::Esc if app.pending_key.is_some() => {
            app.pending_key = None;
            app.pending_key_time = None;
            app.status_message = Some("Command cancelled".to_string());
        }

        // File switching - Previous file
        KeyCode::Char('[') if !app.ui.show_cheatsheet && app.csv_files.len() > 1 => {
            app.current_file_index = if app.current_file_index == 0 {
                app.csv_files.len() - 1
            } else {
                app.current_file_index - 1
            };
            return Ok(true); // Signal to reload file
        }

        // File switching - Next file
        KeyCode::Char(']') if !app.ui.show_cheatsheet && app.csv_files.len() > 1 => {
            app.current_file_index = (app.current_file_index + 1) % app.csv_files.len();
            return Ok(true); // Signal to reload file
        }

        // Start multi-key sequences (only when help is closed)
        KeyCode::Char('g') if !app.ui.show_cheatsheet => {
            app.pending_key = Some(KeyCode::Char('g'));
            app.pending_key_time = Some(std::time::Instant::now());
            return Ok(false);
        }

        KeyCode::Char('z') if !app.ui.show_cheatsheet => {
            app.pending_key = Some(KeyCode::Char('z'));
            app.pending_key_time = Some(std::time::Instant::now());
            return Ok(false);
        }

        // Navigation (only when help is closed)
        _ if !app.ui.show_cheatsheet => navigation::handle_navigation(app, key.code)?,

        _ => {}
    }

    Ok(false)
}

/// Handle multi-key command sequences (gg, zz, zt, zb, etc.)
fn handle_multi_key_command(app: &mut App, first: KeyCode, second: KeyCode) -> Result<bool> {
    app.pending_key = None;
    app.pending_key_time = None;

    match (first, second) {
        // gg - Go to first row
        (KeyCode::Char('g'), KeyCode::Char('g')) => {
            navigation::goto_first_row(app);
            app.status_message = Some("Jumped to first row".to_string());
        }

        // zt - Top of screen
        (KeyCode::Char('z'), KeyCode::Char('t')) => {
            app.ui.viewport_mode = ViewportMode::Top;
            app.status_message = Some("View: top".to_string());
        }

        // zz - Center of screen
        (KeyCode::Char('z'), KeyCode::Char('z')) => {
            app.ui.viewport_mode = ViewportMode::Center;
            app.status_message = Some("View: center".to_string());
        }

        // zb - Bottom of screen
        (KeyCode::Char('z'), KeyCode::Char('b')) => {
            app.ui.viewport_mode = ViewportMode::Bottom;
            app.status_message = Some("View: bottom".to_string());
        }

        _ => {
            app.status_message = Some(format!("Unknown command: {:?} {:?}", first, second));
        }
    }

    Ok(false)
}

/// Handle count prefix (numeric digits for commands like 5j, 10G)
fn handle_count_prefix(app: &mut App, digit: char) -> Result<bool> {
    match &mut app.command_count {
        None => {
            app.command_count = Some(digit.to_string());
        }
        Some(count) => {
            // Limit count string to 5 digits to prevent overflow
            if count.len() < 5 {
                count.push(digit);
            }
        }
    }
    Ok(false)
}
