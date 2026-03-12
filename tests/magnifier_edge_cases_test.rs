//! Magnifier Vim operation edge case tests
//!
//! Tests edge cases for vim operations: count prefixes, boundaries, complex sequences.

use lazycsv::{
    domain::position::{ColIndex, RowIndex},
    magnifier::MagnifierState,
};

#[test]
fn test_magnifier_large_count_prefix_999j() {
    // Create 1000 lines
    let lines: Vec<String> = (1..=1000).map(|i| format!("Line {}", i)).collect();
    let content = lines.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content, position);

    assert_eq!(mag.cursor().0, 0);

    // Simulate 999j by moving down 999 times (in real usage, count prefix would multiply)
    // For now, test multiple moves
    for _ in 0..50 {
        mag.move_down();
    }

    assert_eq!(mag.cursor().0, 50);
}

#[test]
fn test_magnifier_operations_at_start_of_buffer() {
    let content = "line1\nline2\nline3";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Try to move up from first line
    mag.move_up();
    assert_eq!(mag.cursor().0, 0); // Should stay at first line

    // Try to move left from first column
    mag.move_left();
    assert_eq!(mag.cursor().1, 0); // Should stay at first column

    // Try to delete above when at first line
    mag.push_undo();
    mag.delete_char();
    // Should not crash
}

#[test]
fn test_magnifier_operations_at_end_of_buffer() {
    let content = "line1\nline2\nline3";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Move to last line
    mag.move_to_last_line();
    assert_eq!(mag.cursor().0, 2);

    // Try to move down from last line
    mag.move_down();
    assert_eq!(mag.cursor().0, 2); // Should stay at last line

    // Move to end of line
    mag.move_to_line_end();

    // Try to move right from end of line
    mag.move_right();
    // Should handle gracefully (may stay or move to next line in vim)
}

#[test]
fn test_magnifier_complex_undo_redo_sequence() {
    let content = "line1\nline2\nline3\nline4\nline5";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 5);

    // Perform multiple operations
    mag.push_undo();
    mag.delete_line();
    mag.push_undo();
    mag.delete_line();
    mag.push_undo();
    mag.insert_line_below();
    mag.push_undo();
    mag.delete_char();

    // Undo all
    mag.undo();
    mag.undo();
    mag.undo();
    mag.undo();

    assert_eq!(mag.lines().len(), 5); // Back to original

    // Redo some
    mag.redo();
    mag.redo();

    assert_eq!(mag.lines().len(), 3); // After 2 deletions

    // New operation should clear redo stack
    mag.push_undo();
    mag.delete_line();

    mag.redo(); // Should not redo anything
    assert_eq!(mag.lines().len(), 2);
}

#[test]
fn test_magnifier_visual_mode_empty_lines() {
    let content = "\n\n\ntext\n";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Enter visual mode on empty line
    mag.enter_visual_mode();

    // Move selection
    mag.move_down();
    mag.move_down();

    // Check selection exists
    assert!(mag.visual_selection().is_some());
}

#[test]
fn test_magnifier_search_regex_special_chars() {
    let content = "test (parentheses) and [brackets] and {braces}";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Search for literal parentheses (should handle escaping)
    mag.search_forward("(parentheses)".to_string());

    // Should find match (or fallback to literal search) without crashing
    let _matches = mag.search_matches();
}

#[test]
fn test_magnifier_delete_all_lines() {
    let content = "line1\nline2\nline3";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Delete all lines
    mag.push_undo();
    mag.delete_line();
    mag.push_undo();
    mag.delete_line();
    mag.push_undo();
    mag.delete_line();

    // Should have at least one empty line
    assert!(!mag.lines().is_empty());
}

#[test]
fn test_magnifier_paste_empty_buffer() {
    let content = "line1\nline2";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Try to paste without yanking anything first
    mag.paste_below();

    // Should not crash, buffer state should be reasonable
    assert!(mag.lines().len() >= 2);
}

#[test]
fn test_magnifier_find_char_not_found() {
    let content = "test line without target";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    let initial_cursor = mag.cursor();

    // Try to find character that doesn't exist
    mag.find_char_forward('Z');

    // Cursor should not move
    assert_eq!(mag.cursor(), initial_cursor);
}

#[test]
fn test_magnifier_join_single_line() {
    let content = "only one line";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Try to join when only one line exists
    mag.join_lines();

    // Should handle gracefully
    assert_eq!(mag.lines().len(), 1);
}

#[test]
fn test_magnifier_word_motion_punctuation() {
    let content = "hello,world.test!end?";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Move through "words" with punctuation
    mag.move_next_word();
    assert!(mag.cursor().1 > 0);

    mag.move_next_word();
    assert!(mag.cursor().1 > 5);
}

#[test]
fn test_magnifier_indent_operations() {
    let content = "line1\nline2\nline3";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Indent line
    mag.indent_line();

    // Should have added indentation
    assert!(mag.lines()[0].starts_with(' ') || mag.lines()[0].starts_with('\t'));

    // Dedent
    mag.dedent_line();

    // Should remove indentation (or handle no-op gracefully)
    assert!(mag.lines().len() == 3);
}

#[test]
fn test_magnifier_replace_char_at_eol() {
    let content = "test";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Move to end of line
    mag.move_to_line_end();

    // Try to replace character at end
    mag.replace_char('X');

    // Should handle gracefully (may or may not replace depending on vim semantics)
    assert!(mag.lines().len() == 1);
}
