//! Key handler with command pattern for vim editor
//!
//! This module implements a clean command pattern for handling keyboard input
//! in the vim editor, replacing the massive 250-line match statement with
//! composable, testable command objects.

use super::VimEditor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Result of handling a key press
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyResult {
    /// Key was handled, editor state may have changed
    Handled,
    /// Key was not recognized/handled
    Unhandled,
}

impl KeyResult {
    pub fn was_handled(self) -> bool {
        matches!(self, KeyResult::Handled)
    }
}

/// Handle a key event in normal or visual mode
pub fn handle_key(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    // Try handlers in order of priority
    if handle_escape(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    // Handle pending replace: r + next char (must come before count prefix
    // so that digits can be used as replacement characters)
    if editor.pending_command == Some(super::PendingCommand::Replace) {
        editor.pending_command = None;
        if let KeyCode::Char(c) = key.code {
            editor.push_undo();
            editor.replace_char(c);
        }
        editor.count_prefix = None;
        return KeyResult::Handled;
    }

    // Handle count prefix (0-9) - must come before other handlers
    if let KeyCode::Char(c) = key.code {
        if c.is_ascii_digit()
            && key.modifiers == KeyModifiers::NONE
            && handle_count_prefix(editor, c)
        {
            return KeyResult::Handled;
        }
    }

    if handle_mode_switch(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    // Visual mode operations must be checked before edit operations
    // to handle 'd', 'y', 'p' in visual mode correctly
    if handle_visual_mode(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    // Operator + motion (d{motion}, y{motion}, c{motion}) must be checked
    // before plain navigation so motions are applied to the operator range.
    if handle_operator_motion(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    if handle_navigation(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    if handle_edit_operations(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    if handle_search(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    if handle_command_mode(editor, key).was_handled() {
        return KeyResult::Handled;
    }

    KeyResult::Unhandled
}

/// Handle count prefix (1-9, or 0 if count already started)
fn handle_count_prefix(editor: &mut VimEditor, c: char) -> bool {
    let digit = c.to_digit(10).unwrap() as usize;

    // 0 is only valid if we already have a count (e.g., 10, 20)
    if digit == 0 && editor.count_prefix.is_none() {
        return false;
    }

    let current = editor.count_prefix.unwrap_or(0);
    editor.count_prefix = Some(current * 10 + digit);
    true
}

/// Handle Escape key
fn handle_escape(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    if !matches!(key.code, KeyCode::Esc) {
        return KeyResult::Unhandled;
    }

    use super::VimMode;
    if editor.mode == VimMode::Visual || editor.mode == VimMode::VisualLine {
        editor.exit_visual_mode();
    } else {
        editor.count_prefix = None;
        editor.pending_command = None;
    }
    KeyResult::Handled
}

/// Handle mode switching commands (i, a, A, I, o, O)
fn handle_mode_switch(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    if key.modifiers != KeyModifiers::NONE && key.modifiers != KeyModifiers::SHIFT {
        return KeyResult::Unhandled;
    }

    match key.code {
        KeyCode::Char('i') if key.modifiers == KeyModifiers::NONE => {
            editor.push_undo();
            editor.enter_insert_mode();
            KeyResult::Handled
        }
        KeyCode::Char('a') if key.modifiers == KeyModifiers::NONE => {
            editor.push_undo();
            editor.enter_insert_mode();
            editor.move_right();
            KeyResult::Handled
        }
        KeyCode::Char('A') => {
            editor.push_undo();
            editor.move_to_line_end();
            editor.enter_insert_mode();
            editor.cursor.1 += 1; // Move after last char in insert mode
            KeyResult::Handled
        }
        KeyCode::Char('I') => {
            editor.push_undo();
            editor.move_to_line_start();
            editor.enter_insert_mode();
            KeyResult::Handled
        }
        KeyCode::Char('o') if key.modifiers == KeyModifiers::NONE => {
            editor.push_undo();
            editor.insert_line_below();
            KeyResult::Handled
        }
        KeyCode::Char('O') => {
            editor.push_undo();
            editor.insert_line_above();
            KeyResult::Handled
        }
        _ => KeyResult::Unhandled,
    }
}

/// Handle navigation commands (hjkl, w/b/e, 0/$, gg/G, etc.)
/// Handle operator + motion combinations (d{motion}, y{motion}, c{motion}).
/// Records cursor position, executes the motion, then applies the operator
/// over the range between the old and new cursor positions.
fn handle_operator_motion(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    use super::{PendingCommand, VimMode};

    let pending = match editor.pending_command {
        Some(PendingCommand::D) | Some(PendingCommand::Y) | Some(PendingCommand::C) => {
            editor.pending_command.take().unwrap()
        }
        _ => return KeyResult::Unhandled,
    };

    // Check if this key is a valid motion
    let is_motion = matches!(
        (key.code, key.modifiers),
        (KeyCode::Char('w'), KeyModifiers::NONE)
            | (KeyCode::Char('b'), KeyModifiers::NONE)
            | (KeyCode::Char('e'), KeyModifiers::NONE)
            | (KeyCode::Char('$'), KeyModifiers::NONE)
            | (KeyCode::Char('0'), KeyModifiers::NONE)
            | (KeyCode::Char('^'), KeyModifiers::NONE)
            | (KeyCode::Char('h'), KeyModifiers::NONE)
            | (KeyCode::Char('l'), KeyModifiers::NONE)
            | (KeyCode::Char('j'), KeyModifiers::NONE)
            | (KeyCode::Char('k'), KeyModifiers::NONE)
            | (KeyCode::Left, _)
            | (KeyCode::Right, _)
            | (KeyCode::Up, _)
            | (KeyCode::Down, _)
            | (KeyCode::Char('G'), KeyModifiers::NONE | KeyModifiers::SHIFT)
    );

    if !is_motion {
        // Not a motion — restore pending command and let other handlers deal with it
        editor.pending_command = Some(pending);
        return KeyResult::Unhandled;
    }

    // Record start position
    let start_line = editor.cursor.0;
    let start_col = editor.cursor.1;

    // Execute the motion — set the count back so the motion method can consume it.
    // Motion methods like move_next_word() call take_count() internally.
    // Temporarily switch to Insert mode so clamp_cursor allows cursor at end of line
    // (Normal mode clamps to line_len-1, which would cut off the last character).
    let saved_mode = editor.mode;
    editor.mode = VimMode::Insert;
    let count = editor.take_count().max(1);
    editor.count_prefix = Some(count);
    match key.code {
        KeyCode::Char('w') => editor.move_next_word(),
        KeyCode::Char('b') => editor.move_prev_word(),
        KeyCode::Char('e') => {
            editor.move_end_word();
            // 'e' is inclusive — move one past the end for delete range
            editor.cursor.1 += 1;
        }
        KeyCode::Char('$') => {
            editor.move_to_line_end();
            // '$' is inclusive
            editor.cursor.1 += 1;
        }
        KeyCode::Char('0') | KeyCode::Char('^') => editor.move_to_line_start(),
        KeyCode::Char('h') | KeyCode::Left => {
            for _ in 0..count {
                editor.move_left();
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            for _ in 0..count {
                editor.move_right();
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            for _ in 0..count {
                editor.move_down();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            for _ in 0..count {
                editor.move_up();
            }
        }
        KeyCode::Char('G') => editor.move_to_last_line(),
        _ => {}
    }
    // Restore mode and ensure count is cleared
    editor.mode = saved_mode;
    editor.count_prefix = None;

    let end_line = editor.cursor.0;
    let end_col = editor.cursor.1;

    // Determine range (handle backwards motions)
    let (from_line, from_col, to_line, to_col) = if (start_line, start_col) <= (end_line, end_col) {
        (start_line, start_col, end_line, end_col)
    } else {
        (end_line, end_col, start_line, start_col)
    };

    editor.push_undo();

    if from_line == to_line {
        // Same line: delete/yank the character range
        let line = &editor.lines[from_line];
        let chars: Vec<char> = line.chars().collect();
        let safe_from = from_col.min(chars.len());
        let safe_to = to_col.min(chars.len());
        let deleted: String = chars[safe_from..safe_to].iter().collect();

        match pending {
            PendingCommand::D | PendingCommand::C => {
                let new_line = format!(
                    "{}{}",
                    chars[..safe_from].iter().collect::<String>(),
                    chars[safe_to..].iter().collect::<String>()
                );
                editor.lines[from_line] = new_line;
                editor.cursor = (from_line, safe_from);
                editor.clipboard = vec![deleted];
            }
            PendingCommand::Y => {
                editor.clipboard = vec![deleted];
                editor.cursor = (start_line, start_col); // Restore cursor for yank
            }
            _ => {}
        }
    } else {
        // Multi-line: delete/yank full lines in range
        let deleted_lines: Vec<String> = editor.lines[from_line..=to_line].to_vec();

        match pending {
            PendingCommand::D | PendingCommand::C => {
                for _ in from_line..=to_line {
                    if from_line < editor.lines.len() {
                        editor.lines.remove(from_line);
                    }
                }
                if editor.lines.is_empty() {
                    editor.lines.push(String::new());
                }
                editor.cursor = (from_line.min(editor.lines.len() - 1), 0);
                editor.clipboard = deleted_lines;
            }
            PendingCommand::Y => {
                editor.clipboard = deleted_lines;
                editor.cursor = (start_line, start_col);
            }
            _ => {}
        }
    }

    // 'c' enters insert mode after deleting
    if pending == PendingCommand::C {
        editor.mode = VimMode::Insert;
    }

    editor.count_prefix = None;
    KeyResult::Handled
}

fn handle_navigation(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    let handled = match (key.code, key.modifiers) {
        // Basic movement - motions handle count internally via take_count()
        (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, _) => {
            editor.move_left();
            true
        }
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
            editor.move_down();
            true
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
            editor.move_up();
            true
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, _) => {
            editor.move_right();
            true
        }

        // Word motions - motions handle count internally via take_count()
        (KeyCode::Char('w'), KeyModifiers::NONE) => {
            editor.move_next_word();
            true
        }
        (KeyCode::Char('b'), KeyModifiers::NONE) => {
            editor.move_prev_word();
            true
        }
        (KeyCode::Char('e'), KeyModifiers::NONE) => {
            editor.move_end_word();
            true
        }

        // Line start/end
        (KeyCode::Char('0'), KeyModifiers::NONE) => {
            editor.move_to_line_start();
            true
        }
        (KeyCode::Char('$'), KeyModifiers::NONE) => {
            editor.move_to_line_end();
            true
        }
        (KeyCode::Char('^'), KeyModifiers::NONE) => {
            editor.move_to_line_start();
            true
        }

        // File start/end
        (KeyCode::Char('g'), KeyModifiers::NONE) => {
            use super::PendingCommand;
            if editor.pending_command == Some(PendingCommand::G) {
                editor.pending_command = None;
                editor.move_to_first_line();
                true
            } else {
                editor.pending_command = Some(PendingCommand::G);
                editor.count_prefix = None; // Clear count for gg
                return KeyResult::Handled; // Don't clear count yet
            }
        }
        (KeyCode::Char('G'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            if let Some(line) = editor.count_prefix {
                editor.move_to_line(line.saturating_sub(1));
            } else {
                editor.move_to_last_line();
            }
            true
        }

        _ => false,
    };

    if handled {
        // Note: count_prefix is cleared by the motion functions themselves via take_count()
        KeyResult::Handled
    } else {
        KeyResult::Unhandled
    }
}

/// Handle edit operations (x, d, y, p, P, c, s, etc.)
fn handle_edit_operations(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    use super::VimMode;

    let count = editor.count_prefix.unwrap_or(1);

    let handled = match (key.code, key.modifiers) {
        // Delete character
        (KeyCode::Char('x'), KeyModifiers::NONE) => {
            editor.push_undo();
            for _ in 0..count {
                editor.delete_char();
            }
            true
        }

        // Delete operator
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            use super::PendingCommand;
            if editor.pending_command == Some(PendingCommand::D) {
                // dd: delete line
                editor.pending_command = None;
                editor.push_undo();
                for _ in 0..count {
                    editor.delete_line();
                }
                true
            } else {
                editor.pending_command = Some(PendingCommand::D);
                return KeyResult::Handled; // Don't clear count yet
            }
        }

        // Delete to end of line
        (KeyCode::Char('D'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            editor.push_undo();
            editor.delete_to_eol();
            true
        }

        // Yank operator
        (KeyCode::Char('y'), KeyModifiers::NONE) => {
            use super::PendingCommand;
            if editor.mode == VimMode::Visual || editor.mode == VimMode::VisualLine {
                editor.yank_selection();
                editor.exit_visual_mode();
                true
            } else if editor.pending_command == Some(PendingCommand::Y) {
                // yy: yank line
                editor.pending_command = None;
                for _ in 0..count {
                    editor.yank_line();
                }
                true
            } else {
                editor.pending_command = Some(PendingCommand::Y);
                return KeyResult::Handled; // Don't clear count yet
            }
        }

        // Paste
        (KeyCode::Char('p'), KeyModifiers::NONE) => {
            editor.push_undo();
            for _ in 0..count {
                editor.paste_below();
            }
            true
        }
        (KeyCode::Char('P'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            editor.push_undo();
            for _ in 0..count {
                editor.paste_above();
            }
            true
        }

        // Change operator
        (KeyCode::Char('c'), KeyModifiers::NONE) => {
            use super::PendingCommand;
            if editor.pending_command == Some(PendingCommand::C) {
                // cc: change line
                editor.pending_command = None;
                editor.push_undo();
                editor.delete_line();
                editor.enter_insert_mode();
                true
            } else {
                editor.pending_command = Some(PendingCommand::C);
                return KeyResult::Handled; // Don't clear count yet
            }
        }

        // Substitute
        (KeyCode::Char('s'), KeyModifiers::NONE) => {
            editor.push_undo();
            editor.delete_char();
            editor.enter_insert_mode();
            true
        }

        // Join lines
        (KeyCode::Char('J'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            editor.push_undo();
            for _ in 0..count {
                editor.join_lines();
            }
            true
        }

        // Replace single character
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            editor.pending_command = Some(super::PendingCommand::Replace);
            return KeyResult::Handled; // Don't clear count
        }

        // Undo/Redo
        (KeyCode::Char('u'), KeyModifiers::NONE) => {
            for _ in 0..count {
                editor.undo();
            }
            true
        }
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            for _ in 0..count {
                editor.redo();
            }
            true
        }

        _ => false,
    };

    if handled {
        editor.count_prefix = None;
        KeyResult::Handled
    } else {
        KeyResult::Unhandled
    }
}

/// Handle visual mode commands (v, V, gv)
fn handle_visual_mode(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    use super::VimMode;

    match (key.code, key.modifiers) {
        (KeyCode::Char('v'), KeyModifiers::NONE) => {
            if editor.mode == VimMode::Visual {
                editor.exit_visual_mode();
            } else {
                editor.enter_visual_mode();
            }
            editor.count_prefix = None;
            KeyResult::Handled
        }
        (KeyCode::Char('V'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            if editor.mode == VimMode::VisualLine {
                editor.exit_visual_mode();
            } else {
                editor.enter_visual_line_mode();
            }
            editor.count_prefix = None;
            KeyResult::Handled
        }
        (KeyCode::Char('d'), KeyModifiers::NONE)
            if editor.mode == VimMode::Visual || editor.mode == VimMode::VisualLine =>
        {
            editor.push_undo();
            editor.delete_selection();
            editor.exit_visual_mode();
            editor.count_prefix = None;
            KeyResult::Handled
        }
        (KeyCode::Char('p'), KeyModifiers::NONE)
            if editor.mode == VimMode::Visual || editor.mode == VimMode::VisualLine =>
        {
            editor.push_undo();
            editor.delete_selection();
            editor.paste_below();
            editor.exit_visual_mode();
            editor.count_prefix = None;
            KeyResult::Handled
        }
        _ => KeyResult::Unhandled,
    }
}

/// Handle search commands (/, n, N, *)
fn handle_search(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    use super::VimMode;

    match (key.code, key.modifiers) {
        (KeyCode::Char('/'), KeyModifiers::NONE) => {
            editor.mode = VimMode::Command;
            editor.command_buffer.clear();
            editor.command_buffer.push('/');
            editor.count_prefix = None;
            KeyResult::Handled
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            editor.jump_to_next_match();
            editor.count_prefix = None;
            KeyResult::Handled
        }
        (KeyCode::Char('N'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            editor.jump_to_prev_match();
            editor.count_prefix = None;
            KeyResult::Handled
        }
        (KeyCode::Char('*'), KeyModifiers::NONE) => {
            editor.search_word_under_cursor();
            editor.count_prefix = None;
            KeyResult::Handled
        }
        _ => KeyResult::Unhandled,
    }
}

/// Handle command mode (:)
fn handle_command_mode(editor: &mut VimEditor, key: KeyEvent) -> KeyResult {
    use super::VimMode;

    match (key.code, key.modifiers) {
        (KeyCode::Char(':'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            editor.mode = VimMode::Command;
            editor.command_buffer.clear();
            editor.count_prefix = None;
            KeyResult::Handled
        }
        _ => KeyResult::Unhandled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vim_editor::VimEditor;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn key_char(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    fn key_shift(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
    }

    #[test]
    fn test_count_prefix_movement() {
        let mut editor = VimEditor::new("SELECT * FROM table".to_string());

        // Press '5'
        let result = handle_key(&mut editor, key_char('5'));
        assert_eq!(result, KeyResult::Handled);
        assert_eq!(editor.count_prefix, Some(5));
        assert_eq!(editor.cursor(), (0, 0)); // Cursor shouldn't move yet

        // Press 'l'
        let result = handle_key(&mut editor, key_char('l'));
        assert_eq!(result, KeyResult::Handled);
        assert_eq!(editor.count_prefix, None); // Count should be cleared
        assert_eq!(editor.cursor(), (0, 5)); // Should have moved 5 positions
    }

    // ── Operator + motion (d/y/c + w/e/$) ─────────────────────

    #[test]
    fn test_dw_delete_word() {
        let mut editor = VimEditor::new("hello world".to_string());
        // dw should delete "hello "
        handle_key(&mut editor, key_char('d'));
        handle_key(&mut editor, key_char('w'));
        assert_eq!(editor.content(), "world");
    }

    #[test]
    fn test_dw_delete_last_word_on_line() {
        let mut editor = VimEditor::new("select * from numbers".to_string());
        // Move cursor to 'n' in 'numbers' (position 14)
        for _ in 0..14 {
            handle_key(&mut editor, key_char('l'));
        }
        assert_eq!(editor.cursor(), (0, 14));
        // dw should delete "numbers"
        handle_key(&mut editor, key_char('d'));
        handle_key(&mut editor, key_char('w'));
        assert_eq!(editor.content(), "select * from ");
    }

    #[test]
    fn test_de_delete_to_end_of_word() {
        let mut editor = VimEditor::new("hello world".to_string());
        // de should delete "hello" (inclusive)
        handle_key(&mut editor, key_char('d'));
        handle_key(&mut editor, key_char('e'));
        assert_eq!(editor.content(), " world");
    }

    #[test]
    fn test_d_dollar_delete_to_end_of_line() {
        let mut editor = VimEditor::new("hello world".to_string());
        // Move to 'w'
        for _ in 0..6 {
            handle_key(&mut editor, key_char('l'));
        }
        // d$ should delete from 'w' to end
        handle_key(&mut editor, key_char('d'));
        handle_key(&mut editor, key_char('$'));
        assert_eq!(editor.content(), "hello ");
    }

    #[test]
    fn test_db_delete_back_word() {
        let mut editor = VimEditor::new("hello world".to_string());
        // Move to 'w'
        handle_key(&mut editor, key_char('w'));
        // db should delete backwards
        handle_key(&mut editor, key_char('d'));
        handle_key(&mut editor, key_char('b'));
        assert_eq!(editor.content(), "world");
    }

    #[test]
    fn test_yw_yank_word() {
        let mut editor = VimEditor::new("hello world".to_string());
        handle_key(&mut editor, key_char('y'));
        handle_key(&mut editor, key_char('w'));
        // Content shouldn't change
        assert_eq!(editor.content(), "hello world");
        // Clipboard should have the yanked text
        assert_eq!(editor.clipboard, vec!["hello "]);
    }

    #[test]
    fn test_cw_change_word() {
        let mut editor = VimEditor::new("hello world".to_string());
        handle_key(&mut editor, key_char('c'));
        handle_key(&mut editor, key_char('w'));
        // "hello " deleted, now in insert mode
        assert_eq!(editor.content(), "world");
        assert_eq!(editor.mode(), crate::vim_editor::VimMode::Insert);
    }

    // ── J (join lines) ────────────────────────────────────────

    #[test]
    fn test_join_lines_key() {
        let mut editor = VimEditor::new("line one\nline two".to_string());
        handle_key(&mut editor, key_shift('J'));
        assert_eq!(editor.content(), "line one line two");
    }

    #[test]
    fn test_join_lines_last_line_noop() {
        let mut editor = VimEditor::new("only line".to_string());
        handle_key(&mut editor, key_shift('J'));
        assert_eq!(editor.content(), "only line");
    }

    // ── / search via command mode ─────────────────────────────

    #[test]
    fn test_slash_search() {
        let mut editor = VimEditor::new("hello world hello".to_string());
        // Enter search mode
        editor.handle_key(key_char('/'));
        assert_eq!(editor.mode(), crate::vim_editor::VimMode::Command);

        // Type pattern
        editor.handle_key(key_char('w'));
        editor.handle_key(key_char('o'));
        editor.handle_key(key_char('r'));

        // Execute search
        editor.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(editor.mode(), crate::vim_editor::VimMode::Normal);
        assert_eq!(editor.search_pattern(), Some("wor"));
    }

    // ── :%s substitution ──────────────────────────────────────

    #[test]
    fn test_substitution_global() {
        let mut editor = VimEditor::new("foo bar foo".to_string());
        editor.execute_substitution("%s/foo/baz/g");
        assert_eq!(editor.content(), "baz bar baz");
    }

    #[test]
    fn test_substitution_first_only() {
        let mut editor = VimEditor::new("foo bar foo".to_string());
        editor.execute_substitution("%s/foo/baz/");
        assert_eq!(editor.content(), "baz bar foo");
    }

    #[test]
    fn test_substitution_multiline() {
        let mut editor = VimEditor::new("aaa\nbbb\naaa".to_string());
        editor.execute_substitution("%s/aaa/ccc/g");
        assert_eq!(editor.content(), "ccc\nbbb\nccc");
    }

    #[test]
    fn test_substitution_empty_pattern_noop() {
        let mut editor = VimEditor::new("hello".to_string());
        editor.execute_substitution("%s//world/g");
        assert_eq!(editor.content(), "hello");
    }
}
