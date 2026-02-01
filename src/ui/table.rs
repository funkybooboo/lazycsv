use super::{utils::column_index_to_letter, MAX_CELL_WIDTH, MAX_VISIBLE_COLS};
use crate::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::borrow::Cow;

/// Render CSV data table with row/column numbers
pub fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let csv = &app.csv_data;

    // Calculate visible columns (max 10)
    let start_col = app.ui.horizontal_offset;
    let end_col = (start_col + MAX_VISIBLE_COLS).min(csv.column_count());
    let visible_col_count = end_col - start_col;

    if visible_col_count == 0 {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" lazycsv: {} ", csv.filename));
        frame.render_widget(block, area);
        return;
    }

    // Build column letters row (A, B, C...) with indicator for selected column
    let mut col_letter_cells = vec![Cell::from("")]; // Empty cell for row numbers column
    for i in start_col..end_col {
        let letter = column_index_to_letter(i);
        let display = if i == app.ui.selected_col {
            format!("►{}", letter) // Show indicator on selected column
        } else {
            format!(" {}", letter) // Space for alignment
        };
        let style = if i == app.ui.selected_col {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };
        col_letter_cells.push(Cell::from(display).style(style));
    }
    let col_letters_row = Row::new(col_letter_cells).height(1);

    // Build header row with column names
    let mut header_cells = vec![Cell::from("#")]; // Row number column header
    for i in start_col..end_col {
        let header_text = csv.get_header(i);
        header_cells
            .push(Cell::from(header_text).style(Style::default().add_modifier(Modifier::BOLD)));
    }
    let header_row = Row::new(header_cells).height(1);

    // Virtual scrolling: Calculate visible viewport to only process visible rows
    let table_height = area
        .height
        .saturating_sub(4) // borders + column letters + header
        .saturating_sub(6) // status bar + file switcher
        as usize;

    // Get current scroll position from table state
    let selected_idx = app.ui.table_state.selected().unwrap_or(0);

    // Calculate scroll offset based on viewport mode (zt, zz, zb, or auto)
    let scroll_offset = match app.ui.viewport_mode {
        crate::app::ViewportMode::Auto => {
            // Auto-center: keep selected row centered when possible
            if selected_idx < table_height / 2 {
                0 // Near top, no scroll
            } else {
                (selected_idx - table_height / 2).min(csv.row_count().saturating_sub(table_height))
            }
        }
        crate::app::ViewportMode::Top => {
            // zt: selected row at top of screen
            selected_idx.min(csv.row_count().saturating_sub(table_height))
        }
        crate::app::ViewportMode::Center => {
            // zz: selected row at center of screen
            if selected_idx < table_height / 2 {
                0
            } else {
                (selected_idx - table_height / 2).min(csv.row_count().saturating_sub(table_height))
            }
        }
        crate::app::ViewportMode::Bottom => {
            // zb: selected row at bottom of screen
            selected_idx.saturating_sub(table_height.saturating_sub(1))
        }
    };

    // Only process visible rows (huge performance improvement for large files)
    let end_row = (scroll_offset + table_height).min(csv.row_count());
    let visible_rows = if scroll_offset < csv.row_count() {
        &csv.rows[scroll_offset..end_row]
    } else {
        &[]
    };

    let rows = visible_rows.iter().enumerate().map(|(idx_in_window, row)| {
        let row_idx = scroll_offset + idx_in_window; // Actual row index in dataset
        let mut cells = vec![Cell::from(format!("{}", row_idx + 1))]; // Row number

        for col_idx in start_col..end_col {
            let cell_value = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");

            // Truncate long text with ...
            let display_text: Cow<'_, str> = if cell_value.len() > MAX_CELL_WIDTH {
                Cow::Owned(format!("{}...", &cell_value[..MAX_CELL_WIDTH - 3]))
            } else {
                Cow::Borrowed(cell_value)
            };

            // Highlight current cell
            let style = if Some(row_idx) == app.selected_row() && col_idx == app.ui.selected_col {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            cells.push(Cell::from(display_text).style(style));
        }

        Row::new(cells).height(1)
    });

    // Calculate column widths
    let mut widths = vec![Constraint::Length(5)]; // Row number column
    let available_width = area
        .width
        .saturating_sub(5)
        .saturating_sub(visible_col_count as u16 + 2);
    let col_width = if visible_col_count > 0 {
        available_width / visible_col_count as u16
    } else {
        10
    };
    for _ in 0..visible_col_count {
        widths.push(Constraint::Length(col_width));
    }

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

    // Render stateful widget
    // With virtual scrolling, we need to adjust the selected position
    // to be relative to the visible window, then offset by 2 for headers
    let mut adjusted_state = app.ui.table_state.clone();
    if let Some(selected) = adjusted_state.selected() {
        // Calculate position within visible window
        let position_in_window = if selected >= scroll_offset && selected < end_row {
            selected - scroll_offset
        } else {
            0
        };
        adjusted_state.select(Some(position_in_window + 2)); // +2 for column letters and header
    }

    frame.render_stateful_widget(table, area, &mut adjusted_state);
}
