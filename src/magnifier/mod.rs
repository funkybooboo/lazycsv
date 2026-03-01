//! Magnifier Mode - Full vim editor for complex cell editing
//!
//! This module implements a complete vim-like text editor for editing cell content
//! with multi-line support, vim motions, and vim operators.
//!
//! ## Features
//!
//! - Full vim motions: hjkl, w/b/e, 0/$, gg/G, count prefixes
//! - Full vim operators: dd, yy, p, x, s, i/a/o/O
//! - Multi-line editing with proper CSV escaping
//! - Internal clipboard for magnifier operations
//! - Mode switching between Normal and Insert
//!
//! ## Usage
//!
//! Open magnifier on current cell, edit with vim commands, and get the result:
//!
//! 1. Create state: `MagnifierState::new(content, position)`
//! 2. Edit: `move_down()`, `delete_line()`, etc.
//! 3. Get result: `get_content()`

use crate::domain::position::{ColIndex, RowIndex};

/// Vim mode within magnifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnifierMode {
    /// Normal mode - navigation and commands
    Normal,
    /// Insert mode - text input
    Insert,
}

/// State for magnifier mode editing
#[derive(Debug, Clone)]
pub struct MagnifierState {
    /// Text buffer as vector of lines
    lines: Vec<String>,

    /// Current vim mode within magnifier
    mode: MagnifierMode,

    /// Cursor position (line, column) - 0-indexed
    /// Line: 0 to lines.len()-1
    /// Column: 0 to line.len() (can be at end for insert)
    cursor: (usize, usize),

    /// Original cell position in the CSV (for display)
    cell_position: (RowIndex, ColIndex),

    /// Original content (for dirty checking and cancel)
    original_content: String,

    /// Internal clipboard for dd/yy/p operations
    clipboard: Vec<String>,

    /// Count prefix for vim commands (e.g., 5j means count_prefix = 5)
    count_prefix: Option<usize>,
}

impl MagnifierState {
    /// Create a new magnifier state from cell content
    ///
    /// # Arguments
    ///
    /// * `content` - The cell content to edit (may contain newlines)
    /// * `position` - The (row, col) position of the cell in the CSV
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::magnifier::MagnifierState;
    /// use lazycsv::domain::position::{RowIndex, ColIndex};
    ///
    /// let state = MagnifierState::new(
    ///     "Hello\nWorld".to_string(),
    ///     (RowIndex::new(5), ColIndex::new(2))
    /// );
    /// assert_eq!(state.lines().len(), 2);
    /// ```
    pub fn new(content: String, position: (RowIndex, ColIndex)) -> Self {
        // Split content into lines, preserving empty lines
        let lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(String::from).collect()
        };

        Self {
            lines,
            mode: MagnifierMode::Normal,
            cursor: (0, 0),
            cell_position: position,
            original_content: content,
            clipboard: Vec::new(),
            count_prefix: None,
        }
    }

    /// Get the current content as a single string with newlines
    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    /// Check if content has been modified
    pub fn is_dirty(&self) -> bool {
        self.get_content() != self.original_content
    }

    /// Get the current mode
    pub fn mode(&self) -> MagnifierMode {
        self.mode
    }

    /// Get the current cursor position (line, column)
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Get the cell position in the CSV
    pub fn cell_position(&self) -> (RowIndex, ColIndex) {
        self.cell_position
    }

    /// Get the number of lines in the buffer
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get a reference to a specific line
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    /// Get all lines as a slice
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Enter insert mode
    pub fn enter_insert_mode(&mut self) {
        self.mode = MagnifierMode::Insert;
    }

    /// Exit insert mode (return to normal mode)
    pub fn exit_insert_mode(&mut self) {
        self.mode = MagnifierMode::Normal;
        self.clamp_cursor();
    }

    /// Set count prefix for next command
    pub fn set_count_prefix(&mut self, count: usize) {
        self.count_prefix = Some(count);
    }

    /// Get and clear count prefix (returns 1 if no prefix set)
    pub fn take_count(&mut self) -> usize {
        self.count_prefix.take().unwrap_or(1)
    }

    /// Get current line text
    fn current_line(&self) -> &str {
        self.lines
            .get(self.cursor.0)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get mutable reference to current line
    fn current_line_mut(&mut self) -> &mut String {
        let line_idx = self.cursor.0;
        // Ensure line exists
        if line_idx >= self.lines.len() {
            self.lines.resize(line_idx + 1, String::new());
        }
        &mut self.lines[line_idx]
    }

    /// Clamp cursor to valid position within buffer
    ///
    /// In Normal mode: cursor column must be < line.len() (can't be past last char)
    /// In Insert mode: cursor column can be <= line.len() (can be at end)
    fn clamp_cursor(&mut self) {
        // Ensure we have at least one line
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        // Clamp line to valid range
        let max_line = self.lines.len().saturating_sub(1);
        self.cursor.0 = self.cursor.0.min(max_line);

        // Clamp column based on mode
        let line_len = self.current_line().len();
        let max_col = if self.mode == MagnifierMode::Insert {
            line_len // Can be at end in insert mode
        } else {
            line_len.saturating_sub(1) // Must be on a character in normal mode
        };

        self.cursor.1 = self.cursor.1.min(max_col);

        // Special case: empty line in normal mode, cursor at 0
        if line_len == 0 {
            self.cursor.1 = 0;
        }
    }

    // ============================================================================
    // Vim Motions (Phase 2)
    // ============================================================================

    /// Move cursor left (h)
    pub fn move_left(&mut self) {
        let count = self.take_count();
        self.cursor.1 = self.cursor.1.saturating_sub(count);
        self.clamp_cursor();
    }

    /// Move cursor right (l)
    pub fn move_right(&mut self) {
        let count = self.take_count();
        self.cursor.1 = self.cursor.1.saturating_add(count);
        self.clamp_cursor();
    }

    /// Move cursor up (k)
    pub fn move_up(&mut self) {
        let count = self.take_count();
        self.cursor.0 = self.cursor.0.saturating_sub(count);
        self.clamp_cursor();
    }

    /// Move cursor down (j)
    pub fn move_down(&mut self) {
        let count = self.take_count();
        self.cursor.0 = self.cursor.0.saturating_add(count);
        self.clamp_cursor();
    }

    /// Move to start of line (0)
    pub fn move_to_line_start(&mut self) {
        self.cursor.1 = 0;
    }

    /// Move to end of line ($)
    pub fn move_to_line_end(&mut self) {
        let line_len = self.current_line().len();
        self.cursor.1 = if self.mode == MagnifierMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        self.clamp_cursor();
    }

    /// Move to first non-blank character (^)
    pub fn move_to_first_non_blank(&mut self) {
        let line = self.current_line();
        let first_non_blank = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
        self.cursor.1 = first_non_blank;
        self.clamp_cursor();
    }

    /// Move to first line (gg)
    pub fn move_to_first_line(&mut self) {
        self.cursor.0 = 0;
        self.clamp_cursor();
    }

    /// Move to last line (G)
    pub fn move_to_last_line(&mut self) {
        self.cursor.0 = self.lines.len().saturating_sub(1);
        self.clamp_cursor();
    }

    /// Move to specific line number (1-indexed for user, converted to 0-indexed)
    pub fn move_to_line(&mut self, line_number: usize) {
        // Convert 1-indexed to 0-indexed
        self.cursor.0 = line_number.saturating_sub(1);
        self.clamp_cursor();
    }

    /// Move to next word (w)
    pub fn move_next_word(&mut self) {
        let count = self.take_count();
        for _ in 0..count {
            self.move_next_word_once();
        }
    }

    /// Move to previous word (b)
    pub fn move_prev_word(&mut self) {
        let count = self.take_count();
        for _ in 0..count {
            self.move_prev_word_once();
        }
    }

    /// Move to end of word (e)
    pub fn move_end_word(&mut self) {
        let count = self.take_count();
        for _ in 0..count {
            self.move_end_word_once();
        }
    }

    /// Helper: Move to next word once
    fn move_next_word_once(&mut self) {
        let line = self.current_line().to_string();
        let mut col = self.cursor.1;

        if col >= line.len() {
            // At end of line, move to next line
            if self.cursor.0 < self.lines.len() - 1 {
                self.cursor.0 += 1;
                self.cursor.1 = 0;
                let new_line = self.current_line().to_string();
                // Skip leading whitespace
                while self.cursor.1 < new_line.len()
                    && Self::is_whitespace_at(&new_line, self.cursor.1)
                {
                    self.cursor.1 += 1;
                }
            }
            self.clamp_cursor();
            return;
        }

        // Skip current word (non-whitespace)
        while col < line.len() && !Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // Skip whitespace to next word
        while col < line.len() && Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // If we reached end of line, move to next line
        if col >= line.len() && self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
            let new_line = self.current_line().to_string();
            // Skip leading whitespace on new line
            while self.cursor.1 < new_line.len() && Self::is_whitespace_at(&new_line, self.cursor.1)
            {
                self.cursor.1 += 1;
            }
        } else {
            self.cursor.1 = col;
        }

        self.clamp_cursor();
    }

    /// Helper: Move to previous word once
    fn move_prev_word_once(&mut self) {
        let mut col = self.cursor.1;

        // If at start of line, move to end of previous line
        if col == 0 {
            if self.cursor.0 > 0 {
                self.cursor.0 -= 1;
                let line = self.current_line().to_string();
                self.cursor.1 = line.len().saturating_sub(1);
            }
            self.clamp_cursor();
            return;
        }

        let line = self.current_line().to_string();

        // Move back one position
        col = col.saturating_sub(1);

        // Skip whitespace backwards
        while col > 0 && Self::is_whitespace_at(&line, col) {
            col -= 1;
        }

        // Skip word backwards to find start
        while col > 0 && !Self::is_whitespace_at(&line, col.saturating_sub(1)) {
            col -= 1;
        }

        self.cursor.1 = col;
        self.clamp_cursor();
    }

    /// Helper: Move to end of word once
    fn move_end_word_once(&mut self) {
        let line = self.current_line().to_string();
        let mut col = self.cursor.1;

        // Move forward at least one character
        if col < line.len() {
            col += 1;
        }

        // Skip whitespace
        while col < line.len() && Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // Move to end of word (find next whitespace or end)
        while col < line.len() && !Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // Position on last character of word (one before whitespace)
        if col > 0 && col <= line.len() {
            col -= 1;
        }

        self.cursor.1 = col;
        self.clamp_cursor();
    }


    /// Check if character at position is whitespace
    fn is_whitespace_at(line: &str, pos: usize) -> bool {
        line.chars()
            .nth(pos)
            .map(|c| c.is_whitespace())
            .unwrap_or(false)
    }

    // ============================================================================
    // Vim Operators (Phase 3)
    // ============================================================================

    // --- Insert Mode Text Input ---

    /// Insert character at cursor position (in Insert mode)
    pub fn insert_char(&mut self, c: char) {
        let col = self.cursor.1;
        let line = self.current_line_mut();
        let col = col.min(line.len());
        line.insert(col, c);
        self.cursor.1 = col + 1;
    }

    /// Delete character before cursor (Backspace in Insert mode)
    pub fn delete_char_before(&mut self) {
        if self.cursor.1 > 0 {
            let col = self.cursor.1 - 1;
            let line = self.current_line_mut();
            if col < line.len() {
                line.remove(col);
            }
            self.cursor.1 = col;
        } else if self.cursor.0 > 0 {
            // At start of line - join with previous line
            let current_line = self.lines.remove(self.cursor.0);
            self.cursor.0 -= 1;
            let prev_line_len = self.lines[self.cursor.0].len();
            self.lines[self.cursor.0].push_str(&current_line);
            self.cursor.1 = prev_line_len;
        }
    }

    /// Delete character at cursor (Delete key in Insert mode)
    pub fn delete_char_at(&mut self) {
        let col = self.cursor.1;
        let line_idx = self.cursor.0;

        if line_idx < self.lines.len() {
            let line = &mut self.lines[line_idx];
            if col < line.len() {
                line.remove(col);
                return;
            }
        }

        // At end of line - join with next line
        if line_idx < self.lines.len() - 1 {
            let next_line = self.lines.remove(line_idx + 1);
            self.lines[line_idx].push_str(&next_line);
        }
    }

    /// Insert newline at cursor (Enter in Insert mode)
    pub fn newline(&mut self) {
        let col = self.cursor.1;
        let line_idx = self.cursor.0;

        let rest = self.lines[line_idx].split_off(col);
        self.cursor.0 = line_idx + 1;
        self.lines.insert(self.cursor.0, rest);
        self.cursor.1 = 0;
    }

    // --- Normal Mode Operators ---

    /// Delete character under cursor (x in Normal mode)
    pub fn delete_char(&mut self) {
        let col = self.cursor.1;
        let line = self.current_line_mut();
        if col < line.len() {
            line.remove(col);
        }
        self.clamp_cursor();
    }

    /// Delete current line (dd in Normal mode)
    pub fn delete_line(&mut self) {
        if self.lines.len() == 1 {
            // Last line - just clear it
            self.clipboard = vec![self.lines[0].clone()];
            self.lines[0].clear();
            self.cursor.1 = 0;
        } else {
            // Remove line and store in clipboard
            let deleted = self.lines.remove(self.cursor.0);
            self.clipboard = vec![deleted];

            // Adjust cursor if we deleted the last line
            if self.cursor.0 >= self.lines.len() {
                self.cursor.0 = self.lines.len().saturating_sub(1);
            }
        }
        self.clamp_cursor();
    }

    /// Yank (copy) current line (yy in Normal mode)
    pub fn yank_line(&mut self) {
        let line = self.current_line().to_string();
        self.clipboard = vec![line];
    }

    /// Paste clipboard below current line (p in Normal mode)
    pub fn paste_below(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }

        for (i, line) in self.clipboard.iter().enumerate() {
            self.lines.insert(self.cursor.0 + 1 + i, line.clone());
        }

        // Move cursor to first pasted line
        self.cursor.0 += 1;
        self.cursor.1 = 0;
        self.clamp_cursor();
    }

    /// Paste clipboard above current line (P in Normal mode)
    pub fn paste_above(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }

        for (i, line) in self.clipboard.iter().enumerate() {
            self.lines.insert(self.cursor.0 + i, line.clone());
        }

        // Cursor stays on same line (which is now pushed down)
        self.cursor.1 = 0;
        self.clamp_cursor();
    }

    /// Substitute character (s in Normal mode) - delete char and enter insert
    pub fn substitute_char(&mut self) {
        self.delete_char();
        self.enter_insert_mode();
    }

    // --- Enter Insert Mode Variations ---

    /// Enter insert mode before cursor (i)
    pub fn insert_before(&mut self) {
        self.enter_insert_mode();
        // Cursor stays at current position
    }

    /// Enter insert mode after cursor (a)
    pub fn insert_after(&mut self) {
        self.enter_insert_mode();
        // Move cursor one position right
        if self.cursor.1 < self.current_line().len() {
            self.cursor.1 += 1;
        }
    }

    /// Insert new line below and enter insert mode (o)
    pub fn insert_line_below(&mut self) {
        self.cursor.0 += 1;
        self.lines.insert(self.cursor.0, String::new());
        self.cursor.1 = 0;
        self.enter_insert_mode();
    }

    /// Insert new line above and enter insert mode (O)
    pub fn insert_line_above(&mut self) {
        self.lines.insert(self.cursor.0, String::new());
        self.cursor.1 = 0;
        self.enter_insert_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_single_line() {
        let state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(5), ColIndex::new(2)),
        );

        assert_eq!(state.line_count(), 1);
        assert_eq!(state.get_line(0), Some("Hello World"));
        assert_eq!(state.mode(), MagnifierMode::Normal);
        assert_eq!(state.cursor(), (0, 0));
        assert_eq!(state.cell_position(), (RowIndex::new(5), ColIndex::new(2)));
        assert!(!state.is_dirty());
    }

    #[test]
    fn test_new_multiline() {
        let state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );

        assert_eq!(state.line_count(), 3);
        assert_eq!(state.get_line(0), Some("Line 1"));
        assert_eq!(state.get_line(1), Some("Line 2"));
        assert_eq!(state.get_line(2), Some("Line 3"));
    }

    #[test]
    fn test_new_empty() {
        let state = MagnifierState::new(String::new(), (RowIndex::new(0), ColIndex::new(0)));

        assert_eq!(state.line_count(), 1);
        assert_eq!(state.get_line(0), Some(""));
    }

    #[test]
    fn test_get_content_single_line() {
        let state = MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

        assert_eq!(state.get_content(), "Hello");
    }

    #[test]
    fn test_get_content_multiline() {
        let state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );

        assert_eq!(state.get_content(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_is_dirty_unchanged() {
        let state = MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

        assert!(!state.is_dirty());
    }

    #[test]
    fn test_mode_switching() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

        assert_eq!(state.mode(), MagnifierMode::Normal);

        state.enter_insert_mode();
        assert_eq!(state.mode(), MagnifierMode::Insert);

        state.exit_insert_mode();
        assert_eq!(state.mode(), MagnifierMode::Normal);
    }

    #[test]
    fn test_count_prefix() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

        // Default count is 1
        assert_eq!(state.take_count(), 1);

        // Set and take count
        state.set_count_prefix(5);
        assert_eq!(state.take_count(), 5);

        // Count is cleared after take
        assert_eq!(state.take_count(), 1);
    }

    #[test]
    fn test_clamp_cursor_normal_mode() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

        // Try to move cursor past end of line
        state.cursor = (0, 10);
        state.clamp_cursor();

        // In normal mode, max column is len-1 (must be on a character)
        assert_eq!(state.cursor.1, 4); // "Hello" has 5 chars, max col is 4
    }

    #[test]
    fn test_clamp_cursor_insert_mode() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));

        state.enter_insert_mode();
        state.cursor = (0, 10);
        state.clamp_cursor();

        // In insert mode, cursor can be at end (len)
        assert_eq!(state.cursor.1, 5); // "Hello" has 5 chars, can be at position 5
    }

    #[test]
    fn test_clamp_cursor_empty_line() {
        let mut state = MagnifierState::new(String::new(), (RowIndex::new(0), ColIndex::new(0)));

        state.cursor = (0, 5);
        state.clamp_cursor();

        // Empty line, cursor should be at 0
        assert_eq!(state.cursor, (0, 0));
    }

    #[test]
    fn test_clamp_cursor_line_bounds() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );

        // Try to move to non-existent line
        state.cursor = (10, 0);
        state.clamp_cursor();

        // Should clamp to last line
        assert_eq!(state.cursor.0, 1);
    }

    // ============================================================================
    // Phase 2: Vim Motions Tests
    // ============================================================================

    #[test]
    fn test_move_left() {
        let mut state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 5);

        state.move_left();
        assert_eq!(state.cursor.1, 4);
    }

    #[test]
    fn test_move_left_with_count() {
        let mut state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 5);
        state.set_count_prefix(3);

        state.move_left();
        assert_eq!(state.cursor.1, 2);
    }

    #[test]
    fn test_move_left_at_start() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 0);

        state.move_left();
        assert_eq!(state.cursor.1, 0); // Should stay at 0
    }

    #[test]
    fn test_move_right() {
        let mut state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_right();
        assert_eq!(state.cursor.1, 1);
    }

    #[test]
    fn test_move_right_with_count() {
        let mut state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);
        state.set_count_prefix(5);

        state.move_right();
        assert_eq!(state.cursor.1, 5);
    }

    #[test]
    fn test_move_right_at_end() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 4); // Last char

        state.move_right();
        assert_eq!(state.cursor.1, 4); // Should stay at last char in normal mode
    }

    #[test]
    fn test_move_up() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (2, 0);

        state.move_up();
        assert_eq!(state.cursor.0, 1);
    }

    #[test]
    fn test_move_up_with_count() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (2, 0);
        state.set_count_prefix(2);

        state.move_up();
        assert_eq!(state.cursor.0, 0);
    }

    #[test]
    fn test_move_up_at_first_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_up();
        assert_eq!(state.cursor.0, 0); // Should stay at 0
    }

    #[test]
    fn test_move_down() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_down();
        assert_eq!(state.cursor.0, 1);
    }

    #[test]
    fn test_move_down_with_count() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);
        state.set_count_prefix(2);

        state.move_down();
        assert_eq!(state.cursor.0, 2);
    }

    #[test]
    fn test_move_down_at_last_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (1, 0);

        state.move_down();
        assert_eq!(state.cursor.0, 1); // Should stay at last line
    }

    #[test]
    fn test_move_to_line_start() {
        let mut state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 5);

        state.move_to_line_start();
        assert_eq!(state.cursor.1, 0);
    }

    #[test]
    fn test_move_to_line_end() {
        let mut state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_to_line_end();
        assert_eq!(state.cursor.1, 10); // "Hello World" is 11 chars, last index is 10
    }

    #[test]
    fn test_move_to_line_end_insert_mode() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.enter_insert_mode();
        state.cursor = (0, 0);

        state.move_to_line_end();
        assert_eq!(state.cursor.1, 5); // Can be at position 5 in insert mode
    }

    #[test]
    fn test_move_to_first_non_blank() {
        let mut state =
            MagnifierState::new("   Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 0);

        state.move_to_first_non_blank();
        assert_eq!(state.cursor.1, 3); // First 'H' is at position 3
    }

    #[test]
    fn test_move_to_first_non_blank_no_whitespace() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 3);

        state.move_to_first_non_blank();
        assert_eq!(state.cursor.1, 0);
    }

    #[test]
    fn test_move_to_first_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (2, 0);

        state.move_to_first_line();
        assert_eq!(state.cursor.0, 0);
    }

    #[test]
    fn test_move_to_last_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_to_last_line();
        assert_eq!(state.cursor.0, 2);
    }

    #[test]
    fn test_move_to_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3\nLine 4".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_to_line(3); // 1-indexed, so line 3 = index 2
        assert_eq!(state.cursor.0, 2);
    }

    #[test]
    fn test_move_next_word() {
        let mut state = MagnifierState::new(
            "Hello World Test".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_next_word();
        assert_eq!(state.cursor.1, 6); // Start of "World"
    }

    #[test]
    fn test_move_next_word_with_count() {
        let mut state = MagnifierState::new(
            "Hello World Test".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);
        state.set_count_prefix(2);

        state.move_next_word();
        assert_eq!(state.cursor.1, 12); // Start of "Test"
    }

    #[test]
    fn test_move_next_word_across_lines() {
        let mut state = MagnifierState::new(
            "Hello\nWorld".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_next_word();
        assert_eq!(state.cursor, (1, 0)); // Should move to next line
    }

    #[test]
    fn test_move_prev_word() {
        let mut state = MagnifierState::new(
            "Hello World Test".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 12); // At "Test"

        state.move_prev_word();
        assert_eq!(state.cursor.1, 6); // Start of "World"
    }

    #[test]
    fn test_move_prev_word_with_count() {
        let mut state = MagnifierState::new(
            "Hello World Test".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 12);
        state.set_count_prefix(2);

        state.move_prev_word();
        assert_eq!(state.cursor.1, 0); // Start of "Hello"
    }

    #[test]
    fn test_move_prev_word_at_line_start() {
        let mut state = MagnifierState::new(
            "Hello\nWorld".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (1, 0);

        state.move_prev_word();
        assert_eq!(state.cursor.0, 0); // Should move to previous line
    }

    #[test]
    fn test_move_end_word() {
        let mut state = MagnifierState::new(
            "Hello World".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.move_end_word();
        assert_eq!(state.cursor.1, 4); // End of "Hello"
    }

    #[test]
    fn test_move_end_word_with_count() {
        let mut state = MagnifierState::new(
            "Hello World Test".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);
        state.set_count_prefix(2);

        state.move_end_word();
        assert_eq!(state.cursor.1, 10); // End of "World"
    }

    // ============================================================================
    // Phase 3: Vim Operators Tests
    // ============================================================================

    #[test]
    fn test_insert_char() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.enter_insert_mode();
        state.cursor = (0, 2); // Between 'l' and 'l'

        state.insert_char('X');
        assert_eq!(state.get_line(0), Some("HeXllo"));
        assert_eq!(state.cursor.1, 3);
    }

    #[test]
    fn test_insert_char_at_end() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.enter_insert_mode();
        state.cursor = (0, 5);

        state.insert_char('!');
        assert_eq!(state.get_line(0), Some("Hello!"));
        assert_eq!(state.cursor.1, 6);
    }

    #[test]
    fn test_delete_char_before() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.enter_insert_mode();
        state.cursor = (0, 3);

        state.delete_char_before();
        assert_eq!(state.get_line(0), Some("Helo"));
        assert_eq!(state.cursor.1, 2);
    }

    #[test]
    fn test_delete_char_before_at_line_start() {
        let mut state = MagnifierState::new(
            "Hello\nWorld".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.enter_insert_mode();
        state.cursor = (1, 0);

        state.delete_char_before();
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.get_line(0), Some("HelloWorld"));
        assert_eq!(state.cursor, (0, 5));
    }

    #[test]
    fn test_delete_char_at() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.enter_insert_mode();
        state.cursor = (0, 2);

        state.delete_char_at();
        assert_eq!(state.get_line(0), Some("Helo"));
        assert_eq!(state.cursor.1, 2);
    }

    #[test]
    fn test_delete_char_at_end_of_line() {
        let mut state = MagnifierState::new(
            "Hello\nWorld".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.enter_insert_mode();
        state.cursor = (0, 5);

        state.delete_char_at();
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.get_line(0), Some("HelloWorld"));
    }

    #[test]
    fn test_newline() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.enter_insert_mode();
        state.cursor = (0, 2);

        state.newline();
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.get_line(0), Some("He"));
        assert_eq!(state.get_line(1), Some("llo"));
        assert_eq!(state.cursor, (1, 0));
    }

    #[test]
    fn test_delete_char_normal_mode() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 2);

        state.delete_char();
        assert_eq!(state.get_line(0), Some("Helo"));
        assert_eq!(state.cursor.1, 2);
    }

    #[test]
    fn test_delete_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2\nLine 3".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (1, 0);

        state.delete_line();
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.get_line(0), Some("Line 1"));
        assert_eq!(state.get_line(1), Some("Line 3"));
        assert_eq!(state.clipboard, vec!["Line 2".to_string()]);
    }

    #[test]
    fn test_delete_line_last_line() {
        let mut state = MagnifierState::new(
            "Only Line".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );

        state.delete_line();
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.get_line(0), Some(""));
        assert_eq!(state.clipboard, vec!["Only Line".to_string()]);
    }

    #[test]
    fn test_yank_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (1, 0);

        state.yank_line();
        assert_eq!(state.clipboard, vec!["Line 2".to_string()]);
        // Original should be unchanged
        assert_eq!(state.line_count(), 2);
    }

    #[test]
    fn test_paste_below() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.clipboard = vec!["Pasted".to_string()];
        state.cursor = (0, 0);

        state.paste_below();
        assert_eq!(state.line_count(), 3);
        assert_eq!(state.get_line(0), Some("Line 1"));
        assert_eq!(state.get_line(1), Some("Pasted"));
        assert_eq!(state.get_line(2), Some("Line 2"));
        assert_eq!(state.cursor.0, 1);
    }

    #[test]
    fn test_paste_above() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.clipboard = vec!["Pasted".to_string()];
        state.cursor = (1, 0);

        state.paste_above();
        assert_eq!(state.line_count(), 3);
        assert_eq!(state.get_line(0), Some("Line 1"));
        assert_eq!(state.get_line(1), Some("Pasted"));
        assert_eq!(state.get_line(2), Some("Line 2"));
        assert_eq!(state.cursor.0, 1);
    }

    #[test]
    fn test_paste_multiple_lines() {
        let mut state =
            MagnifierState::new("Line 1".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.clipboard = vec!["Paste 1".to_string(), "Paste 2".to_string()];

        state.paste_below();
        assert_eq!(state.line_count(), 3);
        assert_eq!(state.get_line(0), Some("Line 1"));
        assert_eq!(state.get_line(1), Some("Paste 1"));
        assert_eq!(state.get_line(2), Some("Paste 2"));
    }

    #[test]
    fn test_substitute_char() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 2);

        state.substitute_char();
        assert_eq!(state.get_line(0), Some("Helo"));
        assert_eq!(state.mode(), MagnifierMode::Insert);
    }

    #[test]
    fn test_insert_before() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 2);

        state.insert_before();
        assert_eq!(state.mode(), MagnifierMode::Insert);
        assert_eq!(state.cursor.1, 2);
    }

    #[test]
    fn test_insert_after() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        state.cursor = (0, 2);

        state.insert_after();
        assert_eq!(state.mode(), MagnifierMode::Insert);
        assert_eq!(state.cursor.1, 3);
    }

    #[test]
    fn test_insert_line_below() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (0, 0);

        state.insert_line_below();
        assert_eq!(state.line_count(), 3);
        assert_eq!(state.get_line(0), Some("Line 1"));
        assert_eq!(state.get_line(1), Some(""));
        assert_eq!(state.get_line(2), Some("Line 2"));
        assert_eq!(state.cursor, (1, 0));
        assert_eq!(state.mode(), MagnifierMode::Insert);
    }

    #[test]
    fn test_insert_line_above() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        state.cursor = (1, 0);

        state.insert_line_above();
        assert_eq!(state.line_count(), 3);
        assert_eq!(state.get_line(0), Some("Line 1"));
        assert_eq!(state.get_line(1), Some(""));
        assert_eq!(state.get_line(2), Some("Line 2"));
        assert_eq!(state.cursor, (1, 0));
        assert_eq!(state.mode(), MagnifierMode::Insert);
    }

    #[test]
    fn test_is_dirty_after_edit() {
        let mut state =
            MagnifierState::new("Hello".to_string(), (RowIndex::new(0), ColIndex::new(0)));
        assert!(!state.is_dirty());

        state.enter_insert_mode();
        state.insert_char('X');
        assert!(state.is_dirty());
    }

    #[test]
    fn test_is_dirty_after_delete_line() {
        let mut state = MagnifierState::new(
            "Line 1\nLine 2".to_string(),
            (RowIndex::new(0), ColIndex::new(0)),
        );
        assert!(!state.is_dirty());

        state.delete_line();
        assert!(state.is_dirty());
    }
}
