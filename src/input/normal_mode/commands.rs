//! Multi-key command implementations for Normal mode
//!
//! This module contains the actual implementation of multi-key commands.
//! These are called from multi_key.rs after the key sequence is recognized.

use crate::app::{App, Mode, VisualMode, VisualSelection};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::handler::{enter_insert_mode, CursorPosition, InitialContent};
use crate::input::StatusMessage;
use crate::navigation;
use crate::ui::ViewportMode;

/// gg - Go to first row
pub fn goto_first_row(app: &mut App) {
    app.input_state.clear_pending_command();
    navigation::goto_first_row(app);
    app.status_message = Some(StatusMessage::from("Jumped to first row"));
}

/// gv - Reselect last visual selection
pub fn reselect_visual(app: &mut App) {
    app.input_state.clear_pending_command();
    if let Some(last_sel) = app.last_visual_selection {
        // Restore the selection
        app.visual_selection = Some(last_sel);
        // Enter the appropriate visual mode
        app.mode = match last_sel.mode {
            VisualMode::Block => Mode::VisualBlock,
            VisualMode::Line => Mode::VisualLine,
            VisualMode::Column => Mode::VisualColumn,
        };
        // Move cursor to the selection cursor position
        app.view_state
            .table_state
            .select(Some(last_sel.cursor.0.get()));
        app.view_state.selected_column = last_sel.cursor.1;
        app.status_message = Some(StatusMessage::from("Reselected"));
    } else {
        app.status_message = Some(StatusMessage::from("No previous visual selection"));
    }
}

/// zt - Position current line at top of screen
pub fn viewport_top(app: &mut App) {
    app.input_state.clear_pending_command();
    app.view_state.viewport_mode = ViewportMode::Top;
    app.status_message = Some(StatusMessage::from("Viewport: top"));
}

/// zz - Position current line at center of screen
pub fn viewport_center(app: &mut App) {
    app.input_state.clear_pending_command();
    app.view_state.viewport_mode = ViewportMode::Center;
    app.status_message = Some(StatusMessage::from("Viewport: center"));
}

/// zb - Position current line at bottom of screen
pub fn viewport_bottom(app: &mut App) {
    app.input_state.clear_pending_command();
    app.view_state.viewport_mode = ViewportMode::Bottom;
    app.status_message = Some(StatusMessage::from("Viewport: bottom"));
}

/// dd - Delete row(s) with optional count prefix
pub fn delete_rows(app: &mut App) {
    app.input_state.clear_pending_command();
    let count = app
        .input_state
        .command_count
        .take()
        .map(|n| n.get())
        .unwrap_or(1);
    if let Some(row_idx) = app.selected_row() {
        let end_idx = RowIndex::new(row_idx.get() + count - 1);
        let deleted = app.document.delete_rows(row_idx, end_idx);
        let deleted_count = deleted.len();
        if deleted_count > 0 {
            // Record for undo and dot-repeat
            let cmd = crate::history::EditCommand::DeleteRows {
                start: row_idx,
                data: deleted.clone(),
            };
            app.last_edit = Some(cmd.clone());
            app.history.push(cmd);
            app.clipboard.yank_rows(deleted);
            // Adjust selection if needed
            let row_count = app.document.row_count();
            if row_count == 0 {
                app.view_state.table_state.select(None);
            } else if row_idx.get() >= row_count {
                app.view_state.table_state.select(Some(row_count - 1));
            }

            app.status_message = Some(StatusMessage::new_owned(format!(
                "{} row(s) deleted",
                deleted_count
            )));
        }
    }
}

/// yy - Yank (copy) row(s) with optional count prefix
pub fn yank_rows(app: &mut App) {
    app.input_state.clear_pending_command();
    let count = app
        .input_state
        .command_count
        .take()
        .map(|n| n.get())
        .unwrap_or(1);
    if let Some(row_idx) = app.selected_row() {
        let end_idx = RowIndex::new(
            (row_idx.get() + count - 1).min(app.document.row_count().saturating_sub(1)),
        );
        let rows = app.document.rows_range(row_idx, end_idx);
        let yanked_count = rows.len();
        if yanked_count > 0 {
            app.clipboard.yank_rows(rows);
            app.status_message = Some(StatusMessage::new_owned(format!(
                "{} row(s) yanked",
                yanked_count
            )));
        }
    }
}

/// cc - Clear row and enter insert mode
pub fn change_row(app: &mut App) {
    app.input_state.clear_pending_command();
    if let Some(row_idx) = app.selected_row() {
        // Clear all cells in the row
        let col_count = app.document.column_count();
        for col in 0..col_count {
            app.document
                .set_cell(row_idx, ColIndex::new(col), String::new());
        }
        // Move cursor to first column
        app.view_state.selected_column = ColIndex::new(0);
        // Enter insert mode
        enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
        app.status_message = Some(StatusMessage::from("Row cleared"));
    }
}

/// ,v - Enter Visual Column mode
pub fn enter_visual_column_mode(app: &mut App) {
    app.input_state.clear_pending_command();
    let row = app.selected_row().unwrap_or(RowIndex::new(0));
    let col = app.view_state.selected_column;
    app.visual_selection = Some(VisualSelection::new(row, col, VisualMode::Column));
    app.mode = Mode::VisualColumn;
}

/// ,p - Paste column(s) to the right of current column
pub fn paste_columns_after(app: &mut App) {
    app.input_state.clear_pending_command();
    if let Some(columns) = app.clipboard.as_columns() {
        let col_idx = app.view_state.selected_column;
        let pasted_count = columns.len();
        let mut commands = Vec::new();
        for (i, col_data) in columns.into_iter().enumerate() {
            let insert_at = ColIndex::new(col_idx.get() + 1 + i);
            app.document.insert_column(insert_at, col_data.clone());
            commands.push(crate::history::EditCommand::InsertColumn {
                at: insert_at,
                data: col_data,
            });
        }
        if !commands.is_empty() {
            app.history
                .push(crate::history::EditCommand::Compound(commands));
        }
        // Move selection to first pasted column
        app.view_state.selected_column = ColIndex::new(col_idx.get() + 1);
        app.status_message = Some(StatusMessage::new_owned(format!(
            "Pasted {} column(s)",
            pasted_count
        )));
    } else {
        app.status_message = Some(StatusMessage::from("Nothing to paste"));
    }
}

/// ,P - Paste column(s) to the left of current column
pub fn paste_columns_before(app: &mut App) {
    app.input_state.clear_pending_command();
    if let Some(columns) = app.clipboard.as_columns() {
        let col_idx = app.view_state.selected_column;
        let pasted_count = columns.len();
        let mut commands = Vec::new();
        for (i, col_data) in columns.into_iter().enumerate() {
            let insert_at = ColIndex::new(col_idx.get() + i);
            app.document.insert_column(insert_at, col_data.clone());
            commands.push(crate::history::EditCommand::InsertColumn {
                at: insert_at,
                data: col_data,
            });
        }
        if !commands.is_empty() {
            app.history
                .push(crate::history::EditCommand::Compound(commands));
        }
        // Selection stays at current index (first pasted column)
        app.status_message = Some(StatusMessage::new_owned(format!(
            "Pasted {} column(s)",
            pasted_count
        )));
    } else {
        app.status_message = Some(StatusMessage::from("Nothing to paste"));
    }
}

/// ,o - Insert empty column to the right
pub fn insert_column_after(app: &mut App) {
    app.input_state.clear_pending_command();
    let col_idx = app.view_state.selected_column;
    let insert_at = ColIndex::new(col_idx.get() + 1);
    app.document.insert_empty_column(insert_at);
    // Record for undo (empty column data)
    let data: Vec<String> = std::iter::once(format!(
        "Column {}",
        crate::ui::utils::column_to_excel_letter(insert_at.get())
    ))
    .chain(std::iter::repeat_n(
        String::new(),
        app.document.row_count().saturating_sub(1),
    ))
    .collect();
    app.history
        .push(crate::history::EditCommand::InsertColumn { at: insert_at, data });
    app.view_state.selected_column = insert_at;
    app.status_message = Some(StatusMessage::from("Inserted empty column"));
}

/// ,O - Insert empty column to the left
pub fn insert_column_before(app: &mut App) {
    app.input_state.clear_pending_command();
    let col_idx = app.view_state.selected_column;
    app.document.insert_empty_column(col_idx);
    // Record for undo
    let data: Vec<String> = std::iter::once(format!(
        "Column {}",
        crate::ui::utils::column_to_excel_letter(col_idx.get())
    ))
    .chain(std::iter::repeat_n(
        String::new(),
        app.document.row_count().saturating_sub(1),
    ))
    .collect();
    app.history
        .push(crate::history::EditCommand::InsertColumn { at: col_idx, data });
    app.status_message = Some(StatusMessage::from("Inserted empty column"));
}

/// ,dd - Delete column(s) with optional count prefix
pub fn delete_columns(app: &mut App) {
    app.input_state.clear_pending_command();
    let count = app
        .input_state
        .command_count
        .take()
        .map(|n| n.get())
        .unwrap_or(1);
    let col_idx = app.view_state.selected_column;
    let end_idx = ColIndex::new(col_idx.get() + count - 1);
    let deleted = app.document.delete_columns(col_idx, end_idx);
    let deleted_count = deleted.len();
    if deleted_count > 0 {
        // Record for undo
        app.history
            .push(crate::history::EditCommand::DeleteColumns {
                start: col_idx,
                data: deleted.clone(),
            });
        app.clipboard.yank_columns(deleted);
        // Adjust selection if needed
        let col_count = app.document.column_count();
        if col_count == 0 {
            // No columns left — nothing to select
        } else if col_idx.get() >= col_count {
            app.view_state.selected_column = ColIndex::new(col_count - 1);
        }
        app.status_message = Some(StatusMessage::new_owned(format!(
            "{} column(s) deleted",
            deleted_count
        )));
    }
}

/// ,yy - Yank column(s) with optional count prefix
pub fn yank_columns(app: &mut App) {
    app.input_state.clear_pending_command();
    let count = app
        .input_state
        .command_count
        .take()
        .map(|n| n.get())
        .unwrap_or(1);
    let col_idx = app.view_state.selected_column;
    let end_idx = ColIndex::new(
        (col_idx.get() + count - 1).min(app.document.column_count().saturating_sub(1)),
    );
    let columns = app.document.columns_range(col_idx, end_idx);
    let yanked_count = columns.len();
    if yanked_count > 0 {
        app.clipboard.yank_columns(columns);
        app.status_message = Some(StatusMessage::new_owned(format!(
            "{} column(s) yanked",
            yanked_count
        )));
    }
}
