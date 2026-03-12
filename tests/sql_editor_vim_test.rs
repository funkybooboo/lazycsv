//! Integration tests for SQL Editor with Vim editing capabilities (v0.11.0)
//!
//! Tests vim modal editing in SQL query editor: Normal, Insert, Visual modes,
//! navigation, editing, search, undo/redo, and query execution.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::vim_editor::{VimEditor, VimMode};

// ============================================================================
// Helper Functions
// ============================================================================

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn key_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

fn key_char(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

// ============================================================================
// Basic Mode Transitions (5 tests)
// ============================================================================

#[test]
fn test_sql_editor_starts_in_normal_mode() {
    let editor = VimEditor::new("SELECT * FROM table".to_string());
    assert_eq!(editor.mode(), VimMode::Normal);
}

#[test]
fn test_enter_insert_mode_with_i() {
    let mut editor = VimEditor::new("SELECT".to_string());
    assert_eq!(editor.mode(), VimMode::Normal);

    editor.handle_key(key_char('i'));
    assert_eq!(editor.mode(), VimMode::Insert);
}

#[test]
fn test_exit_insert_mode_with_esc() {
    let mut editor = VimEditor::new("SELECT".to_string());
    editor.handle_key(key_char('i')); // Enter insert
    assert_eq!(editor.mode(), VimMode::Insert);

    editor.handle_key(key(KeyCode::Esc));
    assert_eq!(editor.mode(), VimMode::Normal);
}

#[test]
fn test_enter_visual_mode_with_v() {
    let mut editor = VimEditor::new("SELECT".to_string());
    assert_eq!(editor.mode(), VimMode::Normal);

    editor.handle_key(key_char('v'));
    assert_eq!(editor.mode(), VimMode::Visual);
}

#[test]
fn test_enter_command_mode_with_colon() {
    let mut editor = VimEditor::new("SELECT".to_string());
    assert_eq!(editor.mode(), VimMode::Normal);

    editor.handle_key(key_char(':'));
    assert_eq!(editor.mode(), VimMode::Command);
}

// ============================================================================
// Navigation Tests (8 tests)
// ============================================================================

#[test]
fn test_hjkl_navigation() {
    let mut editor = VimEditor::new("SELECT *\nFROM table".to_string());
    assert_eq!(editor.cursor(), (0, 0));

    // l: move right
    editor.handle_key(key_char('l'));
    assert_eq!(editor.cursor(), (0, 1));

    // j: move down
    editor.handle_key(key_char('j'));
    assert_eq!(editor.cursor(), (1, 1));

    // h: move left
    editor.handle_key(key_char('h'));
    assert_eq!(editor.cursor(), (1, 0));

    // k: move up
    editor.handle_key(key_char('k'));
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_arrow_key_navigation() {
    let mut editor = VimEditor::new("SELECT *\nFROM table".to_string());

    editor.handle_key(key(KeyCode::Right));
    assert_eq!(editor.cursor(), (0, 1));

    editor.handle_key(key(KeyCode::Down));
    assert_eq!(editor.cursor(), (1, 1));

    editor.handle_key(key(KeyCode::Left));
    assert_eq!(editor.cursor(), (1, 0));

    editor.handle_key(key(KeyCode::Up));
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_word_navigation() {
    let mut editor = VimEditor::new("SELECT COUNT FROM table".to_string());
    assert_eq!(editor.cursor(), (0, 0));

    // w: next word
    editor.handle_key(key_char('w'));
    assert_eq!(editor.cursor(), (0, 7)); // "COUNT"

    editor.handle_key(key_char('w'));
    assert_eq!(editor.cursor(), (0, 13)); // "FROM"

    // b: previous word
    editor.handle_key(key_char('b'));
    assert_eq!(editor.cursor(), (0, 7)); // Back to "COUNT"
}

#[test]
fn test_line_start_end_navigation() {
    let mut editor = VimEditor::new("SELECT * FROM table".to_string());
    editor.handle_key(key_char('$')); // End of line
    assert_eq!(editor.cursor(), (0, 18)); // Last char

    editor.handle_key(key_char('0')); // Start of line
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_file_start_end_navigation() {
    let mut editor = VimEditor::new("SELECT *\nFROM table\nWHERE id = 1".to_string());

    editor.handle_key(key_char('G')); // End of file
    assert_eq!(editor.cursor(), (2, 0));

    editor.handle_key(key_char('g'));
    editor.handle_key(key_char('g')); // Start of file
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_count_prefix_navigation() {
    let mut editor = VimEditor::new("SELECT * FROM table".to_string());

    // 5l: move right 5 times
    editor.handle_key(key_char('5'));
    editor.handle_key(key_char('l'));
    assert_eq!(editor.cursor(), (0, 5));
}

#[test]
fn test_multiline_navigation() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());

    // Navigate to line 2
    editor.handle_key(key_char('j'));
    assert_eq!(editor.cursor(), (1, 0));

    // Navigate to line 3
    editor.handle_key(key_char('j'));
    assert_eq!(editor.cursor(), (2, 0));

    // Try to go past end (should stay at line 3)
    editor.handle_key(key_char('j'));
    assert_eq!(editor.cursor(), (2, 0));
}

#[test]
fn test_navigation_preserves_column() {
    let mut editor = VimEditor::new("SELECT COUNT\nFROM table".to_string());

    // Move to column 5
    for _ in 0..5 {
        editor.handle_key(key_char('l'));
    }
    assert_eq!(editor.cursor(), (0, 5));

    // Move down - should try to preserve column
    editor.handle_key(key_char('j'));
    // Line 2 is "FROM table", column 5 is valid
    assert_eq!(editor.cursor(), (1, 5));
}

// ============================================================================
// Insert Mode Editing (6 tests)
// ============================================================================

#[test]
fn test_insert_characters() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('i')); // Enter insert mode
    editor.handle_key(key_char(' '));
    editor.handle_key(key_char('*'));

    assert_eq!(editor.content(), " *SELECT");
}

#[test]
fn test_append_mode() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('a')); // Append after cursor
    assert_eq!(editor.mode(), VimMode::Insert);
    editor.handle_key(key_char(' '));
    editor.handle_key(key_char('*'));

    assert_eq!(editor.content(), "S *ELECT");
}

#[test]
fn test_append_at_line_end() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('A')); // Append at end of line
    assert_eq!(editor.mode(), VimMode::Insert);
    editor.handle_key(key_char(' '));
    editor.handle_key(key_char('*'));

    assert_eq!(editor.content(), "SELECT *");
}

#[test]
fn test_open_line_below() {
    let mut editor = VimEditor::new("SELECT *".to_string());

    editor.handle_key(key_char('o')); // Open line below
    assert_eq!(editor.mode(), VimMode::Insert);
    editor.handle_key(key_char('F'));
    editor.handle_key(key_char('R'));
    editor.handle_key(key_char('O'));
    editor.handle_key(key_char('M'));

    assert_eq!(editor.content(), "SELECT *\nFROM");
}

#[test]
fn test_open_line_above() {
    let mut editor = VimEditor::new("FROM table".to_string());

    editor.handle_key(key_char('O')); // Open line above
    assert_eq!(editor.mode(), VimMode::Insert);
    editor.handle_key(key_char('S'));
    editor.handle_key(key_char('E'));
    editor.handle_key(key_char('L'));
    editor.handle_key(key_char('E'));
    editor.handle_key(key_char('C'));
    editor.handle_key(key_char('T'));

    assert_eq!(editor.content(), "SELECT\nFROM table");
}

#[test]
fn test_backspace_in_insert_mode() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('A')); // Append at end
    editor.handle_key(key_char('X'));
    editor.handle_key(key(KeyCode::Backspace));

    assert_eq!(editor.content(), "SELECT");
}

// ============================================================================
// Delete Operations (5 tests)
// ============================================================================

#[test]
fn test_delete_char_with_x() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('x')); // Delete 'S'
    assert_eq!(editor.content(), "ELECT");
}

#[test]
fn test_delete_line_with_dd() {
    let mut editor = VimEditor::new("SELECT *\nFROM table\nWHERE id = 1".to_string());

    editor.handle_key(key_char('d'));
    editor.handle_key(key_char('d')); // Delete first line

    assert_eq!(editor.content(), "FROM table\nWHERE id = 1");
}

#[test]
fn test_delete_multiple_lines() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3\nLine 4".to_string());

    editor.handle_key(key_char('2')); // Count prefix
    editor.handle_key(key_char('d'));
    editor.handle_key(key_char('d')); // Delete 2 lines

    assert_eq!(editor.content(), "Line 3\nLine 4");
}

#[test]
fn test_delete_preserves_clipboard() {
    let mut editor = VimEditor::new("SELECT *\nFROM table".to_string());

    editor.handle_key(key_char('d'));
    editor.handle_key(key_char('d')); // Delete first line

    // Paste below
    editor.handle_key(key_char('p'));

    assert_eq!(editor.content(), "FROM table\nSELECT *");
}

#[test]
fn test_delete_last_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());

    editor.handle_key(key_char('j')); // Move to line 2
    editor.handle_key(key_char('d'));
    editor.handle_key(key_char('d'));

    assert_eq!(editor.content(), "Line 1");
}

// ============================================================================
// Yank and Paste Operations (4 tests)
// ============================================================================

#[test]
fn test_yank_line_with_yy() {
    let mut editor = VimEditor::new("SELECT *\nFROM table".to_string());

    editor.handle_key(key_char('y'));
    editor.handle_key(key_char('y')); // Yank first line

    editor.handle_key(key_char('p')); // Paste below

    assert_eq!(editor.content(), "SELECT *\nSELECT *\nFROM table");
}

#[test]
fn test_paste_below() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());

    editor.handle_key(key_char('y'));
    editor.handle_key(key_char('y'));

    editor.handle_key(key_char('j')); // Move to line 2
    editor.handle_key(key_char('p')); // Paste below line 2

    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 1");
}

#[test]
fn test_paste_above() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());

    editor.handle_key(key_char('y'));
    editor.handle_key(key_char('y'));

    editor.handle_key(key_char('j')); // Move to line 2
    editor.handle_key(key_char('P')); // Paste above line 2

    assert_eq!(editor.content(), "Line 1\nLine 1\nLine 2");
}

#[test]
fn test_multiple_paste() {
    let mut editor = VimEditor::new("Line 1".to_string());

    editor.handle_key(key_char('y'));
    editor.handle_key(key_char('y'));

    editor.handle_key(key_char('p')); // Paste once
    editor.handle_key(key_char('p')); // Paste twice

    assert_eq!(editor.content(), "Line 1\nLine 1\nLine 1");
}

// ============================================================================
// Undo/Redo Operations (4 tests)
// ============================================================================

#[test]
fn test_undo_insert() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('A'));
    editor.handle_key(key_char(' '));
    editor.handle_key(key_char('*'));
    editor.handle_key(key(KeyCode::Esc));

    assert_eq!(editor.content(), "SELECT *");

    editor.handle_key(key_char('u')); // Undo
    assert_eq!(editor.content(), "SELECT");
}

#[test]
fn test_undo_delete() {
    let mut editor = VimEditor::new("SELECT *\nFROM table".to_string());

    editor.handle_key(key_char('d'));
    editor.handle_key(key_char('d'));

    assert_eq!(editor.content(), "FROM table");

    editor.handle_key(key_char('u')); // Undo
    assert_eq!(editor.content(), "SELECT *\nFROM table");
}

#[test]
fn test_redo() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('x')); // Delete 'S'
    assert_eq!(editor.content(), "ELECT");

    editor.handle_key(key_char('u')); // Undo
    assert_eq!(editor.content(), "SELECT");

    editor.handle_key(key_with_mod(KeyCode::Char('r'), KeyModifiers::CONTROL)); // Redo
    assert_eq!(editor.content(), "ELECT");
}

#[test]
fn test_multiple_undo() {
    let mut editor = VimEditor::new("A".to_string());

    editor.handle_key(key_char('A'));
    editor.handle_key(key_char('B'));
    editor.handle_key(key(KeyCode::Esc));

    editor.handle_key(key_char('A'));
    editor.handle_key(key_char('C'));
    editor.handle_key(key(KeyCode::Esc));

    assert_eq!(editor.content(), "ABC");

    editor.handle_key(key_char('u')); // Undo C
    assert_eq!(editor.content(), "AB");

    editor.handle_key(key_char('u')); // Undo B
    assert_eq!(editor.content(), "A");
}

// ============================================================================
// Visual Mode Operations (3 tests)
// ============================================================================

#[test]
fn test_visual_mode_yank() {
    let mut editor = VimEditor::new("SELECT *\nFROM table".to_string());

    editor.handle_key(key_char('v')); // Enter visual
    assert_eq!(editor.mode(), VimMode::Visual);

    // Select some text
    editor.handle_key(key_char('l'));
    editor.handle_key(key_char('l'));

    editor.handle_key(key_char('y')); // Yank selection
    assert_eq!(editor.mode(), VimMode::Normal);
}

#[test]
fn test_visual_line_mode() {
    let mut editor = VimEditor::new("SELECT *\nFROM table\nWHERE id = 1".to_string());

    editor.handle_key(key_char('V')); // Enter visual line
    assert_eq!(editor.mode(), VimMode::VisualLine);

    editor.handle_key(key_char('j')); // Select 2 lines

    editor.handle_key(key_char('d')); // Delete selection
    assert_eq!(editor.content(), "WHERE id = 1");
}

#[test]
fn test_exit_visual_mode_with_esc() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char('v'));
    assert_eq!(editor.mode(), VimMode::Visual);

    editor.handle_key(key(KeyCode::Esc));
    assert_eq!(editor.mode(), VimMode::Normal);
}

// ============================================================================
// Search Operations (4 tests)
// ============================================================================

#[test]
fn test_search_word_under_cursor() {
    let mut editor = VimEditor::new("SELECT id FROM table WHERE id = 1".to_string());

    // Move to first 'id'
    for _ in 0..7 {
        editor.handle_key(key_char('l'));
    }

    editor.handle_key(key_char('*')); // Search word under cursor
    assert_eq!(editor.search_pattern(), Some("id"));
    assert_eq!(editor.search_match_count(), 2);
}

#[test]
fn test_jump_to_next_match() {
    let mut editor = VimEditor::new("SELECT id FROM table WHERE id = 1".to_string());

    for _ in 0..7 {
        editor.handle_key(key_char('l'));
    }

    editor.handle_key(key_char('*'));
    let first_pos = editor.cursor();

    editor.handle_key(key_char('n')); // Next match
    assert!(editor.cursor() != first_pos);
}

#[test]
fn test_jump_to_prev_match() {
    let mut editor = VimEditor::new("SELECT id FROM table WHERE id = 1".to_string());

    for _ in 0..7 {
        editor.handle_key(key_char('l'));
    }

    editor.handle_key(key_char('*'));
    editor.handle_key(key_char('n')); // Go to next
    let second_pos = editor.cursor();

    editor.handle_key(key_char('N')); // Previous match
    assert!(editor.cursor() != second_pos);
}

#[test]
fn test_search_persists_across_edits() {
    let mut editor = VimEditor::new("SELECT id FROM id".to_string());

    for _ in 0..7 {
        editor.handle_key(key_char('l'));
    }

    editor.handle_key(key_char('*')); // Search for 'id'
    assert_eq!(editor.search_match_count(), 2);

    // Make an edit
    editor.handle_key(key_char('i'));
    editor.handle_key(key_char('X'));
    editor.handle_key(key(KeyCode::Esc));

    // Search should still be active
    assert_eq!(editor.search_pattern(), Some("id"));
}

// ============================================================================
// Ex Command Operations (5 tests)
// ============================================================================

#[test]
fn test_enter_command_mode() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char(':'));
    assert_eq!(editor.mode(), VimMode::Command);
}

#[test]
fn test_command_buffer_input() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char(':'));
    editor.handle_key(key_char('w'));
    editor.handle_key(key_char('q'));

    assert_eq!(editor.command_buffer(), "wq");
}

#[test]
fn test_command_backspace() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char(':'));
    editor.handle_key(key_char('w'));
    editor.handle_key(key_char('q'));
    editor.handle_key(key(KeyCode::Backspace));

    assert_eq!(editor.command_buffer(), "w");
}

#[test]
fn test_execute_noh_command() {
    let mut editor = VimEditor::new("SELECT id FROM id".to_string());

    // Setup search
    for _ in 0..7 {
        editor.handle_key(key_char('l'));
    }
    editor.handle_key(key_char('*'));
    assert_eq!(editor.search_match_count(), 2);

    // Execute :noh
    editor.handle_key(key_char(':'));
    editor.handle_key(key_char('n'));
    editor.handle_key(key_char('o'));
    editor.handle_key(key_char('h'));
    editor.handle_key(key(KeyCode::Enter));

    assert_eq!(editor.search_pattern(), None);
    assert_eq!(editor.mode(), VimMode::Normal);
}

#[test]
fn test_cancel_command_with_esc() {
    let mut editor = VimEditor::new("SELECT".to_string());

    editor.handle_key(key_char(':'));
    editor.handle_key(key_char('w'));
    editor.handle_key(key(KeyCode::Esc));

    assert_eq!(editor.mode(), VimMode::Normal);
    assert_eq!(editor.command_buffer(), "");
}

// ============================================================================
// Multi-line SQL Editing (3 tests)
// ============================================================================

#[test]
fn test_edit_multiline_query() {
    let mut editor = VimEditor::new("SELECT *".to_string());

    editor.handle_key(key_char('o'));
    editor.handle_key(key_char('F'));
    editor.handle_key(key_char('R'));
    editor.handle_key(key_char('O'));
    editor.handle_key(key_char('M'));
    editor.handle_key(key(KeyCode::Esc));

    editor.handle_key(key_char('o'));
    editor.handle_key(key_char('W'));
    editor.handle_key(key_char('H'));
    editor.handle_key(key_char('E'));
    editor.handle_key(key_char('R'));
    editor.handle_key(key_char('E'));
    editor.handle_key(key(KeyCode::Esc));

    assert_eq!(editor.content(), "SELECT *\nFROM\nWHERE");
    assert_eq!(editor.line_count(), 3);
}

#[test]
fn test_navigate_between_lines() {
    let mut editor =
        VimEditor::new("SELECT *\nFROM table\nWHERE id = 1\nORDER BY name".to_string());

    assert_eq!(editor.cursor(), (0, 0)); // Line 1

    editor.handle_key(key_char('j'));
    assert_eq!(editor.cursor().0, 1); // Line 2

    editor.handle_key(key_char('j'));
    assert_eq!(editor.cursor().0, 2); // Line 3

    editor.handle_key(key_char('k'));
    assert_eq!(editor.cursor().0, 1); // Back to line 2
}

#[test]
fn test_delete_line_in_multiline_query() {
    let mut editor =
        VimEditor::new("SELECT *\nFROM table\nWHERE id = 1\nORDER BY name".to_string());

    editor.handle_key(key_char('j')); // Move to line 2
    editor.handle_key(key_char('j')); // Move to line 3 (WHERE)

    editor.handle_key(key_char('d'));
    editor.handle_key(key_char('d')); // Delete WHERE line

    assert_eq!(editor.content(), "SELECT *\nFROM table\nORDER BY name");
}

// ============================================================================
// Edge Cases (3 tests)
// ============================================================================

#[test]
fn test_empty_query() {
    let editor = VimEditor::new("".to_string());
    assert_eq!(editor.content(), "");
    assert_eq!(editor.line_count(), 1);
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_single_line_query() {
    let mut editor = VimEditor::new("SELECT * FROM table".to_string());

    editor.handle_key(key_char('A'));
    editor.handle_key(key_char(' '));
    editor.handle_key(key_char('W'));
    editor.handle_key(key(KeyCode::Esc));

    assert_eq!(editor.content(), "SELECT * FROM table W");
}

#[test]
fn test_navigation_at_boundaries() {
    let mut editor = VimEditor::new("A".to_string());

    // Try to move left at start
    editor.handle_key(key_char('h'));
    assert_eq!(editor.cursor(), (0, 0));

    // Try to move up at first line
    editor.handle_key(key_char('k'));
    assert_eq!(editor.cursor(), (0, 0));

    // Try to move right past end
    editor.handle_key(key_char('l'));
    editor.handle_key(key_char('l'));
    assert_eq!(editor.cursor(), (0, 0)); // Should clamp to last char
}
