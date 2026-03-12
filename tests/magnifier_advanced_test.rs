//! Advanced magnifier mode tests for new vim features
//!
//! Tests for:
//! - Visual mode (v, V)
//! - Search (/, n, N, *)
//! - Character find (f, F, t, T, ;, ,)
//! - Change operator (c, cc, C)
//! - Replace (r)
//! - Join lines (J)
//! - Indent (>>, <<)
//! - Undo/redo (u, Ctrl+r)
//! - Ex commands (:w, :q, :wq, :q!)

use lazycsv::domain::position::{ColIndex, RowIndex};
use lazycsv::magnifier::{MagnifierMode, MagnifierState};

#[test]
fn test_visual_mode_char_wise() {
    let mut state = MagnifierState::new(
        "Hello World".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Enter visual mode
    state.enter_visual_mode();
    assert_eq!(state.mode(), MagnifierMode::Visual);

    // Move to extend selection
    state.move_right();
    state.move_right();
    state.move_right();

    // Get selection
    let selection = state.visual_selection();
    assert!(selection.is_some());
}

#[test]
fn test_visual_mode_line_wise() {
    let mut state = MagnifierState::new(
        "Line 1\nLine 2\nLine 3".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Enter visual line mode
    state.enter_visual_line_mode();
    assert_eq!(state.mode(), MagnifierMode::VisualLine);

    // Move down to select multiple lines
    state.move_down();

    // Get selection
    let selection = state.visual_selection();
    assert!(selection.is_some());
}

#[test]
fn test_visual_delete_selection() {
    let mut state = MagnifierState::new(
        "Line 1\nLine 2\nLine 3".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Select and delete first two lines
    state.enter_visual_line_mode();
    state.move_down();
    state.delete_selection();

    assert_eq!(state.line_count(), 1);
    assert_eq!(state.line(0), Some("Line 3"));
}

#[test]
fn test_visual_yank_selection() {
    let mut state = MagnifierState::new(
        "Line 1\nLine 2\nLine 3".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Select and yank first two lines
    state.enter_visual_line_mode();
    state.move_down();
    state.yank_selection();

    // Verify clipboard has content
    // Paste to verify
    state.move_to_last_line();
    state.paste_below();

    assert_eq!(state.line_count(), 5); // 3 original + 2 pasted
}

#[test]
fn test_search_forward() {
    let mut state = MagnifierState::new(
        "hello world\nhello again\nworld hello".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Search for "hello" - cursor starts at (0,0), so finds next match at (1,0)
    state.search_forward("hello".to_string());
    assert_eq!(state.cursor(), (1, 0));

    // Jump to next match - should go to third occurrence
    state.jump_to_next_match();
    assert_eq!(state.cursor(), (2, 6));

    // Jump to next match - should wrap to first
    state.jump_to_next_match();
    assert_eq!(state.cursor(), (0, 0));

    // Jump to next match - should go to second
    state.jump_to_next_match();
    assert_eq!(state.cursor(), (1, 0));
}

#[test]
fn test_search_previous() {
    let mut state = MagnifierState::new(
        "hello world\nhello again\nworld hello".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Search - jumps to first match at (0,0)
    state.search_forward("hello".to_string());

    // Move cursor past all matches
    state.set_cursor_for_test(2, 11);

    // Jump to previous match - should go to last occurrence
    state.jump_to_prev_match();
    assert_eq!(state.cursor(), (2, 6));

    // Jump to previous again
    state.jump_to_prev_match();
    assert_eq!(state.cursor(), (1, 0));
}

#[test]
fn test_search_word_under_cursor() {
    let mut state = MagnifierState::new(
        "hello world hello".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Get word under cursor
    let word = state.word_under_cursor();
    assert_eq!(word, Some("hello".to_string()));

    // Move to "world"
    state.set_cursor_for_test(0, 6);
    let word = state.word_under_cursor();
    assert_eq!(word, Some("world".to_string()));
}

#[test]
fn test_clear_search() {
    let mut state = MagnifierState::new(
        "hello world".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    state.search_forward("hello".to_string());
    assert!(state.search_pattern().is_some());

    state.clear_search();
    assert!(state.search_pattern().is_none());
    assert_eq!(state.search_matches().len(), 0);
}

#[test]
fn test_find_char_forward() {
    let mut state = MagnifierState::new(
        "hello world".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Find 'o' forward
    state.find_char_forward('o');
    assert_eq!(state.cursor(), (0, 4)); // First 'o' in "hello"

    // Find 'o' again
    state.find_char_forward('o');
    assert_eq!(state.cursor(), (0, 7)); // 'o' in "world"
}

#[test]
fn test_find_char_backward() {
    let mut state = MagnifierState::new(
        "hello world".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    state.set_cursor_for_test(0, 10); // End of line

    // Find 'o' backward
    state.find_char_backward('o');
    assert_eq!(state.cursor(), (0, 7)); // 'o' in "world"

    // Find 'o' backward again
    state.find_char_backward('o');
    assert_eq!(state.cursor(), (0, 4)); // 'o' in "hello"
}

#[test]
fn test_till_char_forward() {
    let mut state = MagnifierState::new(
        "hello world".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Till 'o' forward (stops before 'o')
    state.till_char_forward('o');
    assert_eq!(state.cursor(), (0, 3)); // Before first 'o'
}

#[test]
fn test_repeat_find() {
    let mut state = MagnifierState::new(
        "hello world hello".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Find 'o' - should find first 'o' in "hello"
    state.find_char_forward('o');
    assert_eq!(state.cursor(), (0, 4));

    // Repeat find - should find 'o' in "world"
    state.repeat_find();
    assert_eq!(state.cursor(), (0, 7));

    // Repeat again - should find 'o' in second "hello"
    state.repeat_find();
    assert_eq!(state.cursor(), (0, 16)); // "hello world hello"
                                         //                 ^
}

#[test]
fn test_repeat_find_reverse() {
    let mut state = MagnifierState::new(
        "hello world hello".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Find 'o' forward
    state.find_char_forward('o');
    state.find_char_forward('o');
    assert_eq!(state.cursor(), (0, 7));

    // Repeat in reverse
    state.repeat_find_reverse();
    assert_eq!(state.cursor(), (0, 4));
}

#[test]
fn test_change_char() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.change_char();
    assert_eq!(state.mode(), MagnifierMode::Insert);
    assert_eq!(state.line(0), Some("ello")); // 'h' deleted
}

#[test]
fn test_change_line() {
    let mut state = MagnifierState::new(
        "hello world".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    state.change_line();
    assert_eq!(state.mode(), MagnifierMode::Insert);
    assert_eq!(state.line(0), Some("")); // Line cleared
    assert_eq!(state.cursor(), (0, 0));
}

#[test]
fn test_change_to_eol() {
    let mut state = MagnifierState::new(
        "hello world".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    state.set_cursor_for_test(0, 6); // At 'w'
    state.change_to_eol();
    assert_eq!(state.mode(), MagnifierMode::Insert);
    assert_eq!(state.line(0), Some("hello ")); // "world" deleted
}

#[test]
fn test_replace_char() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.replace_char('H');
    assert_eq!(state.line(0), Some("Hello"));
    assert_eq!(state.mode(), MagnifierMode::Normal); // Stays in normal
}

#[test]
fn test_join_lines() {
    let mut state = MagnifierState::new(
        "hello\nworld".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    state.join_lines();
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.line(0), Some("hello world"));
}

#[test]
fn test_join_lines_empty() {
    let mut state = MagnifierState::new(
        "hello\n\nworld".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    state.join_lines();
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.line(0), Some("hello"));
}

#[test]
fn test_indent_line() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.indent_line();
    assert_eq!(state.line(0), Some("  hello"));
    assert_eq!(state.cursor(), (0, 2)); // Cursor moved right
}

#[test]
fn test_dedent_line() {
    let mut state =
        MagnifierState::new("  hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.set_cursor_for_test(0, 2);
    state.dedent_line();
    assert_eq!(state.line(0), Some("hello"));
    assert_eq!(state.cursor(), (0, 0)); // Cursor moved left
}

#[test]
fn test_dedent_line_with_tab() {
    let mut state =
        MagnifierState::new("\thello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.set_cursor_for_test(0, 1);
    state.dedent_line();
    assert_eq!(state.line(0), Some("hello"));
}

#[test]
fn test_undo() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    // Make a change
    state.push_undo();
    state.enter_insert_mode();
    state.insert_char('X');
    state.exit_insert_mode();

    // Undo
    state.undo();
    assert_eq!(state.line(0), Some("hello")); // Back to original
}

#[test]
fn test_redo() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    // Make a change
    state.push_undo();
    state.enter_insert_mode();
    state.insert_char('X');
    state.exit_insert_mode();

    // Undo then redo
    state.undo();
    state.redo();
    assert_eq!(state.line(0), Some("Xhello")); // Change restored
}

#[test]
fn test_multiple_undo_redo() {
    let mut state = MagnifierState::new("a".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    // Make multiple changes
    state.push_undo();
    state.enter_insert_mode();
    state.insert_char('b');
    state.exit_insert_mode();

    state.push_undo();
    state.enter_insert_mode();
    state.insert_char('c');
    state.exit_insert_mode();

    // Undo twice
    state.undo();
    assert_eq!(state.line(0), Some("ba"));
    state.undo();
    assert_eq!(state.line(0), Some("a"));

    // Redo once
    state.redo();
    assert_eq!(state.line(0), Some("ba"));
}

#[test]
fn test_pending_command_gg() {
    let mut state = MagnifierState::new(
        "Line 1\nLine 2\nLine 3".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    state.set_cursor_for_test(2, 0); // Last line

    // Set pending 'g'
    use lazycsv::magnifier::PendingCommand;
    state.set_pending(PendingCommand::G);
    assert!(state.has_pending());

    // Complete 'gg'
    state.take_pending();
    state.move_to_first_line();
    assert_eq!(state.cursor(), (0, 0));
}

#[test]
fn test_pending_command_dd() {
    let mut state = MagnifierState::new(
        "Line 1\nLine 2\nLine 3".to_string(),
        (RowIndex::new(0), ColIndex::new(0)),
    );

    // Set pending 'd'
    use lazycsv::magnifier::PendingCommand;
    state.set_pending(PendingCommand::D);
    assert!(state.has_pending());

    // Complete 'dd'
    state.take_pending();
    state.push_undo();
    state.delete_line();
    assert_eq!(state.line_count(), 2);
}

#[test]
fn test_command_mode_enter_exit() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.enter_command_mode();
    assert_eq!(state.mode(), MagnifierMode::Command);
    assert_eq!(state.command_buffer(), "");

    state.exit_command_mode();
    assert_eq!(state.mode(), MagnifierMode::Normal);
}

#[test]
fn test_command_mode_input() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.enter_command_mode();
    state.command_insert_char('w');
    state.command_insert_char('q');
    assert_eq!(state.command_buffer(), "wq");

    state.command_backspace();
    assert_eq!(state.command_buffer(), "w");
}

#[test]
fn test_command_mode_with_prefix() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.enter_command_mode_with("/");
    assert_eq!(state.command_buffer(), "/");

    state.command_insert_char('h');
    state.command_insert_char('e');
    assert_eq!(state.command_buffer(), "/he");
}

#[test]
fn test_mark_clean_with_content() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    // Make a change
    state.enter_insert_mode();
    state.insert_char('X');
    state.exit_insert_mode();

    assert!(state.is_dirty());

    // Mark as clean with new content
    state.mark_clean_with_content("Xhello".to_string());
    assert!(!state.is_dirty());
}

#[test]
fn test_visual_mode_exit() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    state.enter_visual_mode();
    assert_eq!(state.mode(), MagnifierMode::Visual);

    state.exit_visual_mode();
    assert_eq!(state.mode(), MagnifierMode::Normal);
}

#[test]
fn test_pending_display() {
    let mut state = MagnifierState::new("hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

    use lazycsv::magnifier::PendingCommand;
    state.set_pending(PendingCommand::G);
    assert_eq!(state.pending_display(), Some("g"));

    state.set_pending(PendingCommand::FindForward);
    assert_eq!(state.pending_display(), Some("f"));

    state.take_pending();
    assert_eq!(state.pending_display(), None);
}
