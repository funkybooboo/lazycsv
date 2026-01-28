use super::utils::column_index_to_letter;
use crate::app::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

/// Render CSV data table with row/column numbers
pub fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let csv = &app.csv_data;

    // Calculate visible columns (max 10)
    let max_visible_cols = 10;
    let start_col = app.horizontal_offset;
    let end_col = (start_col + max_visible_cols).min(csv.column_count());
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
        let display = if i == app.selected_col {
            format!("►{}", letter) // Show indicator on selected column
        } else {
            format!(" {}", letter) // Space for alignment
        };
        let style = if i == app.selected_col {
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

    // Build data rows with row numbers
    let rows = csv.rows.iter().enumerate().map(|(row_idx, row)| {
        let mut cells = vec![Cell::from(format!("{}", row_idx + 1))]; // Row number

        for col_idx in start_col..end_col {
            let cell_value = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");

            // Truncate long text with ...
            let max_cell_width = 20;
            let display_text = if cell_value.len() > max_cell_width {
                format!("{}...", &cell_value[..max_cell_width - 3])
            } else {
                cell_value.to_string()
            };

            // Highlight current cell
            let style = if Some(row_idx) == app.selected_row() && col_idx == app.selected_col {
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
    // Offset by 2 to account for column letters and header rows
    let mut adjusted_state = app.table_state.clone();
    if let Some(selected) = adjusted_state.selected() {
        adjusted_state.select(Some(selected + 2));
    }

    frame.render_stateful_widget(table, area, &mut adjusted_state);
}
