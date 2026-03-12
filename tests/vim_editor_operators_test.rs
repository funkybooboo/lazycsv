//! Tests for vim_editor operator commands (x, dd, yy, p, i, a, o, O, r, J, etc.)

use lazycsv::vim_editor::VimEditor;

// ============================================================================
// Insert Mode Text Input Tests
// ============================================================================

#[test]
fn test_insert_char() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 2); // Between 'l' and 'l'

    editor.insert_char('X');
    assert_eq!(editor.content(), "HeXllo");
    assert_eq!(editor.cursor().1, 3);
}

#[test]
fn test_insert_char_at_end() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 5);

    editor.insert_char('!');
    assert_eq!(editor.content(), "Hello!");
    assert_eq!(editor.cursor().1, 6);
}

#[test]
fn test_insert_char_at_start() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 0);

    editor.insert_char('X');
    assert_eq!(editor.content(), "XHello");
    assert_eq!(editor.cursor().1, 1);
}

#[test]
fn test_delete_char_before() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 3);

    editor.delete_char_before();
    assert_eq!(editor.content(), "Helo");
    assert_eq!(editor.cursor().1, 2);
}

#[test]
fn test_delete_char_before_at_line_start() {
    let mut editor = VimEditor::new("Hello\nWorld".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(1, 0);

    editor.delete_char_before();
    assert_eq!(editor.content(), "HelloWorld");
    assert_eq!(editor.cursor(), (0, 5));
}

#[test]
fn test_delete_char_at() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 2);

    editor.delete_char_at();
    assert_eq!(editor.content(), "Helo");
    assert_eq!(editor.cursor().1, 2);
}

#[test]
fn test_delete_char_at_end_of_line() {
    let mut editor = VimEditor::new("Hello\nWorld".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 5);

    editor.delete_char_at();
    assert_eq!(editor.content(), "HelloWorld");
    assert_eq!(editor.cursor(), (0, 5));
}

#[test]
fn test_newline() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 2);

    editor.newline();
    assert_eq!(editor.content(), "He\nllo");
    assert_eq!(editor.cursor(), (1, 0));
}

#[test]
fn test_newline_at_end() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 5);

    editor.newline();
    assert_eq!(editor.content(), "Hello\n");
    assert_eq!(editor.cursor(), (1, 0));
}

// ============================================================================
// Normal Mode Basic Operators Tests
// ============================================================================

#[test]
fn test_delete_char() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.delete_char();
    assert_eq!(editor.content(), "Helo");
    assert_eq!(editor.cursor().1, 2);
}

#[test]
fn test_delete_char_at_end() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 4);

    editor.delete_char();
    assert_eq!(editor.content(), "Hell");
    assert_eq!(editor.cursor().1, 3); // Cursor clamped to last char
}

#[test]
fn test_delete_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.delete_line();
    assert_eq!(editor.content(), "Line 1\nLine 3");
    assert_eq!(editor.cursor().0, 1);
}

#[test]
fn test_delete_line_last_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.delete_line();
    assert_eq!(editor.content(), "Line 1");
    assert_eq!(editor.cursor().0, 0);
}

#[test]
fn test_delete_line_only_line() {
    let mut editor = VimEditor::new("Only Line".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.delete_line();
    assert_eq!(editor.content(), "");
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_yank_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.yank_line();
    editor.move_down();
    editor.paste_below();

    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 1");
}

#[test]
fn test_paste_below() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.yank_line();
    editor.paste_below();

    assert_eq!(editor.content(), "Line 1\nLine 1\nLine 2");
    assert_eq!(editor.cursor(), (1, 0));
}

#[test]
fn test_paste_above() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.yank_line();
    editor.paste_above();

    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 2");
    assert_eq!(editor.cursor(), (1, 0));
}

#[test]
fn test_paste_multiple_lines() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.delete_line();
    editor.delete_line();

    // Clipboard now has "Line 2" (from last delete)
    editor.set_cursor_for_test(0, 0);
    editor.paste_below();

    assert_eq!(editor.content(), "Line 3\nLine 2");
}

#[test]
fn test_substitute_char() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.substitute_char();
    assert_eq!(editor.content(), "Helo");
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

// ============================================================================
// Enter Insert Mode Variations Tests
// ============================================================================

#[test]
fn test_insert_before() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.insert_before();
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
    assert_eq!(editor.cursor().1, 2); // Cursor stays at position
}

#[test]
fn test_insert_after() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.insert_after();
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
    assert_eq!(editor.cursor().1, 3); // Cursor moves right
}

#[test]
fn test_insert_line_below() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.insert_line_below();
    assert_eq!(editor.content(), "Line 1\n\nLine 2");
    assert_eq!(editor.cursor(), (1, 0));
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

#[test]
fn test_insert_line_above() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.insert_line_above();
    assert_eq!(editor.content(), "Line 1\n\nLine 2");
    assert_eq!(editor.cursor(), (1, 0));
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

#[test]
fn test_insert_at_line_start() {
    let mut editor = VimEditor::new("  Hello".to_string());
    editor.set_cursor_for_test(0, 5);

    editor.insert_at_line_start();
    assert_eq!(editor.cursor().1, 2); // First non-blank is at position 2
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

#[test]
fn test_insert_at_line_end() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.insert_at_line_end();
    assert_eq!(editor.cursor().1, 5); // Insert mode can be at end
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

// ============================================================================
// Change Operators Tests
// ============================================================================

#[test]
fn test_change_char() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.change_char();
    assert_eq!(editor.content(), "Helo");
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

#[test]
fn test_change_line() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 5);

    editor.change_line();
    assert_eq!(editor.content(), "");
    assert_eq!(editor.cursor(), (0, 0));
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

#[test]
fn test_change_to_eol() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 5);

    editor.change_to_eol();
    assert_eq!(editor.content(), "Hello");
    assert_eq!(editor.cursor().1, 5);
    assert!(matches!(
        editor.mode(),
        lazycsv::vim_editor::VimMode::Insert
    ));
}

// ============================================================================
// Replace & Join Tests
// ============================================================================

#[test]
fn test_replace_char() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.replace_char('X');
    assert_eq!(editor.content(), "HeXlo");
    assert_eq!(editor.cursor().1, 2);
}

#[test]
fn test_replace_char_at_end() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 4);

    editor.replace_char('!');
    assert_eq!(editor.content(), "Hell!");
}

#[test]
fn test_join_lines() {
    let mut editor = VimEditor::new("Hello\nWorld".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.join_lines();
    assert_eq!(editor.content(), "Hello World");
}

#[test]
fn test_join_lines_empty() {
    let mut editor = VimEditor::new("Hello\n\nWorld".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.join_lines();
    assert_eq!(editor.content(), "Hello\nWorld");
}

#[test]
fn test_join_lines_last_line() {
    let mut editor = VimEditor::new("Hello\nWorld".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.join_lines();
    assert_eq!(editor.content(), "Hello\nWorld"); // No change
}

// ============================================================================
// Indent/Dedent Tests
// ============================================================================

#[test]
fn test_indent_line() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.indent_line();
    assert_eq!(editor.content(), "  Hello");
    assert_eq!(editor.cursor().1, 2);
}

#[test]
fn test_indent_line_preserves_cursor() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.indent_line();
    assert_eq!(editor.content(), "  Hello");
    assert_eq!(editor.cursor().1, 4); // Cursor moves right by 2
}

#[test]
fn test_dedent_line() {
    let mut editor = VimEditor::new("  Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.dedent_line();
    assert_eq!(editor.content(), "Hello");
    assert_eq!(editor.cursor().1, 0);
}

#[test]
fn test_dedent_line_with_tab() {
    let mut editor = VimEditor::new("\tHello".to_string());
    editor.set_cursor_for_test(0, 1);

    editor.dedent_line();
    assert_eq!(editor.content(), "Hello");
    assert_eq!(editor.cursor().1, 0);
}

#[test]
fn test_dedent_line_no_indent() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.dedent_line();
    assert_eq!(editor.content(), "Hello"); // No change
}

// ============================================================================
// Combined Operations Tests
// ============================================================================

#[test]
fn test_delete_and_paste() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);

    // Delete line 1
    editor.delete_line();
    assert_eq!(editor.content(), "Line 2\nLine 3");

    // Paste after line 2
    editor.move_down();
    editor.paste_below();
    assert_eq!(editor.content(), "Line 2\nLine 3\nLine 1");
}

#[test]
fn test_yank_and_paste_multiple_times() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.yank_line();
    editor.paste_below();
    editor.paste_below();

    assert_eq!(editor.content(), "Line 1\nLine 1\nLine 1\nLine 2");
}

#[test]
fn test_insert_and_delete_sequence() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 5);

    editor.insert_after();
    editor.insert_char(' ');
    editor.insert_char('W');
    editor.insert_char('o');
    editor.insert_char('r');
    editor.insert_char('l');
    editor.insert_char('d');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "Hello World");
}

#[test]
fn test_change_and_type() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 6);

    editor.change_to_eol();
    editor.insert_char('R');
    editor.insert_char('u');
    editor.insert_char('s');
    editor.insert_char('t');
    editor.exit_insert_mode();

    assert_eq!(editor.content(), "Hello Rust");
}
