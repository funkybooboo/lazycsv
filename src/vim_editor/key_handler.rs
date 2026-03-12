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
            editor.move_right();
            editor.enter_insert_mode();
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
}
