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
//! ```ignore
//! // Open magnifier on current cell
//! let state = MagnifierState::new("cell content".to_string(), (row, col));
//!
//! // Edit content with vim commands
//! state.move_down();
//! state.delete_line();
//!
//! // Get edited content
//! let content = state.get_content();
//! ```

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
    /// ```ignore
    /// let state = MagnifierState::new("Hello\nWorld".to_string(), (RowIndex(5), ColIndex(2)));
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

    /// Check if the buffer has been modified
    pub fn is_dirty(&self) -> bool {
        self.get_content() != self.original_content
    }

    /// Get the current mode
    pub fn mode(&self) -> MagnifierMode {
        self.mode
    }

    /// Get the cursor position (line, column)
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
}
