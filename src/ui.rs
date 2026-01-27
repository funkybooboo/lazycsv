use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

/// Main UI rendering function
pub fn render(frame: &mut Frame, app: &mut App) {
    // Split terminal into main area + sheet switcher + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Table area
            Constraint::Length(3), // Sheet switcher
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    // Render table with row/column numbers
    render_table(frame, app, chunks[0]);

    // Render sheet switcher (always visible)
    render_sheet_switcher(frame, app, chunks[1]);

    // Render status bar
    render_status_bar(frame, app, chunks[2]);

    // Render help overlay if active
    if app.show_cheatsheet {
        render_cheatsheet(frame);
    }
}

/// Render CSV data table with row/column numbers
fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
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

    // Build column letters row (A, B, C...)
    let mut col_letter_cells = vec![Cell::from("")]; // Empty cell for row numbers column
    for i in start_col..end_col {
        let letter = column_index_to_letter(i);
        col_letter_cells
            .push(Cell::from(letter).style(Style::default().add_modifier(Modifier::DIM)));
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

/// Render sheet/file switcher at bottom
fn render_sheet_switcher(frame: &mut Frame, app: &App, area: Rect) {
    if app.csv_files.is_empty() {
        return;
    }

    let count_info = if app.csv_files.len() > 1 {
        format!(
            "Files ({}/{}): ",
            app.current_file_index + 1,
            app.csv_files.len()
        )
    } else {
        "File: ".to_string()
    };

    // Build file list with indicator
    let mut file_list = String::new();
    for (idx, path) in app.csv_files.iter().enumerate() {
        if idx > 0 {
            file_list.push_str(" | ");
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if idx == app.current_file_index {
            file_list.push_str(&format!("► {}", filename));
        } else {
            file_list.push_str(filename);
        }
    }

    let switcher_text = format!("{}{}", count_info, file_list);

    let switcher = Paragraph::new(switcher_text)
        .block(Block::default().borders(Borders::ALL).title(" Files "))
        .style(Style::default());

    frame.render_widget(switcher, area);
}

/// Render status bar at bottom
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let selected_row = app.selected_row().map(|i| i + 1).unwrap_or(0);
    let total_rows = app.csv_data.row_count();
    let current_col = app.selected_col + 1;
    let total_cols = app.csv_data.column_count();
    let col_name = app.csv_data.get_header(app.selected_col);

    let status_text = if let Some(ref msg) = app.status_message {
        // Show status message if present
        format!(" {} ", msg)
    } else {
        // Default status info
        format!(
            " Row {}/{} │ Col {}/{} {} │ [?] help │ [q] quit {}",
            selected_row,
            total_rows,
            current_col,
            total_cols,
            col_name,
            if app.csv_files.len() > 1 {
                "│ [/]] switch files"
            } else {
                ""
            }
        )
    };

    let status = Paragraph::new(status_text).style(Style::default());

    frame.render_widget(status, area);
}

/// Render centered help overlay
fn render_cheatsheet(frame: &mut Frame) {
    // Create centered area (60% width, 70% height)
    let area = centered_rect(60, 70, frame.area());

    let help_text = vec![
        Line::from(Span::styled(
            "LazyCSV - Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  hjkl / arrows      Move cursor"),
        Line::from("  gg / Home          First row"),
        Line::from("  G / End            Last row"),
        Line::from("  w / b              Next/previous column"),
        Line::from("  0 / $              First/last column"),
        Line::from("  PageUp / PageDown  Page up/down"),
        Line::from("  [ / ]              Previous/next file"),
        Line::from(""),
        Line::from("Future - Cell Editing (Phase 2):"),
        Line::from("  i / Enter          Edit cell (not yet implemented)"),
        Line::from("  Esc                Cancel edit"),
        Line::from("  Ctrl+S             Save file"),
        Line::from(""),
        Line::from("Future - Rows/Columns (Phase 3):"),
        Line::from("  o / O              Add row below/above"),
        Line::from("  dd                 Delete row"),
        Line::from("  yy / p             Copy/paste row"),
        Line::from(""),
        Line::from("Future - Search (Phase 4):"),
        Line::from("  /                  Fuzzy search"),
        Line::from("  s                  Sort by column"),
        Line::from(""),
        Line::from("Other:"),
        Line::from("  ?                  Toggle this help"),
        Line::from("  q                  Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? or Esc to close",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];

    let help =
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title(" Help "));

    // Clear background
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(help, area);
}

/// Helper to create centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Convert column index to letter (0 -> A, 1 -> B, ..., 26 -> AA, etc.)
fn column_index_to_letter(index: usize) -> String {
    let mut result = String::new();
    let mut num = index + 1; // 1-based

    while num > 0 {
        let remainder = (num - 1) % 26;
        result.insert(0, (b'A' + remainder as u8) as char);
        num = (num - 1) / 26;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_index_to_letter() {
        assert_eq!(column_index_to_letter(0), "A");
        assert_eq!(column_index_to_letter(1), "B");
        assert_eq!(column_index_to_letter(25), "Z");
        assert_eq!(column_index_to_letter(26), "AA");
        assert_eq!(column_index_to_letter(27), "AB");
        assert_eq!(column_index_to_letter(51), "AZ");
        assert_eq!(column_index_to_letter(52), "BA");
    }
}
