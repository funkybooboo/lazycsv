//! Main CSV table rendering with virtual scrolling.
//!
//! This module renders the CSV data table with row numbers, column letters,
//! and headers. Implements virtual scrolling for performance with large files.

use super::{utils::column_to_excel_letter, MAX_CELL_WIDTH, MAX_VISIBLE_COLS};
use crate::domain::position::ColIndex;
use crate::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::borrow::Cow;

/// Height reserved for table borders, column letters, and header row
const TABLE_HEADER_HEIGHT: u16 = 4;

/// Height reserved for status bar and file switcher
const STATUS_BAR_HEIGHT: u16 = 6;

/// Width allocated for the row number column
const ROW_NUMBER_COLUMN_WIDTH: u16 = 5;

/// Offset added to selected position to account for column letters and header rows
const HEADER_ROW_OFFSET: usize = 2;

/// Calculate the visible column range based on horizontal scroll offset
fn calculate_visible_columns(start_col: usize, total_cols: usize) -> (usize, usize) {
    let end_col = (start_col + MAX_VISIBLE_COLS).min(total_cols);
    (start_col, end_col)
}

/// Build the column letters row (A, B, C...) with indicator for selected column
fn build_column_letters_row<'a>(
    start_col: usize,
    end_col: usize,
    selected_column: ColIndex,
) -> Row<'a> {
    let mut col_letter_cells = vec![Cell::from("")]; // Empty cell for row numbers column

    for i in start_col..end_col {
        let letter = column_to_excel_letter(i);
        let col_idx = ColIndex::new(i);
        let display = if col_idx == selected_column {
            format!("►{}", letter) // Show indicator on selected column
        } else {
            format!(" {}", letter) // Space for alignment
        };
        let style = if col_idx == selected_column {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };
        col_letter_cells.push(Cell::from(display).style(style));
    }

    Row::new(col_letter_cells).height(1)
}

/// Build the header row with column names
fn build_header_row<'a>(app: &'a App, start_col: usize, end_col: usize) -> Row<'a> {
    let mut header_cells = vec![Cell::from("#")]; // Row number column header

    for i in start_col..end_col {
        let header_text = app.document.get_header(ColIndex::new(i));
        header_cells
            .push(Cell::from(header_text).style(Style::default().add_modifier(Modifier::BOLD)));
    }

    Row::new(header_cells).height(1)
}

/// Calculate scroll offset based on viewport mode and selected row
fn calculate_scroll_offset(
    selected_idx: usize,
    table_height: usize,
    total_rows: usize,
    viewport_mode: &crate::ui::ViewportMode,
) -> usize {
    match viewport_mode {
        crate::ui::ViewportMode::Auto => {
            // Auto-center: keep selected row centered when possible
            if selected_idx < table_height / 2 {
                0 // Near top, no scroll
            } else {
                (selected_idx - table_height / 2).min(total_rows.saturating_sub(table_height))
            }
        }
        crate::ui::ViewportMode::Top => {
            // zt: selected row at top of screen
            selected_idx.min(total_rows.saturating_sub(table_height))
        }
        crate::ui::ViewportMode::Center => {
            // zz: selected row at center of screen
            if selected_idx < table_height / 2 {
                0
            } else {
                (selected_idx - table_height / 2).min(total_rows.saturating_sub(table_height))
            }
        }
        crate::ui::ViewportMode::Bottom => {
            // zb: selected row at bottom of screen
            selected_idx.saturating_sub(table_height.saturating_sub(1))
        }
    }
}

/// Build data rows with proper styling for the current selection
fn build_data_rows<'a>(
    app: &App,
    visible_rows: &'a [Vec<String>],
    scroll_offset: usize,
    start_col: usize,
    end_col: usize,
) -> impl Iterator<Item = Row<'a>> + 'a {
    let selected_column = app.view_state.selected_column;
    let selected_row_idx = app.get_selected_row().map(|r| r.get());

    visible_rows
        .iter()
        .enumerate()
        .map(move |(idx_in_window, row)| {
            let row_idx = scroll_offset + idx_in_window;
            let mut cells = vec![Cell::from(format!("{}", row_idx + 1))]; // Row number

            for col_idx in start_col..end_col {
                let cell_value = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");

                // Truncate long text with ellipsis
                let display_text: Cow<'_, str> = if cell_value.len() > MAX_CELL_WIDTH {
                    Cow::Owned(format!("{}...", &cell_value[..MAX_CELL_WIDTH - 3]))
                } else {
                    Cow::Borrowed(cell_value)
                };

                // Highlight current cell
                let is_selected =
                    selected_row_idx == Some(row_idx) && ColIndex::new(col_idx) == selected_column;
                let style = if is_selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };

                cells.push(Cell::from(display_text).style(style));
            }

            Row::new(cells).height(1)
        })
}

/// Calculate column widths for the table
fn calculate_column_widths(area: &Rect, visible_col_count: usize) -> Vec<Constraint> {
    let mut widths = vec![Constraint::Length(ROW_NUMBER_COLUMN_WIDTH)];
    let available_width = area
        .width
        .saturating_sub(ROW_NUMBER_COLUMN_WIDTH)
        .saturating_sub(visible_col_count as u16 + 2);
    let col_width = if visible_col_count > 0 {
        available_width / visible_col_count as u16
    } else {
        10
    };
    for _ in 0..visible_col_count {
        widths.push(Constraint::Length(col_width));
    }
    widths
}

/// Render the main CSV table with virtual scrolling support.
///
///This function renders the complete table including column letters (A, B, C...),
/// headers, row numbers, and data cells. Uses virtual scrolling to efficiently
/// handle large files by only rendering visible rows.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `app` - Application state containing the CSV data and view state
/// * `area` - The rectangle area to render the table within
pub fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let csv = &app.document;

    // Calculate visible columns
    let start_col = app.view_state.column_scroll_offset;
    let (start_col, end_col) = calculate_visible_columns(start_col, csv.column_count());
    let visible_col_count = end_col - start_col;

    if visible_col_count == 0 {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" lazycsv: {} ", csv.filename));
        frame.render_widget(block, area);
        return;
    }

    // Build column letters and header rows
    let col_letters_row =
        build_column_letters_row(start_col, end_col, app.view_state.selected_column);
    let header_row = build_header_row(app, start_col, end_col);

    // Calculate visible viewport for virtual scrolling
    let table_height = area
        .height
        .saturating_sub(TABLE_HEADER_HEIGHT)
        .saturating_sub(STATUS_BAR_HEIGHT) as usize;

    let selected_idx = app.view_state.table_state.selected().unwrap_or(0);

    // Calculate scroll offset based on viewport mode
    let scroll_offset = calculate_scroll_offset(
        selected_idx,
        table_height,
        csv.row_count(),
        &app.view_state.viewport_mode,
    );

    // Get visible rows for current viewport
    let end_row = (scroll_offset + table_height).min(csv.row_count());
    let visible_rows = if scroll_offset < csv.row_count() {
        &csv.rows[scroll_offset..end_row]
    } else {
        &[]
    };

    // Build data rows
    let rows = build_data_rows(app, visible_rows, scroll_offset, start_col, end_col);

    // Calculate column widths
    let widths = calculate_column_widths(&area, visible_col_count);

    // Combine column letters + headers + data
    let all_rows = std::iter::once(col_letters_row)
        .chain(std::iter::once(header_row))
        .chain(rows);

    // Create table widget
    let table = Table::new(all_rows, widths)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " lazycsv: {}{} ",
            csv.filename,
            if csv.is_dirty { "*" } else { "" }
        )))
        .highlight_symbol("► ");

    // Render stateful widget with adjusted selection state
    // Virtual scrolling requires adjusting the selected position to be relative
    // to the visible window, plus offset for column letters and header rows
    let mut adjusted_state = app.view_state.table_state.clone();
    if let Some(selected) = adjusted_state.selected() {
        let position_in_window = if selected >= scroll_offset && selected < end_row {
            selected - scroll_offset
        } else {
            0
        };
        adjusted_state.select(Some(position_in_window + HEADER_ROW_OFFSET));
    }

    frame.render_stateful_widget(table, area, &mut adjusted_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::ViewportMode;

    #[test]
    fn test_calculate_scroll_offset_auto_mode_near_top() {
        let selected_idx = 5;
        let table_height = 20;
        let total_rows = 100;

        let offset =
            calculate_scroll_offset(selected_idx, table_height, total_rows, &ViewportMode::Auto);

        // Near top, should not scroll
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_scroll_offset_auto_mode_centered() {
        let selected_idx = 50;
        let table_height = 20;
        let total_rows = 100;

        let offset =
            calculate_scroll_offset(selected_idx, table_height, total_rows, &ViewportMode::Auto);

        // Should center around selected row
        let expected = 50 - 20 / 2; // 40
        assert_eq!(offset, expected);
    }

    #[test]
    fn test_calculate_scroll_offset_auto_mode_near_bottom() {
        let selected_idx = 95;
        let table_height = 20;
        let total_rows = 100;

        let offset =
            calculate_scroll_offset(selected_idx, table_height, total_rows, &ViewportMode::Auto);

        // Should clamp to max scroll (100 - 20 = 80)
        assert_eq!(offset, 80);
    }

    #[test]
    fn test_calculate_scroll_offset_top_mode() {
        let selected_idx = 50;
        let table_height = 20;
        let total_rows = 100;

        let offset =
            calculate_scroll_offset(selected_idx, table_height, total_rows, &ViewportMode::Top);

        // Selected row should be at top
        assert_eq!(offset, 50);
    }

    #[test]
    fn test_calculate_scroll_offset_top_mode_at_end() {
        let selected_idx = 95;
        let table_height = 20;
        let total_rows = 100;

        let offset =
            calculate_scroll_offset(selected_idx, table_height, total_rows, &ViewportMode::Top);

        // Should clamp to max scroll
        assert_eq!(offset, 80);
    }

    #[test]
    fn test_calculate_scroll_offset_center_mode() {
        let selected_idx = 50;
        let table_height = 20;
        let total_rows = 100;

        let offset = calculate_scroll_offset(
            selected_idx,
            table_height,
            total_rows,
            &ViewportMode::Center,
        );

        // Should center around selected row
        let expected = 50 - 20 / 2; // 40
        assert_eq!(offset, expected);
    }

    #[test]
    fn test_calculate_scroll_offset_center_mode_near_start() {
        let selected_idx = 5;
        let table_height = 20;
        let total_rows = 100;

        let offset = calculate_scroll_offset(
            selected_idx,
            table_height,
            total_rows,
            &ViewportMode::Center,
        );

        // Can't center at start, should be 0
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_scroll_offset_bottom_mode() {
        let selected_idx = 50;
        let table_height = 20;
        let total_rows = 100;

        let offset = calculate_scroll_offset(
            selected_idx,
            table_height,
            total_rows,
            &ViewportMode::Bottom,
        );

        // Selected row should be at bottom (50 - (20 - 1) = 31)
        let expected = (50 as isize - (table_height as isize - 1)).max(0) as usize;
        assert_eq!(offset, expected);
    }

    #[test]
    fn test_calculate_scroll_offset_bottom_mode_near_start() {
        let selected_idx = 5;
        let table_height = 20;
        let total_rows = 100;

        let offset = calculate_scroll_offset(
            selected_idx,
            table_height,
            total_rows,
            &ViewportMode::Bottom,
        );

        // Can't position at bottom near start, should be 0
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_scroll_offset_small_table() {
        let selected_idx = 2;
        let table_height = 10;
        let total_rows = 5;

        // All modes should return 0 when table fits on screen
        assert_eq!(
            calculate_scroll_offset(selected_idx, table_height, total_rows, &ViewportMode::Auto),
            0
        );
        assert_eq!(
            calculate_scroll_offset(selected_idx, table_height, total_rows, &ViewportMode::Top),
            0
        );
        assert_eq!(
            calculate_scroll_offset(
                selected_idx,
                table_height,
                total_rows,
                &ViewportMode::Center
            ),
            0
        );
        assert_eq!(
            calculate_scroll_offset(
                selected_idx,
                table_height,
                total_rows,
                &ViewportMode::Bottom
            ),
            0
        );
    }

    #[test]
    fn test_calculate_visible_columns_normal() {
        let (start, end) = calculate_visible_columns(0, 50);
        assert_eq!(start, 0);
        assert!(end <= 50);
        assert!(end <= start + MAX_VISIBLE_COLS);
    }

    #[test]
    fn test_calculate_visible_columns_scrolled() {
        let (start, end) = calculate_visible_columns(10, 50);
        assert_eq!(start, 10);
        assert!(end <= 50);
        assert_eq!(end - start, MAX_VISIBLE_COLS.min(50 - 10));
    }

    #[test]
    fn test_calculate_visible_columns_at_end() {
        let total_cols = 30;
        let start_col = 25;
        let (start, end) = calculate_visible_columns(start_col, total_cols);
        assert_eq!(start, 25);
        assert_eq!(end, 30);
        assert!(end - start <= MAX_VISIBLE_COLS);
    }
}
