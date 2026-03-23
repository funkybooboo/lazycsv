//! Visual delete operations
//!
//! Handles deletion of visual selections (Block, Line, Column modes).
//! Automatically yanks selection to appropriate clipboard buffer before deleting.

use crate::app::{App, Mode, VisualMode};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;

/// Delete the visual selection
pub fn handle_visual_delete(app: &mut App) -> Result<InputResult> {
    let selection = match app.visual_selection.take() {
        Some(sel) => sel,
        None => {
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
    };

    let (start_row, end_row, start_col, end_col) = selection.bounds();

    match selection.mode {
        VisualMode::Block => delete_block(app, start_row, end_row, start_col, end_col),
        VisualMode::Line => delete_lines(app, start_row, end_row),
        VisualMode::Column => delete_columns(app, start_col, end_col),
    }

    // Save selection for gv
    app.last_visual_selection = Some(selection);
    app.mode = Mode::Normal;

    Ok(InputResult::Continue)
}

/// Delete rectangular block selection
fn delete_block(
    app: &mut App,
    start_row: RowIndex,
    end_row: RowIndex,
    start_col: ColIndex,
    end_col: ColIndex,
) {
    // Yank rectangular region to region buffer before deleting
    let mut region = Vec::new();
    for row_idx in start_row.get()..=end_row.get() {
        let mut row = Vec::new();
        for col_idx in start_col.get()..=end_col.get() {
            let cell = app
                .document
                .cell(RowIndex::new(row_idx), ColIndex::new(col_idx))
                .to_string();
            row.push(cell);
        }
        region.push(row);
    }
    app.clipboard.yank_region(region);

    // Clear cells in rectangular region (preserve structure) and record for undo
    let mut commands = Vec::new();
    for row_idx in start_row.get()..=end_row.get() {
        for col_idx in start_col.get()..=end_col.get() {
            let r = RowIndex::new(row_idx);
            let c = ColIndex::new(col_idx);
            let old_value = app.document.cell(r, c);
            app.document.set_cell(r, c, String::new());
            if !old_value.is_empty() {
                commands.push(crate::history::EditCommand::SetCell {
                    row: r,
                    col: c,
                    old_value,
                    new_value: String::new(),
                });
            }
        }
    }
    if !commands.is_empty() {
        app.history
            .push(crate::history::EditCommand::Compound(commands));
    }

    let row_count = end_row.get() - start_row.get() + 1;
    let col_count = end_col.get() - start_col.get() + 1;
    app.status_message = Some(StatusMessage::from(format!(
        "Cleared {}x{} cells",
        row_count, col_count
    )));
}

/// Delete whole rows (Line mode)
fn delete_lines(app: &mut App, start_row: RowIndex, end_row: RowIndex) {
    // Yank entire rows to row buffer before deleting
    let rows: Vec<Vec<String>> = (start_row.get()..=end_row.get())
        .map(|row_idx| {
            (0..app.document.column_count())
                .map(|col_idx| {
                    app.document
                        .cell(RowIndex::new(row_idx), ColIndex::new(col_idx))
                        .to_string()
                })
                .collect()
        })
        .collect();
    app.clipboard.yank_rows(rows);

    // Delete entire rows
    let deleted = app.document.delete_rows(start_row, end_row);
    if !deleted.is_empty() {
        app.history.push(crate::history::EditCommand::DeleteRows {
            start: start_row,
            data: deleted.clone(),
        });
    }
    app.status_message = Some(StatusMessage::from(format!(
        "Deleted {} row(s)",
        deleted.len()
    )));

    // Adjust cursor position
    if app.document.row_count() > 0 {
        let new_row = start_row.get().min(app.document.row_count() - 1);
        app.view_state.table_state.select(Some(new_row));
    } else {
        app.view_state.table_state.select(Some(0));
    }
}

/// Delete whole columns (Column mode)
fn delete_columns(app: &mut App, start_col: ColIndex, end_col: ColIndex) {
    // Yank entire columns to column buffer before deleting
    let mut columns = Vec::new();
    for col_idx in start_col.get()..=end_col.get() {
        let column: Vec<String> = (0..app.document.row_count())
            .map(|row_idx| {
                app.document
                    .cell(RowIndex::new(row_idx), ColIndex::new(col_idx))
                    .to_string()
            })
            .collect();
        columns.push(column);
    }
    app.clipboard.yank_columns(columns);

    // Delete entire columns — use delete_columns for a single undo step
    let deleted = app.document.delete_columns(start_col, end_col);
    let col_count = deleted.len();
    if col_count > 0 {
        app.history
            .push(crate::history::EditCommand::DeleteColumns {
                start: start_col,
                data: deleted,
            });
    }
    app.status_message = Some(StatusMessage::from(format!(
        "Deleted {} column(s)",
        col_count
    )));

    // Adjust cursor position
    if start_col.get() >= app.document.column_count() {
        app.view_state.selected_column =
            ColIndex::new(app.document.column_count().saturating_sub(1));
    }
}
