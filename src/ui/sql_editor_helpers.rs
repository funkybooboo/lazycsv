//! Helper functions for SQL editor rendering.
//!
//! Extracted from render_sql_editor_overlay to keep functions under 50 lines.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Build text lines with cursor highlighting for SQL editor.
///
/// Returns the lines ready to render with ratatui.
pub fn build_cursor_highlighted_lines(sql_buffer: &str, sql_cursor: usize) -> Vec<Line<'static>> {
    let cursor_style = Style::default().bg(Color::White).fg(Color::Black);
    let normal_style = Style::default();

    // Handle empty buffer
    if sql_buffer.is_empty() {
        return vec![Line::from(vec![Span::styled(" ", cursor_style)])];
    }

    let chars: Vec<char> = sql_buffer.chars().collect();
    let total = chars.len();

    // Extract text before cursor, cursor char, and text after cursor
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

    build_multiline_with_cursor(
        &before_cursor,
        &cursor_char,
        &after_cursor,
        cursor_style,
        normal_style,
    )
}

/// Build multiline text with cursor highlighting.
///
/// Splits text by newlines and inserts cursor at the correct position.
fn build_multiline_with_cursor(
    before: &str,
    cursor_char: &str,
    after: &str,
    cursor_style: Style,
    normal_style: Style,
) -> Vec<Line<'static>> {
    let before_lines: Vec<&str> = before.split('\n').collect();
    let after_lines: Vec<&str> = after.split('\n').collect();

    // Single line case
    if before_lines.len() == 1 && after_lines.len() == 1 {
        return vec![Line::from(vec![
            Span::styled(before_lines[0].to_string(), normal_style),
            Span::styled(cursor_char.to_string(), cursor_style),
            Span::styled(after_lines[0].to_string(), normal_style),
        ])];
    }

    // Multiline case
    let mut lines: Vec<Line> = Vec::new();

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
        Span::styled(cursor_char.to_string(), cursor_style),
        Span::styled(cursor_line_after.to_string(), normal_style),
    ]));

    // Lines after cursor line
    for line in after_lines.iter().skip(1) {
        lines.push(Line::from(Span::styled(line.to_string(), normal_style)));
    }

    lines
}

/// Build error message paragraph for SQL editor.
pub fn build_error_line(error_msg: &str) -> Line<'static> {
    let error_style = Style::default().fg(Color::Red);
    Line::from(Span::styled(error_msg.to_string(), error_style))
}
