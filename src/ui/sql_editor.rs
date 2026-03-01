//! SQL editor overlay rendering.
//!
//! Displays a centered modal popup for typing and executing SQL queries
//! against loaded CSV tables.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Width percentage for SQL editor overlay
const SQL_EDITOR_WIDTH_PERCENT: u16 = 70;

/// Height percentage for SQL editor overlay
const SQL_EDITOR_HEIGHT_PERCENT: u16 = 50;

/// Render the SQL editor overlay.
///
/// Displays a centered modal window where the user types SQL queries.
/// The cursor is shown as a reversed-color character.
pub fn render_sql_editor_overlay(
    frame: &mut Frame,
    sql_buffer: &str,
    sql_cursor: usize,
    sql_error: Option<&str>,
) {
    let area = super::help::centered_rect(
        SQL_EDITOR_WIDTH_PERCENT,
        SQL_EDITOR_HEIGHT_PERCENT,
        frame.area(),
    );

    // Clear background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" SQL Query (Enter: execute, Shift+Enter: newline, Esc: cancel) ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area: query text on top, error line at bottom (if error present)
    let has_error = sql_error.is_some();
    let chunks = if has_error {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(0)])
            .split(inner)
    };
    let query_area = chunks[0];
    let error_area = chunks[1];

    // Build text with cursor highlight
    let chars: Vec<char> = sql_buffer.chars().collect();
    let total = chars.len();

    // Split into lines and render with cursor
    let before_cursor: String = chars[..sql_cursor].iter().collect();
    let cursor_char = if sql_cursor < total {
        chars[sql_cursor].to_string()
    } else {
        " ".to_string()
    };
    let after_cursor: String = if sql_cursor < total {
        chars[sql_cursor + 1..].iter().collect()
    } else {
        String::new()
    };

    // Build lines from the content, splitting on newlines
    let full_text = format!("{}\x00{}", before_cursor, after_cursor);
    let parts: Vec<&str> = full_text.split('\x00').collect();
    let before = parts[0];
    let after = parts.get(1).unwrap_or(&"");

    // Build the display with cursor
    // We need to handle multiline content properly
    let before_lines: Vec<&str> = before.split('\n').collect();
    let after_lines: Vec<&str> = after.split('\n').collect();

    let cursor_style = Style::default().bg(Color::White).fg(Color::Black);
    let normal_style = Style::default();

    let mut lines: Vec<Line> = Vec::new();

    if before_lines.len() == 1 && after_lines.len() == 1 {
        // Single line case
        lines.push(Line::from(vec![
            Span::styled(before_lines[0].to_string(), normal_style),
            Span::styled(cursor_char, cursor_style),
            Span::styled(after_lines[0].to_string(), normal_style),
        ]));
    } else {
        // Multiline: cursor is at end of last before_line / start of first after_line
        // Lines before cursor line
        for (i, line) in before_lines.iter().enumerate() {
            if i < before_lines.len() - 1 {
                lines.push(Line::from(Span::styled(line.to_string(), normal_style)));
            }
        }

        // Cursor line: last part of before + cursor char + first part of after
        let cursor_line_before = before_lines.last().unwrap_or(&"");
        let cursor_line_after = after_lines.first().unwrap_or(&"");
        lines.push(Line::from(vec![
            Span::styled(cursor_line_before.to_string(), normal_style),
            Span::styled(cursor_char, cursor_style),
            Span::styled(cursor_line_after.to_string(), normal_style),
        ]));

        // Lines after cursor line
        for line in after_lines.iter().skip(1) {
            lines.push(Line::from(Span::styled(line.to_string(), normal_style)));
        }
    }

    // If buffer is empty, show placeholder
    if sql_buffer.is_empty() {
        lines = vec![Line::from(vec![Span::styled(" ", cursor_style)])];
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, query_area);

    // Render error message at the bottom if present
    if let Some(err) = sql_error {
        let error_style = Style::default().fg(Color::Red);
        let error_line = Line::from(Span::styled(err.to_string(), error_style));
        let error_paragraph = Paragraph::new(vec![error_line]);
        frame.render_widget(error_paragraph, error_area);
    }
}
