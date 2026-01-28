use super::{navigation, App, Mode};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard input events
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Returns true if we need to reload a different file
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        // Phase 2: Mode::Edit => handle_edit_mode(app, key),
    }
}

/// Handle keyboard input in Normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Quit - vim-style (warns if unsaved in Phase 2)
        KeyCode::Char('q') if !app.show_cheatsheet => {
            if app.csv_data.is_dirty {
                app.status_message = Some("Unsaved changes! Use :q! to force quit".to_string());
            } else {
                app.should_quit = true;
            }
        }

        // Toggle help/cheatsheet
        KeyCode::Char('?') => {
            app.show_cheatsheet = !app.show_cheatsheet;
        }

        // Close help overlay
        KeyCode::Esc if app.show_cheatsheet => {
            app.show_cheatsheet = false;
        }

        // File switching - Previous file
        KeyCode::Char('[') if !app.show_cheatsheet && app.csv_files.len() > 1 => {
            app.current_file_index = if app.current_file_index == 0 {
                app.csv_files.len() - 1
            } else {
                app.current_file_index - 1
            };
            return Ok(true); // Signal to reload file
        }

        // File switching - Next file
        KeyCode::Char(']') if !app.show_cheatsheet && app.csv_files.len() > 1 => {
            app.current_file_index = (app.current_file_index + 1) % app.csv_files.len();
            return Ok(true); // Signal to reload file
        }

        // Navigation (only when help is closed)
        _ if !app.show_cheatsheet => navigation::handle_navigation(app, key.code),

        _ => {}
    }

    Ok(false)
}
