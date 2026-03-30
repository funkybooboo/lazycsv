//! Main CSV table rendering with virtual scrolling.
//!
//! This module renders the CSV data table with row numbers, column letters,
//! and headers. Implements virtual scrolling for performance with large files.

use super::utils::column_to_excel_letter;
use crate::app::Mode;
use crate::domain::position::{ColIndex, RowIndex};
use crate::session::Session;
use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table},
    Frame,
};

/// Height reserved for title bar, horizontal rule, and column letters
const TABLE_HEADER_HEIGHT: u16 = 3;

/// Height reserved for status bar (1)
const STATUS_BAR_HEIGHT: u16 = 1;

/// Compute the width needed for the row number gutter based on total row count.
/// Adds 1 for a space separator after the number.
fn row_number_col_width(row_count: usize) -> u16 {
    let digits = if row_count == 0 {
        1
    } else {
        row_count.ilog10() as usize + 1
    };
    (digits + 1) as u16
}

/// Calculate cell style based on selection, search matches, and visual mode
///
/// # Arguments
///
/// * `app` - Application state containing visual selection
/// * `search_state` - Optional search state for match highlighting
/// * `row` - Row index of the cell
/// * `col` - Column index of the cell
/// * `is_selected` - Whether this is the currently selected cell
///
/// # Returns
///
/// A `Style` object with appropriate colors and modifiers
fn calculate_cell_style(
    app: &App,
    search_state: Option<&crate::search::SearchState>,
    row: RowIndex,
    col: ColIndex,
    is_selected: bool,
) -> Style {
    let theme = &app.config.theme;
    if is_selected {
        return super::modal::cursor_style_from(theme);
    }

    let base = if is_in_visual_selection(app, row, col) {
        let mut style = super::modal::visual_selection_style_from(theme);
        if let Some(sel) = &app.visual_selection {
            let (_, end_row, _, _) = sel.bounds();
            if row == end_row {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
        }
        style
    } else if search_state
        .map(|s| s.is_current_match(row, col))
        .unwrap_or(false)
    {
        super::modal::search_match_style_from(theme).add_modifier(Modifier::BOLD)
    } else if search_state.map(|s| s.is_match(row, col)).unwrap_or(false) {
        super::modal::visual_selection_style_from(theme)
    } else if app.config.defaults.zebra_striping && row.get().is_multiple_of(2) {
        super::modal::zebra_stripe_style_from(theme)
    } else {
        Style::default()
    };

    // Apply per-column colors only to normal/zebra cells (not visual selection or search matches)
    let is_visual = is_in_visual_selection(app, row, col);
    let is_search = search_state.map(|s| s.is_match(row, col)).unwrap_or(false);
    if !is_visual && !is_search {
        let col_idx = col.get();
        let mut style = base;
        if let Some(&bg) = app.view_state.column_bg_colors.get(&col_idx) {
            style = style.bg(bg);
        }
        if let Some(&fg) = app.view_state.column_fg_colors.get(&col_idx) {
            style = style.fg(fg);
        }
        style
    } else {
        base
    }
}

/// Check if a cell is within the current visual selection
fn is_in_visual_selection(app: &App, row: RowIndex, col: ColIndex) -> bool {
    if let Some(sel) = &app.visual_selection {
        let (start_row, end_row, start_col, end_col) = sel.bounds();
        match sel.mode {
            crate::app::VisualMode::Block => {
                // Rectangular block: both row and column must be in range
                row >= start_row && row <= end_row && col >= start_col && col <= end_col
            }
            crate::app::VisualMode::Line => {
                // Whole rows: only row needs to be in range
                row >= start_row && row <= end_row
            }
            crate::app::VisualMode::Column => {
                // Whole columns: only column needs to be in range
                col >= start_col && col <= end_col
            }
        }
    } else {
        false
    }
}

/// Calculate the visible column range dynamically based on available terminal width.
/// Fits as many columns as possible given their ideal widths.
fn calculate_visible_columns(
    app: &crate::App,
    start_col: usize,
    total_cols: usize,
    available_width: u16,
    row_num_width: u16,
) -> (usize, usize) {
    let usable_width = available_width.saturating_sub(row_num_width);
    let mut used_width: u16 = 0;
    let mut end_col = start_col;

    for col_idx in start_col..total_cols {
        let col_width = ideal_column_width(app, col_idx);
        if used_width + col_width > usable_width && end_col > start_col {
            break;
        }
        used_width += col_width;
        end_col += 1;
    }

    // Always show at least one column
    if end_col == start_col && start_col < total_cols {
        end_col = start_col + 1;
    }

    (start_col, end_col)
}

/// Compute the ideal display width for a single column (used for layout decisions).
fn ideal_column_width(app: &crate::App, col_idx: usize) -> u16 {
    // Manual override
    if let Some(manual_width) = app.session.column_width(col_idx) {
        return manual_width.max(MIN_COLUMN_WIDTH);
    }

    let header_len = app
        .document
        .header(ColIndex::new(col_idx))
        .len()
        .max(column_to_excel_letter(col_idx).len());

    let sample_rows = app
        .document
        .get_rows_range(0, 100.min(app.document.row_count()));
    let max_data_len = sample_rows
        .iter()
        .filter_map(|row| row.get(col_idx))
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);

    let max_width = app.config.defaults.max_column_width;
    let ideal = (header_len.max(max_data_len) + 2) as u16;
    ideal.clamp(MIN_COLUMN_WIDTH, max_width)
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
/// Format edit buffer with cursor, scrolled to keep cursor visible within `max_width`.
fn format_edit_buffer(content: &str, cursor: usize, max_width: usize) -> String {
    // Insert a visible cursor character at cursor position
    let mut with_cursor = String::new();
    for (i, ch) in content.chars().enumerate() {
        if i == cursor {
            with_cursor.push('│'); // Cursor indicator
        }
        with_cursor.push(ch);
    }
    if cursor >= content.chars().count() {
        with_cursor.push('│');
    }

    let total_chars = with_cursor.chars().count();
    if max_width == 0 || total_chars <= max_width {
        return with_cursor;
    }

    // Find cursor position in the with_cursor string (the '│' character)
    let cursor_pos = with_cursor.chars().position(|c| c == '│').unwrap_or(cursor);

    // Calculate a window around the cursor that fits in max_width
    let half = max_width / 2;
    let start = if cursor_pos <= half {
        0
    } else if cursor_pos + half >= total_chars {
        total_chars.saturating_sub(max_width)
    } else {
        cursor_pos - half
    };

    with_cursor.chars().skip(start).take(max_width).collect()
}

/// Minimum column width in characters
const MIN_COLUMN_WIDTH: u16 = 8;

/// Truncation threshold - only truncate truly massive content
const TRUNCATE_THRESHOLD: usize = 100;

/// Build column letters row from an explicit list of column indices.
/// Frozen columns get an underline indicator, typed columns get a type suffix.
fn build_column_letters_row_v2<'a>(
    display_cols: &[usize],
    frozen: &[usize],
    selected_column: ColIndex,
    theme: &crate::config::Theme,
    session: &Session,
    row_num_width: usize,
) -> Row<'a> {
    let base_style = if let Some(bg) = theme.header_bg {
        Style::default().bg(bg)
    } else {
        Style::default()
    };
    let mut cells = vec![Cell::from(" ".repeat(row_num_width)).style(base_style)];

    for &col_idx in display_cols {
        let mut letter = column_to_excel_letter(col_idx);
        // Append type indicator if column has a type set
        if let Some(col_type) = session.column_type(col_idx) {
            let ind = col_type.indicator();
            if !ind.is_empty() {
                letter = format!("{}{}", letter, ind).into();
            }
        }
        let is_frozen = frozen.contains(&col_idx);
        let is_selected = ColIndex::new(col_idx) == selected_column;
        let style = if is_selected {
            let mut s = super::modal::bold_style().patch(base_style);
            if is_frozen {
                s = s.add_modifier(Modifier::UNDERLINED);
            }
            s
        } else if is_frozen {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED)
                .patch(base_style)
        } else {
            super::modal::dim_style().patch(base_style)
        };
        cells.push(Cell::from(letter).style(style));
    }

    Row::new(cells).height(1)
}

/// Calculate column widths from an explicit list of column indices.
fn calculate_column_widths_v2(
    app: &crate::App,
    area: &Rect,
    display_cols: &[usize],
    row_num_width: u16,
) -> (Vec<Constraint>, Vec<u16>) {
    let mut constraints = vec![Constraint::Length(row_num_width)];
    let mut raw_widths = vec![row_num_width];

    let available_width = area.width.saturating_sub(row_num_width);

    if display_cols.is_empty() {
        return (constraints, raw_widths);
    }

    let ideal_widths: Vec<u16> = display_cols
        .iter()
        .map(|&col_idx| ideal_column_width(app, col_idx))
        .collect();

    let total_ideal: u16 = ideal_widths.iter().sum();

    if total_ideal <= available_width {
        for width in ideal_widths {
            constraints.push(Constraint::Length(width));
            raw_widths.push(width);
        }
    } else {
        let scale = available_width as f64 / total_ideal as f64;
        for ideal in ideal_widths {
            let scaled = ((ideal as f64 * scale) as u16).max(MIN_COLUMN_WIDTH);
            constraints.push(Constraint::Length(scaled));
            raw_widths.push(scaled);
        }
    }

    (constraints, raw_widths)
}

/// Build data rows from an explicit list of column indices.
fn build_data_rows_v2(
    app: &App,
    visible_rows: &[Vec<String>],
    scroll_offset: usize,
    display_cols: &[usize],
    column_widths: &[u16],
    row_num_width: usize,
) -> Vec<Row<'static>> {
    let frozen_rows = app.session.frozen_rows();
    let selected_column = app.view_state.selected_column;
    let selected_row_idx = app.selected_row().map(|r| r.get());
    let is_insert_mode = app.mode == Mode::Insert;
    let search_state = app.search_state.as_ref();

    let edit_info = if is_insert_mode {
        app.edit_buffer
            .as_ref()
            .map(|buf| (buf.content.clone(), buf.cursor))
    } else {
        None
    };

    visible_rows
        .iter()
        .enumerate()
        .map(|(idx_in_window, row)| {
            let row_idx = scroll_offset + idx_in_window;
            let is_selected_row = selected_row_idx == Some(row_idx);
            let is_pinned = frozen_rows.contains(&row_idx);

            let num_digits = row_num_width.saturating_sub(1);
            let row_num_display = format!("{:>width$}", row_idx + 1, width = num_digits);
            let mut row_num_style = if is_selected_row {
                super::modal::row_number_style()
            } else if app.config.defaults.zebra_striping && row_idx.is_multiple_of(2) {
                super::modal::zebra_stripe_style_from(&app.config.theme)
            } else {
                Style::default()
            };
            if is_pinned {
                row_num_style = row_num_style
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED);
            }
            let mut cells = vec![Cell::from(row_num_display).style(row_num_style)];

            for (i, &col_idx) in display_cols.iter().enumerate() {
                let is_selected = is_selected_row && ColIndex::new(col_idx) == selected_column;

                let col_width = column_widths
                    .get(i + 1)
                    .copied()
                    .unwrap_or(MIN_COLUMN_WIDTH) as usize;

                let raw_value = if is_selected && is_insert_mode {
                    if let Some((ref content, cursor)) = edit_info {
                        format_edit_buffer(content, cursor, col_width.saturating_sub(1))
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

                let ri = RowIndex::new(row_idx);
                let ci = ColIndex::new(col_idx);
                let mut style = calculate_cell_style(app, search_state, ri, ci, is_selected);
                if is_pinned && !is_selected {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }

                cells.push(Cell::from(display_text).style(style));
            }

            Row::new(cells)
        })
        .collect()
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

    // Compute row number gutter width based on actual row count
    let row_num_width = row_number_col_width(csv.row_count());

    // Frozen columns: always displayed first, before the scrollable region.
    let frozen: Vec<usize> = app
        .session
        .frozen_columns()
        .iter()
        .copied()
        .filter(|&c| c < csv.column_count())
        .collect();
    let frozen_width: u16 = frozen.iter().map(|&c| ideal_column_width(app, c)).sum();

    // Calculate visible scrollable columns (excluding frozen) from remaining width
    let scroll_available = area.width.saturating_sub(frozen_width);
    let start_col = app.view_state.column_scroll_offset;
    let (start_col, end_col) = calculate_visible_columns(
        app,
        start_col,
        csv.column_count(),
        scroll_available,
        row_num_width,
    );

    // Build display_cols: frozen first, then scrollable (skipping any that are frozen)
    let mut display_cols: Vec<usize> = frozen.clone();
    for c in start_col..end_col {
        if !frozen.contains(&c) {
            display_cols.push(c);
        }
    }

    let visible_col_count = display_cols.len();

    // Store for navigation code to use
    app.view_state.visible_cols_count = visible_col_count;

    if visible_col_count == 0 {
        let title = Paragraph::new(format!(" {} (no columns)", csv.filename))
            .style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(title, area);
        return;
    }

    // Build column letters row
    let col_letters_row = build_column_letters_row_v2(
        &display_cols,
        &frozen,
        app.view_state.selected_column,
        &app.config.theme,
        &app.session,
        row_num_width as usize,
    );

    // Frozen rows: always displayed at top, before the scrollable region.
    let frozen_row_indices: Vec<usize> = app
        .session
        .frozen_rows()
        .iter()
        .copied()
        .filter(|&r| r < csv.row_count())
        .collect();
    let frozen_row_count = frozen_row_indices.len();

    // Calculate visible viewport for virtual scrolling
    // Subtract: title(1) + rule(1) + col letters(1) + frozen rows + status bar(1)
    let scrollable_height = area
        .height
        .saturating_sub(TABLE_HEADER_HEIGHT)
        .saturating_sub(STATUS_BAR_HEIGHT)
        .saturating_sub(frozen_row_count as u16) as usize;

    let selected_idx = app.view_state.table_state.selected().unwrap_or(0);

    // Calculate scroll offset based on viewport mode
    let scroll_offset = calculate_scroll_offset(
        selected_idx,
        scrollable_height,
        csv.row_count(),
        &app.view_state.viewport_mode,
    );

    // Calculate column widths first (needed for cell padding)
    let (widths, raw_widths) = calculate_column_widths_v2(app, &area, &display_cols, row_num_width);

    // Build frozen rows (rendered as a separate table above the scrollable table)
    let mut frozen_data_rows: Vec<Row<'static>> = Vec::new();
    for &fr_idx in &frozen_row_indices {
        let fr_rows = csv.get_rows_range(fr_idx, fr_idx + 1);
        let rows = build_data_rows_v2(
            app,
            &fr_rows,
            fr_idx,
            &display_cols,
            &raw_widths,
            row_num_width as usize,
        );
        frozen_data_rows.extend(rows);
    }

    // Build scrollable rows: walk from scroll_offset, skip frozen rows,
    // collect until we fill the scrollable viewport height.
    let data_start = scroll_offset;
    let mut scrollable_indices: Vec<usize> = Vec::with_capacity(scrollable_height);
    let mut idx = data_start;
    while scrollable_indices.len() < scrollable_height && idx < csv.row_count() {
        if !frozen_row_indices.contains(&idx) {
            scrollable_indices.push(idx);
        }
        idx += 1;
    }

    let mut scrollable_data_rows: Vec<Row<'static>> = Vec::new();
    for &row_idx in &scrollable_indices {
        let row_data = csv.get_rows_range(row_idx, row_idx + 1);
        let rows = build_data_rows_v2(
            app,
            &row_data,
            row_idx,
            &display_cols,
            &raw_widths,
            row_num_width as usize,
        );
        scrollable_data_rows.extend(rows);
    }

    // Combine: column letters row + frozen rows + scrollable rows into one table.
    // This avoids ratatui stateful widget scrolling issues with multiple tables.
    let mut all_rows: Vec<Row<'static>> = Vec::new();
    all_rows.push(col_letters_row);
    all_rows.extend(frozen_data_rows);
    all_rows.extend(scrollable_data_rows);

    // Split area: title bar + horizontal rule + table content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Length(1), // Horizontal rule
            Constraint::Min(0),    // Table content
        ])
        .split(area);

    // Title bar: cell value on left (not truncated)
    let cell_value = if let Some(row_idx) = app.selected_row() {
        let content = app.cell_formula_or_value(row_idx, app.view_state.selected_column);
        if content.is_empty() {
            String::new()
        } else {
            content
        }
    } else {
        String::new()
    };

    let title_text = format!(" {}", cell_value);
    let title_bar = Paragraph::new(title_text).style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title_bar, chunks[0]);

    // Horizontal rule
    let rule = Paragraph::new("─".repeat(area.width as usize));
    frame.render_widget(rule, chunks[1]);

    // Render as a plain (non-stateful) widget — we handle scrolling ourselves
    // via scroll_offset and the scrollable_indices list.
    let table = Table::new(all_rows, widths);
    frame.render_widget(table, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use crate::session::FileConfig;
    use crate::ui::ViewportMode;
    use std::path::PathBuf;

    fn create_test_app_with_cols(num_cols: usize) -> App {
        let headers: Vec<String> = (0..num_cols).map(|i| format!("Col{}", i)).collect();
        let mut data_rows = Vec::new();
        for i in 0..5 {
            let row: Vec<String> = (0..num_cols).map(|c| format!("{}", i + c)).collect();
            data_rows.push(row);
        }
        let document = Document::new(headers, data_rows, "test.csv".to_string());
        let csv_files = vec![PathBuf::from("test.csv")];
        App::new(document, csv_files, 0, FileConfig::new())
    }

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
    fn test_calculate_visible_columns_fits_based_on_width() {
        // With a wide terminal, more columns should fit
        let app = create_test_app_with_cols(5);
        let rnw = row_number_col_width(app.document.row_count());
        let (start, end) = calculate_visible_columns(&app, 0, 5, 200, rnw);
        assert_eq!(start, 0);
        assert_eq!(end, 5); // All 5 columns fit in 200 chars
    }

    #[test]
    fn test_calculate_visible_columns_narrow_terminal() {
        let app = create_test_app_with_cols(5);
        // Very narrow — only a couple columns should fit
        let rnw = row_number_col_width(app.document.row_count());
        let (start, end) = calculate_visible_columns(&app, 0, 5, 25, rnw);
        assert_eq!(start, 0);
        assert!(end >= 1); // At least one column always shown
        assert!(end <= 5);
    }

    #[test]
    fn test_calculate_visible_columns_scrolled() {
        let app = create_test_app_with_cols(5);
        let rnw = row_number_col_width(app.document.row_count());
        let (start, end) = calculate_visible_columns(&app, 2, 5, 200, rnw);
        assert_eq!(start, 2);
        assert_eq!(end, 5); // Remaining 3 columns fit
    }

    #[test]
    fn test_calculate_visible_columns_many_cols_wide_terminal() {
        // With 20 columns and a wide terminal, should show more than old limit of 10
        let app = create_test_app_with_cols(20);
        let rnw = row_number_col_width(app.document.row_count());
        let (start, end) = calculate_visible_columns(&app, 0, 20, 250, rnw);
        assert_eq!(start, 0);
        assert!(end > 10); // Should exceed old MAX_VISIBLE_COLS of 10
    }

    #[test]
    fn test_row_number_col_width() {
        // 0 rows → 1 digit placeholder + 1 = 2
        assert_eq!(row_number_col_width(0), 2);
        // 1–9 rows → 1 digit + 1 = 2
        assert_eq!(row_number_col_width(1), 2);
        assert_eq!(row_number_col_width(9), 2);
        // 10–99 rows → 2 digits + 1 = 3
        assert_eq!(row_number_col_width(10), 3);
        assert_eq!(row_number_col_width(99), 3);
        // 100–999 rows → 3 digits + 1 = 4
        assert_eq!(row_number_col_width(100), 4);
        assert_eq!(row_number_col_width(999), 4);
        // 9999 rows → 4 digits + 1 = 5 (old hardcoded constant value)
        assert_eq!(row_number_col_width(9999), 5);
        // 1,048,576 rows (Excel max) → 7 digits + 1 = 8
        assert_eq!(row_number_col_width(1_048_576), 8);
    }
}
