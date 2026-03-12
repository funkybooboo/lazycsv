//! Magnifier Mode - Full vim editor for complex cell editing
//!
//! This module is a thin wrapper around the `vim_editor` module, adding
//! magnifier-specific functionality like cell position tracking and dirty state.

use crate::domain::position::{ColIndex, RowIndex};
use crate::vim_editor::{
    PendingCommand as VimPendingCommand, Selection as VimSelection, VimEditor, VimMode,
};

// Re-export types from vim_editor for public API
pub type MagnifierMode = VimMode;
pub type PendingCommand = VimPendingCommand;

/// Selection range for visual mode operations (converted from vim_editor::Selection)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// Character-wise selection (like vim's `v`)
    CharWise {
        start: (usize, usize),
        end: (usize, usize),
    },
    /// Line-wise selection (like vim's `V`)
    LineWise { start_line: usize, end_line: usize },
}

impl From<VimSelection> for Selection {
    fn from(sel: VimSelection) -> Self {
        match sel {
            VimSelection::CharWise { start, end } => Selection::CharWise { start, end },
            VimSelection::LineWise {
                start_line,
                end_line,
            } => Selection::LineWise {
                start_line,
                end_line,
            },
        }
    }
}

/// Complete state for vim-style text editor within magnifier mode
///
/// Thin wrapper around `VimEditor` that adds magnifier-specific functionality.
#[derive(Debug, Clone)]
pub struct MagnifierState {
    /// Vim editor for all text editing operations
    editor: VimEditor,

    /// Original cell position in the CSV (for display)
    cell_position: (RowIndex, ColIndex),

    /// Original content (for dirty checking)
    original_content: String,

    /// Command buffer for ex mode commands (:w, :q, etc.)
    command_buffer: String,
}

impl MagnifierState {
    /// Create a new magnifier state from cell content
    pub fn new(content: String, position: (RowIndex, ColIndex)) -> Self {
        let editor = VimEditor::new(content.clone());

        Self {
            editor,
            cell_position: position,
            original_content: content,
            command_buffer: String::new(),
        }
    }

    // ============================================================================
    // State Access Methods
    // ============================================================================

    /// Get the current content as a single string with newlines
    pub fn content(&self) -> String {
        self.editor.content()
    }

    /// Check if content has been modified
    pub fn is_dirty(&self) -> bool {
        self.content() != self.original_content
    }

    /// Mark content as clean (after saving to document)
    pub fn mark_clean_with_content(&mut self, content: String) {
        self.original_content = content;
    }

    /// Get the current mode
    pub fn mode(&self) -> MagnifierMode {
        self.editor.mode()
    }

    /// Get cursor position
    pub fn cursor(&self) -> (usize, usize) {
        self.editor.cursor()
    }

    /// Get cell position in CSV
    pub fn cell_position(&self) -> (RowIndex, ColIndex) {
        self.cell_position
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.editor.line_count()
    }

    /// Get a specific line
    pub fn line(&self, line: usize) -> Option<&str> {
        self.editor.line(line)
    }

    /// Get all lines
    pub fn lines(&self) -> &[String] {
        self.editor.lines()
    }

    /// Set cursor position (for testing)
    pub fn set_cursor_for_test(&mut self, line: usize, col: usize) {
        self.editor.set_cursor_for_test(line, col);
    }

    // ============================================================================
    // Mode Management
    // ============================================================================

    pub fn enter_insert_mode(&mut self) {
        self.editor.enter_insert_mode();
    }

    pub fn exit_insert_mode(&mut self) {
        self.editor.exit_insert_mode();
    }

    pub fn enter_visual_mode(&mut self) {
        self.editor.enter_visual_mode();
    }

    pub fn enter_visual_line_mode(&mut self) {
        self.editor.enter_visual_line_mode();
    }

    pub fn exit_visual_mode(&mut self) {
        self.editor.exit_visual_mode();
    }

    pub fn visual_selection(&self) -> Option<Selection> {
        self.editor.visual_selection().map(Selection::from)
    }

    // ============================================================================
    // Command Mode (Magnifier-specific for :w/:q)
    // ============================================================================

    pub fn enter_command_mode(&mut self) {
        self.command_buffer.clear();
        self.editor.enter_command_mode();
    }

    pub fn enter_command_mode_with(&mut self, prefix: &str) {
        self.command_buffer = prefix.to_string();
        self.editor.enter_command_mode();
    }

    pub fn exit_command_mode(&mut self) {
        self.command_buffer.clear();
        self.editor.exit_command_mode();
    }

    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    pub fn command_insert_char(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    pub fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    // ============================================================================
    // Count Prefix
    // ============================================================================

    pub fn set_count_prefix(&mut self, count: usize) {
        self.editor.set_count_prefix(count);
    }

    pub fn take_count(&mut self) -> usize {
        self.editor.take_count()
    }

    // ============================================================================
    // Pending Commands
    // ============================================================================

    pub fn set_pending(&mut self, cmd: PendingCommand) {
        self.editor.set_pending(cmd);
    }

    pub fn take_pending(&mut self) -> Option<PendingCommand> {
        self.editor.take_pending()
    }

    pub fn has_pending(&self) -> bool {
        self.editor.has_pending()
    }

    pub fn pending_display(&self) -> Option<&str> {
        self.editor.pending_display()
    }

    // ============================================================================
    // Motion Commands - Delegate to VimEditor
    // ============================================================================

    pub fn move_left(&mut self) {
        self.editor.move_left();
    }

    pub fn move_right(&mut self) {
        self.editor.move_right();
    }

    pub fn move_up(&mut self) {
        self.editor.move_up();
    }

    pub fn move_down(&mut self) {
        self.editor.move_down();
    }

    pub fn move_to_line_start(&mut self) {
        self.editor.move_to_line_start();
    }

    pub fn move_to_line_end(&mut self) {
        self.editor.move_to_line_end();
    }

    pub fn move_to_first_non_blank(&mut self) {
        self.editor.move_to_first_non_blank();
    }

    pub fn move_to_first_line(&mut self) {
        self.editor.move_to_first_line();
    }

    pub fn move_to_last_line(&mut self) {
        self.editor.move_to_last_line();
    }

    pub fn move_to_line(&mut self, line_number: usize) {
        self.editor.move_to_line(line_number);
    }

    pub fn move_next_word(&mut self) {
        self.editor.move_next_word();
    }

    pub fn move_prev_word(&mut self) {
        self.editor.move_prev_word();
    }

    pub fn move_end_word(&mut self) {
        self.editor.move_end_word();
    }

    pub fn find_char_forward(&mut self, ch: char) {
        self.editor.find_char_forward(ch);
    }

    pub fn find_char_backward(&mut self, ch: char) {
        self.editor.find_char_backward(ch);
    }

    pub fn till_char_forward(&mut self, ch: char) {
        self.editor.till_char_forward(ch);
    }

    pub fn till_char_backward(&mut self, ch: char) {
        self.editor.till_char_backward(ch);
    }

    pub fn repeat_find(&mut self) {
        self.editor.repeat_find();
    }

    pub fn repeat_find_reverse(&mut self) {
        self.editor.repeat_find_reverse();
    }

    pub fn repeat_find_forward(&mut self) {
        self.editor.repeat_find();
    }

    pub fn repeat_find_backward(&mut self) {
        self.editor.repeat_find_reverse();
    }

    // ============================================================================
    // Operator Commands - Delegate to VimEditor
    // ============================================================================

    pub fn insert_char(&mut self, c: char) {
        self.editor.insert_char(c);
    }

    pub fn delete_char_before(&mut self) {
        self.editor.delete_char_before();
    }

    pub fn delete_char_at(&mut self) {
        self.editor.delete_char_at();
    }

    pub fn newline(&mut self) {
        self.editor.newline();
    }

    pub fn delete_char(&mut self) {
        self.editor.delete_char();
    }

    pub fn delete_line(&mut self) {
        self.editor.delete_line();
    }

    pub fn yank_line(&mut self) {
        self.editor.yank_line();
    }

    pub fn paste_below(&mut self) {
        self.editor.paste_below();
    }

    pub fn paste_above(&mut self) {
        self.editor.paste_above();
    }

    pub fn substitute_char(&mut self) {
        self.editor.substitute_char();
    }

    pub fn insert_before(&mut self) {
        self.editor.insert_before();
    }

    pub fn insert_after(&mut self) {
        self.editor.insert_after();
    }

    pub fn insert_line_below(&mut self) {
        self.editor.insert_line_below();
    }

    pub fn insert_line_above(&mut self) {
        self.editor.insert_line_above();
    }

    pub fn insert_at_line_start(&mut self) {
        self.editor.insert_at_line_start();
    }

    pub fn insert_at_line_end(&mut self) {
        self.editor.insert_at_line_end();
    }

    pub fn change_char(&mut self) {
        self.editor.change_char();
    }

    pub fn change_line(&mut self) {
        self.editor.change_line();
    }

    pub fn change_to_eol(&mut self) {
        self.editor.change_to_eol();
    }

    pub fn replace_char(&mut self, c: char) {
        self.editor.replace_char(c);
    }

    pub fn join_lines(&mut self) {
        self.editor.join_lines();
    }

    pub fn indent_line(&mut self) {
        self.editor.indent_line();
    }

    pub fn dedent_line(&mut self) {
        self.editor.dedent_line();
    }

    // ============================================================================
    // Visual Mode Operations
    // ============================================================================

    pub fn delete_selection(&mut self) {
        self.editor.delete_selection();
    }

    pub fn yank_selection(&mut self) {
        self.editor.yank_selection();
    }

    pub fn change_selection(&mut self) {
        self.editor.change_selection();
    }

    pub fn indent_selection(&mut self) {
        self.editor.indent_selection();
    }

    pub fn dedent_selection(&mut self) {
        self.editor.dedent_selection();
    }

    // ============================================================================
    // Search Operations
    // ============================================================================

    pub fn search_forward(&mut self, pattern: String) {
        self.editor.search_forward(pattern);
    }

    pub fn search_word_under_cursor(&mut self) {
        self.editor.search_word_under_cursor();
    }

    /// Get word under cursor (for display or * search)
    pub fn word_under_cursor(&self) -> Option<String> {
        self.editor.word_under_cursor()
    }

    pub fn jump_to_next_match(&mut self) {
        self.editor.jump_to_next_match();
    }

    pub fn jump_to_prev_match(&mut self) {
        self.editor.jump_to_prev_match();
    }

    pub fn clear_search(&mut self) {
        self.editor.clear_search();
    }

    pub fn search_pattern(&self) -> Option<&str> {
        self.editor.search_pattern()
    }

    pub fn search_matches(&self) -> &[(usize, usize)] {
        self.editor.search_matches()
    }

    pub fn current_match_index(&self) -> Option<usize> {
        self.editor.current_match_index()
    }

    pub fn search_match_count(&self) -> usize {
        self.editor.search_match_count()
    }

    // ============================================================================
    // Undo/Redo Operations
    // ============================================================================

    pub fn push_undo(&mut self) {
        self.editor.push_undo();
    }

    pub fn undo(&mut self) {
        self.editor.undo();
    }

    pub fn redo(&mut self) {
        self.editor.redo();
    }

    pub fn can_undo(&self) -> bool {
        self.editor.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.editor.can_redo()
    }
}
