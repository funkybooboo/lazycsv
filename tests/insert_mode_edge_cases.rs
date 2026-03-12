//! Edge case tests for Insert Mode functionality
//!
//! This module tests:
//! - Unicode and multi-byte character handling
//! - Cursor position at boundaries (start, end)
//! - Empty cell editing
//! - Very long content
//! - Special CSV characters (quotes, commas, newlines)

use std::io::Write;
use tempfile::NamedTempFile;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::session::FileConfig;
use lazycsv::{App, Document};

/// Create a test app with sample CSV data
fn create_test_app() -> App {
    let csv = "name,value,category\nAlice,100,A\nBob,200,B\nCharlie,300,C\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv.as_bytes()).unwrap();
    let path = temp_file.path().to_path_buf();
    temp_file.keep().unwrap();

    let csv_data = Document::from_file(&path, None, false, None).unwrap();
    let file_config = FileConfig::with_options(None, false, None);
    App::new(csv_data, vec![path], 0, file_config)
}

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

// ============================================================================
// Unicode and Multi-byte Character Tests
// ============================================================================

#[test]
fn test_emoji_insertion_and_cursor_movement() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Insert emoji
    app.handle_key(key_event(KeyCode::Char('🚀'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('🎉'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('🔥'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "🚀🎉🔥");
    assert_eq!(buffer.cursor, 3); // 3 chars, not 12 bytes

    // Move cursor left
    app.handle_key(key_event(KeyCode::Left)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 2);

    // Delete emoji at cursor
    app.handle_key(key_event(KeyCode::Delete)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "🚀🎉");
}

#[test]
fn test_multibyte_unicode_backspace() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type Japanese characters
    for c in "こんにちは".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "こんにちは");
    assert_eq!(buffer.cursor, 5);

    // Backspace should delete one character at a time
    app.handle_key(key_event(KeyCode::Backspace)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "こんにち");
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 4);
}

#[test]
fn test_combining_characters() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type accented characters with combining marks
    for c in "café".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "café");

    // Cursor should be at char count, not byte count
    assert!(buffer.cursor > 0);
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_backspace_at_start_of_content() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    app.handle_key(key_event(KeyCode::Char('x'))).unwrap();

    // Move to start
    app.handle_key(key_event(KeyCode::Home)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 0);

    // Backspace at start should do nothing
    app.handle_key(key_event(KeyCode::Backspace)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "x");
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 0);
}

#[test]
fn test_delete_at_end_of_content() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    app.handle_key(key_event(KeyCode::Char('x'))).unwrap();

    // Already at end, cursor = 1
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 1);

    // Delete at end should do nothing
    app.handle_key(key_event(KeyCode::Delete)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "x");
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 1);
}

#[test]
fn test_ctrl_w_with_only_spaces() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type only spaces
    for _ in 0..5 {
        app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    }

    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "     ");

    // Ctrl+w should delete all trailing spaces
    app.handle_key(ctrl_key_event(KeyCode::Char('w'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "");
    assert_eq!(buffer.cursor, 0);
}

#[test]
fn test_ctrl_w_with_no_content() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Ctrl+w on empty content should do nothing
    app.handle_key(ctrl_key_event(KeyCode::Char('w'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "");
    assert_eq!(buffer.cursor, 0);
}

#[test]
fn test_ctrl_u_on_empty_content() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Ctrl+u on empty content should do nothing
    app.handle_key(ctrl_key_event(KeyCode::Char('u'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "");
    assert_eq!(buffer.cursor, 0);
}

// ============================================================================
// Very Long Content Tests
// ============================================================================

#[test]
fn test_very_long_content_editing() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type 1000 characters
    let long_text = "a".repeat(1000);
    for c in long_text.chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content.len(), 1000);
    assert_eq!(buffer.cursor, 1000);

    // Move to middle
    for _ in 0..500 {
        app.handle_key(key_event(KeyCode::Left)).unwrap();
    }
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 500);

    // Insert at middle
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content.len(), 1001);
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 501);
}

#[test]
fn test_cursor_movement_at_boundaries() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();

    // Move left beyond start (should stay at 0)
    app.handle_key(key_event(KeyCode::Home)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 0);

    app.handle_key(key_event(KeyCode::Left)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 0);

    // Move right beyond end (should stay at char count)
    app.handle_key(key_event(KeyCode::End)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 2);

    app.handle_key(key_event(KeyCode::Right)).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 2);
}

// ============================================================================
// Special CSV Character Tests
// ============================================================================

#[test]
fn test_insert_comma_in_cell() {
    let mut app = create_test_app();
    let row_idx = app.selected_row().unwrap();
    let col_idx = app.view_state.selected_column;

    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type "a,b,c"
    for c in "a,b,c".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "a,b,c");

    // Commit
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Verify cell contains commas (should be quoted when saved)
    let cell_value = app.document.cell(row_idx, col_idx);
    assert_eq!(cell_value, "a,b,c");
}

#[test]
fn test_insert_quotes_in_cell() {
    let mut app = create_test_app();
    let row_idx = app.selected_row().unwrap();
    let col_idx = app.view_state.selected_column;

    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type text with quotes
    for c in "say \"hello\"".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "say \"hello\"");

    // Commit
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    let cell_value = app.document.cell(row_idx, col_idx);
    assert_eq!(cell_value, "say \"hello\"");
}

#[test]
fn test_ctrl_w_multiple_words() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type "one two three"
    for c in "one two three".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    // Delete "three"
    app.handle_key(ctrl_key_event(KeyCode::Char('w'))).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "one two ");

    // Delete "two "
    app.handle_key(ctrl_key_event(KeyCode::Char('w'))).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "one ");

    // Delete "one "
    app.handle_key(ctrl_key_event(KeyCode::Char('w'))).unwrap();
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "");
}
