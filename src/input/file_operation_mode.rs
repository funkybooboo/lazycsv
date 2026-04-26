//! File operation prompt mode handler

use crate::app::{App, FileOperation, Mode};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;

/// Handle keyboard input in file operation prompt mode (with keymap pre-pass).
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    if let Some(result) = crate::input::keymap_dispatch::try_keymap(
        app,
        key,
        crate::config::keys::KeymapScope::FileOperation,
        handle_raw,
    )? {
        return Ok(result);
    }
    handle_raw(app, key)
}

/// Legacy match-based file-operation-prompt handler.
pub fn handle_raw(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match key.code {
        // Cancel operation
        KeyCode::Esc => {
            cancel(app);
            Ok(InputResult::Continue)
        }

        // Execute operation
        KeyCode::Enter => execute_operation(app),

        // Backspace
        KeyCode::Backspace => {
            app.file_operation_buffer.pop();
            Ok(InputResult::Continue)
        }

        // Type characters
        KeyCode::Char(c) => {
            app.file_operation_buffer.push(c);
            Ok(InputResult::Continue)
        }

        _ => Ok(InputResult::Continue),
    }
}

/// Cancel file operation
fn cancel(app: &mut App) {
    app.file_operation = None;
    app.file_operation_buffer.clear();
    app.mode = Mode::FileList;
}

/// Execute the pending file operation
fn execute_operation(app: &mut App) -> Result<InputResult> {
    let operation = match app.file_operation.take() {
        Some(op) => op,
        None => {
            cancel(app);
            return Ok(InputResult::Continue);
        }
    };

    let result = match operation {
        FileOperation::Rename(old_path) => execute_rename(app, &old_path),
        FileOperation::Delete(path) => execute_delete(app, &path),
        FileOperation::Move(source) => execute_move(app, &source),
        FileOperation::Copy(source) => execute_copy(app, &source),
        FileOperation::Create => execute_create(app),
    };

    // Return to file list mode
    app.file_operation_buffer.clear();
    app.mode = Mode::FileList;

    match result {
        Ok(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
            Ok(InputResult::Continue)
        }
        Err(err) => {
            app.status_message = Some(StatusMessage::from(format!("Error: {}", err)));
            Ok(InputResult::Continue)
        }
    }
}

/// Execute rename operation
fn execute_rename(app: &App, old_path: &PathBuf) -> Result<String> {
    let new_name = &app.file_operation_buffer;

    if new_name.is_empty() {
        anyhow::bail!("Filename cannot be empty");
    }

    let parent = old_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;
    let new_path = parent.join(new_name);

    if new_path.exists() {
        anyhow::bail!("File already exists: {}", new_name);
    }

    std::fs::rename(old_path, &new_path)?;
    Ok(format!("Renamed to {}", new_name))
}

/// Execute delete operation
fn execute_delete(app: &App, path: &PathBuf) -> Result<String> {
    // Require confirmation by typing "yes"
    if app.file_operation_buffer != "yes" {
        anyhow::bail!("Type 'yes' to confirm deletion");
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
        Ok(format!("Deleted directory: {}", filename))
    } else {
        std::fs::remove_file(path)?;
        Ok(format!("Deleted file: {}", filename))
    }
}

/// Execute move operation
fn execute_move(app: &App, source: &PathBuf) -> Result<String> {
    let dest_input = &app.file_operation_buffer;

    if dest_input.is_empty() {
        anyhow::bail!("Destination cannot be empty");
    }

    // Resolve destination path (could be relative or absolute)
    let dest_path = if dest_input.starts_with('/') || dest_input.starts_with('~') {
        PathBuf::from(dest_input)
    } else {
        app.view_state.current_directory.join(dest_input)
    };

    if dest_path.exists() {
        anyhow::bail!("Destination already exists: {}", dest_input);
    }

    std::fs::rename(source, &dest_path)?;

    let source_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    Ok(format!("Moved {} to {}", source_name, dest_input))
}

/// Execute copy operation
fn execute_copy(app: &App, source: &PathBuf) -> Result<String> {
    let dest_name = &app.file_operation_buffer;

    if dest_name.is_empty() {
        anyhow::bail!("Filename cannot be empty");
    }

    let dest_path = app.view_state.current_directory.join(dest_name);

    if dest_path.exists() {
        anyhow::bail!("File already exists: {}", dest_name);
    }

    if source.is_dir() {
        anyhow::bail!("Cannot copy directories (not implemented)");
    }

    std::fs::copy(source, &dest_path)?;
    Ok(format!("Copied to {}", dest_name))
}

/// Execute create operation
fn execute_create(app: &App) -> Result<String> {
    let filename = &app.file_operation_buffer;

    if filename.is_empty() {
        anyhow::bail!("Filename cannot be empty");
    }

    let file_path = app.view_state.current_directory.join(filename);

    if file_path.exists() {
        anyhow::bail!("File already exists: {}", filename);
    }

    // Create empty CSV file with headers
    std::fs::write(&file_path, "Column1,Column2,Column3\n")?;
    Ok(format!("Created {}", filename))
}
