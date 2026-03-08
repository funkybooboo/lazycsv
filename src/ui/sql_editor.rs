//! SQL editor overlay rendering.
//!
//! Displays a centered modal popup for typing and executing SQL queries
//! against loaded CSV tables.

use super::sql_editor_helpers;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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

    // Clear background and render border
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" SQL Query (Enter: execute, Shift+Enter: newline, Esc: cancel) ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split area for query text and optional error line
    let (query_area, error_area) = split_editor_area(inner, sql_error.is_some());

    // Render query text with cursor highlighting
    let lines = sql_editor_helpers::build_cursor_highlighted_lines(sql_buffer, sql_cursor);
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, query_area);

    // Render error message if present
    if let Some(err) = sql_error {
        let error_line = sql_editor_helpers::build_error_line(err);
        let error_paragraph = Paragraph::new(vec![error_line]);
        frame.render_widget(error_paragraph, error_area);
    }
}

/// Split the editor area into query text area and error area.
///
/// Returns (query_area, error_area).
fn split_editor_area(inner: Rect, has_error: bool) -> (Rect, Rect) {
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
    (chunks[0], chunks[1])
}
