//! File list mode handlers
//!
//! Handles keyboard input when in FileList mode with yazi-like keybindings.

mod browser;
mod operations;

use crate::app::{App, Mode};
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use anyhow::Result;
pub use browser::{scan_directory, BrowserEntry};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard input in file list mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Handle search mode separately
    if app.input_state.file_list_search_active {
        return handle_search_mode(app, key);
    }

    match (key.code, key.modifiers) {
        // Exit file manager
        (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
            cancel(app);
            Ok(InputResult::Continue)
        }

        // Enter search mode
        (KeyCode::Char('/'), KeyModifiers::NONE) => {
            app.input_state.file_list_search_active = true;
            app.input_state.clear_file_filter();
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

        // Jump to top - first 'g' sets pending, second 'g' executes
        (KeyCode::Char('g'), KeyModifiers::NONE) => {
            if app.input_state.pending_command == Some(crate::input::PendingCommand::G) {
                // Second 'g' - go to top
                app.input_state.clear_pending_command();
                app.view_state.file_list_selected = 0;
            } else {
                // First 'g' - set pending
                app.input_state
                    .set_pending_command(crate::input::PendingCommand::G);
            }
            Ok(InputResult::Continue)
        }

        // Jump to bottom (G in vim)
        (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
            // Get browser entries for current directory
            let current_dir = app.view_state.current_directory.clone();
            if let Ok(entries) = scan_directory(&current_dir) {
                let filter = app.input_state.file_filter_buffer.to_lowercase();
                let filtered_count = count_filtered_browser_entries(&entries, &filter);
                if filtered_count > 0 {
                    app.view_state.file_list_selected = filtered_count - 1;
                }
            }
            Ok(InputResult::Continue)
        }

        // Open selected file
        (KeyCode::Enter, _) => select_file(app),

        // Yazi-style directory navigation
        (KeyCode::Char('h'), KeyModifiers::NONE) => {
            // Go up to parent directory
            navigate_to_parent(app);
            Ok(InputResult::Continue)
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) => {
            // Enter directory or open file
            navigate_into_selected(app)
        }

        // File operations
        (KeyCode::Char('r'), KeyModifiers::NONE) => operations::prompt_rename(app),
        (KeyCode::Char('d'), KeyModifiers::NONE) => operations::prompt_delete(app),
        (KeyCode::Char('m'), KeyModifiers::NONE) => operations::prompt_move(app),
        (KeyCode::Char('y'), KeyModifiers::NONE) => operations::prompt_copy(app),
        (KeyCode::Char('n'), KeyModifiers::NONE) => operations::prompt_create(app),

        _ => Ok(InputResult::Continue),
    }
}

/// Handle keyboard input when in search mode
fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match key.code {
        // Exit search mode
        KeyCode::Esc => {
            app.input_state.file_list_search_active = false;
            app.input_state.clear_file_filter();
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }

        // Apply filter and exit search mode (or show error if no matches)
        KeyCode::Enter => {
            app.input_state.file_list_search_active = false;
            // Get browser entries for current directory
            let current_dir = app.view_state.current_directory.clone();
            if let Ok(entries) = scan_directory(&current_dir) {
                let filter = app.input_state.file_filter_buffer.to_lowercase();
                let filtered_count = count_filtered_browser_entries(&entries, &filter);
                if filtered_count == 0 && !app.input_state.file_filter_buffer.is_empty() {
                    app.status_message = Some(StatusMessage::from("No matching files"));
                }
            }
            Ok(InputResult::Continue)
        }

        // Backspace in search
        KeyCode::Backspace => {
            app.input_state.pop_file_filter_char();
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }

        // Accept any character in search mode
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
    app.input_state.file_list_search_active = false;
    app.view_state.file_list_selected = 0;
}

/// Move file list selection down
fn move_down(app: &mut App) {
    // Get browser entries for current directory
    let current_dir = app.view_state.current_directory.clone();
    if let Ok(entries) = scan_directory(&current_dir) {
        let filter = app.input_state.file_filter_buffer.to_lowercase();
        let filtered_count = count_filtered_browser_entries(&entries, &filter);
        if app.view_state.file_list_selected + 1 < filtered_count {
            app.view_state.file_list_selected += 1;
        }
    }
}

/// Count browser entries matching current filter
fn count_filtered_browser_entries(entries: &[BrowserEntry], filter: &str) -> usize {
    entries
        .iter()
        .filter(|entry| entry_matches_filter(entry, filter))
        .count()
}

/// Check if browser entry matches filter
fn entry_matches_filter(entry: &BrowserEntry, filter: &str) -> bool {
    if filter.is_empty() {
        true
    } else if let Some(name) = entry.filename() {
        name.to_lowercase().contains(filter)
    } else {
        false
    }
}

/// Check if file path matches filter (for session files)
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

/// Navigate to parent directory (h key - yazi-style)
fn navigate_to_parent(app: &mut App) {
    let current_dir = &app.view_state.current_directory;
    if let Some(parent) = current_dir.parent() {
        app.view_state.current_directory = parent.to_path_buf();
        app.view_state.file_list_selected = 0;
        app.input_state.clear_file_filter();
    }
}

/// Navigate into selected directory or open file (l key - yazi-style)
fn navigate_into_selected(app: &mut App) -> Result<InputResult> {
    // Get browser entries for current directory
    let current_dir = app.view_state.current_directory.clone();
    let entries = match scan_directory(&current_dir) {
        Ok(e) => e,
        Err(err) => {
            app.status_message = Some(StatusMessage::from(format!(
                "Error reading directory: {}",
                err
            )));
            return Ok(InputResult::Continue);
        }
    };

    // Apply filter
    let filter = app.input_state.file_filter_buffer.to_lowercase();
    let filtered_entries: Vec<&BrowserEntry> = entries
        .iter()
        .filter(|entry| {
            if filter.is_empty() {
                true
            } else if let Some(name) = entry.filename() {
                name.to_lowercase().contains(&filter)
            } else {
                false
            }
        })
        .collect();

    if filtered_entries.is_empty() {
        app.status_message = Some(StatusMessage::from("No items to navigate"));
        return Ok(InputResult::Continue);
    }

    let selected_idx = app
        .view_state
        .file_list_selected
        .min(filtered_entries.len() - 1);
    let selected_entry = filtered_entries[selected_idx];

    match selected_entry {
        BrowserEntry::Directory(path) => {
            // Navigate into directory
            let canonical_path = match path.canonicalize() {
                Ok(p) => p,
                Err(err) => {
                    app.status_message = Some(StatusMessage::from(format!(
                        "Cannot access directory: {}",
                        err
                    )));
                    return Ok(InputResult::Continue);
                }
            };
            app.view_state.current_directory = canonical_path;
            app.view_state.file_list_selected = 0;
            app.input_state.clear_file_filter();
            Ok(InputResult::Continue)
        }
        BrowserEntry::CsvFile(path) => {
            // Load the CSV file
            load_csv_file(app, path.clone())
        }
    }
}

/// Load a CSV file (opens it in the app)
fn load_csv_file(app: &mut App, path: std::path::PathBuf) -> Result<InputResult> {
    // Check if this file is already in the session
    let files = app.session.files();
    if let Some(index) = files.iter().position(|p| p == &path) {
        // File already loaded, just switch to it
        let current = app.session.active_file_index();
        if index != current {
            switch_to_index(app, index, current);
        }
        cancel(app);
        if index != current {
            return Ok(InputResult::ReloadFile);
        } else {
            return Ok(InputResult::Continue);
        }
    }

    // File not loaded yet - load it and add to session
    use crate::Document;

    let config = app.session.config();
    let document = match Document::from_file(
        &path,
        config.delimiter,
        config.no_headers,
        config.encoding.clone(),
    ) {
        Ok(doc) => doc,
        Err(err) => {
            app.status_message = Some(StatusMessage::from(format!("Failed to load: {}", err)));
            cancel(app);
            return Ok(InputResult::Continue);
        }
    };

    // Add file to session and switch to it
    let new_index = app.session.add_file(path.clone());
    app.session.set_active_file_index(new_index);

    // Update document
    app.document = document;

    // Reset view state for new file
    app.view_state.table_state.select(Some(0));
    app.view_state.selected_column = crate::domain::position::ColIndex::new(0);
    app.view_state.column_scroll_offset = 0;

    app.status_message = Some(StatusMessage::from(format!(
        "Loaded: {}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
    )));

    cancel(app);
    Ok(InputResult::Continue)
}
