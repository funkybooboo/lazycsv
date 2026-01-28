use crate::app::App;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render sheet/file switcher at bottom
pub fn render_sheet_switcher(frame: &mut Frame, app: &App, area: Rect) {
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
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    use super::utils::column_index_to_letter;

    let selected_row = app.selected_row().map(|i| i + 1).unwrap_or(0);
    let total_rows = app.csv_data.row_count();
    let col_letter = column_index_to_letter(app.selected_col);
    let col_name = app.csv_data.get_header(app.selected_col);
    let total_cols = app.csv_data.column_count();

    // Get current cell value
    let cell_value = if let Some(row_idx) = app.selected_row() {
        let value = app.csv_data.get_cell(row_idx, app.selected_col);
        if value.is_empty() {
            "<empty>".to_string()
        } else if value.len() > 30 {
            format!("\"{}...\"", &value[..27])
        } else {
            format!("\"{}\"", value)
        }
    } else {
        "<no data>".to_string()
    };

    let status_text = if let Some(ref msg) = app.status_message {
        // Show status message if present
        format!(" {} ", msg)
    } else {
        // Build left side: help, quit, files
        let left_side = if app.csv_files.len() > 1 {
            "[?] help │ [q] quit │ [ ] files"
        } else {
            "[?] help │ [q] quit"
        };

        // Build right side: row, col, cell info
        let right_side = format!(
            "Row {}/{} │ Col {}: {} ({}/{}) │ Cell: {}",
            selected_row,
            total_rows,
            col_letter,
            col_name,
            app.selected_col + 1,
            total_cols,
            cell_value
        );

        format!(" {} │ {}", left_side, right_side)
    };

    let status = Paragraph::new(status_text).style(Style::default());

    frame.render_widget(status, area);
}
