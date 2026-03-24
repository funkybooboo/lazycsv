//! Status bar rendering for the main table view.
//!
//! This module handles rendering the bottom status bar that displays current
//! mode, cursor position, pending commands, and status messages.

use crate::App;
use ratatui::{layout::Rect, style::Style, widgets::Paragraph, Frame};

/// Insert a visible cursor indicator (block) at the given char position in a string.
fn insert_cursor_indicator(text: &str, cursor: usize) -> String {
    let mut result = String::new();
    let char_count = text.chars().count();
    for (i, ch) in text.chars().enumerate() {
        if i == cursor {
            result.push('\u{2588}'); // Full block cursor █
        }
        result.push(ch);
    }
    if cursor >= char_count {
        result.push('\u{2588}');
    }
    result
}

/// Build a status line with left (mode), center (filename), and right (stats + help)
fn build_three_part_status_line(left: &str, center: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let center_len = center.chars().count();
    let right_len = right.chars().count();

    // Calculate center position
    let center_start = (width / 2).saturating_sub(center_len / 2);

    // Calculate padding
    let left_padding = center_start.saturating_sub(left_len);
    let right_start = center_start + center_len;
    let right_padding = width.saturating_sub(right_start).saturating_sub(right_len);

    format!(
        "{}{}{}{}{}",
        left,
        " ".repeat(left_padding),
        center,
        " ".repeat(right_padding),
        right
    )
}

/// Build the right side of the status bar with enhanced position and cell info
fn build_right_side(app: &App) -> String {
    use crate::ui::utils::{column_to_excel_letter, format_number};

    let selected_row = app.selected_row().map(|r| r.get()).unwrap_or(0);
    let col_letter = column_to_excel_letter(app.view_state.selected_column.get());
    let total_rows = app.document.row_count();
    let total_cols = app.document.column_count();

    // Calculate percentage through document (by row)
    let row_percent = if total_rows > 0 {
        ((selected_row as f64 / total_rows.saturating_sub(1).max(1) as f64) * 100.0) as usize
    } else {
        0
    };

    // Get current cell content info
    let cell_info = if let Some(row_idx) = app.selected_row() {
        let cell_content = app.document.cell(row_idx, app.view_state.selected_column);
        let cell_len = cell_content.len();

        if cell_len > 0 {
            // Detect cell type
            let cell_type = if cell_content.chars().all(|c| c.is_ascii_digit() || c == '-') {
                "int"
            } else if cell_content.parse::<f64>().is_ok() {
                "num"
            } else if cell_content.eq_ignore_ascii_case("true")
                || cell_content.eq_ignore_ascii_case("false")
            {
                "bool"
            } else {
                "text"
            };
            format!(" | {}:{}", cell_type, cell_len)
        } else {
            " | empty".to_string()
        }
    } else {
        String::new()
    };

    format!(
        "{}{} | Row {}/{} ({}%) | Col {}/{}{}",
        col_letter,
        format_number(selected_row),
        format_number(selected_row + 1),
        format_number(total_rows),
        row_percent,
        format_number(app.view_state.selected_column.get() + 1),
        format_number(total_cols),
        cell_info
    )
}

/// Build the pending command/count indicator
fn build_pending_indicator(app: &App) -> String {
    match &app.input_state.pending_command {
        Some(crate::input::PendingCommand::G) => "g".to_string(),
        Some(crate::input::PendingCommand::Z) => "z".to_string(),
        Some(crate::input::PendingCommand::GotoColumn(letters)) => format!("g{}", letters),
        Some(crate::input::PendingCommand::D) => "d".to_string(),
        Some(crate::input::PendingCommand::Y) => "y".to_string(),
        Some(crate::input::PendingCommand::C) => "c".to_string(),
        Some(crate::input::PendingCommand::Comma) => ",".to_string(),
        Some(crate::input::PendingCommand::CommaD) => ",d".to_string(),
        Some(crate::input::PendingCommand::CommaY) => ",y".to_string(),
        Some(crate::input::PendingCommand::Space) => "Space".to_string(),
        None => app
            .input_state
            .command_count
            .map(|c| c.to_string())
            .unwrap_or_default(),
    }
}

/// Build the status text based on current mode
fn build_status_text(app: &App, right_side: &str, pending_indicator: &str, width: usize) -> String {
    match app.mode {
        crate::app::Mode::Command => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let buf = &app.input_state.command_buffer;
            let cursor = app.input_state.command_cursor;
            let left = format!(" :{}", insert_cursor_indicator(buf, cursor));
            let center = format!("{}{}", filename, dirty);
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::Search => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let left = format!(" /{}", app.search_buffer);
            let center = format!("{}{}", filename, dirty);
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::Normal => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;

            let left = if let Some(ref msg) = app.status_message {
                format!(" {}", msg.as_str())
            } else if !pending_indicator.is_empty() {
                format!(" {}", pending_indicator)
            } else if let Some(ref search) = app.search_state {
                format!(" /{} {}", search.pattern, search.display_position())
            } else {
                format!(" NORMAL{}", dirty)
            };

            let center = format!("{}{}", filename, dirty);
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::Insert => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let left = format!(" INSERT{}", dirty);
            let center = filename.to_string();
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::Magnifier => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let left = format!(" MAGNIFIER{}", dirty);
            let center = filename.to_string();
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::VisualBlock => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let selection_info = if let Some(sel) = &app.visual_selection {
                let (start_row, end_row, start_col, end_col) = sel.bounds();
                format!(
                    " {}-{},{}-{}",
                    start_row.get() + 1,
                    end_row.get() + 1,
                    crate::ui::utils::column_to_excel_letter(start_col.get()),
                    crate::ui::utils::column_to_excel_letter(end_col.get())
                )
            } else {
                String::new()
            };
            let left = format!(" VISUAL{}{}", selection_info, dirty);
            let center = filename.to_string();
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::VisualLine => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let selection_info = if let Some(sel) = &app.visual_selection {
                let (start_row, end_row, _, _) = sel.bounds();
                format!(" LINE {}-{}", start_row.get() + 1, end_row.get() + 1)
            } else {
                String::new()
            };
            let left = format!(" VISUAL{}{}", selection_info, dirty);
            let center = filename.to_string();
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::VisualColumn => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let selection_info = if let Some(sel) = &app.visual_selection {
                let (_, _, start_col, end_col) = sel.bounds();
                format!(
                    " COLUMN {}-{}",
                    crate::ui::utils::column_to_excel_letter(start_col.get()),
                    crate::ui::utils::column_to_excel_letter(end_col.get())
                )
            } else {
                String::new()
            };
            let left = format!(" VISUAL{}{}", selection_info, dirty);
            let center = filename.to_string();
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::SqlEditor => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let left = if let Some(ref err) = app.sql_error {
                format!(" {}", err)
            } else {
                format!(" SQL EDITOR{}", dirty)
            };
            let center = format!("{}{}", filename, dirty);
            let right = format!("{} | ? for help ", right_side);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::FileList => {
            use crate::input::file_list_mode::scan_directory;

            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let filter = &app.input_state.file_filter_buffer;

            // Count filtered entries in current directory
            let entries = scan_directory(&app.view_state.current_directory).unwrap_or_default();
            let filter_lower = filter.to_lowercase();
            let filtered_count = entries
                .iter()
                .filter(|entry| {
                    if filter.is_empty() {
                        true
                    } else if let Some(name) = entry.filename() {
                        name.to_lowercase().contains(&filter_lower)
                    } else {
                        false
                    }
                })
                .count();

            let selected = app.view_state.file_list_selected + 1;

            let left = if app.input_state.file_list_search_active {
                format!(" /{}", filter)
            } else if !filter.is_empty() {
                format!(" FILE MENU (filtered: {})", filter)
            } else {
                " FILE MENU".to_string()
            };

            let center = format!("{}{}", filename, dirty);
            let right = format!("{} of {} items | ? for help ", selected, filtered_count);
            build_three_part_status_line(&left, &center, &right, width)
        }
        crate::app::Mode::FileOperationPrompt => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let filename = &app.document.filename;
            let left = " FILE OPERATION".to_string();
            let center = format!("{}{}", filename, dirty);
            let right = "Enter: confirm | Esc: cancel ";
            build_three_part_status_line(&left, &center, right, width)
        }
    }
}

/// Render the main status bar showing position information.
///
/// Displays current row/column position and any pending status messages.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `app` - Application state containing cursor position and document data
/// * `area` - The rectangle area to render the status bar within
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let right_side = build_right_side(app);
    let pending_indicator = build_pending_indicator(app);
    let status_text = build_status_text(app, &right_side, &pending_indicator, area.width as usize);

    let style = if app.external_modification_pending {
        super::modal::mode_indicator_style()
    } else {
        Style::default()
    };

    let status = Paragraph::new(status_text).style(style);
    frame.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Mode, VisualMode, VisualSelection};
    use crate::csv::Document;
    use crate::domain::position::{ColIndex, RowIndex};
    use crate::input::PendingCommand;
    use crate::session::FileConfig;
    use crate::App;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::num::NonZeroUsize;
    use std::path::PathBuf;

    fn create_test_app() -> App {
        let document = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![
                vec!["a1".to_string(), "b1".to_string(), "c1".to_string()],
                vec!["a2".to_string(), "".to_string(), "c2".to_string()], // empty cell
                vec![
                    "a3".to_string(),
                    "This is a very long cell value that exceeds the maximum status cell length and should be truncated".to_string(),
                    "c3".to_string(),
                ],
            ],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        App::new(document, csv_files, 0, FileConfig::new())
    }

    #[test]
    fn test_build_right_side_normal_cell() {
        let app = create_test_app();
        let right = build_right_side(&app);
        // Should show Excel-style cell reference with stats
        assert!(right.starts_with("A")); // Column letter first
        assert!(right.contains("Row")); // Contains row info
        assert!(right.contains("Col")); // Contains column info
        assert!(right.contains("%")); // Contains percentage
    }

    #[test]
    fn test_build_right_side_empty_cell() {
        let mut app = create_test_app();
        // Ensure row is selected first
        app.view_state.table_state.select(Some(2)); // Row 2 (has empty cell at B)
        app.view_state.selected_column = ColIndex::new(1); // Column B (empty in row 2)

        let right = build_right_side(&app);
        // Should show Excel-style cell reference with empty cell indicator
        assert!(app.selected_row().is_some());
        assert!(right.starts_with("B2")); // Excel-style format
        assert!(right.contains("empty")); // Empty cell indicator
    }

    #[test]
    fn test_build_right_side_long_cell() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(2)); // Row with long cell
        app.view_state.selected_column = ColIndex::new(1); // Column B (long value)

        let right = build_right_side(&app);
        // Should show position with cell type info
        assert!(right.starts_with("B2")); // Excel-style format
        assert!(right.contains("Row")); // Contains row info
        assert!(right.contains("Col")); // Contains column info
    }

    #[test]
    fn test_build_pending_indicator_g() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::G);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, "g");
    }

    #[test]
    fn test_build_pending_indicator_z() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::Z);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, "z");
    }

    #[test]
    fn test_build_pending_indicator_goto_column() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::GotoColumn("AB".to_string()));

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, "gAB");
    }

    #[test]
    fn test_build_pending_indicator_d() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::D);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, "d");
    }

    #[test]
    fn test_build_pending_indicator_y() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::Y);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, "y");
    }

    #[test]
    fn test_build_pending_indicator_c() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::C);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, "c");
    }

    #[test]
    fn test_build_pending_indicator_comma() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::Comma);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, ",");
    }

    #[test]
    fn test_build_pending_indicator_comma_d() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::CommaD);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, ",d");
    }

    #[test]
    fn test_build_pending_indicator_comma_y() {
        let mut app = create_test_app();
        app.input_state.pending_command = Some(PendingCommand::CommaY);

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, ",y");
    }

    #[test]
    fn test_build_pending_indicator_count() {
        let mut app = create_test_app();
        app.input_state.command_count = Some(NonZeroUsize::new(5).unwrap());

        let indicator = build_pending_indicator(&app);
        assert_eq!(indicator, "5");
    }

    #[test]
    fn test_build_status_text_command_mode() {
        let mut app = create_test_app();
        app.mode = Mode::Command;
        app.input_state.command_buffer = "sort".to_string();
        app.input_state.command_cursor = 4;

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains(":sort"));
    }

    #[test]
    fn test_build_status_text_search_mode() {
        let mut app = create_test_app();
        app.mode = Mode::Search;
        app.search_buffer = "pattern".to_string();

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("/pattern"));
    }

    #[test]
    fn test_build_status_text_insert_mode() {
        let mut app = create_test_app();
        app.mode = Mode::Insert;

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("INSERT"));
    }

    #[test]
    fn test_build_status_text_insert_mode_dirty() {
        let mut app = create_test_app();
        app.mode = Mode::Insert;
        app.document.is_dirty = true;

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("*"));
        assert!(status.contains("INSERT"));
    }

    #[test]
    fn test_build_status_text_magnifier_mode() {
        let mut app = create_test_app();
        app.mode = Mode::Magnifier;

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("MAGNIFIER"));
    }

    #[test]
    fn test_build_status_text_visual_block_mode() {
        let mut app = create_test_app();
        app.mode = Mode::VisualBlock;
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(0),
            ColIndex::new(0),
            VisualMode::Block,
        ));

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("VISUAL"));
        assert!(status.contains("1-1")); // Row range (1-indexed)
        assert!(status.contains("A-A")); // Column range
    }

    #[test]
    fn test_build_status_text_visual_block_mode_dirty() {
        let mut app = create_test_app();
        app.mode = Mode::VisualBlock;
        app.document.is_dirty = true;
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(0),
            ColIndex::new(0),
            VisualMode::Block,
        ));

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("VISUAL"));
        assert!(status.contains("*")); // Dirty indicator
    }

    #[test]
    fn test_build_status_text_visual_line_mode() {
        let mut app = create_test_app();
        app.mode = Mode::VisualLine;
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(0),
            ColIndex::new(0),
            VisualMode::Line,
        ));

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("VISUAL"));
        assert!(status.contains("LINE"));
        assert!(status.contains("1-1")); // Row range
    }

    #[test]
    fn test_build_status_text_visual_line_mode_dirty() {
        let mut app = create_test_app();
        app.mode = Mode::VisualLine;
        app.document.is_dirty = true;
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(0),
            ColIndex::new(0),
            VisualMode::Line,
        ));

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("*")); // Dirty indicator
    }

    #[test]
    fn test_build_status_text_visual_column_mode() {
        let mut app = create_test_app();
        app.mode = Mode::VisualColumn;
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(0),
            ColIndex::new(0),
            VisualMode::Column,
        ));

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("VISUAL"));
        assert!(status.contains("COLUMN"));
        assert!(status.contains("A-A")); // Column range
    }

    #[test]
    fn test_build_status_text_visual_column_mode_dirty() {
        let mut app = create_test_app();
        app.mode = Mode::VisualColumn;
        app.document.is_dirty = true;
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(0),
            ColIndex::new(0),
            VisualMode::Column,
        ));

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("*")); // Dirty indicator
    }

    #[test]
    fn test_build_status_text_sql_editor_mode() {
        let mut app = create_test_app();
        app.mode = Mode::SqlEditor;

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("SQL EDITOR"));
    }

    #[test]
    fn test_build_status_text_sql_editor_mode_with_error() {
        let mut app = create_test_app();
        app.mode = Mode::SqlEditor;
        app.sql_error = Some("Syntax error".to_string());

        let status = build_status_text(&app, "right", "", 80);
        assert!(status.contains("Syntax error"));
    }

    #[test]
    fn test_build_status_text_normal_mode_with_pending() {
        let mut app = create_test_app();
        app.mode = Mode::Normal;

        let status = build_status_text(&app, "right", "5", 80);
        assert!(status.contains("5")); // Pending indicator shown
    }

    #[test]
    fn test_render_external_modification() {
        let mut app = create_test_app();
        app.external_modification_pending = true;

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render(f, &app, area);
            })
            .unwrap();

        // Should render without crashing (green background applied)
    }
}
