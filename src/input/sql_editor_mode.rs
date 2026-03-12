//! SQL Editor mode input handling
//!
//! This module handles keyboard input when the user is editing SQL queries (after pressing 'q' in Normal mode).

use crate::app::{App, Mode};
use crate::input::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard input in SQL editor mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match (key.code, key.modifiers) {
        (KeyCode::Esc, _) => {
            app.mode = Mode::Normal;
        }
        (KeyCode::Enter, KeyModifiers::SHIFT) => {
            insert_char(app, '\n');
        }
        (KeyCode::Enter, KeyModifiers::NONE) => {
            return execute_query(app);
        }
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            app.sql_error = None;
            insert_char(app, c);
        }
        (KeyCode::Backspace, _) => {
            delete_before_cursor(app);
        }
        (KeyCode::Delete, _) => {
            delete_at_cursor(app);
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.sql_buffer.clear();
            app.sql_cursor = 0;
        }
        (KeyCode::Left, _) => {
            app.sql_cursor = app.sql_cursor.saturating_sub(1);
        }
        (KeyCode::Right, _) => {
            let char_count = app.sql_buffer.chars().count();
            app.sql_cursor = (app.sql_cursor + 1).min(char_count);
        }
        (KeyCode::Up, _) => {
            app.sql_cursor = move_cursor_up(&app.sql_buffer, app.sql_cursor);
        }
        (KeyCode::Down, _) => {
            app.sql_cursor = move_cursor_down(&app.sql_buffer, app.sql_cursor);
        }
        (KeyCode::Home, _) => {
            app.sql_cursor = cursor_to_line_start(&app.sql_buffer, app.sql_cursor);
        }
        (KeyCode::End, _) => {
            app.sql_cursor = cursor_to_line_end(&app.sql_buffer, app.sql_cursor);
        }
        _ => {}
    }
    Ok(InputResult::Continue)
}

/// Move the SQL cursor up one line within the sql_buffer.
fn move_cursor_up(buffer: &str, cursor: usize) -> usize {
    let chars: Vec<char> = buffer.chars().collect();
    // Find position of start of current line
    let mut line_start = cursor;
    while line_start > 0 && chars[line_start - 1] != '\n' {
        line_start -= 1;
    }
    if line_start == 0 {
        return cursor; // Already on first line
    }
    let col = cursor - line_start;
    // Find start of previous line
    let prev_line_end = line_start - 1; // the '\n' char
    let mut prev_line_start = prev_line_end;
    while prev_line_start > 0 && chars[prev_line_start - 1] != '\n' {
        prev_line_start -= 1;
    }
    let prev_line_len = prev_line_end - prev_line_start;
    prev_line_start + col.min(prev_line_len)
}

/// Move the SQL cursor down one line within the sql_buffer.
fn move_cursor_down(buffer: &str, cursor: usize) -> usize {
    let chars: Vec<char> = buffer.chars().collect();
    let total = chars.len();
    // Find start of current line
    let mut line_start = cursor;
    while line_start > 0 && chars[line_start - 1] != '\n' {
        line_start -= 1;
    }
    let col = cursor - line_start;
    // Find end of current line
    let mut line_end = cursor;
    while line_end < total && chars[line_end] != '\n' {
        line_end += 1;
    }
    if line_end >= total {
        return cursor; // Already on last line
    }
    let next_line_start = line_end + 1;
    // Find end of next line
    let mut next_line_end = next_line_start;
    while next_line_end < total && chars[next_line_end] != '\n' {
        next_line_end += 1;
    }
    let next_line_len = next_line_end - next_line_start;
    next_line_start + col.min(next_line_len)
}

/// Insert a character at the SQL cursor position
fn insert_char(app: &mut App, c: char) {
    let byte_pos = app
        .sql_buffer
        .char_indices()
        .nth(app.sql_cursor)
        .map(|(i, _)| i)
        .unwrap_or(app.sql_buffer.len());
    app.sql_buffer.insert(byte_pos, c);
    app.sql_cursor += 1;
}

/// Delete character before cursor in SQL buffer
fn delete_before_cursor(app: &mut App) {
    if app.sql_cursor > 0 {
        app.sql_cursor -= 1;
        let byte_pos = app
            .sql_buffer
            .char_indices()
            .nth(app.sql_cursor)
            .map(|(i, _)| i)
            .unwrap_or(0);
        app.sql_buffer.remove(byte_pos);
    }
}

/// Delete character at cursor in SQL buffer
fn delete_at_cursor(app: &mut App) {
    let char_count = app.sql_buffer.chars().count();
    if app.sql_cursor < char_count {
        let byte_pos = app
            .sql_buffer
            .char_indices()
            .nth(app.sql_cursor)
            .map(|(i, _)| i)
            .unwrap_or(0);
        app.sql_buffer.remove(byte_pos);
    }
}

/// Execute SQL query from buffer
fn execute_query(app: &mut App) -> Result<InputResult> {
    let query = app.sql_buffer.trim().to_string();
    if query.is_empty() {
        app.status_message = Some(StatusMessage::new_owned("Empty query".to_string()));
        app.mode = Mode::Normal;
        return Ok(InputResult::Continue);
    }
    Ok(InputResult::ExecuteQuery { query })
}

/// Move cursor to start of current line
fn cursor_to_line_start(buffer: &str, cursor: usize) -> usize {
    let chars: Vec<char> = buffer.chars().collect();
    let mut pos = cursor;
    while pos > 0 && chars[pos - 1] != '\n' {
        pos -= 1;
    }
    pos
}

/// Move cursor to end of current line
fn cursor_to_line_end(buffer: &str, cursor: usize) -> usize {
    let chars: Vec<char> = buffer.chars().collect();
    let total = chars.len();
    let mut pos = cursor;
    while pos < total && chars[pos] != '\n' {
        pos += 1;
    }
    pos
}
