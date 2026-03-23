//! Editing operations for Normal mode

use crate::app::App;
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::handler::{enter_insert_mode, CursorPosition, InitialContent};
use crate::input::StatusMessage;

/// Add row below and enter Insert mode (o)
pub fn insert_row_below(app: &mut App) {
    if let Some(row_idx) = app.selected_row() {
        let new_row_idx = RowIndex::new(row_idx.get() + 1);
        app.document.insert_row(new_row_idx);
        app.history
            .push(crate::history::EditCommand::InsertRow { at: new_row_idx });
        app.view_state.table_state.select(Some(new_row_idx.get()));
        enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
    }
}

/// Add row above and enter Insert mode (O)
pub fn insert_row_above(app: &mut App) {
    if let Some(row_idx) = app.selected_row() {
        app.document.insert_row(row_idx);
        app.history
            .push(crate::history::EditCommand::InsertRow { at: row_idx });
        // Selection stays at current index which is now the new row
        enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
    }
}

/// Paste row(s) above cursor (P)
pub fn paste_rows_above(app: &mut App) {
    if let Some(rows) = app.clipboard.rows() {
        if let Some(row_idx) = app.selected_row() {
            let pasted_count = rows.len();
            let mut commands = Vec::new();
            for (i, clipboard_row) in rows.iter().enumerate() {
                let insert_idx = RowIndex::new(row_idx.get() + i);
                app.document.insert_row(insert_idx);
                commands.push(crate::history::EditCommand::InsertRow { at: insert_idx });
                for (col_idx, value) in clipboard_row.iter().enumerate() {
                    if col_idx < app.document.column_count() && !value.is_empty() {
                        app.document
                            .set_cell(insert_idx, ColIndex::new(col_idx), value.clone());
                        commands.push(crate::history::EditCommand::SetCell {
                            row: insert_idx,
                            col: ColIndex::new(col_idx),
                            old_value: String::new(),
                            new_value: value.clone(),
                        });
                    }
                }
            }
            if !commands.is_empty() {
                app.history
                    .push(crate::history::EditCommand::Compound(commands));
            }
            // Selection stays at current index (the first pasted row)
            app.status_message = Some(StatusMessage::new_owned(format!(
                "Pasted {} row(s)",
                pasted_count
            )));
        }
    } else {
        app.status_message = Some(StatusMessage::from("Nothing to paste"));
    }
}

/// Paste row(s) below cursor (p)
pub fn paste_rows_below(app: &mut App) {
    if let Some(rows) = app.clipboard.rows() {
        if let Some(row_idx) = app.selected_row() {
            let pasted_count = rows.len();
            let mut commands = Vec::new();
            for (i, clipboard_row) in rows.iter().enumerate() {
                let insert_idx = RowIndex::new(row_idx.get() + 1 + i);
                app.document.insert_row(insert_idx);
                commands.push(crate::history::EditCommand::InsertRow { at: insert_idx });
                for (col_idx, value) in clipboard_row.iter().enumerate() {
                    if col_idx < app.document.column_count() && !value.is_empty() {
                        app.document
                            .set_cell(insert_idx, ColIndex::new(col_idx), value.clone());
                        commands.push(crate::history::EditCommand::SetCell {
                            row: insert_idx,
                            col: ColIndex::new(col_idx),
                            old_value: String::new(),
                            new_value: value.clone(),
                        });
                    }
                }
            }
            if !commands.is_empty() {
                app.history
                    .push(crate::history::EditCommand::Compound(commands));
            }
            // Move selection to last pasted row
            let last_pasted = row_idx.get() + pasted_count;
            app.view_state.table_state.select(Some(last_pasted));
            app.status_message = Some(StatusMessage::new_owned(format!(
                "Pasted {} row(s)",
                pasted_count
            )));
        }
    } else {
        app.status_message = Some(StatusMessage::from("Nothing to paste"));
    }
}

/// Clear current cell (Delete key)
pub fn clear_cell(app: &mut App) {
    if let Some(row_idx) = app.selected_row() {
        let col_idx = app.view_state.selected_column;
        let old_value = app.document.cell(row_idx, col_idx);
        app.document.set_cell(row_idx, col_idx, String::new());
        app.history.push(crate::history::EditCommand::SetCell {
            row: row_idx,
            col: col_idx,
            old_value,
            new_value: String::new(),
        });
        app.status_message = Some(StatusMessage::from("Cell cleared"));
    }
}
