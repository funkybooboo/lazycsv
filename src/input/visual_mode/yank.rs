//! Visual yank operations
//!
//! Handles yanking (copying) visual selections without deleting.
//! Each visual mode yanks to its own clipboard buffer.

use crate::app::{App, Mode, VisualMode};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;

/// Yank the visual selection
pub fn handle_visual_yank(app: &mut App) -> Result<InputResult> {
    let selection = match &app.visual_selection {
        Some(sel) => *sel,
        None => {
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
    };

    let (start_row, end_row, start_col, end_col) = selection.bounds();

    match selection.mode {
        VisualMode::Block => yank_block(app, start_row, end_row, start_col, end_col),
        VisualMode::Line => yank_lines(app, start_row, end_row),
        VisualMode::Column => yank_columns(app, start_col, end_col),
    }

    // Move cursor to the start of the selection (minimum coordinates) like vim
    let (anchor_row, cursor_row) = (selection.anchor.0, selection.cursor.0);
    let (anchor_col, cursor_col) = (selection.anchor.1, selection.cursor.1);
    let start_row = anchor_row.min(cursor_row);
    let start_col = anchor_col.min(cursor_col);
    app.view_state.table_state.select(Some(start_row.get()));
    app.view_state.selected_column = start_col;

    // Save selection for gv and exit visual mode
    app.last_visual_selection = app.visual_selection.take();
    app.mode = Mode::Normal;

    Ok(InputResult::Continue)
}

/// Yank rectangular block selection
fn yank_block(
    app: &mut App,
    start_row: RowIndex,
    end_row: RowIndex,
    start_col: ColIndex,
    end_col: ColIndex,
) {
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
    app.clipboard.yank_region(region.clone());

    let row_count = end_row.get() - start_row.get() + 1;
    let col_count = end_col.get() - start_col.get() + 1;
    app.status_message = Some(StatusMessage::from(format!(
        "Yanked {}x{} cells",
        row_count, col_count
    )));
}

/// Yank whole rows (Line mode)
fn yank_lines(app: &mut App, start_row: RowIndex, end_row: RowIndex) {
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
    app.clipboard.yank_rows(rows.clone());

    let row_count = end_row.get() - start_row.get() + 1;
    app.status_message = Some(StatusMessage::from(format!("Yanked {} row(s)", row_count)));
}

/// Yank whole columns (Column mode)
fn yank_columns(app: &mut App, start_col: ColIndex, end_col: ColIndex) {
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

    let col_count = end_col.get() - start_col.get() + 1;
    app.status_message = Some(StatusMessage::from(format!(
        "Yanked {} column(s)",
        col_count
    )));
}
