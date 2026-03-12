//! File operations for file manager mode

use crate::app::App;
use crate::input::StatusMessage;
use std::path::PathBuf;

/// Pending file operation state
#[derive(Debug, Clone, PartialEq)]
pub enum FileOperation {
    None,
    /// Rename file - waiting for new name input
    Rename {
        original_path: PathBuf,
    },
    /// Delete file - waiting for confirmation
    Delete {
        path: PathBuf,
    },
    /// Copy file - waiting for new name input
    Copy {
        source_path: PathBuf,
    },
    /// Create new file - waiting for name input
    Create,
}

impl FileOperation {
    pub fn is_active(&self) -> bool {
        !matches!(self, FileOperation::None)
    }

    pub fn prompt_text(&self) -> &str {
        match self {
            FileOperation::None => "",
            FileOperation::Rename { .. } => "New name: ",
            FileOperation::Delete { .. } => "Delete? (y/n): ",
            FileOperation::Copy { .. } => "Copy to: ",
            FileOperation::Create => "New file name: ",
        }
    }
}

/// Start rename operation for selected file
pub fn start_rename(app: &mut App) {
    if let Some(_path) = get_selected_file_path(app) {
        app.status_message = Some(StatusMessage::from("Enter new name..."));
        // TODO: Set file operation state (needs to be added to App)
    } else {
        app.status_message = Some(StatusMessage::from("No file selected"));
    }
}

/// Start delete operation for selected file
pub fn start_delete(app: &mut App) {
    if let Some(path) = get_selected_file_path(app) {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        app.status_message = Some(StatusMessage::from(format!(
            "Delete {}? Press 'y' to confirm",
            filename
        )));
        // TODO: Set file operation state
    } else {
        app.status_message = Some(StatusMessage::from("No file selected"));
    }
}

/// Start copy operation for selected file
pub fn start_copy(app: &mut App) {
    if let Some(_path) = get_selected_file_path(app) {
        app.status_message = Some(StatusMessage::from("Enter destination name..."));
        // TODO: Set file operation state
    } else {
        app.status_message = Some(StatusMessage::from("No file selected"));
    }
}

/// Start create new file operation
pub fn start_create(app: &mut App) {
    app.status_message = Some(StatusMessage::from("Enter new file name..."));
    // TODO: Set file operation state
}

/// Get the path of the currently selected file in the file list
fn get_selected_file_path(app: &App) -> Option<PathBuf> {
    let filter = app.input_state.file_filter_buffer.to_lowercase();
    let files = app.session.files();
    let selected_idx = app.view_state.file_list_selected;

    let filtered_files: Vec<&PathBuf> = files
        .iter()
        .filter(|path| {
            if filter.is_empty() {
                true
            } else {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase().contains(&filter))
                    .unwrap_or(false)
            }
        })
        .collect();

    if selected_idx < filtered_files.len() {
        Some(filtered_files[selected_idx].clone())
    } else {
        None
    }
}
