//! Tests for vim_editor undo/redo functionality
//!
//! Tests undo (u) and redo (Ctrl+r) operations to ensure proper state restoration.

use lazycsv::vim_editor::VimEditor;

// ============================================================================
// Basic Undo/Redo Tests
// ============================================================================

#[test]
fn test_undo_single_char_insert() {
    let mut editor = VimEditor::new("hello".to_string());
    editor.set_cursor_for_test(0, 4); // On last 'o'

    editor.push_undo(); // Save state before edit
    editor.insert_after(); // 'a' command - move cursor right and enter insert
    editor.insert_char('!');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "hello!");
    assert_eq!(editor.cursor(), (0, 5));

    editor.undo();
    assert_eq!(editor.content(), "hello");
    assert_eq!(editor.cursor(), (0, 4));
}

#[test]
fn test_undo_multiple_chars() {
    let mut editor = VimEditor::new("test".to_string());
    editor.set_cursor_for_test(0, 3); // On 't'

    editor.push_undo();
    editor.insert_after(); // Move to end and enter insert
    editor.insert_char('i');
    editor.insert_char('n');
    editor.insert_char('g');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "testing");

    editor.undo();
    assert_eq!(editor.content(), "test");
}

#[test]
fn test_undo_delete_char() {
    let mut editor = VimEditor::new("hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.push_undo();
    editor.delete_char_at(); // x command

    assert_eq!(editor.content(), "ello");

    editor.undo();
    assert_eq!(editor.content(), "hello");
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_undo_delete_line() {
    let mut editor = VimEditor::new("line1\nline2\nline3".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.push_undo();
    editor.delete_line(); // dd command

    assert_eq!(editor.content(), "line1\nline3");

    editor.undo();
    assert_eq!(editor.content(), "line1\nline2\nline3");
    assert_eq!(editor.cursor(), (1, 0));
}

#[test]
fn test_undo_with_cursor_restoration() {
    let mut editor = VimEditor::new("abcdef".to_string());
    editor.set_cursor_for_test(0, 3); // On 'd'

    editor.push_undo();
    editor.delete_char_at(); // Delete 'd'
    editor.move_right(); // Move to next char

    assert_eq!(editor.content(), "abcef");
    assert_eq!(editor.cursor(), (0, 4));

    editor.undo();
    assert_eq!(editor.content(), "abcdef");
    assert_eq!(editor.cursor(), (0, 3)); // Cursor restored
}

// ============================================================================
// Redo Tests
// ============================================================================

#[test]
fn test_redo_after_undo() {
    let mut editor = VimEditor::new("hello".to_string());
    editor.set_cursor_for_test(0, 4); // On last 'o'

    editor.push_undo();
    editor.insert_after();
    editor.insert_char('!');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "hello!");

    editor.undo();
    assert_eq!(editor.content(), "hello");

    editor.redo();
    assert_eq!(editor.content(), "hello!");
    assert_eq!(editor.cursor(), (0, 5));
}

#[test]
fn test_multiple_undo_redo() {
    let mut editor = VimEditor::new("a".to_string());
    editor.set_cursor_for_test(0, 0);

    // Add 'b'
    editor.push_undo();
    editor.insert_after();
    editor.insert_char('b');
    editor.exit_insert_mode();
    assert_eq!(editor.content(), "ab");

    // Add 'c'
    editor.push_undo();
    editor.insert_after();
    editor.insert_char('c');
    editor.exit_insert_mode();
    assert_eq!(editor.content(), "abc");

    // Undo twice
    editor.undo();
    assert_eq!(editor.content(), "ab");
    editor.undo();
    assert_eq!(editor.content(), "a");

    // Redo twice
    editor.redo();
    assert_eq!(editor.content(), "ab");
    editor.redo();
    assert_eq!(editor.content(), "abc");
}

#[test]
fn test_redo_cleared_after_new_edit() {
    let mut editor = VimEditor::new("hello".to_string());
    editor.set_cursor_for_test(0, 4);

    // Edit 1: Add '!'
    editor.push_undo();
    editor.insert_after();
    editor.insert_char('!');
    editor.exit_insert_mode();

    // Undo
    editor.undo();
    assert_eq!(editor.content(), "hello");
    assert!(editor.can_redo());

    // Edit 2: Add '?'
    editor.push_undo();
    editor.insert_after();
    editor.insert_char('?');
    editor.exit_insert_mode();
    assert_eq!(editor.content(), "hello?");

    // Redo should not be available since we made a new edit
    assert!(!editor.can_redo());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_undo_on_empty_stack() {
    let mut editor = VimEditor::new("hello".to_string());

    editor.undo(); // Should do nothing
    assert_eq!(editor.content(), "hello");
}

#[test]
fn test_redo_on_empty_stack() {
    let mut editor = VimEditor::new("hello".to_string());

    editor.redo(); // Should do nothing
    assert_eq!(editor.content(), "hello");
}

#[test]
fn test_can_undo_can_redo() {
    let mut editor = VimEditor::new("hello".to_string());

    assert!(!editor.can_undo());
    assert!(!editor.can_redo());

    editor.push_undo();
    editor.enter_insert_mode();
    editor.insert_char('!');
    editor.exit_insert_mode();

    assert!(editor.can_undo());
    assert!(!editor.can_redo());

    editor.undo();
    assert!(!editor.can_undo());
    assert!(editor.can_redo());

    editor.redo();
    assert!(editor.can_undo());
    assert!(!editor.can_redo());
}

// ============================================================================
// Multi-line Undo/Redo Tests
// ============================================================================

#[test]
fn test_undo_multiline_insert() {
    let mut editor = VimEditor::new("line1".to_string());
    editor.set_cursor_for_test(0, 4); // On '1'

    editor.push_undo();
    editor.insert_after(); // Move to end
    editor.newline();
    editor.insert_char('l');
    editor.insert_char('i');
    editor.insert_char('n');
    editor.insert_char('e');
    editor.insert_char('2');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "line1\nline2");
    assert_eq!(editor.cursor(), (1, 4));

    editor.undo();
    assert_eq!(editor.content(), "line1");
    assert_eq!(editor.cursor(), (0, 4));
}

#[test]
fn test_undo_join_lines() {
    let mut editor = VimEditor::new("hello\nworld".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.push_undo();
    editor.join_lines(); // J command

    assert_eq!(editor.content(), "hello world");

    editor.undo();
    assert_eq!(editor.content(), "hello\nworld");
}

#[test]
fn test_undo_new_line_below() {
    let mut editor = VimEditor::new("line1\nline3".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.push_undo();
    editor.insert_line_below(); // o command
    editor.insert_char('l');
    editor.insert_char('i');
    editor.insert_char('n');
    editor.insert_char('e');
    editor.insert_char('2');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "line1\nline2\nline3");

    editor.undo();
    assert_eq!(editor.content(), "line1\nline3");
}

// ============================================================================
// Complex Edit Sequences
// ============================================================================

#[test]
fn test_undo_sequence_of_edits() {
    let mut editor = VimEditor::new("start".to_string());

    // Edit 1: append " one"
    editor.push_undo();
    editor.set_cursor_for_test(0, 4); // On 't'
    editor.insert_after();
    editor.insert_char(' ');
    editor.insert_char('o');
    editor.insert_char('n');
    editor.insert_char('e');
    editor.exit_insert_mode();
    assert_eq!(editor.content(), "start one");

    // Edit 2: append " two"
    editor.push_undo();
    editor.insert_after();
    editor.insert_char(' ');
    editor.insert_char('t');
    editor.insert_char('w');
    editor.insert_char('o');
    editor.exit_insert_mode();
    assert_eq!(editor.content(), "start one two");

    // Edit 3: append " three"
    editor.push_undo();
    editor.insert_after();
    editor.insert_char(' ');
    editor.insert_char('t');
    editor.insert_char('h');
    editor.insert_char('r');
    editor.insert_char('e');
    editor.insert_char('e');
    editor.exit_insert_mode();
    assert_eq!(editor.content(), "start one two three");

    // Undo all three edits
    editor.undo();
    assert_eq!(editor.content(), "start one two");
    editor.undo();
    assert_eq!(editor.content(), "start one");
    editor.undo();
    assert_eq!(editor.content(), "start");

    // Redo all three
    editor.redo();
    assert_eq!(editor.content(), "start one");
    editor.redo();
    assert_eq!(editor.content(), "start one two");
    editor.redo();
    assert_eq!(editor.content(), "start one two three");
}

#[test]
fn test_undo_redo_with_navigation() {
    let mut editor = VimEditor::new("abc\ndef\nghi".to_string());

    // Delete first line
    editor.set_cursor_for_test(0, 0);
    editor.push_undo();
    editor.delete_line();
    assert_eq!(editor.content(), "def\nghi");

    // Move to second line and delete it
    editor.set_cursor_for_test(1, 0);
    editor.push_undo();
    editor.delete_line();
    assert_eq!(editor.content(), "def");

    // Undo both deletes
    editor.undo();
    assert_eq!(editor.content(), "def\nghi");
    editor.undo();
    assert_eq!(editor.content(), "abc\ndef\nghi");
}

// ============================================================================
// Undo History Limit Tests
// ============================================================================

#[test]
fn test_undo_history_limit() {
    let mut editor = VimEditor::new("".to_string());

    // Push more than MAX_UNDO_HISTORY (1000) edits
    for _i in 0..1100 {
        editor.push_undo();
        editor.enter_insert_mode();
        editor.insert_char('a');
        editor.exit_insert_mode();
    }

    // Should only be able to undo up to 1000 times
    let mut undo_count = 0;
    while editor.can_undo() {
        editor.undo();
        undo_count += 1;
    }

    assert_eq!(undo_count, 1000);
}

// ============================================================================
// Undo with Visual Mode Tests
// ============================================================================

#[test]
fn test_undo_visual_delete() {
    let mut editor = VimEditor::new("hello world".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.push_undo();
    editor.enter_visual_mode(); // v
    editor.move_right(); // Select to 'e'
    editor.move_right(); // Select to 'l'
    editor.move_right(); // Select to 'l'
    editor.move_right(); // Select to 'o'
    editor.delete_selection(); // d - deletes "hello" (inclusive)

    assert_eq!(editor.content(), " world");

    editor.undo();
    assert_eq!(editor.content(), "hello world");
}

#[test]
fn test_undo_visual_line_delete() {
    let mut editor = VimEditor::new("line1\nline2\nline3".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.push_undo();
    editor.enter_visual_line_mode(); // V
    editor.move_down(); // Select two lines
    editor.delete_selection(); // d

    assert_eq!(editor.content(), "line3");

    editor.undo();
    assert_eq!(editor.content(), "line1\nline2\nline3");
}

// ============================================================================
// Undo with Paste Tests
// ============================================================================

#[test]
fn test_undo_paste() {
    let mut editor = VimEditor::new("hello\nworld".to_string());
    editor.set_cursor_for_test(0, 0);

    // Yank first line
    editor.yank_line();

    // Move to second line and paste
    editor.set_cursor_for_test(1, 0);
    editor.push_undo();
    editor.paste_below();

    assert_eq!(editor.content(), "hello\nworld\nhello");

    editor.undo();
    assert_eq!(editor.content(), "hello\nworld");
}

#[test]
fn test_undo_change_line() {
    let mut editor = VimEditor::new("old line".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.push_undo();
    editor.change_line(); // cc command - should enter insert mode
    editor.insert_char('n');
    editor.insert_char('e');
    editor.insert_char('w');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "new");

    editor.undo();
    assert_eq!(editor.content(), "old line");
}
