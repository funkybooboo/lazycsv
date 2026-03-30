//! File operations (rename, delete, move, copy, create) for file menu

use super::{scan_directory_filtered, BrowserEntry};
use crate::app::{App, FileOperation, Mode};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;

/// Get the currently selected entry in the file browser
fn get_selected_entry(app: &App) -> Result<BrowserEntry> {
    let entries = scan_directory_filtered(
        &app.view_state.current_directory,
        app.view_state.show_hidden_files,
    )?;
    let filter = app.input_state.file_filter_buffer.to_lowercase();

    let filtered: Vec<&BrowserEntry> = entries
        .iter()
        .filter(|e| {
            if filter.is_empty() {
                true
            } else if let Some(name) = e.filename() {
                name.to_lowercase().contains(&filter)
            } else {
                false
            }
        })
        .collect();

    if filtered.is_empty() {
        anyhow::bail!("No file selected");
    }

    let idx = app.view_state.file_list_selected.min(filtered.len() - 1);
    Ok(filtered[idx].clone())
}

/// Prompt for renaming the selected file
pub fn prompt_rename(app: &mut App) -> Result<InputResult> {
    let entry = match get_selected_entry(app) {
        Ok(e) => e,
        Err(_) => {
            app.status_message = Some(StatusMessage::from("No file selected"));
            return Ok(InputResult::Continue);
        }
    };

    // Don't allow renaming ".."
    if entry.filename() == Some("..") {
        app.status_message = Some(StatusMessage::from("Cannot rename parent directory"));
        return Ok(InputResult::Continue);
    }

    // Set up rename operation
    app.file_operation = Some(FileOperation::Rename(entry.path().to_path_buf()));
    app.file_operation_buffer = entry.filename().unwrap_or("").to_string();
    app.mode = Mode::FileOperationPrompt;
    Ok(InputResult::Continue)
}

/// Prompt for deleting the selected file
pub fn prompt_delete(app: &mut App) -> Result<InputResult> {
    let entry = match get_selected_entry(app) {
        Ok(e) => e,
        Err(_) => {
            app.status_message = Some(StatusMessage::from("No file selected"));
            return Ok(InputResult::Continue);
        }
    };

    // Don't allow deleting ".."
    if entry.filename() == Some("..") {
        app.status_message = Some(StatusMessage::from("Cannot delete parent directory"));
        return Ok(InputResult::Continue);
    }

    // Set up delete operation (requires confirmation)
    app.file_operation = Some(FileOperation::Delete(entry.path().to_path_buf()));
    app.file_operation_buffer.clear();
    app.mode = Mode::FileOperationPrompt;
    Ok(InputResult::Continue)
}

/// Prompt for moving the selected file
pub fn prompt_move(app: &mut App) -> Result<InputResult> {
    let entry = match get_selected_entry(app) {
        Ok(e) => e,
        Err(_) => {
            app.status_message = Some(StatusMessage::from("No file selected"));
            return Ok(InputResult::Continue);
        }
    };

    // Don't allow moving ".."
    if entry.filename() == Some("..") {
        app.status_message = Some(StatusMessage::from("Cannot move parent directory"));
        return Ok(InputResult::Continue);
    }

    // Set up move operation
    app.file_operation = Some(FileOperation::Move(entry.path().to_path_buf()));
    app.file_operation_buffer.clear();
    app.mode = Mode::FileOperationPrompt;
    Ok(InputResult::Continue)
}

/// Prompt for copying the selected file
pub fn prompt_copy(app: &mut App) -> Result<InputResult> {
    let entry = match get_selected_entry(app) {
        Ok(e) => e,
        Err(_) => {
            app.status_message = Some(StatusMessage::from("No file selected"));
            return Ok(InputResult::Continue);
        }
    };

    // Set up copy operation
    app.file_operation = Some(FileOperation::Copy(entry.path().to_path_buf()));
    app.file_operation_buffer = format!("{}.copy", entry.filename().unwrap_or("file"));
    app.mode = Mode::FileOperationPrompt;
    Ok(InputResult::Continue)
}

/// Prompt for creating a new file
pub fn prompt_create(app: &mut App) -> Result<InputResult> {
    // Set up create operation
    app.file_operation = Some(FileOperation::Create);
    app.file_operation_buffer = "new.csv".to_string();
    app.mode = Mode::FileOperationPrompt;
    Ok(InputResult::Continue)
}
