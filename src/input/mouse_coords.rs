//! Coordinate resolution — map terminal positions to logical cells.

use crate::app::App;

pub(crate) fn resolve_row_gutter(
    layout: &crate::ui::view_state::MouseLayout,
    area: ratatui::layout::Rect,
    x: u16,
    y: u16,
) -> Option<usize> {
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    let rel_y = (y - area.y) as usize;

    // Must be in the gutter column (before first data column)
    let gutter_end = layout
        .col_positions
        .get(1)
        .copied()
        .unwrap_or(area.x + layout.row_num_width);
    if x >= gutter_end {
        return None;
    }

    // Skip column letters row
    if rel_y == 0 {
        return None;
    }

    let frozen_count = layout.frozen_row_indices.len();
    let data_rel_y = rel_y - 1;

    if data_rel_y < frozen_count {
        layout.frozen_row_indices.get(data_rel_y).copied()
    } else {
        let scroll_rel = data_rel_y - frozen_count;
        layout.scrollable_indices.get(scroll_rel).copied()
    }
}

/// Map an absolute x coordinate to a display column index using resolved col_positions.
/// Returns None if x falls in the gutter (index 0) or outside all columns.
pub(crate) fn resolve_x_to_display_col(
    layout: &crate::ui::view_state::MouseLayout,
    x: u16,
) -> Option<usize> {
    let positions = &layout.col_positions;
    for i in 0..positions.len().saturating_sub(1) {
        if x >= positions[i] && x < positions[i + 1] {
            if i == 0 {
                return None; // row number gutter
            }
            return Some(i - 1); // display_cols index
        }
    }
    None
}

/// Resolve a click on the column letters row to a column index, if any.
pub(crate) fn resolve_column_header(
    layout: &crate::ui::view_state::MouseLayout,
    area: ratatui::layout::Rect,
    x: u16,
    y: u16,
) -> Option<usize> {
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }
    if (y - area.y) != 0 {
        return None;
    }
    let idx = resolve_x_to_display_col(layout, x)?;
    layout.display_cols.get(idx).copied()
}

/// Resolve terminal coordinates to a (row_index, col_index) table cell, if any.
pub(crate) fn resolve_table_cell(
    layout: &crate::ui::view_state::MouseLayout,
    area: ratatui::layout::Rect,
    x: u16,
    y: u16,
) -> Option<(usize, usize)> {
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }
    let rel_y = (y - area.y) as usize;
    if rel_y == 0 {
        return None;
    }

    let frozen_count = layout.frozen_row_indices.len();
    let data_rel_y = rel_y - 1;
    let target_row = if data_rel_y < frozen_count {
        layout.frozen_row_indices.get(data_rel_y).copied()?
    } else {
        let scroll_rel = data_rel_y - frozen_count;
        layout.scrollable_indices.get(scroll_rel).copied()?
    };

    let idx = resolve_x_to_display_col(layout, x)?;
    let col = *layout.display_cols.get(idx)?;
    Some((target_row, col))
}

/// Move the insert-mode edit cursor to the clicked position within the current cell.
pub(crate) fn move_insert_cursor(
    app: &mut App,
    layout: &crate::ui::view_state::MouseLayout,
    _area: ratatui::layout::Rect,
    x: u16,
) {
    let selected_col = app.view_state.selected_column.get();

    // Find the x start and width of the selected column using col_positions
    let positions = &layout.col_positions;
    let mut col_start_x: u16 = 0;
    let mut col_width: u16 = 0;
    let mut found = false;

    for (i, &dc) in layout.display_cols.iter().enumerate() {
        if dc == selected_col {
            col_start_x = positions.get(i + 1).copied().unwrap_or(0);
            let col_end_x = positions.get(i + 2).copied().unwrap_or(col_start_x);
            col_width = col_end_x.saturating_sub(col_start_x).saturating_sub(1);
            found = true;
            break;
        }
    }

    if !found {
        return;
    }

    let char_offset = if x >= col_start_x {
        (x - col_start_x) as usize
    } else {
        0
    };

    if let Some(ref mut buf) = app.edit_buffer {
        let content_len = buf.content.chars().count();
        let max_width = col_width.saturating_sub(1) as usize;
        let display_len = content_len + 1; // +1 for the cursor '│'

        if display_len <= max_width {
            // No scrolling — account for the '│' indicator
            let new_cursor = if char_offset <= buf.cursor {
                char_offset
            } else {
                (char_offset - 1).min(content_len)
            };
            buf.cursor = new_cursor;
        } else {
            // Scrolled content — compute the visible window
            let cursor_in_display = buf.cursor;
            let half = max_width / 2;
            let window_start = if cursor_in_display <= half {
                0
            } else if cursor_in_display + half >= display_len {
                display_len.saturating_sub(max_width)
            } else {
                cursor_in_display - half
            };

            let pos_in_display = window_start + char_offset;
            let new_cursor = if pos_in_display <= buf.cursor {
                pos_in_display
            } else {
                (pos_in_display - 1).min(content_len)
            };
            buf.cursor = new_cursor;
        }
    }
}
