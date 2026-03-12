//! Tests for vim_editor visual mode (v, V, visual delete/yank/change)

use lazycsv::vim_editor::{VimEditor, VimMode};

// ============================================================================
// Visual Mode Entry/Exit Tests
// ============================================================================

#[test]
fn test_enter_visual_mode() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 2);

    editor.enter_visual_mode();
    assert!(matches!(editor.mode(), VimMode::Visual));
    assert_eq!(editor.visual_anchor(), Some((0, 2)));
}

#[test]
fn test_enter_visual_line_mode() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(1, 3);

    editor.enter_visual_line_mode();
    assert!(matches!(editor.mode(), VimMode::VisualLine));
    assert_eq!(editor.visual_anchor(), Some((1, 0)));
}

#[test]
fn test_exit_visual_mode() {
    let mut editor = VimEditor::new("Hello".to_string());
    editor.enter_visual_mode();

    editor.exit_visual_mode();
    assert!(matches!(editor.mode(), VimMode::Normal));
    assert_eq!(editor.visual_anchor(), None);
}

#[test]
fn test_toggle_visual_mode() {
    let mut editor = VimEditor::new("Hello".to_string());

    editor.toggle_visual_mode();
    assert!(matches!(editor.mode(), VimMode::Visual));

    editor.toggle_visual_mode();
    assert!(matches!(editor.mode(), VimMode::Normal));
}

#[test]
fn test_toggle_visual_line_mode() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());

    editor.toggle_visual_line_mode();
    assert!(matches!(editor.mode(), VimMode::VisualLine));

    editor.toggle_visual_line_mode();
    assert!(matches!(editor.mode(), VimMode::Normal));
}

// ============================================================================
// Visual Selection Query Tests
// ============================================================================

#[test]
fn test_get_visual_selection_charwise_forward() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 2);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(0, 6);

    let selection = editor.visual_selection();
    assert!(selection.is_some());
    if let Some(lazycsv::vim_editor::Selection::CharWise { start, end }) = selection {
        assert_eq!(start, (0, 2));
        assert_eq!(end, (0, 6));
    } else {
        panic!("Expected CharWise selection");
    }
}

#[test]
fn test_get_visual_selection_charwise_backward() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 6);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(0, 2);

    let selection = editor.visual_selection();
    assert!(selection.is_some());
    if let Some(lazycsv::vim_editor::Selection::CharWise { start, end }) = selection {
        assert_eq!(start, (0, 2));
        assert_eq!(end, (0, 6));
    } else {
        panic!("Expected CharWise selection");
    }
}

#[test]
fn test_get_visual_selection_linewise_forward() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(2, 0);

    let selection = editor.visual_selection();
    assert!(selection.is_some());
    if let Some(lazycsv::vim_editor::Selection::LineWise {
        start_line,
        end_line,
    }) = selection
    {
        assert_eq!(start_line, 0);
        assert_eq!(end_line, 2);
    } else {
        panic!("Expected LineWise selection");
    }
}

#[test]
fn test_get_visual_selection_linewise_backward() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(2, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(0, 0);

    let selection = editor.visual_selection();
    assert!(selection.is_some());
    if let Some(lazycsv::vim_editor::Selection::LineWise {
        start_line,
        end_line,
    }) = selection
    {
        assert_eq!(start_line, 0);
        assert_eq!(end_line, 2);
    } else {
        panic!("Expected LineWise selection");
    }
}

#[test]
fn test_get_visual_selection_normal_mode() {
    let editor = VimEditor::new("Hello".to_string());
    assert!(editor.visual_selection().is_none());
}

// ============================================================================
// Visual Delete Tests
// ============================================================================

#[test]
fn test_delete_selection_charwise_single_line() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 6);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(0, 10);

    editor.delete_selection();
    assert_eq!(editor.content(), "Hello ");
    assert_eq!(editor.cursor(), (0, 5)); // Cursor clamped to last valid position
    assert!(matches!(editor.mode(), VimMode::Normal));
}

#[test]
fn test_delete_selection_charwise_at_start() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(0, 4);

    editor.delete_selection();
    assert_eq!(editor.content(), " World");
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_delete_selection_charwise_multiline() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(1, 5);

    editor.delete_selection();
    assert_eq!(editor.content(), "Line 3");
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_delete_selection_linewise() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.delete_selection();
    assert_eq!(editor.content(), "Line 3");
    assert_eq!(editor.cursor(), (0, 0));
}

#[test]
fn test_delete_selection_linewise_single_line() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(1, 0);
    editor.enter_visual_line_mode();

    editor.delete_selection();
    assert_eq!(editor.content(), "Line 1\nLine 3");
    assert_eq!(editor.cursor(), (1, 0));
}

#[test]
fn test_delete_selection_linewise_all_lines() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.delete_selection();
    assert_eq!(editor.content(), "");
    assert_eq!(editor.cursor(), (0, 0));
}

// ============================================================================
// Visual Yank Tests
// ============================================================================

#[test]
fn test_yank_selection_charwise_single_line() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(0, 4);

    editor.yank_selection();
    assert!(matches!(editor.mode(), VimMode::Normal));

    editor.set_cursor_for_test(0, 11);
    editor.paste_below();
    // Paste below on single line doesn't make sense for char selection
    // but clipboard should contain "Hello"
    assert_eq!(editor.content(), "Hello World\nHello");
}

#[test]
fn test_yank_selection_charwise_multiline() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(1, 5);

    editor.yank_selection();
    editor.move_to_last_line();
    editor.paste_below();

    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 3\nLine 1\nLine 2");
}

#[test]
fn test_yank_selection_linewise() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.yank_selection();
    editor.move_to_last_line();
    editor.paste_below();

    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 3\nLine 1\nLine 2");
}

// ============================================================================
// Visual Change Tests
// ============================================================================

#[test]
fn test_change_selection_charwise() {
    let mut editor = VimEditor::new("Hello World".to_string());
    editor.set_cursor_for_test(0, 6);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(0, 10);

    editor.change_selection();
    assert_eq!(editor.content(), "Hello ");
    assert!(matches!(editor.mode(), VimMode::Insert));
}

#[test]
fn test_change_selection_linewise() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.change_selection();
    assert_eq!(editor.content(), "Line 3");
    assert!(matches!(editor.mode(), VimMode::Insert));
}

// ============================================================================
// Visual Indent/Dedent Tests
// ============================================================================

#[test]
fn test_indent_selection_charwise() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(1, 0);

    editor.indent_selection();
    assert_eq!(editor.content(), "  Line 1\n  Line 2\nLine 3");
    assert!(matches!(editor.mode(), VimMode::Normal));
}

#[test]
fn test_indent_selection_linewise() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.indent_selection();
    assert_eq!(editor.content(), "  Line 1\n  Line 2\nLine 3");
}

#[test]
fn test_indent_selection_all_lines() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.indent_selection();
    assert_eq!(editor.content(), "  Line 1\n  Line 2");
}

#[test]
fn test_dedent_selection_charwise() {
    let mut editor = VimEditor::new("  Line 1\n  Line 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_mode();
    editor.set_cursor_for_test(1, 0);

    editor.dedent_selection();
    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 3");
    assert!(matches!(editor.mode(), VimMode::Normal));
}

#[test]
fn test_dedent_selection_linewise() {
    let mut editor = VimEditor::new("  Line 1\n  Line 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.dedent_selection();
    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_dedent_selection_with_tabs() {
    let mut editor = VimEditor::new("\tLine 1\n\tLine 2".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.dedent_selection();
    assert_eq!(editor.content(), "Line 1\nLine 2");
}

#[test]
fn test_dedent_selection_mixed_indent() {
    let mut editor = VimEditor::new("  Line 1\n\tLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(2, 0);

    editor.dedent_selection();
    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 3");
}

// ============================================================================
// Combined Visual Operations Tests
// ============================================================================

#[test]
fn test_visual_delete_and_paste() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();

    editor.delete_selection();
    editor.move_to_last_line();
    editor.paste_below();

    assert_eq!(editor.content(), "Line 2\nLine 3\nLine 1");
}

#[test]
fn test_visual_yank_and_paste_multiple() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();

    editor.yank_selection();
    editor.move_down();
    editor.paste_below();
    editor.paste_below();

    assert_eq!(editor.content(), "Line 1\nLine 2\nLine 1\nLine 1");
}

#[test]
fn test_visual_select_change_type() {
    let mut editor = VimEditor::new("Line 1\nLine 2\nLine 3".to_string());
    editor.set_cursor_for_test(1, 0);

    // Start with character-wise
    editor.enter_visual_mode();
    assert!(matches!(editor.mode(), VimMode::Visual));

    // Switch to line-wise
    editor.exit_visual_mode();
    editor.enter_visual_line_mode();
    assert!(matches!(editor.mode(), VimMode::VisualLine));
}

#[test]
fn test_visual_indent_twice() {
    let mut editor = VimEditor::new("Line 1\nLine 2".to_string());
    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);

    editor.indent_selection();
    assert_eq!(editor.content(), "  Line 1\n  Line 2");

    editor.set_cursor_for_test(0, 0);
    editor.enter_visual_line_mode();
    editor.set_cursor_for_test(1, 0);
    editor.indent_selection();

    assert_eq!(editor.content(), "    Line 1\n    Line 2");
}

#[test]
fn test_visual_empty_line_selection() {
    let mut editor = VimEditor::new("Line 1\n\nLine 3".to_string());
    editor.set_cursor_for_test(1, 0);
    editor.enter_visual_line_mode();

    editor.delete_selection();
    assert_eq!(editor.content(), "Line 1\nLine 3");
}
