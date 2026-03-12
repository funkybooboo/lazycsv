//! Tests for vim_editor motion commands (hjkl, w/b/e, 0/$, gg/G, f/t, etc.)

use lazycsv::vim_editor::VimEditor;

// ============================================================================
// Basic Directional Movement (hjkl) Tests
// ============================================================================

#[test]
fn test_move_left() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 3);

    editor.move_left();
    assert_eq!(editor.cursor(), (0, 2));
}

#[test]
fn test_move_left_at_line_start() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_left();
    assert_eq!(editor.cursor(), (0, 0)); // Should not move beyond start
}

#[test]
fn test_move_left_with_count() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 5);
    editor.set_count_prefix(3);

    editor.move_left();
    assert_eq!(editor.cursor(), (0, 2));
}

#[test]
fn test_move_right() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.move_right();
    assert_eq!(editor.cursor(), (0, 3));
}

#[test]
fn test_move_right_at_line_end() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 4);

    editor.move_right();
    assert_eq!(editor.cursor(), (0, 4)); // Should clamp at last char
}

#[test]
fn test_move_right_with_count() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.set_count_prefix(5);

    editor.move_right();
    assert_eq!(editor.cursor(), (0, 5));
}

#[test]
fn test_move_up() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(2, 0);

    editor.move_up();
    assert_eq!(editor.cursor().0, 1);
}

#[test]
fn test_move_up_at_first_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_up();
    assert_eq!(editor.cursor().0, 0); // Should not move beyond first line
}

#[test]
fn test_move_up_with_count() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3\nLine 4".to_string());
    editor.set_cursor_for_test(3, 0);
    editor.set_count_prefix(2);

    editor.move_up();
    assert_eq!(editor.cursor().0, 1);
}

#[test]
fn test_move_down() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_down();
    assert_eq!(editor.cursor().0, 1);
}

#[test]
fn test_move_down_at_last_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.move_down();
    assert_eq!(editor.cursor().0, 1); // Should not move beyond last line
}

#[test]
fn test_move_down_with_count() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3\nLine 4".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.set_count_prefix(3);

    editor.move_down();
    assert_eq!(editor.cursor().0, 3);
}

// ============================================================================
// Line-Level Movement (0, $, ^) Tests
// ============================================================================

#[test]
fn test_move_to_line_start() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 6);

    editor.move_to_line_start();
    assert_eq!(editor.cursor().1, 0);
}

#[test]
fn test_move_to_line_end_normal_mode() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_to_line_end();
    assert_eq!(editor.cursor().1, 4); // Last char is at index 4
}

#[test]
fn test_move_to_line_end_insert_mode() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_insert_mode();
    editor.set_cursor_for_test(0, 0);

    editor.move_to_line_end();
    assert_eq!(editor.cursor().1, 5); // Can be at position 5 in insert mode
}

#[test]
fn test_move_to_first_non_blank() {
    let mut editor = VimEditor::new("   Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_to_first_non_blank();
    assert_eq!(editor.cursor().1, 3); // First 'H' is at position 3
}

#[test]
fn test_move_to_first_non_blank_no_whitespace() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 3);

    editor.move_to_first_non_blank();
    assert_eq!(editor.cursor().1, 0);
}

// ============================================================================
// Document-Level Movement (gg, G, line number) Tests
// ============================================================================

#[test]
fn test_move_to_first_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(2, 0);

    editor.move_to_first_line();
    assert_eq!(editor.cursor().0, 0);
}

#[test]
fn test_move_to_last_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_to_last_line();
    assert_eq!(editor.cursor().0, 2);
}

#[test]
fn test_move_to_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3\nLine 4".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_to_line(3); // 1-indexed, so line 3 = index 2
    assert_eq!(editor.cursor().0, 2);
}

// ============================================================================
// Word Movement (w, b, e) Tests
// ============================================================================

#[test]
fn test_move_next_word() {
    let mut editor = VimEditor::new("Hello World Test".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_next_word();
    assert_eq!(editor.cursor().1, 6); // Start of "World"
}

#[test]
fn test_move_next_word_with_count() {
    let mut editor = VimEditor::new("Hello World Test".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.set_count_prefix(2);

    editor.move_next_word();
    assert_eq!(editor.cursor().1, 12); // Start of "Test"
}

#[test]
fn test_move_next_word_across_lines() {
    let mut editor = VimEditor::new("Hello\nWorld".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_next_word();
    assert_eq!(editor.cursor(), (1, 0)); // Should move to next line
}

#[test]
fn test_move_next_word_multiple_spaces() {
    let mut editor = VimEditor::new("Hello    World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_next_word();
    assert_eq!(editor.cursor().1, 9); // Start of "World"
}

#[test]
fn test_move_prev_word() {
    let mut editor = VimEditor::new("Hello World Test".to_string());
    editor.set_cursor_for_test(0, 12); // At "Test"

    editor.move_prev_word();
    assert_eq!(editor.cursor().1, 6); // Start of "World"
}

#[test]
fn test_move_prev_word_with_count() {
    let mut editor = VimEditor::new("Hello World Test".to_string());
    editor.set_cursor_for_test(0, 12);
    editor.set_count_prefix(2);

    editor.move_prev_word();
    assert_eq!(editor.cursor().1, 0); // Start of "Hello"
}

#[test]
fn test_move_prev_word_at_line_start() {
    let mut editor = VimEditor::new("Hello\nWorld".to_string());
    editor.set_cursor_for_test(1, 0);

    editor.move_prev_word();
    assert_eq!(editor.cursor().0, 0); // Should move to previous line
}

#[test]
fn test_move_end_word() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.move_end_word();
    assert_eq!(editor.cursor().1, 4); // End of "Hello"
}

#[test]
fn test_move_end_word_with_count() {
    let mut editor = VimEditor::new("Hello World Test".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.set_count_prefix(2);

    editor.move_end_word();
    assert_eq!(editor.cursor().1, 10); // End of "World"
}

// ============================================================================
// Find/Till Character Movement (f, F, t, T, ;, ,) Tests
// ============================================================================

#[test]
fn test_find_char_forward() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.find_char_forward('W');
    assert_eq!(editor.cursor().1, 6); // Position of 'W'
}

#[test]
fn test_find_char_forward_not_found() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.find_char_forward('X');
    assert_eq!(editor.cursor().1, 0); // Should not move
}

#[test]
fn test_find_char_forward_multiple_occurrences() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.find_char_forward('l');
    assert_eq!(editor.cursor().1, 2); // First 'l' after cursor
}

#[test]
fn test_find_char_backward() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 10);

    editor.find_char_backward('H');
    assert_eq!(editor.cursor().1, 0); // Position of 'H'
}

#[test]
fn test_find_char_backward_not_found() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 5);

    editor.find_char_backward('X');
    assert_eq!(editor.cursor().1, 5); // Should not move
}

#[test]
fn test_till_char_forward() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.till_char_forward('W');
    assert_eq!(editor.cursor().1, 5); // One position before 'W'
}

#[test]
fn test_till_char_forward_not_found() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.till_char_forward('X');
    assert_eq!(editor.cursor().1, 0); // Should not move
}

#[test]
fn test_till_char_backward() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 10);

    editor.till_char_backward('H');
    assert_eq!(editor.cursor().1, 1); // One position after 'H'
}

#[test]
fn test_till_char_backward_not_found() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 5);

    editor.till_char_backward('X');
    assert_eq!(editor.cursor().1, 5); // Should not move
}

#[test]
fn test_repeat_find_forward() {
    let mut editor = VimEditor::new("abcabcabc".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.find_char_forward('c');
    assert_eq!(editor.cursor().1, 2);

    editor.repeat_find(); // Should find next 'c'
    assert_eq!(editor.cursor().1, 5);

    editor.repeat_find(); // Should find next 'c'
    assert_eq!(editor.cursor().1, 8);
}

#[test]
fn test_repeat_find_backward() {
    let mut editor = VimEditor::new("abcabcabc".to_string());
    editor.set_cursor_for_test(0, 8);

    editor.find_char_backward('a');
    assert_eq!(editor.cursor().1, 6);

    editor.repeat_find(); // Should find previous 'a'
    assert_eq!(editor.cursor().1, 3);

    editor.repeat_find(); // Should find previous 'a'
    assert_eq!(editor.cursor().1, 0);
}

#[test]
fn test_repeat_find_reverse_forward() {
    let mut editor = VimEditor::new("abcabcabc".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.find_char_forward('c');
    assert_eq!(editor.cursor().1, 2);

    editor.repeat_find_reverse(); // Should find previous 'c' (but there is none)
    assert_eq!(editor.cursor().1, 2); // Should not move
}

#[test]
fn test_repeat_find_reverse_backward() {
    let mut editor = VimEditor::new("abcabcabc".to_string());
    editor.set_cursor_for_test(0, 8);

    editor.find_char_backward('a');
    assert_eq!(editor.cursor().1, 6);

    editor.repeat_find_reverse(); // Should find next 'a' (forward)
    assert_eq!(editor.cursor().1, 6); // Should not move (no 'a' after)
}

#[test]
fn test_repeat_till_forward() {
    let mut editor = VimEditor::new("abcabcabc".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.till_char_forward('c');
    assert_eq!(editor.cursor().1, 1); // One before 'c'

    editor.repeat_find(); // Should till next 'c'
    assert_eq!(editor.cursor().1, 4); // One before next 'c'
}
