//! Visual paste operations
//!
//! Handles pasting over visual selections (Block, Line, Column modes).
//! Pastes from the appropriate clipboard buffer based on visual mode.

use crate::app::{App, Mode, VisualMode};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;

/// Paste over the visual selection
pub fn handle_visual_paste(app: &mut App) -> Result<InputResult> {
    let selection = match app.visual_selection.take() {
        Some(sel) => sel,
        None => {
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
    };

    let (start_row, end_row, start_col, end_col) = selection.bounds();

    match selection.mode {
        VisualMode::Block => paste_block(app, start_row, start_col),
        VisualMode::Line => paste_lines(app, start_row, end_row),
        VisualMode::Column => paste_columns(app, start_col, end_col),
    }

    // Save selection for gv
    app.last_visual_selection = Some(selection);
    app.mode = Mode::Normal;

    Ok(InputResult::Continue)
}

/// Paste rectangular region over block selection
fn paste_block(app: &mut App, start_row: RowIndex, start_col: ColIndex) {
    if let Some(region) = app.clipboard.region() {
        let region_rows = region.len();
        let max_cols = region.iter().map(|r| r.len()).max().unwrap_or(0);

        for (r_offset, row_data) in region.iter().enumerate() {
            let target_row = start_row.get() + r_offset;

            // Extend table if needed
            while target_row >= app.document.row_count() {
                app.document
                    .insert_row(RowIndex::new(app.document.row_count()));
            }

            for (c_offset, cell_value) in row_data.iter().enumerate() {
                let target_col = start_col.get() + c_offset;
                if target_col < app.document.column_count() {
                    app.document.set_cell(
                        RowIndex::new(target_row),
                        ColIndex::new(target_col),
                        cell_value.clone(),
                    );
                }
            }
        }
        app.status_message = Some(StatusMessage::from(format!(
            "Pasted {}x{} region",
            region_rows, max_cols
        )));
    } else {
        app.status_message = Some(StatusMessage::from("Nothing to paste"));
    }
}

/// Paste rows over line selection
fn paste_lines(app: &mut App, start_row: RowIndex, end_row: RowIndex) {
    if let Some(rows) = app.clipboard.rows() {
        // Delete selected rows first
        app.document.delete_rows(start_row, end_row);

        // Insert rows at start position
        for (offset, row_data) in rows.iter().enumerate() {
            let insert_row = RowIndex::new(start_row.get() + offset);
            app.document.insert_row(insert_row);
            for (col_idx, cell_value) in row_data.iter().enumerate() {
                if col_idx < app.document.column_count() {
                    app.document
                        .set_cell(insert_row, ColIndex::new(col_idx), cell_value.clone());
                }
            }
        }
        app.status_message = Some(StatusMessage::from(format!("Pasted {} row(s)", rows.len())));
    } else {
        app.status_message = Some(StatusMessage::from("Nothing to paste"));
    }
}

/// Paste columns over column selection
fn paste_columns(app: &mut App, start_col: ColIndex, end_col: ColIndex) {
    if let Some(columns) = app.clipboard.columns() {
        // Delete selected columns first
        let col_count = end_col.get() - start_col.get() + 1;
        for _ in 0..col_count {
            app.document.delete_column(start_col);
        }

        // Insert columns at start position
        for (offset, col_data) in columns.iter().enumerate() {
            let insert_col = ColIndex::new(start_col.get() + offset);
            app.document.insert_column(insert_col, col_data.clone());
        }
        app.status_message = Some(StatusMessage::from(format!(
            "Pasted {} column(s)",
            columns.len()
        )));
    } else {
        app.status_message = Some(StatusMessage::from("Nothing to paste"));
    }
}
