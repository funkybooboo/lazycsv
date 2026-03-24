//! Tests for v0.4.0 Insert Mode functionality
//!
//! This module tests:
//! - Entering Insert mode (i, a, I, A, s, F2)
//! - Text editing in Insert mode (typing, backspace, delete, cursor movement)
//! - Committing edits (Enter, Tab, Shift+Enter, Shift+Tab)
//! - Canceling edits (Esc)
//! - Row operations (o, O, dd, yy, p, Delete)

use std::io::Write;
use tempfile::NamedTempFile;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::Mode;
use lazycsv::session::FileConfig;
use lazycsv::{App, ColIndex, Document};

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

/// Create a key event with no modifiers
fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Create a key event with shift modifier
fn shift_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::SHIFT)
}

/// Create a key event with control modifier
fn ctrl_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

// ============================================================================
// Enter Insert Mode Tests
// ============================================================================

#[test]
fn test_i_enters_insert_mode() {
    let mut app = create_test_app();
    assert_eq!(app.mode, Mode::Normal);

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    assert_eq!(app.mode, Mode::Insert);
    assert!(app.edit_buffer.is_some());
}

#[test]
fn test_i_sets_cursor_at_end() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    // Cursor should be at end of content
    assert_eq!(buffer.cursor, buffer.content.len());
}

#[test]
fn test_a_enters_insert_mode() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('a'))).unwrap();
    assert_eq!(app.mode, Mode::Insert);
}

#[test]
fn test_capital_i_cursor_at_start() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('I'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    // Cursor should be at start
    assert_eq!(buffer.cursor, 0);
}

#[test]
fn test_capital_a_cursor_at_end() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('A'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    // Cursor should be at end
    assert_eq!(buffer.cursor, buffer.content.len());
}

#[test]
fn test_s_clears_content_and_enters_insert() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    assert_eq!(app.mode, Mode::Insert);
    let buffer = app.edit_buffer.as_ref().unwrap();
    // Content should be cleared
    assert!(buffer.content.is_empty());
    // Original should be preserved for cancel
    assert!(!buffer.original.is_empty());
    assert_eq!(buffer.cursor, 0);
}

#[test]
fn test_f2_enters_insert_mode() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::F(2))).unwrap();
    assert_eq!(app.mode, Mode::Insert);
}

// ============================================================================
// Insert Mode Text Editing Tests
// ============================================================================

#[test]
fn test_typing_inserts_characters() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap(); // Clear and enter Insert

    app.handle_key(key_event(KeyCode::Char('H'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "Hi");
    assert_eq!(buffer.cursor, 2);
}

#[test]
fn test_backspace_deletes_before_cursor() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();

    // Type some characters
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap();
    let len_before = app.edit_buffer.as_ref().unwrap().content.len();

    // Backspace
    app.handle_key(key_event(KeyCode::Backspace)).unwrap();
    let len_after = app.edit_buffer.as_ref().unwrap().content.len();

    assert_eq!(len_after, len_before - 1);
}

#[test]
fn test_ctrl_h_acts_as_backspace() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('A'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('B'))).unwrap();

    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "AB");

    app.handle_key(ctrl_key_event(KeyCode::Char('h'))).unwrap();

    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "A");
}

#[test]
fn test_delete_key_deletes_at_cursor() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('I'))).unwrap(); // Cursor at start

    // Move to position and delete
    let content_before = app.edit_buffer.as_ref().unwrap().content.clone();
    app.handle_key(key_event(KeyCode::Delete)).unwrap();

    let content_after = app.edit_buffer.as_ref().unwrap().content.clone();
    // First character should be deleted
    assert_eq!(content_after.len(), content_before.len() - 1);
}

#[test]
fn test_left_arrow_moves_cursor() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();

    let cursor_before = app.edit_buffer.as_ref().unwrap().cursor;
    app.handle_key(key_event(KeyCode::Left)).unwrap();
    let cursor_after = app.edit_buffer.as_ref().unwrap().cursor;

    if cursor_before > 0 {
        assert_eq!(cursor_after, cursor_before - 1);
    } else {
        assert_eq!(cursor_after, 0); // Can't go negative
    }
}

#[test]
fn test_right_arrow_moves_cursor() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('I'))).unwrap(); // Cursor at start

    let cursor_before = app.edit_buffer.as_ref().unwrap().cursor;
    let content_len = app.edit_buffer.as_ref().unwrap().content.len();

    app.handle_key(key_event(KeyCode::Right)).unwrap();
    let cursor_after = app.edit_buffer.as_ref().unwrap().cursor;

    if cursor_before < content_len {
        assert_eq!(cursor_after, cursor_before + 1);
    } else {
        assert_eq!(cursor_after, content_len);
    }
}

#[test]
fn test_home_moves_cursor_to_start() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Home)).unwrap();

    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 0);
}

#[test]
fn test_end_moves_cursor_to_end() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('I'))).unwrap(); // Start at beginning
    app.handle_key(key_event(KeyCode::End)).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.cursor, buffer.content.len());
}

#[test]
fn test_ctrl_w_deletes_word() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type "hello world"
    for c in "hello world".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "hello world");

    // Delete word
    app.handle_key(ctrl_key_event(KeyCode::Char('w'))).unwrap();

    // Should delete "world"
    let content = &app.edit_buffer.as_ref().unwrap().content;
    assert!(content.starts_with("hello"));
    assert!(!content.contains("world"));
}

#[test]
fn test_ctrl_u_deletes_to_start() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type text
    for c in "hello".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    app.handle_key(ctrl_key_event(KeyCode::Char('u'))).unwrap();

    // Content should be empty and cursor at 0
    let buffer = app.edit_buffer.as_ref().unwrap();
    assert!(buffer.content.is_empty());
    assert_eq!(buffer.cursor, 0);
}

// ============================================================================
// Commit Edit Tests
// ============================================================================

#[test]
fn test_enter_commits_and_moves_down() {
    let mut app = create_test_app();
    let initial_row = app.selected_row().unwrap().get();

    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should stay in Insert mode (Enter now persists insert mode)
    assert_eq!(app.mode, Mode::Insert);
    // Should have moved down
    let new_row = app.selected_row().unwrap().get();
    assert_eq!(new_row, initial_row + 1);
    // Edit buffer should be populated for the new cell
    assert!(app.edit_buffer.is_some());
}

#[test]
fn test_tab_commits_and_moves_right() {
    let mut app = create_test_app();
    let initial_col = app.view_state.selected_column.get();

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Tab)).unwrap();

    // Should stay in Insert mode (Tab now persists insert mode)
    assert_eq!(app.mode, Mode::Insert);
    let new_col = app.view_state.selected_column.get();
    assert_eq!(new_col, initial_col + 1);
}

#[test]
fn test_shift_enter_commits_and_moves_up() {
    let mut app = create_test_app();
    // Move down first so we can move up
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    let initial_row = app.selected_row().unwrap().get();
    assert!(initial_row > 0);

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(shift_key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.mode, Mode::Normal);
    let new_row = app.selected_row().unwrap().get();
    assert_eq!(new_row, initial_row - 1);
}

#[test]
fn test_shift_tab_commits_and_moves_left() {
    let mut app = create_test_app();
    // Move right first so we can move left
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    let initial_col = app.view_state.selected_column.get();
    assert!(initial_col > 0);

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(shift_key_event(KeyCode::Tab)).unwrap();

    // Should stay in Insert mode
    assert_eq!(app.mode, Mode::Insert);
    let new_col = app.view_state.selected_column.get();
    assert_eq!(new_col, initial_col - 1);
}

#[test]
fn test_commit_sets_dirty_flag() {
    let mut app = create_test_app();
    assert!(!app.document.is_dirty);

    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Document should be marked dirty
    assert!(app.document.is_dirty);
}

#[test]
fn test_commit_unchanged_does_not_set_dirty() {
    let mut app = create_test_app();
    assert!(!app.document.is_dirty);

    // Enter Insert mode without changing anything
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Document should not be dirty
    assert!(!app.document.is_dirty);
}

#[test]
fn test_commit_updates_cell_value() {
    let mut app = create_test_app();
    let row_idx = app.selected_row().unwrap();
    let col_idx = app.view_state.selected_column;

    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('N'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('E'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('W'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    let cell_value = app.document.cell(row_idx, col_idx);
    assert_eq!(cell_value, "NEW");
}

// ============================================================================
// Cancel Edit Tests
// ============================================================================

#[test]
fn test_escape_cancels_edit() {
    let mut app = create_test_app();
    let row_idx = app.selected_row().unwrap();
    let col_idx = app.view_state.selected_column;
    let original_value = app.document.cell(row_idx, col_idx).to_string();

    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap();
    app.handle_key(key_event(KeyCode::Esc)).unwrap();

    // Should be back in Normal mode
    assert_eq!(app.mode, Mode::Normal);
    // Edit buffer should be cleared
    assert!(app.edit_buffer.is_none());
    // Cell value should be unchanged
    let new_value = app.document.cell(row_idx, col_idx);
    assert_eq!(new_value, original_value);
    // Document should not be dirty
    assert!(!app.document.is_dirty);
}

// ============================================================================
// Row Operations Tests
// ============================================================================

#[test]
fn test_o_inserts_row_below() {
    let mut app = create_test_app();
    let initial_row_count = app.document.row_count();
    let initial_row = app.selected_row().unwrap().get();

    app.handle_key(key_event(KeyCode::Char('o'))).unwrap();

    // Row count should increase
    assert_eq!(app.document.row_count(), initial_row_count + 1);
    // Should be in Insert mode
    assert_eq!(app.mode, Mode::Insert);
    // Should be on the new row (one below original)
    assert_eq!(app.selected_row().unwrap().get(), initial_row + 1);
}

#[test]
fn test_capital_o_inserts_row_above() {
    let mut app = create_test_app();
    // Move down first
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    let initial_row_count = app.document.row_count();
    let initial_row = app.selected_row().unwrap().get();

    app.handle_key(key_event(KeyCode::Char('O'))).unwrap();

    // Row count should increase
    assert_eq!(app.document.row_count(), initial_row_count + 1);
    // Should be in Insert mode
    assert_eq!(app.mode, Mode::Insert);
    // Should be on same index (which is now the new row)
    assert_eq!(app.selected_row().unwrap().get(), initial_row);
}

#[test]
fn test_dd_deletes_row() {
    let mut app = create_test_app();
    let initial_row_count = app.document.row_count();

    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    // Row count should decrease
    assert_eq!(app.document.row_count(), initial_row_count - 1);
    // Should have status message
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("deleted"))
        .unwrap_or(false));
    // Document should be dirty
    assert!(app.document.is_dirty);
    // Row should be in clipboard
    assert!(!app.clipboard.is_empty());
}

#[test]
fn test_yy_yanks_row() {
    let mut app = create_test_app();
    let row_idx = app.selected_row().unwrap();
    // With new navigation model, row_idx.get() is already absolute index
    let expected_row: Vec<String> = app
        .document
        .get_rows_range(row_idx.get(), row_idx.get() + 1)[0]
        .clone();

    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();

    // Row should be in clipboard
    assert!(!app.clipboard.is_empty());
    assert_eq!(app.clipboard.as_row().as_ref().unwrap(), &expected_row);
    // Should have status message
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("yanked"))
        .unwrap_or(false));
    // Document should NOT be dirty (yank doesn't modify)
    assert!(!app.document.is_dirty);
}

#[test]
fn test_p_pastes_row_below() {
    let mut app = create_test_app();

    // First yank a row
    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();

    let initial_row_count = app.document.row_count();
    let initial_row = app.selected_row().unwrap().get();

    app.handle_key(key_event(KeyCode::Char('p'))).unwrap();

    // Row count should increase
    assert_eq!(app.document.row_count(), initial_row_count + 1);
    // Should be on the new row (one below original)
    assert_eq!(app.selected_row().unwrap().get(), initial_row + 1);
    // Document should be dirty
    assert!(app.document.is_dirty);
}

#[test]
fn test_p_without_clipboard_shows_error() {
    let mut app = create_test_app();
    assert!(app.clipboard.is_empty());

    app.handle_key(key_event(KeyCode::Char('p'))).unwrap();

    // Should have error message
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Nothing to paste"))
        .unwrap_or(false));
}

#[test]
fn test_delete_key_clears_cell() {
    let mut app = create_test_app();
    let row_idx = app.selected_row().unwrap();
    let col_idx = app.view_state.selected_column;

    // Make sure cell has content
    let original = app.document.cell(row_idx, col_idx).to_string();
    assert!(!original.is_empty());

    app.handle_key(key_event(KeyCode::Delete)).unwrap();

    // Cell should be empty
    let new_value = app.document.cell(row_idx, col_idx);
    assert!(new_value.is_empty());
    // Still in Normal mode
    assert_eq!(app.mode, Mode::Normal);
    // Document should be dirty
    assert!(app.document.is_dirty);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_edit_empty_cell() {
    let mut app = create_test_app();

    // First clear the cell
    app.handle_key(key_event(KeyCode::Delete)).unwrap();
    // Reset dirty flag
    app.document.is_dirty = false;

    // Now edit the empty cell
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert!(buffer.content.is_empty());
    assert!(buffer.original.is_empty());
    assert_eq!(buffer.cursor, 0);
}

#[test]
fn test_backspace_at_start_does_nothing() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('I'))).unwrap(); // Cursor at start

    let content_before = app.edit_buffer.as_ref().unwrap().content.clone();
    app.handle_key(key_event(KeyCode::Backspace)).unwrap();
    let content_after = app.edit_buffer.as_ref().unwrap().content.clone();

    // Content should be unchanged
    assert_eq!(content_before, content_after);
}

#[test]
fn test_delete_at_end_does_nothing() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap(); // Cursor at end

    let content_before = app.edit_buffer.as_ref().unwrap().content.clone();
    app.handle_key(key_event(KeyCode::Delete)).unwrap();
    let content_after = app.edit_buffer.as_ref().unwrap().content.clone();

    // Content should be unchanged
    assert_eq!(content_before, content_after);
}

#[test]
fn test_dd_on_last_row_adjusts_selection() {
    let mut app = create_test_app();

    // Move to last row
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    let last_row = app.selected_row().unwrap().get();
    let initial_count = app.document.row_count();

    // Delete the last row
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    // Row count should decrease
    assert_eq!(app.document.row_count(), initial_count - 1);
    // Selection should adjust to new last row
    let new_selection = app.selected_row().unwrap().get();
    assert!(new_selection < last_row);
    assert_eq!(new_selection, app.document.row_count() - 1);
}

#[test]
fn test_pending_d_can_be_cancelled() {
    let mut app = create_test_app();

    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    assert!(app.input_state.pending_command.is_some());

    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(app.input_state.pending_command.is_none());
}

#[test]
fn test_pending_y_can_be_cancelled() {
    let mut app = create_test_app();

    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();
    assert!(app.input_state.pending_command.is_some());

    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(app.input_state.pending_command.is_none());
}

#[test]
fn test_last_edit_position_tracked() {
    let mut app = create_test_app();
    assert!(app.last_edit_position.is_none());

    // Make an edit at row 0 (index 0), col 0 (app starts at row 0)
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Last edit position should be set to row index 0, col 0
    assert!(app.last_edit_position.is_some());
    let (row, col) = app.last_edit_position.unwrap();
    assert_eq!(row.get(), 0);
    assert_eq!(col.get(), 0);
}

// ============================================================================
// Unicode and Special Character Tests
// ============================================================================

#[test]
fn test_typing_unicode_characters() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    // Type unicode characters
    for c in "日本語".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "日本語");
}

#[test]
fn test_typing_emoji() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    app.handle_key(key_event(KeyCode::Char('👍'))).unwrap();

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert!(buffer.content.contains('👍'));
}

#[test]
fn test_typing_accented_characters() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();

    for c in "café".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }

    let buffer = app.edit_buffer.as_ref().unwrap();
    assert_eq!(buffer.content, "café");
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_tab_at_last_column_wraps_or_stays() {
    let mut app = create_test_app();
    let col_count = app.document.column_count();
    let initial_row_count = app.document.row_count();

    // Move to last column
    for _ in 0..col_count {
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    }

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Tab)).unwrap();

    // Should stay in Insert mode, insert new row, and wrap to first column
    assert_eq!(app.mode, Mode::Insert);
    assert_eq!(app.view_state.selected_column.get(), 0);
    assert_eq!(app.document.row_count(), initial_row_count + 1);
}

#[test]
fn test_shift_tab_at_first_column() {
    let mut app = create_test_app();
    assert_eq!(app.view_state.selected_column.get(), 0);

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(shift_key_event(KeyCode::Tab)).unwrap();

    // Should stay in Insert mode
    assert_eq!(app.mode, Mode::Insert);
    // Column should still be 0 (can't go negative)
    assert_eq!(app.view_state.selected_column.get(), 0);
}

#[test]
fn test_enter_at_last_row() {
    let mut app = create_test_app();
    let initial_row_count = app.document.row_count();

    // Move to last row
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    let last_row = app.selected_row().unwrap().get();

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should stay in Insert mode, insert new row, and move to it
    assert_eq!(app.mode, Mode::Insert);
    assert_eq!(app.selected_row().unwrap().get(), last_row + 1);
    assert_eq!(app.document.row_count(), initial_row_count + 1);
}

#[test]
fn test_shift_enter_at_first_row() {
    let mut app = create_test_app();
    assert_eq!(app.selected_row().unwrap().get(), 0); // App starts at row 0

    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(shift_key_event(KeyCode::Enter)).unwrap();

    // Should be in Normal mode
    assert_eq!(app.mode, Mode::Normal);
    // Should stay at row 0 (can't go above first row)
    assert_eq!(app.selected_row().unwrap().get(), 0);
}

#[test]
fn test_left_at_cursor_zero() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('I'))).unwrap(); // Cursor at start

    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 0);

    // Try to move left
    app.handle_key(key_event(KeyCode::Left)).unwrap();

    // Cursor should still be at 0
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, 0);
}

#[test]
fn test_right_at_cursor_end() {
    let mut app = create_test_app();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap(); // Cursor at end

    let content_len = app.edit_buffer.as_ref().unwrap().content.len();
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, content_len);

    // Try to move right
    app.handle_key(key_event(KeyCode::Right)).unwrap();

    // Cursor should still be at end
    assert_eq!(app.edit_buffer.as_ref().unwrap().cursor, content_len);
}

// ============================================================================
// Sequence and Combination Tests
// ============================================================================

#[test]
fn test_yy_then_dd_updates_clipboard() {
    let mut app = create_test_app();

    // Yank first row
    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();
    let yanked_row = app.clipboard.as_row();

    // Move down and delete
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    // Clipboard should now have the deleted row
    assert!(!app.clipboard.is_empty());
    assert_ne!(app.clipboard.as_row(), yanked_row);
}

#[test]
fn test_multiple_row_inserts() {
    let mut app = create_test_app();
    let initial_count = app.document.row_count();

    // Insert 3 rows
    for _ in 0..3 {
        app.handle_key(key_event(KeyCode::Char('o'))).unwrap();
        app.handle_key(key_event(KeyCode::Esc)).unwrap(); // Cancel to stay in Normal
    }

    assert_eq!(app.document.row_count(), initial_count + 3);
}

#[test]
fn test_multiple_row_deletes() {
    let mut app = create_test_app();
    let initial_count = app.document.row_count();

    // Delete 2 rows
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.row_count(), initial_count - 2);
}

#[test]
fn test_edit_then_navigate_then_edit() {
    let mut app = create_test_app();
    let initial_row = app.selected_row().unwrap();

    // Edit first cell with Tab (stays in Insert mode, moves right)
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('A'))).unwrap();
    app.handle_key(key_event(KeyCode::Tab)).unwrap(); // Commits and moves right, stays in Insert

    // Already in Insert mode on second cell — clear and type B
    // Need to clear existing content first: Ctrl+u clears line in insert mode
    app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL))
        .unwrap();
    app.handle_key(key_event(KeyCode::Char('B'))).unwrap();
    app.handle_key(key_event(KeyCode::Tab)).unwrap(); // Commits and moves right

    // Both cells should be updated on the initial row
    let cell_a = app.document.cell(initial_row, ColIndex::new(0));
    let cell_b = app.document.cell(initial_row, ColIndex::new(1));
    assert_eq!(cell_a, "A");
    assert_eq!(cell_b, "B");
}

#[test]
fn test_insert_mode_preserves_column_position() {
    let mut app = create_test_app();

    // Move to column 2
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    let col_before = app.view_state.selected_column.get();

    // Enter and exit insert mode with Esc
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Esc)).unwrap();

    // Column should be unchanged
    assert_eq!(app.view_state.selected_column.get(), col_before);
}

// ============================================================================
// Document Mutation Tests
// ============================================================================

#[test]
fn test_insert_row_has_correct_column_count() {
    let mut app = create_test_app();
    let col_count = app.document.column_count();

    app.handle_key(key_event(KeyCode::Char('o'))).unwrap();
    app.handle_key(key_event(KeyCode::Esc)).unwrap();

    // New row should have same number of columns
    let new_row_idx = app.selected_row().unwrap();
    // With new navigation model, row_idx.get() is already absolute index
    let new_row = app
        .document
        .get_rows_range(new_row_idx.get(), new_row_idx.get() + 1)[0]
        .clone();
    assert_eq!(new_row.len(), col_count);
}

#[test]
fn test_insert_row_cells_are_empty() {
    let mut app = create_test_app();

    app.handle_key(key_event(KeyCode::Char('o'))).unwrap();
    app.handle_key(key_event(KeyCode::Esc)).unwrap();

    let new_row_idx = app.selected_row().unwrap();
    // With new navigation model, row_idx.get() is already absolute index
    let new_row = app
        .document
        .get_rows_range(new_row_idx.get(), new_row_idx.get() + 1)[0]
        .clone();

    // All cells should be empty
    for cell in &new_row {
        assert!(cell.is_empty());
    }
}

#[test]
fn test_paste_row_content_matches_yanked() {
    let mut app = create_test_app();

    // Move to row 1 (first data row)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // Get the first data row content (row 1)
    let original_row: Vec<String> = app.document.get_rows_range(1, 2)[0].clone();

    // Yank row 1
    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();

    // Move down and paste
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('p'))).unwrap();

    // Get the pasted row using absolute row index
    let pasted_row_idx = app.selected_row().unwrap();
    let pasted_row = app
        .document
        .get_rows_range(pasted_row_idx.get(), pasted_row_idx.get() + 1)[0]
        .clone();

    // Content should match
    assert_eq!(pasted_row, original_row);
}

// ============================================================================
// Cursor Position After Commit Tests
// ============================================================================

#[test]
fn test_cursor_position_after_typing_and_commit() {
    let mut app = create_test_app();

    // Start at row 0, col 0 (app starts at row 0)
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap();
    app.handle_key(key_event(KeyCode::Tab)).unwrap();

    // Should now be at row 0, col 1
    assert_eq!(app.selected_row().unwrap().get(), 0);
    assert_eq!(app.view_state.selected_column.get(), 1);
}

#[test]
fn test_multiple_tab_commits_traverse_row() {
    let mut app = create_test_app();
    let col_count = app.document.column_count();

    // Edit and Tab through columns (stop before last since Tab won't go beyond)
    for col in 0..(col_count - 1) {
        assert_eq!(app.view_state.selected_column.get(), col);
        app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
        app.handle_key(key_event(KeyCode::Tab)).unwrap();
    }

    // Should be at second-to-last column (since we can Tab at most col_count-1 times)
    assert_eq!(app.view_state.selected_column.get(), col_count - 1);
}

#[test]
fn test_multiple_enter_commits_traverse_column() {
    let mut app = create_test_app();
    let row_count = app.document.row_count();

    // Enter insert mode once, then Enter traverses down staying in Insert mode
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    for row in 0..(row_count - 1) {
        assert_eq!(app.selected_row().unwrap().get(), row);
        app.handle_key(key_event(KeyCode::Enter)).unwrap();
        assert_eq!(app.mode, Mode::Insert);
    }

    // Should be at last row, still in Insert mode
    assert_eq!(app.selected_row().unwrap().get(), row_count - 1);
    assert_eq!(app.mode, Mode::Insert);
}

// ============================================================================
// Error and Invalid State Tests
// ============================================================================

#[test]
fn test_invalid_d_sequence_shows_error() {
    let mut app = create_test_app();

    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('x'))).unwrap(); // Invalid after d

    // Should have error message
    assert!(app.status_message.is_some());
}

#[test]
fn test_invalid_y_sequence_shows_error() {
    let mut app = create_test_app();

    app.handle_key(key_event(KeyCode::Char('y'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('x'))).unwrap(); // Invalid after y

    // Should have error message
    assert!(app.status_message.is_some());
}

// ============================================================================
// o/O Edge Cases
// ============================================================================

#[test]
fn test_o_at_last_row_inserts_at_end() {
    let mut app = create_test_app();
    let initial_count = app.document.row_count();

    // Move to last row
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

    app.handle_key(key_event(KeyCode::Char('o'))).unwrap();

    // New row should be at end
    assert_eq!(app.document.row_count(), initial_count + 1);
    assert_eq!(
        app.selected_row().unwrap().get(),
        app.document.row_count() - 1
    );
}

#[test]
fn test_capital_o_at_first_row_inserts_at_beginning() {
    let mut app = create_test_app();
    let initial_count = app.document.row_count();

    // App starts at row 0
    assert_eq!(app.selected_row().unwrap().get(), 0);

    app.handle_key(key_event(KeyCode::Char('O'))).unwrap();

    // New row should be inserted at row 0 (before current row)
    assert_eq!(app.document.row_count(), initial_count + 1);
    assert_eq!(app.selected_row().unwrap().get(), 0);
}

#[test]
fn test_o_enters_insert_mode_preserves_column() {
    let mut app = create_test_app();

    // Move to a different column first
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    let col_before = app.view_state.selected_column.get();
    assert_eq!(col_before, 2);

    app.handle_key(key_event(KeyCode::Char('o'))).unwrap();

    // Should be in Insert mode, column position preserved
    assert_eq!(app.mode, Mode::Insert);
    assert_eq!(app.view_state.selected_column.get(), col_before);
}
