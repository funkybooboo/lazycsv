//! Main CSV table rendering with virtual scrolling.
//!
//! This module renders the CSV data table with row numbers, column letters,
//! and headers. Implements virtual scrolling for performance with large files.

use super::{utils::column_to_excel_letter, MAX_VISIBLE_COLS};
use crate::app::Mode;
use crate::domain::position::ColIndex;
use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table},
    Frame,
};

/// Height reserved for title bar, horizontal rule, column letters, and header row
const TABLE_HEADER_HEIGHT: u16 = 4;

/// Height reserved for status bar (1) and file switcher (2)
const STATUS_BAR_HEIGHT: u16 = 3;

/// Width allocated for the row number column
const ROW_NUMBER_COLUMN_WIDTH: u16 = 5;

/// Offset added to selected position to account for column letters and header rows
const HEADER_ROW_OFFSET: usize = 2;

/// Calculate the visible column range based on horizontal scroll offset
fn calculate_visible_columns(start_col: usize, total_cols: usize) -> (usize, usize) {
    let end_col = (start_col + MAX_VISIBLE_COLS).min(total_cols);
    (start_col, end_col)
}

/// Build the column letters row (A, B, C...) with highlighting for selected column
fn build_column_letters_row<'a>(
    start_col: usize,
    end_col: usize,
    selected_column: ColIndex,
) -> Row<'a> {
    let mut col_letter_cells = vec![Cell::from("    ")]; // Align with row numbers column

    for i in start_col..end_col {
        let letter = column_to_excel_letter(i);
        let col_idx = ColIndex::new(i);
        let style = if col_idx == selected_column {
            // Highlight selected column with bold only
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };
        col_letter_cells.push(Cell::from(letter).style(style));
    }

    Row::new(col_letter_cells).height(1)
}

/// Build the header row with column names
fn build_header_row<'a>(app: &'a App, start_col: usize, end_col: usize) -> Row<'a> {
    let mut header_cells = vec![Cell::from("")]; // Empty cell for row number column

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

/// Format edit buffer content with visible cursor
fn format_edit_buffer(content: &str, cursor: usize) -> String {
    // Insert a visible cursor character at cursor position
    let mut result = String::new();
    for (i, ch) in content.chars().enumerate() {
        if i == cursor {
            result.push('│'); // Cursor indicator
        }
        result.push(ch);
    }
    // If cursor is at end of content
    if cursor >= content.chars().count() {
        result.push('│');
    }
    result
}

/// Build data rows with proper styling for the current selection
fn build_data_rows(
    app: &App,
    visible_rows: &[Vec<String>],
    scroll_offset: usize,
    start_col: usize,
    end_col: usize,
    column_widths: &[u16],
) -> Vec<Row<'static>> {
    let selected_column = app.view_state.selected_column;
    let selected_row_idx = app.get_selected_row().map(|r| r.get());
    let is_insert_mode = app.mode == Mode::Insert;

    // Get edit buffer content if in Insert mode
    let edit_content = if is_insert_mode {
        app.edit_buffer
            .as_ref()
            .map(|buf| format_edit_buffer(&buf.content, buf.cursor))
    } else {
        None
    };

    visible_rows
        .iter()
        .enumerate()
        .map(|(idx_in_window, row)| {
            let row_idx = scroll_offset + idx_in_window;
            let is_selected_row = selected_row_idx == Some(row_idx);

            // Row number: bold for selected row, normal for others
            let row_num_display = format!("{:>4}", row_idx + 1);
            let row_num_style = if is_selected_row {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mut cells = vec![Cell::from(row_num_display).style(row_num_style)];

            for (i, col_idx) in (start_col..end_col).enumerate() {
                let is_selected = is_selected_row && ColIndex::new(col_idx) == selected_column;

                // Get column width (skip first element which is row number column)
                let col_width = column_widths
                    .get(i + 1)
                    .copied()
                    .unwrap_or(MIN_COLUMN_WIDTH) as usize;

                // Show edit buffer content when editing this cell
                let raw_value = if is_selected && is_insert_mode {
                    if let Some(ref content) = edit_content {
                        content.clone()
                    } else {
                        row.get(col_idx).cloned().unwrap_or_default()
                    }
                } else {
                    row.get(col_idx).cloned().unwrap_or_default()
                };

                // Truncate only truly massive content
                let cell_value = if raw_value.chars().count() > TRUNCATE_THRESHOLD {
                    let truncated: String =
                        raw_value.chars().take(TRUNCATE_THRESHOLD - 3).collect();
                    format!("{}...", truncated)
                } else {
                    raw_value
                };

                // Pad content to fill column width for consistent highlighting
                let display_text = if is_selected {
                    // Pad to column width minus 1 for some margin
                    let char_count = cell_value.chars().count();
                    let pad_width = col_width.saturating_sub(1);
                    if char_count < pad_width {
                        format!("{}{}", cell_value, " ".repeat(pad_width - char_count))
                    } else {
                        cell_value
                    }
                } else {
                    cell_value
                };

                // Highlight current cell with background color
                let style = if is_selected {
                    Style::default().bg(Color::White).fg(Color::Black)
                } else {
                    Style::default()
                };

                cells.push(Cell::from(display_text).style(style));
            }

            Row::new(cells).height(1)
        })
        .collect()
}

/// Minimum column width in characters
const MIN_COLUMN_WIDTH: u16 = 8;

/// Maximum column width in characters (generous to avoid truncation)
const MAX_COLUMN_WIDTH: u16 = 100;

/// Truncation threshold - only truncate truly massive content
const TRUNCATE_THRESHOLD: usize = 100;

/// Calculate column widths based on content
/// Returns (constraints for Table widget, raw widths in characters)
fn calculate_column_widths(
    app: &crate::App,
    area: &Rect,
    start_col: usize,
    end_col: usize,
) -> (Vec<Constraint>, Vec<u16>) {
    let mut constraints = vec![Constraint::Length(ROW_NUMBER_COLUMN_WIDTH)];
    let mut raw_widths = vec![ROW_NUMBER_COLUMN_WIDTH];

    // Calculate available width for data columns
    let available_width = area.width.saturating_sub(ROW_NUMBER_COLUMN_WIDTH);
    let visible_col_count = end_col - start_col;

    if visible_col_count == 0 {
        return (constraints, raw_widths);
    }

    // Calculate ideal width for each column based on content
    let mut ideal_widths: Vec<u16> = Vec::with_capacity(visible_col_count);
    for col_idx in start_col..end_col {
        // Get header width
        let header_len = app
            .document
            .get_header(ColIndex::new(col_idx))
            .len()
            .max(column_to_excel_letter(col_idx).len());

        // Sample data rows to find max width (sample first 100 rows for performance)
        let max_data_len = app
            .document
            .rows
            .iter()
            .take(100)
            .filter_map(|row| row.get(col_idx))
            .map(|s| s.chars().count()) // Use char count for unicode support
            .max()
            .unwrap_or(0);

        // Calculate ideal width with min/max constraints
        let ideal = (header_len.max(max_data_len) + 2) as u16; // +2 for padding
        let constrained = ideal.clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH);
        ideal_widths.push(constrained);
    }

    // Calculate total ideal width
    let total_ideal: u16 = ideal_widths.iter().sum();

    // If we have room, use ideal widths; otherwise scale proportionally
    if total_ideal <= available_width {
        // Use ideal widths
        for width in ideal_widths {
            constraints.push(Constraint::Length(width));
            raw_widths.push(width);
        }
    } else {
        // Scale down proportionally to fit available space
        let scale = available_width as f64 / total_ideal as f64;
        for ideal in ideal_widths {
            let scaled = ((ideal as f64 * scale) as u16).max(MIN_COLUMN_WIDTH);
            constraints.push(Constraint::Length(scaled));
            raw_widths.push(scaled);
        }
    }

    (constraints, raw_widths)
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
        let title = Paragraph::new(format!(" lazycsv: {} (no columns)", csv.filename))
            .style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(title, area);
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

    // Calculate column widths first (needed for cell padding)
    let (widths, raw_widths) = calculate_column_widths(app, &area, start_col, end_col);

    // Build data rows with column widths for proper cell padding
    let rows = build_data_rows(
        app,
        visible_rows,
        scroll_offset,
        start_col,
        end_col,
        &raw_widths,
    );

    // Combine column letters + headers + data
    let all_rows = std::iter::once(col_letters_row)
        .chain(std::iter::once(header_row))
        .chain(rows);

    // Split area: title bar + horizontal rule + table content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Length(1), // Horizontal rule
            Constraint::Min(0),    // Table content
        ])
        .split(area);

    // Title bar: filename left, row count right
    let dirty_indicator = if csv.is_dirty { "*" } else { "" };
    let title_left = format!(" lazycsv: {}{}", csv.filename, dirty_indicator);
    let title_right = format!("{}/{} ", selected_idx + 1, csv.row_count());
    let title_padding = (area.width as usize)
        .saturating_sub(title_left.len())
        .saturating_sub(title_right.len());
    let title_text = format!("{}{}{}", title_left, " ".repeat(title_padding), title_right);
    let title_bar = Paragraph::new(title_text).style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title_bar, chunks[0]);

    // Horizontal rule (using unicode box-drawing character)
    let rule = Paragraph::new("─".repeat(area.width as usize));
    frame.render_widget(rule, chunks[1]);

    // Create table widget without borders
    let table = Table::new(all_rows, widths);

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

    frame.render_stateful_widget(table, chunks[2], &mut adjusted_state);
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
        let expected = (50_isize - (table_height as isize - 1)).max(0) as usize;
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
