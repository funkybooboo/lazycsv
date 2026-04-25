//! File switcher rendering for multi-file sessions.
//!
//! This module handles rendering a compact file list showing all open CSV files
//! in the current session. The active file is highlighted and the view auto-scrolls
//! to keep it visible.

use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

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
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
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
    let file_positions = calculate_file_positions(app);
    let total_len = file_positions.last().map(|(_, end)| *end).unwrap_or(0);

    // Calculate scroll offset to keep current file visible
    let active_idx = app.session.active_file_index();
    let (active_start, active_end) = file_positions[active_idx];
    let visible_width = available_width.saturating_sub(count_width + 1);

    let scroll_offset = calculate_scroll_offset(active_start, active_end, visible_width);

    // Build visible portion of file list
    let spans = build_file_spans(
        app,
        &file_positions,
        scroll_offset,
        visible_width,
        total_len,
        dim_style,
        bold_style,
    );

    // Calculate current display length
    let display_len: usize = spans.iter().map(|s| s.content.len()).sum();

    // Add padding and count indicator
    let mut final_spans = spans;
    let padding_needed = available_width.saturating_sub(display_len + count_width);
    if padding_needed > 0 {
        final_spans.push(Span::raw(" ".repeat(padding_needed)));
    }
    final_spans.push(Span::styled(count_indicator, dim_style));

    let line = Line::from(final_spans);
    let switcher = Paragraph::new(line);
    frame.render_widget(switcher, chunks[1]);
}

/// Calculate the start and end position of each file in the switcher
fn calculate_file_positions(app: &App) -> Vec<(usize, usize)> {
    let mut file_positions: Vec<(usize, usize)> = Vec::new();
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

    file_positions
}

/// Calculate scroll offset to keep active file visible
fn calculate_scroll_offset(active_start: usize, active_end: usize, visible_width: usize) -> usize {
    if active_end <= visible_width || active_start < visible_width / 2 {
        0 // File fits without scrolling or is near the start
    } else {
        // Scroll to show active file
        active_start.saturating_sub(visible_width / 4)
    }
}

/// Build the spans for the file list with proper styling and scroll indicators
#[allow(clippy::too_many_arguments)]
fn build_file_spans(
    app: &App,
    _file_positions: &[(usize, usize)],
    scroll_offset: usize,
    visible_width: usize,
    total_len: usize,
    dim_style: Style,
    bold_style: Style,
) -> Vec<Span<'static>> {
    let mut spans: Vec<Span> = Vec::new();

    // Add scroll indicator if scrolled
    if scroll_offset > 0 {
        spans.push(Span::styled("< ", dim_style));
    }

    let active_idx = app.session.active_file_index();
    let mut current_pos = 0usize;

    for (idx, path) in app.session.files().iter().enumerate() {
        let separator = if idx > 0 { " | " } else { "" };
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let is_dirty = app.session.is_dirty(path);
        let dirty_indicator = if is_dirty { "*" } else { "" };
        let display_len = filename.len() + dirty_indicator.len();

        let sep_start = current_pos;
        let sep_end = sep_start + separator.len();
        let file_start = sep_end;
        let file_end = file_start + display_len;

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
                spans.push(Span::styled(filename.to_string(), style));
                if is_dirty {
                    let dirty_style = Style::default().fg(app.config.theme.table.dirty_fg);
                    spans.push(Span::styled("*", dirty_style));
                }
            }
        }

        current_pos = file_end;
    }

    // Add scroll indicator if there's more content
    if total_len > scroll_offset + visible_width {
        spans.push(Span::styled(" >", dim_style));
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use crate::session::FileConfig;
    use crate::App;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::PathBuf;

    fn create_test_app() -> App {
        let document = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![
                vec!["a1".to_string(), "b1".to_string(), "c1".to_string()],
                vec!["a2".to_string(), "".to_string(), "c2".to_string()],
            ],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        App::new(document, csv_files, 0, FileConfig::new())
    }

    #[test]
    fn test_render_single_file() {
        let app = create_test_app();

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let area = f.area();
                render(f, &app, area);
            })
            .unwrap();

        // Should render without crashing
    }

    #[test]
    fn test_render_multiple_files() {
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
                render(f, &app, area);
            })
            .unwrap();

        // Should render without crashing and show file count
    }

    #[test]
    fn test_render_many_files_scrolling() {
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
                render(f, &app, area);
            })
            .unwrap();

        // Should render without crashing and handle scrolling
    }

    #[test]
    fn test_render_empty_files_list() {
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
                render(f, &app, area);
            })
            .unwrap();

        // Should handle empty file list gracefully (early return)
    }

    #[test]
    fn test_calculate_file_positions_single() {
        let app = create_test_app();
        let positions = calculate_file_positions(&app);
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].0, 0); // starts at 0
    }

    #[test]
    fn test_calculate_scroll_offset_no_scroll_needed() {
        let offset = calculate_scroll_offset(0, 10, 100);
        assert_eq!(offset, 0); // File fits, no scroll
    }

    #[test]
    fn test_calculate_scroll_offset_scroll_needed() {
        let offset = calculate_scroll_offset(200, 210, 50);
        assert!(offset > 0); // Should scroll to show active file
    }
}
