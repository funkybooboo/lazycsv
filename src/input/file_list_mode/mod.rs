//! File list mode handlers
//!
//! Handles keyboard input when in FileList mode (Ctrl+P fuzzy file finder).

use crate::app::{App, Mode};
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard input in file list mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match key.code {
        KeyCode::Esc => {
            cancel(app);
            Ok(InputResult::Continue)
        }
        KeyCode::Backspace => {
            app.input_state.pop_file_filter_char();
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.view_state.file_list_selected > 0 {
                app.view_state.file_list_selected -= 1;
            }
            Ok(InputResult::Continue)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_down(app);
            Ok(InputResult::Continue)
        }
        KeyCode::Enter => select_file(app),
        KeyCode::Char(c) => {
            app.input_state.push_file_filter_char(c);
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
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
