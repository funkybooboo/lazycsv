//! Column and row reorder via drag-and-drop.

use super::mouse_coords::{resolve_column_header, resolve_row_gutter, resolve_table_cell};
use crate::app::App;
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::InputResult;
use crate::ui::ViewportMode;

pub(crate) fn handle_reorder_drag(app: &mut App, x: u16, y: u16) -> InputResult {
    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    if let Some((src, _)) = app.view_state.mouse_layout.col_reorder {
        // Update drop target column
        let target = resolve_column_header(&layout, area, x, y)
            .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(_, c)| c));
        if let Some(col_idx) = target {
            app.view_state.mouse_layout.col_reorder = Some((src, col_idx));
            app.view_state.selected_column = ColIndex::new(col_idx);
        }
    }

    if let Some((src, _)) = app.view_state.mouse_layout.row_reorder {
        // Update drop target row
        let target = resolve_row_gutter(&layout, area, x, y)
            .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(r, _)| r));
        if let Some(row_idx) = target {
            app.view_state.mouse_layout.row_reorder = Some((src, row_idx));
            app.view_state.table_state.select(Some(row_idx));
            app.view_state.viewport_mode = ViewportMode::Auto;
        }
    }

    InputResult::Continue
}

/// Finalize a column reorder when the mouse is released.
pub(crate) fn finalize_column_reorder(
    app: &mut App,
    src_col: usize,
    x: u16,
    y: u16,
) -> InputResult {
    use crate::input::StatusMessage;

    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    // Resolve the drop target column from the header or any cell
    let dst_col = resolve_column_header(&layout, area, x, y)
        .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(_, c)| c));

    let Some(dst_col) = dst_col else {
        return InputResult::Continue;
    };

    if src_col == dst_col {
        return InputResult::Continue;
    }

    let from_start = ColIndex::new(src_col);
    let from_end = ColIndex::new(src_col);
    // Insert before dst_col if moving left, after dst_col if moving right
    let to_before = if dst_col > src_col {
        dst_col + 1
    } else {
        dst_col
    };

    let actual_insert = app.document.move_columns(from_start, from_end, to_before);

    app.history.push(crate::history::EditCommand::MoveColumns {
        from_start,
        from_end,
        to_before,
        actual_insert,
    });

    app.view_state.selected_column = ColIndex::new(actual_insert);
    app.status_message = Some(StatusMessage::from(format!(
        "Moved column {} → {}",
        crate::ui::utils::column_to_excel_letter(src_col),
        crate::ui::utils::column_to_excel_letter(actual_insert),
    )));

    InputResult::Continue
}

/// Finalize a row reorder when the mouse is released.
pub(crate) fn finalize_row_reorder(app: &mut App, src_row: usize, x: u16, y: u16) -> InputResult {
    use crate::input::StatusMessage;

    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    // Resolve the drop target row from the gutter or any cell
    let dst_row = resolve_row_gutter(&layout, area, x, y)
        .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(r, _)| r));

    let Some(dst_row) = dst_row else {
        return InputResult::Continue;
    };

    if src_row == dst_row {
        return InputResult::Continue;
    }

    // Move row via sequential swaps
    let from = src_row;
    let to = dst_row;
    if from < to {
        for i in from..to {
            app.document
                .swap_rows(RowIndex::new(i), RowIndex::new(i + 1));
        }
    } else {
        for i in (to..from).rev() {
            app.document
                .swap_rows(RowIndex::new(i), RowIndex::new(i + 1));
        }
    }

    app.history.push(crate::history::EditCommand::MoveRow {
        from: RowIndex::new(from),
        to: RowIndex::new(to),
    });

    app.view_state.table_state.select(Some(dst_row));
    app.view_state.viewport_mode = ViewportMode::Auto;
    app.status_message = Some(StatusMessage::from(format!(
        "Moved row {} → {}",
        from + 1,
        to + 1,
    )));

    InputResult::Continue
}
