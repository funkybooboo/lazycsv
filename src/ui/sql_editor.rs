//! SQL editor overlay rendering.
//!
//! Displays a centered modal popup for typing and executing SQL queries
//! against loaded CSV tables with full vim editing capabilities.

use crate::vim_editor::{VimEditor, VimMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Width percentage for SQL editor overlay
const SQL_EDITOR_WIDTH_PERCENT: u16 = 80;

/// Height percentage for SQL editor overlay
const SQL_EDITOR_HEIGHT_PERCENT: u16 = 80;

/// Render the SQL editor overlay with vim editing
///
/// Displays a centered modal window where the user can edit SQL queries
/// using full vim modal editing (Normal, Insert, Visual modes).
pub fn render_sql_editor_vim(frame: &mut Frame, vim_editor: &VimEditor, sql_error: Option<&str>) {
    let area = super::help::centered_rect(
        SQL_EDITOR_WIDTH_PERCENT,
        SQL_EDITOR_HEIGHT_PERCENT,
        frame.area(),
    );

    // Clear background and render border with mode indicator
    frame.render_widget(Clear, area);

    let mode_str = vim_editor.mode().display_name();

    let title = format!(" SQL Query - {} ", mode_str);
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split area for query text and status line
    let (query_area, status_area) = split_editor_area(inner);

    // Render query text with line numbers and cursor
    let lines = build_vim_editor_lines(vim_editor);
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, query_area);

    // Render status bar with error/command/help
    let status_line = build_status_line(vim_editor, sql_error, status_area.width as usize);
    let status_paragraph = Paragraph::new(vec![status_line]);
    frame.render_widget(status_paragraph, status_area);
}

/// Build status line with mode/command/error and help tip
fn build_status_line<'a>(
    vim_editor: &VimEditor,
    sql_error: Option<&str>,
    width: usize,
) -> Line<'a> {
    let help_text = "? for help";

    if let Some(err) = sql_error {
        // Error message on left, help on right
        let error_text = format!("Error: {}", err);
        let padding = width.saturating_sub(error_text.len() + help_text.len() + 2);
        Line::from(vec![
            Span::styled(
                error_text,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat(padding)),
            Span::raw(help_text),
        ])
    } else if vim_editor.mode() == VimMode::Command {
        // Command buffer on left, help on right
        let cmd_text = format!(":{}", vim_editor.command_buffer());
        let padding = width.saturating_sub(cmd_text.len() + help_text.len() + 1);
        Line::from(vec![
            Span::styled(cmd_text, Style::default()),
            Span::raw(" ".repeat(padding)),
            Span::raw(help_text),
        ])
    } else {
        // Just help on right
        let padding = width.saturating_sub(help_text.len());
        Line::from(vec![Span::raw(" ".repeat(padding)), Span::raw(help_text)])
    }
}

/// Build display lines from vim editor with line numbers and cursor highlighting
fn build_vim_editor_lines(vim_editor: &VimEditor) -> Vec<Line<'static>> {
    let (cursor_line, cursor_col) = vim_editor.cursor();
    let line_count = vim_editor.line_count();
    let line_num_width = format!("{}", line_count).len();

    let mut display_lines = Vec::new();

    for (line_idx, line_text) in vim_editor.lines().iter().enumerate() {
        let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width);
        let line_num_span = Span::styled(line_num, Style::default().fg(Color::DarkGray));

        if line_idx == cursor_line {
            // This line contains the cursor - highlight cursor position
            let chars: Vec<char> = line_text.chars().collect();
            let mut spans = vec![line_num_span];

            // Text before cursor
            if cursor_col > 0 {
                let before: String = chars[..cursor_col.min(chars.len())].iter().collect();
                spans.push(Span::raw(before));
            }

            // Cursor character (inverted)
            if cursor_col < chars.len() {
                let cursor_char = chars[cursor_col].to_string();
                spans.push(Span::styled(
                    cursor_char,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // Cursor at end of line (show as space)
                spans.push(Span::styled(
                    " ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            // Text after cursor
            if cursor_col + 1 < chars.len() {
                let after: String = chars[cursor_col + 1..].iter().collect();
                spans.push(Span::raw(after));
            }

            display_lines.push(Line::from(spans));
        } else {
            // Regular line without cursor
            display_lines.push(Line::from(vec![
                line_num_span,
                Span::raw(line_text.clone()),
            ]));
        }
    }

    display_lines
}

/// Split the editor area into query text area and status area.
///
/// Returns (query_area, status_area).
fn split_editor_area(inner: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    (chunks[0], chunks[1])
}
