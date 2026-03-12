//! Status bar and file switcher rendering.
//!
//! This module handles rendering the bottom status bar showing current cell
//! position, plus the file switcher for multi-file sessions.

use crate::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Build a status line with left and right content, padding between them
fn build_status_line(left: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let total = left_len + right_len + 2; // +2 for spacing

    if total >= width {
        // If too long, truncate left side
        let available = width.saturating_sub(right_len + 2);
        let truncated_left: String = left.chars().take(available).collect();
        format!(" {} {}", truncated_left, right)
    } else {
        let padding = width - total;
        format!(" {}{}{}", left, " ".repeat(padding), right)
    }
}

/// Render the file switcher showing all open CSV files (minimal single-line format).
///
/// Displays a list of all CSV files in the current directory.
/// Format: "file1.csv | file2.csv | file3.csv [1/3]"
/// Active file is shown first/highlighted.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `app` - Application state containing session file list
/// * `area` - The rectangle area to render the switcher within
pub fn render_file_switcher(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};

    if app.session.files().is_empty() {
        return;
    }

    // Split: horizontal rule + file list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Horizontal rule above file list
    let rule = Paragraph::new("─".repeat(area.width as usize));
    frame.render_widget(rule, chunks[0]);

    let dim_style = Style::default().add_modifier(Modifier::DIM);
    let bold_style = Style::default().add_modifier(Modifier::BOLD);
    let available_width = area.width as usize;

    // File count indicator (shown at end)
    let count_indicator = if app.session.files().len() > 1 {
        format!(
            " [{}/{}]",
            app.session.active_file_index() + 1,
            app.session.files().len()
        )
    } else {
        String::new()
    };
    let count_width = count_indicator.len();

    // Calculate position of each file and find current file's position
    let mut file_positions: Vec<(usize, usize)> = Vec::new(); // (start, end) for each file
    let mut pos = 0usize;

    for (idx, path) in app.session.files().iter().enumerate() {
        if idx > 0 {
            pos += 3; // " | "
        }
        let start = pos;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        pos += filename.len();
        file_positions.push((start, pos));
    }

    let total_len = pos;

    // Calculate scroll offset to keep current file visible
    let active_idx = app.session.active_file_index();
    let (active_start, active_end) = file_positions[active_idx];
    let visible_width = available_width.saturating_sub(count_width + 1);

    // Auto-scroll to keep active file visible
    let scroll_offset = if active_end <= visible_width || active_start < visible_width / 2 {
        0 // File fits without scrolling or is near the start
    } else {
        // Scroll to show active file
        active_start.saturating_sub(visible_width / 4)
    };

    // Build visible portion of file list
    let mut spans: Vec<Span> = Vec::new();

    // Add scroll indicator if scrolled
    if scroll_offset > 0 {
        spans.push(Span::styled("< ", dim_style));
    }

    let mut current_pos = 0usize;
    for (idx, path) in app.session.files().iter().enumerate() {
        let separator = if idx > 0 { " | " } else { "" };
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Add dirty indicator if file is dirty
        let dirty_indicator = if app.session.is_dirty(path) { "*" } else { "" };
        let display_name = format!("{}{}", filename, dirty_indicator);

        let sep_start = current_pos;
        let sep_end = sep_start + separator.len();
        let file_start = sep_end;
        let file_end = file_start + display_name.len();

        // Check if this segment is visible
        if file_end > scroll_offset && sep_start < scroll_offset + visible_width {
            // Add separator if visible
            if !separator.is_empty() && sep_end > scroll_offset {
                spans.push(Span::styled(separator.to_string(), dim_style));
            }

            // Add filename if visible
            if file_end > scroll_offset {
                let style = if idx == active_idx {
                    bold_style
                } else {
                    dim_style
                };
                spans.push(Span::styled(display_name, style));
            }
        }

        current_pos = file_end;
    }

    // Add scroll indicator if there's more content
    if total_len > scroll_offset + visible_width {
        spans.push(Span::styled(" >", dim_style));
    }

    // Calculate current display length
    let display_len: usize = spans.iter().map(|s| s.content.len()).sum();

    // Add padding to push count indicator to the right
    let padding_needed = available_width.saturating_sub(display_len + count_width);
    if padding_needed > 0 {
        spans.push(Span::raw(" ".repeat(padding_needed)));
    }

    // Add count indicator
    spans.push(Span::styled(count_indicator, dim_style));

    let line = Line::from(spans);
    let switcher = Paragraph::new(line);
    frame.render_widget(switcher, chunks[1]);
}

/// Build the right side of the status bar (position only, Excel-style)
fn build_right_side(app: &App) -> String {
    use crate::ui::utils::column_to_excel_letter;

    let selected_row = app.selected_row().map(|r| r.get()).unwrap_or(0);
    let col_letter = column_to_excel_letter(app.view_state.selected_column.get());

    format!("{}{}", col_letter, selected_row)
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
            // Show command input: ":sort_" on left, position on right
            let left = format!(":{}", app.input_state.command_buffer);
            build_status_line(&left, right_side, width)
        }
        crate::app::Mode::Search => {
            // Show search input: "/pattern" on left, position on right
            let left = format!("/{}", app.search_buffer);
            build_status_line(&left, right_side, width)
        }
        crate::app::Mode::Normal => {
            // Show notification or mode indicator
            let left = if let Some(ref msg) = app.status_message {
                msg.as_str().to_string()
            } else if !pending_indicator.is_empty() {
                pending_indicator.to_string()
            } else if let Some(ref search) = app.search_state {
                format!("/{} {}", search.pattern, search.display_position())
            } else {
                let dirty = if app.document.is_dirty { "*" } else { "" };
                format!("NORMAL{}", dirty)
            };
            build_status_line(&left, right_side, width)
        }
        crate::app::Mode::Insert => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            build_status_line(&format!("INSERT{}", dirty), right_side, width)
        }
        crate::app::Mode::Magnifier => build_status_line("MAGNIFIER", right_side, width),
        crate::app::Mode::VisualBlock => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
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
            build_status_line(
                &format!("VISUAL{}{}", selection_info, dirty),
                right_side,
                width,
            )
        }
        crate::app::Mode::VisualLine => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
            let selection_info = if let Some(sel) = &app.visual_selection {
                let (start_row, end_row, _, _) = sel.bounds();
                format!(" LINE {}-{}", start_row.get() + 1, end_row.get() + 1)
            } else {
                String::new()
            };
            build_status_line(
                &format!("VISUAL{}{}", selection_info, dirty),
                right_side,
                width,
            )
        }
        crate::app::Mode::VisualColumn => {
            let dirty = if app.document.is_dirty { "*" } else { "" };
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
            build_status_line(
                &format!("VISUAL{}{}", selection_info, dirty),
                right_side,
                width,
            )
        }
        crate::app::Mode::SqlEditor => {
            let left = if let Some(ref err) = app.sql_error {
                err.clone()
            } else {
                "SQL EDITOR".to_string()
            };
            build_status_line(&left, right_side, width)
        }
        crate::app::Mode::FileList => {
            // Show file list with cursor indicator and filter
            let files = app.session.files();
            let filter = &app.input_state.file_filter_buffer;
            let filter_lower = filter.to_lowercase();

            // Filter files based on search
            let filtered_files: Vec<(usize, &std::path::PathBuf)> = files
                .iter()
                .enumerate()
                .filter(|(_, path)| {
                    if filter.is_empty() {
                        true
                    } else {
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_lowercase().contains(&filter_lower))
                            .unwrap_or(false)
                    }
                })
                .collect();

            // Build file list with cursor indicator: "> file1.csv* | file2.csv | file3.csv"
            let mut file_list = String::new();
            let selected_idx = app.view_state.file_list_selected;

            for (display_num, (_orig_idx, path)) in filtered_files.iter().enumerate() {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                // Add dirty indicator
                let dirty = if app.session.is_dirty(path) { "*" } else { "" };

                // Add cursor indicator for selected file
                let cursor = if display_num == selected_idx {
                    "> "
                } else {
                    ""
                };

                // Add separator between files
                if display_num > 0 {
                    file_list.push_str(" | ");
                }

                file_list.push_str(&format!("{}{}{}", cursor, filename, dirty));
            }

            let left = if filter.is_empty() {
                format!(
                    "FILES (j/k or arrows to navigate, Enter to select): {}",
                    file_list
                )
            } else {
                format!(
                    "FILTER: \"{}\" ({} matches): {}",
                    filter,
                    filtered_files.len(),
                    file_list
                )
            };

            build_status_line(&left, "", width)
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
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let right_side = build_right_side(app);
    let pending_indicator = build_pending_indicator(app);
    let status_text = build_status_text(app, &right_side, &pending_indicator, area.width as usize);

    let style = if app.external_modification_pending {
        Style::default().fg(Color::Black).bg(Color::Green)
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
    fn test_build_status_line_normal() {
        let result = build_status_line("LEFT", "RIGHT", 80);
        assert!(result.contains("LEFT"));
        assert!(result.contains("RIGHT"));
        // Line includes leading space, so actual content is 79 chars + leading space
        assert!(result.len() >= 79 && result.len() <= 81);
    }

    #[test]
    fn test_build_status_line_truncation() {
        let result = build_status_line("VERY_LONG_LEFT_SIDE", "RIGHT", 20);
        assert!(result.contains("RIGHT"));
        assert!(result.len() <= 22); // Allow for small variation
    }

    #[test]
    fn test_build_right_side_normal_cell() {
        let app = create_test_app();
        let right = build_right_side(&app);
        // Should show Excel-style cell reference (e.g., "A0")
        assert!(right.starts_with("A")); // Column letter first
        assert!(right.len() < 10); // Reasonable length for position only
    }

    #[test]
    fn test_build_right_side_empty_cell() {
        let mut app = create_test_app();
        // Ensure row is selected first
        app.view_state.table_state.select(Some(1)); // Row 1 (has empty cell at B)
        app.view_state.selected_column = ColIndex::new(1); // Column B (empty in row 1)

        let right = build_right_side(&app);
        // Should show Excel-style cell reference
        assert!(app.selected_row().is_some());
        assert_eq!(right, "B1"); // Excel-style format
    }

    #[test]
    fn test_build_right_side_long_cell() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(2)); // Row with long cell
        app.view_state.selected_column = ColIndex::new(1); // Column B (long value)

        let right = build_right_side(&app);
        // Cell value no longer shown, only position
        assert_eq!(right, "B2"); // Excel-style format
        assert!(right.len() < 10); // Reasonable length for position only
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
        assert!(status.contains("INSERT*"));
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
    fn test_render_status_bar_external_modification() {
        let mut app = create_test_app();
        app.external_modification_pending = true;

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_status_bar(f, &app, area);
            })
            .unwrap();

        // Should render without crashing (green background applied)
    }

    #[test]
    fn test_render_file_switcher_single_file() {
        let app = create_test_app();

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_file_switcher(f, &app, area);
            })
            .unwrap();

        // Should render without crashing
    }

    #[test]
    fn test_render_file_switcher_multiple_files() {
        let document = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test1.csv".to_string(),
        );
        let csv_files = vec![
            PathBuf::from("test1.csv"),
            PathBuf::from("test2.csv"),
            PathBuf::from("test3.csv"),
        ];
        let app = App::new(document, csv_files, 0, FileConfig::new());

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_file_switcher(f, &app, area);
            })
            .unwrap();

        // Should render without crashing and show file count
    }

    #[test]
    fn test_render_file_switcher_many_files_scrolling() {
        let document = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );
        let csv_files: Vec<PathBuf> = (0..50)
            .map(|i| PathBuf::from(format!("file{}.csv", i)))
            .collect();
        let app = App::new(document, csv_files, 25, FileConfig::new()); // Active file in middle

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_file_switcher(f, &app, area);
            })
            .unwrap();

        // Should render without crashing and handle scrolling
    }

    #[test]
    fn test_render_file_switcher_empty_files_list() {
        let document = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );
        let csv_files = vec![]; // Empty list
        let app = App::new(document, csv_files, 0, FileConfig::new());

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render_file_switcher(f, &app, area);
            })
            .unwrap();

        // Should handle empty file list gracefully (early return)
    }
}
