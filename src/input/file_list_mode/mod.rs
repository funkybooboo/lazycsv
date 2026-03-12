//! File list mode handlers
//!
//! Handles keyboard input when in FileList mode with yazi-like keybindings.

pub mod operations;

use crate::app::{App, Mode};
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard input in file list mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match (key.code, key.modifiers) {
        // Exit file manager
        (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
            cancel(app);
            Ok(InputResult::Continue)
        }

        // Search/filter - backspace
        (KeyCode::Backspace, _) => {
            app.input_state.pop_file_filter_char();
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }

        // Navigation - vim keys and arrows
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
            if app.view_state.file_list_selected > 0 {
                app.view_state.file_list_selected -= 1;
            }
            Ok(InputResult::Continue)
        }
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
            move_down(app);
            Ok(InputResult::Continue)
        }

        // Jump to top (gg in vim, but 'g' alone for simplicity)
        (KeyCode::Char('g'), KeyModifiers::NONE) => {
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }

        // Jump to bottom (G in vim)
        (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
            let filtered_count = count_filtered_files(app);
            if filtered_count > 0 {
                app.view_state.file_list_selected = filtered_count - 1;
            }
            Ok(InputResult::Continue)
        }

        // Open selected file
        (KeyCode::Enter, _) | (KeyCode::Char('o'), KeyModifiers::NONE) => select_file(app),

        // File operations - yazi-like
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            operations::start_rename(app);
            Ok(InputResult::Continue)
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            operations::start_delete(app);
            Ok(InputResult::Continue)
        }
        (KeyCode::Char('y'), KeyModifiers::NONE) => {
            operations::start_copy(app);
            Ok(InputResult::Continue)
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            operations::start_create(app);
            Ok(InputResult::Continue)
        }

        // Search/filter - alphanumeric characters (but not operation keys)
        (KeyCode::Char(c), mods) if mods == KeyModifiers::NONE || mods == KeyModifiers::SHIFT => {
            // Skip operation keys
            if matches!(c, 'k' | 'j' | 'g' | 'G' | 'o' | 'r' | 'd' | 'y' | 'n' | 'q') {
                Ok(InputResult::Continue)
            } else {
                app.input_state.push_file_filter_char(c);
                app.view_state.file_list_selected = 0;
                Ok(InputResult::Continue)
            }
        }

        _ => Ok(InputResult::Continue),
    }
}

/// Cancel file list mode and return to normal
fn cancel(app: &mut App) {
    app.mode = Mode::Normal;
    app.status_message = None;
    app.input_state.clear_file_filter();
    app.view_state.file_list_selected = 0;
}

/// Move file list selection down
fn move_down(app: &mut App) {
    let filtered_count = count_filtered_files(app);
    if app.view_state.file_list_selected + 1 < filtered_count {
        app.view_state.file_list_selected += 1;
    }
}

/// Count files matching current filter
fn count_filtered_files(app: &App) -> usize {
    let filter = app.input_state.file_filter_buffer.to_lowercase();
    app.session
        .files()
        .iter()
        .filter(|path| file_matches_filter(path, &filter))
        .count()
}

/// Check if file matches filter
fn file_matches_filter(path: &std::path::Path, filter: &str) -> bool {
    if filter.is_empty() {
        true
    } else {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase().contains(filter))
            .unwrap_or(false)
    }
}

/// Select file from filtered list
fn select_file(app: &mut App) -> Result<InputResult> {
    let filter = app.input_state.file_filter_buffer.to_lowercase();
    let filtered_files: Vec<(usize, &std::path::PathBuf)> = app
        .session
        .files()
        .iter()
        .enumerate()
        .filter(|(_, path)| file_matches_filter(path, &filter))
        .collect();

    if filtered_files.is_empty() {
        app.status_message = Some(StatusMessage::from("No matching files"));
        return Ok(InputResult::Continue);
    }

    let selected_idx = app
        .view_state
        .file_list_selected
        .min(filtered_files.len() - 1);
    let target_index = filtered_files[selected_idx].0;
    let current = app.session.active_file_index();

    if target_index != current {
        switch_to_index(app, target_index, current);
    }

    cancel(app);

    if target_index != current {
        Ok(InputResult::ReloadFile)
    } else {
        Ok(InputResult::Continue)
    }
}

/// Switch to file at target index
fn switch_to_index(app: &mut App, target: usize, current: usize) {
    let file_count = app.session.file_count();
    let diff = if target > current {
        target - current
    } else {
        file_count - current + target
    };

    for _ in 0..diff {
        app.session.next_file();
    }
}
