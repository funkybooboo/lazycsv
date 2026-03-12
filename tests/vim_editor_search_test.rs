//! Tests for vim_editor search functionality (/, n, N, *, search highlighting)

use lazycsv::vim_editor::VimEditor;

// ============================================================================
// Basic Search Tests
// ============================================================================

#[test]
fn test_search_forward_single_match() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("World".to_string());
    assert_eq!(editor.cursor(), (0, 6));
    assert_eq!(editor.search_pattern(), Some("World"));
}

#[test]
fn test_search_forward_multiple_matches() {
    let mut editor = VimEditor::new("foo bar foo baz foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.cursor(), (0, 8)); // Jumps to second "foo" (after cursor)
    assert_eq!(editor.search_match_count(), 3);
}

#[test]
fn test_search_forward_no_match() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("xyz".to_string());
    assert_eq!(editor.cursor(), (0, 0)); // Cursor doesn't move
    assert_eq!(editor.search_match_count(), 0);
}

#[test]
fn test_search_forward_multiline() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("Line".to_string());
    assert_eq!(editor.cursor(), (1, 0)); // Skips match at (0,0), jumps to (1,0)
    assert_eq!(editor.search_match_count(), 3);
}

#[test]
fn test_search_forward_case_sensitive() {
    let mut editor = VimEditor::new("Hello hello HELLO".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("hello".to_string());
    assert_eq!(editor.cursor(), (0, 6)); // Matches lowercase "hello"
    assert_eq!(editor.search_match_count(), 1);
}

#[test]
fn test_search_forward_partial_match() {
    let mut editor = VimEditor::new("testing test".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("test".to_string());
    assert_eq!(editor.cursor(), (0, 8)); // Skips "test" in "testing" at cursor, goes to standalone "test"
    assert_eq!(editor.search_match_count(), 2);
}

// ============================================================================
// Jump to Next/Previous Match Tests
// ============================================================================

#[test]
fn test_jump_to_next_match() {
    let mut editor = VimEditor::new("foo bar foo baz foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.cursor(), (0, 8)); // Skips first match, jumps to second

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 16)); // Third match

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 0)); // Wraps to first
}

#[test]
fn test_jump_to_next_match_wraps() {
    let mut editor = VimEditor::new("foo bar foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.cursor(), (0, 8)); // Skips first match at cursor

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 0)); // Wraps to first match

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 8)); // Back to second match
}

#[test]
fn test_jump_to_prev_match() {
    let mut editor = VimEditor::new("foo bar foo baz foo".to_string());
    editor.set_cursor_for_test(0, 16); // At third "foo"

    editor.search_forward("foo".to_string());
    assert_eq!(editor.cursor(), (0, 0)); // Wraps to first match (no match after 16)

    editor.jump_to_prev_match();
    assert_eq!(editor.cursor(), (0, 16)); // Wraps to last match (no match before 0)

    editor.jump_to_prev_match();
    assert_eq!(editor.cursor(), (0, 8)); // Second match (before 16)
}

#[test]
fn test_jump_to_prev_match_wraps() {
    let mut editor = VimEditor::new("foo bar foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.cursor(), (0, 8)); // Skips match at cursor, jumps to second

    editor.jump_to_prev_match();
    assert_eq!(editor.cursor(), (0, 0)); // Prev from (0,8) is (0,0)
}

#[test]
fn test_jump_to_next_match_multiline() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("Line".to_string());
    assert_eq!(editor.cursor(), (1, 0)); // Skips (0,0), jumps to (1,0)

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (2, 0));

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 0)); // Wraps
}

// ============================================================================
// Word Under Cursor Search Tests
// ============================================================================

#[test]
fn test_search_word_under_cursor() {
    let mut editor = VimEditor::new("foo bar foo baz".to_string());
    editor.set_cursor_for_test(0, 0); // On "foo"

    editor.search_word_under_cursor();
    assert_eq!(editor.search_pattern(), Some("foo"));
    assert_eq!(editor.search_match_count(), 2);
}

#[test]
fn test_search_word_under_cursor_middle() {
    let mut editor = VimEditor::new("hello world hello".to_string());
    editor.set_cursor_for_test(0, 7); // On "w" in "world"

    editor.search_word_under_cursor();
    assert_eq!(editor.search_pattern(), Some("world"));
    assert_eq!(editor.search_match_count(), 1);
}

#[test]
fn test_search_word_under_cursor_with_underscore() {
    let mut editor = VimEditor::new("my_var = my_var + 1".to_string());
    editor.set_cursor_for_test(0, 0); // On "my_var"

    editor.search_word_under_cursor();
    assert_eq!(editor.search_pattern(), Some("my_var"));
    assert_eq!(editor.search_match_count(), 2);
}

#[test]
fn test_search_word_under_cursor_end_of_word() {
    let mut editor = VimEditor::new("test testing".to_string());
    editor.set_cursor_for_test(0, 3); // On "t" at end of "test"

    editor.search_word_under_cursor();
    assert_eq!(editor.search_pattern(), Some("test"));
    assert_eq!(editor.search_match_count(), 2); // Matches "test" in "testing" too
}

// ============================================================================
// Clear Search Tests
// ============================================================================

#[test]
fn test_clear_search() {
    let mut editor = VimEditor::new("foo bar foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.search_match_count(), 2);

    editor.clear_search();
    assert_eq!(editor.search_pattern(), None);
    assert_eq!(editor.search_match_count(), 0);
    assert_eq!(editor.current_match_index(), None);
}

// ============================================================================
// Search Matches and Highlighting Tests
// ============================================================================

#[test]
fn test_search_matches() {
    let mut editor = VimEditor::new("foo bar foo baz foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    let matches = editor.search_matches();

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0], (0, 0));
    assert_eq!(matches[1], (0, 8));
    assert_eq!(matches[2], (0, 16));
}

#[test]
fn test_search_matches_multiline() {
    let mut editor = VimEditor::new("foo\nbar\nfoo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    let matches = editor.search_matches();

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0], (0, 0));
    assert_eq!(matches[1], (2, 0));
}

#[test]
fn test_current_match_index() {
    let mut editor = VimEditor::new("foo bar foo baz foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.current_match_index(), Some(1)); // Skips match at cursor (index 0), lands at index 1

    editor.jump_to_next_match();
    assert_eq!(editor.current_match_index(), Some(2));

    editor.jump_to_next_match();
    assert_eq!(editor.current_match_index(), Some(0)); // Wraps back to first match
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_search_empty_pattern() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("".to_string());
    assert_eq!(editor.search_match_count(), 0);
}

#[test]
fn test_search_single_character() {
    let mut editor = VimEditor::new("a b a c a".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("a".to_string());
    assert_eq!(editor.search_match_count(), 3);
    assert_eq!(editor.cursor(), (0, 4)); // Skips match at cursor (0,0), jumps to next (0,4)
}

#[test]
fn test_search_overlapping_matches() {
    let mut editor = VimEditor::new("aaaa".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("aa".to_string());
    // Should find non-overlapping matches: positions 0 and 2
    assert_eq!(editor.search_match_count(), 2);
}

#[test]
fn test_search_at_end_of_line() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 10);

    editor.search_forward("Hello".to_string());
    assert_eq!(editor.cursor(), (0, 0)); // Wraps to beginning
}

#[test]
fn test_jump_without_search() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 5);

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 5)); // No movement without search

    editor.jump_to_prev_match();
    assert_eq!(editor.cursor(), (0, 5)); // No movement without search
}

#[test]
fn test_search_unicode() {
    let mut editor = VimEditor::new("Hello 世界 Hello".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("Hello".to_string());
    assert_eq!(editor.search_match_count(), 2);
    assert_eq!(editor.cursor(), (0, 9)); // Skips match at cursor (0,0), jumps to (0,9)

    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 0)); // Wraps back to first match
}

#[test]
fn test_search_special_characters() {
    let mut editor = VimEditor::new("price: $10.99 or $20.99".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("$".to_string());
    assert_eq!(editor.search_match_count(), 2);
    assert_eq!(editor.cursor(), (0, 7));
}

// ============================================================================
// Search and Edit Tests
// ============================================================================

#[test]
fn test_search_then_edit() {
    let mut editor = VimEditor::new("foo bar foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("bar".to_string());
    assert_eq!(editor.cursor(), (0, 4));

    // Delete the word
    editor.delete_char();
    editor.delete_char();
    editor.delete_char();

    assert_eq!(editor.content(), "foo  foo");
}

#[test]
fn test_search_persists_after_movement() {
    let mut editor = VimEditor::new("foo bar foo baz".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.cursor(), (0, 8)); // Jumped to second "foo"
    editor.move_right();
    editor.move_right();

    // Search pattern should still be active
    assert_eq!(editor.search_pattern(), Some("foo"));
    assert_eq!(editor.search_match_count(), 2);

    // Can still jump to next match
    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 0)); // Wraps to first match
}

// ============================================================================
// Combined Search Operations Tests
// ============================================================================

#[test]
fn test_sequential_searches() {
    let mut editor = VimEditor::new("foo bar baz".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.search_match_count(), 1);

    editor.search_forward("bar".to_string());
    assert_eq!(editor.search_match_count(), 1);
    assert_eq!(editor.search_pattern(), Some("bar"));
}

#[test]
fn test_search_navigate_and_clear() {
    let mut editor = VimEditor::new("foo bar foo baz foo".to_string());
    editor.set_cursor_for_test(0, 0);

    editor.search_forward("foo".to_string());
    assert_eq!(editor.cursor(), (0, 8)); // Jumped to second match
    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 16)); // Third match
    editor.jump_to_next_match();
    assert_eq!(editor.cursor(), (0, 0)); // Wraps to first

    editor.clear_search();
    editor.jump_to_next_match(); // Should do nothing
    assert_eq!(editor.cursor(), (0, 0));
}
