//! File list mode handlers
//!
//! Handles keyboard input when in FileList mode with yazi-like keybindings.

mod browser;
mod operations;

use crate::app::{App, Mode};
use crate::config::views;
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use anyhow::Result;
pub use browser::{scan_directory, scan_directory_filtered, BrowserEntry};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard input in file list mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Handle search mode separately
    if app.input_state.file_list_search_active {
        return handle_search_mode(app, key);
    }

    match (key.code, key.modifiers) {
        // Exit file manager (or close spot popup first)
        (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
            if app.view_state.file_spot_visible {
                app.view_state.file_spot_visible = false;
            } else {
                cancel(app);
            }
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
            move_up(app);
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
            if let Ok(entries) =
                scan_directory_filtered(&current_dir, app.view_state.show_hidden_files)
            {
                let filter = app.input_state.file_filter_buffer.to_lowercase();
                let filtered_count = count_filtered_browser_entries(&entries, &filter);
                if filtered_count > 0 {
                    app.view_state.file_list_selected = filtered_count - 1;
                }
            }
            Ok(InputResult::Continue)
        }

        // Open selected file / enter directory
        (KeyCode::Enter, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), KeyModifiers::NONE) => {
            navigate_into_selected(app)
        }

        // Go up to parent directory
        (KeyCode::Left, _) | (KeyCode::Char('h'), KeyModifiers::NONE) => {
            navigate_to_parent(app);
            Ok(InputResult::Continue)
        }

        // Toggle hidden files (yazi-style)
        (KeyCode::Char('.'), KeyModifiers::NONE) => {
            app.view_state.show_hidden_files = !app.view_state.show_hidden_files;
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }

        // Toggle file details popup (yazi Spot)
        (KeyCode::Tab, _) => {
            app.view_state.file_spot_visible = !app.view_state.file_spot_visible;
            Ok(InputResult::Continue)
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
            if let Ok(entries) =
                scan_directory_filtered(&current_dir, app.view_state.show_hidden_files)
            {
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

        // Arrow keys / vim nav: exit search mode and navigate the filtered list
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
            app.input_state.file_list_search_active = false;
            handle(app, key)
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

/// Move file list selection up (wraps to bottom)
fn move_up(app: &mut App) {
    if app.view_state.file_list_selected > 0 {
        app.view_state.file_list_selected -= 1;
    } else {
        // Wrap to bottom
        let current_dir = app.view_state.current_directory.clone();
        if let Ok(entries) = scan_directory_filtered(&current_dir, app.view_state.show_hidden_files)
        {
            let filter = app.input_state.file_filter_buffer.to_lowercase();
            let filtered_count = count_filtered_browser_entries(&entries, &filter);
            if filtered_count > 0 {
                app.view_state.file_list_selected = filtered_count - 1;
            }
        }
    }
}

/// Move file list selection down (wraps to top)
fn move_down(app: &mut App) {
    let current_dir = app.view_state.current_directory.clone();
    if let Ok(entries) = scan_directory_filtered(&current_dir, app.view_state.show_hidden_files) {
        let filter = app.input_state.file_filter_buffer.to_lowercase();
        let filtered_count = count_filtered_browser_entries(&entries, &filter);
        if app.view_state.file_list_selected + 1 < filtered_count {
            app.view_state.file_list_selected += 1;
        } else {
            // Wrap to top
            app.view_state.file_list_selected = 0;
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

/// Navigate to a clicked entry from the parent-column listing (mouse support).
/// The parent column shows siblings of the current directory; clicking one
/// changes the current directory to that sibling. Non-directory entries are ignored.
pub fn navigate_to_parent_column_index(app: &mut App, idx: usize) {
    let parent_dir = match app.view_state.current_directory.parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };
    let entries = match scan_directory_filtered(&parent_dir, app.view_state.show_hidden_files) {
        Ok(e) => e,
        Err(_) => return,
    };
    // Parent column skips the ".." entry.
    let visible: Vec<&BrowserEntry> = entries
        .iter()
        .filter(|e| e.filename().is_some_and(|n| n != ".."))
        .collect();
    let Some(entry) = visible.get(idx) else {
        return;
    };
    let BrowserEntry::Directory(path) = entry else {
        return;
    };
    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return,
    };
    app.view_state.current_directory = canonical_path;
    app.view_state.file_list_selected = 0;
    app.input_state.clear_file_filter();
}

/// Navigate to parent directory (h key - yazi-style)
fn navigate_to_parent(app: &mut App) {
    let current_dir = app.view_state.current_directory.clone();
    if let Some(parent) = current_dir.parent() {
        let parent_buf = parent.to_path_buf();
        // Find the index of the directory we just came from in the parent listing
        let selected = scan_directory_filtered(&parent_buf, app.view_state.show_hidden_files)
            .ok()
            .and_then(|entries| {
                let dir_name = current_dir.file_name()?;
                entries
                    .iter()
                    .position(|e| e.filename() == dir_name.to_str())
            })
            .unwrap_or(0);
        app.view_state.current_directory = parent_buf;
        app.view_state.file_list_selected = selected;
        app.input_state.clear_file_filter();
    }
}

/// Navigate into selected directory or open file (l key - yazi-style)
pub fn navigate_into_selected(app: &mut App) -> Result<InputResult> {
    // Get browser entries for current directory
    let current_dir = app.view_state.current_directory.clone();
    let entries = match scan_directory_filtered(&current_dir, app.view_state.show_hidden_files) {
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
        // Apply saved view settings
        {
            let store = views::load_views();
            let key = views::canonical_key(&path);
            if let Some(fv) = store.files.get(&key) {
                views::apply_file_view(&path, fv, &mut app.session, &mut app.view_state);
            }
        }
        cancel(app);
        if index != current {
            return Ok(InputResult::ReloadFile);
        } else {
            return Ok(InputResult::Continue);
        }
    }

    // File not loaded yet — exit file list mode and defer the load to the main
    // loop so it can render a "Loading…" screen (the handler has no terminal).
    cancel(app);
    Ok(InputResult::OpenFile(path))
}
