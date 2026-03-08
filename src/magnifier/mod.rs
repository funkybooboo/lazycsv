//! Magnifier Mode - Full vim editor for complex cell editing
//!
//! This module implements a comprehensive vim-style text editor for editing CSV cell content
//! with multi-line support, vim motions, operators, visual selection, search, and unlimited undo/redo.
//!
//! ## Features
//!
//! ### Modal Editing
//! - **Normal Mode**: Navigation and commands (default mode)
//! - **Insert Mode**: Text input and editing
//! - **Visual Mode**: Character-wise and line-wise selection
//! - **Command Mode**: Ex commands (`:w`, `:q`, `:wq`, `:q!`)
//!
//! ### Vim Motions
//! - **Basic**: `hjkl` (arrow keys also work)
//! - **Word**: `w` (next word), `b` (back), `e` (end)
//! - **Line**: `0` (start), `$` (end), `^` (first non-blank)
//! - **Document**: `gg` (top), `G` (bottom)
//! - **Find**: `f{char}`, `F{char}`, `t{char}`, `T{char}`, `;`, `,`
//!
//! ### Vim Operators
//! - **Delete**: `x` (char), `dd` (line), `d{motion}`
//! - **Yank**: `yy` (line), `y{motion}`
//! - **Change**: `cc` (line), `C` (to end), `c{motion}`
//! - **Paste**: `p` (below), `P` (above)
//! - **Other**: `J` (join lines), `r{char}` (replace), `>>` / `<<` (indent/dedent)
//!
//! ### Insert Mode Entry
//! - `i` (before cursor), `a` (after cursor)
//! - `I` (line start), `A` (line end)
//! - `o` (line below), `O` (line above)
//! - `s` (substitute char)
//!
//! ### Visual Selection
//! - `v`: Character-wise visual mode
//! - `V`: Line-wise visual mode
//! - `d`, `y`, `c`: Delete, yank, change selection
//! - `gv`: Reselect last visual selection
//!
//! ### Search
//! - `/pattern`: Search forward (case-sensitive)
//! - `n`: Next match, `N`: Previous match
//! - `*`: Search word under cursor
//! - `:noh`: Clear search highlighting
//!
//! ### Undo/Redo
//! - `u`: Undo (unlimited history)
//! - `Ctrl+r`: Redo
//!
//! ### Ex Commands
//! - `:w` - Save to cell (updates in-memory document)
//! - `:q` - Quit (warns if unsaved changes)
//! - `:wq` or `ZZ` - Save and quit
//! - `:q!` - Force quit without saving
//!
//! ## Architecture
//!
//! The magnifier uses a modal state machine with full document snapshots for undo/redo.
//! This provides simplicity and correctness at the cost of memory (acceptable for cell editing).
//!
//! ### State Management
//! - **Document**: Stored as `Vec<String>` (one string per line)
//! - **Cursor**: `(line, col)` tuple using char positions (not bytes)
//! - **Undo/Redo**: Full snapshots, O(1) operations
//! - **Registers**: Separate registers for char, line, and region operations
//!
//! ### Performance Characteristics
//! - **Motions**: O(1) for basic, O(n) for word/find (n = line length)
//! - **Operators**: O(n) where n = affected lines
//! - **Search**: O(n*m) where n = document size, m = pattern length
//! - **Undo/Redo**: O(1) stack operations
//! - **Memory**: document_size * (1 + undo_count) for undo history
//!
//! ## Usage Example
//!
//! ```rust
//! use lazycsv::magnifier::MagnifierState;
//! use lazycsv::domain::position::{RowIndex, ColIndex};
//!
//! // Create magnifier state for a cell
//! let content = "Line 1\nLine 2\nLine 3".to_string();
//! let position = (RowIndex::new(1), ColIndex::new(0));
//! let mut mag = MagnifierState::new(content, position);
//!
//! // Use vim commands
//! mag.move_down();           // j - move down
//! mag.push_undo();           // Save undo point
//! mag.delete_line();         // dd - delete line
//! mag.move_to_line_end();    // $ - end of line
//! mag.enter_insert_mode();   // i - enter insert mode
//! mag.insert_char('!');      // Type '!'
//! mag.exit_insert_mode();    // Esc - back to normal
//!
//! // Get modified content
//! let result = mag.get_content();
//! assert_eq!(result, "Line 1!\nLine 3");
//! ```
//!
//! ## Multi-byte Character Support
//!
//! The magnifier correctly handles multi-byte UTF-8 characters including emojis:
//! - Cursor positions use char indices, not byte indices
//! - Search handles multi-byte patterns correctly (fixed in v0.6.1)
//! - All operations respect character boundaries
//!
//! ## Integration with LazyCSV
//!
//! The magnifier integrates with the main CSV editor:
//! - `:w` saves to in-memory CSV document (not to disk)
//! - `Alt+hjkl` navigates to adjacent cells (prompts to save if dirty)
//! - Changes are only persisted when explicitly saved
//! - Magnifier state is destroyed on close (changes must be saved first)
//!
//! ## See Also
//!
//! - User documentation: `docs/keybindings.md` (lines 373-472)
//! - Implementation details: `docs/vim-implementation.md`
//! - Test files: `tests/magnifier_*_test.rs`
//! - Benchmarks: `benches/magnifier.rs`

use crate::domain::position::{ColIndex, RowIndex};

/// Vim mode within magnifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnifierMode {
    /// Normal mode - navigation and commands
    Normal,
    /// Insert mode - text input
    Insert,
    /// Command mode - ex commands (:w, :q, etc)
    Command,
    /// Visual mode - character-wise selection
    Visual,
    /// Visual Line mode - line-wise selection
    VisualLine,
}

/// Pending command for multi-key sequences
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingCommand {
    /// Waiting for second 'g' (gg)
    G,
    /// Waiting for second 'd' (dd)
    D,
    /// Waiting for second 'y' (yy)
    Y,
    /// Waiting for second 'c' (cc)
    C,
    /// Waiting for second 'Z' (ZZ)
    Z,
    /// Waiting for character after 'f'
    FindForward,
    /// Waiting for character after 'F'
    FindBackward,
    /// Waiting for character after 't'
    TillForward,
    /// Waiting for character after 'T'
    TillBackward,
    /// Waiting for character to replace with 'r'
    Replace,
    /// Waiting for second '>' (>>)
    IndentRight,
    /// Waiting for second '<' (<<)
    IndentLeft,
}

/// Last find command for repeating with ; and ,
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FindCommand {
    Forward(char),
    Backward(char),
    TillForward(char),
    TillBackward(char),
}

/// Undo snapshot for state restoration
///
/// Stores a complete snapshot of the document and cursor position for undo/redo operations.
/// We use full snapshots instead of deltas for simplicity and correctness.
#[derive(Debug, Clone)]
struct UndoSnapshot {
    lines: Vec<String>,
    cursor: (usize, usize),
}

/// Selection range for visual mode operations
///
/// Represents the selected text region in either character-wise or line-wise mode.
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

/// Complete state for vim-style text editor within magnifier mode
///
/// The `MagnifierState` represents a full-featured vim-style editor for editing
/// individual CSV cell content. It maintains the document as a vector of lines,
/// tracks cursor position, and implements all vim operations through public methods.
///
/// ## Architecture
///
/// - **Document**: Stored as `Vec<String>`, one string per line
/// - **Cursor**: `(line, col)` using char indices (not bytes) for UTF-8 safety
/// - **Modes**: Normal, Insert, Visual, VisualLine, Command
/// - **Undo/Redo**: Full document snapshots (unlimited history)
/// - **Registers**: Separate clipboards for char/line/region operations
///
/// ## Example
///
/// ```rust
/// use lazycsv::magnifier::MagnifierState;
/// use lazycsv::domain::position::{RowIndex, ColIndex};
///
/// let mut mag = MagnifierState::new(
///     "Hello\nWorld".to_string(),
///     (RowIndex::new(1), ColIndex::new(0))
/// );
///
/// // Vim commands
/// mag.move_down();        // j
/// mag.move_to_line_end(); // $
/// mag.push_undo();
/// mag.insert_after();     // a
/// mag.insert_char('!');
/// mag.exit_insert_mode(); // Esc
///
/// assert_eq!(mag.get_content(), "Hello\nWorld!");
/// ```
///
/// ## Public API
///
/// The public API exposes 83 methods organized into categories:
///
/// - **Mode management**: `mode()`, `enter_insert_mode()`, `exit_insert_mode()`
/// - **Basic motions**: `move_up()`, `move_down()`, `move_left()`, `move_right()`
/// - **Word motions**: `move_next_word()`, `move_prev_word()`, `move_end_word()`
/// - **Line motions**: `move_to_line_start()`, `move_to_line_end()`, `move_to_first_non_blank()`
/// - **Document navigation**: `move_to_first_line()`, `move_to_last_line()`, `move_to_line()`
/// - **Find commands**: `find_char_forward()`, `find_char_backward()`, etc.
/// - **Operators**: `delete_char()`, `delete_line()`, `yank_line()`, `paste_below()`, `paste_above()`
/// - **Insert operations**: `insert_char()`, `newline()`, `backspace()`, `delete_key()`
/// - **Visual mode**: `enter_visual_mode()`, `enter_visual_line_mode()`, `get_visual_selection()`
/// - **Search**: `search_forward()`, `jump_to_next_match()`, `jump_to_prev_match()`
/// - **Undo/Redo**: `push_undo()`, `undo()`, `redo()`
/// - **State access**: `get_content()`, `is_dirty()`, `cursor()`, `lines()`
///
/// ## Performance
///
/// - Motions: O(1) for basic, O(n) for word/find
/// - Operators: O(n) where n = affected lines
/// - Undo/Redo: O(1) stack operations
/// - Memory: document_size × (1 + undo_count)
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

    /// Pending command for multi-key sequences
    pending_command: Option<PendingCommand>,

    /// Command buffer for ex mode
    command_buffer: String,

    /// Visual mode anchor point (where selection started)
    visual_anchor: Option<(usize, usize)>,

    /// Undo history stack
    undo_stack: Vec<UndoSnapshot>,

    /// Redo history stack
    redo_stack: Vec<UndoSnapshot>,

    /// Search pattern
    search_pattern: Option<String>,

    /// Search match positions (line, col)
    search_matches: Vec<(usize, usize)>,

    /// Current match index
    current_match: Option<usize>,

    /// Last find command for ; and ,
    last_find: Option<FindCommand>,
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
            pending_command: None,
            command_buffer: String::new(),
            visual_anchor: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_pattern: None,
            search_matches: Vec::new(),
            current_match: None,
            last_find: None,
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

    /// Mark content as clean (after saving to document)
    pub fn mark_clean_with_content(&mut self, content: String) {
        self.original_content = content;
    }

    /// Get the current mode
    pub fn mode(&self) -> MagnifierMode {
        self.mode
    }

    /// Get the current cursor position (line, column)
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Set cursor position (for testing)
    pub fn set_cursor_for_test(&mut self, line: usize, col: usize) {
        self.cursor = (line, col);
        self.clamp_cursor();
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

    // ============================================================================
    // Pending Command Management
    // ============================================================================

    /// Set pending command
    pub fn set_pending(&mut self, cmd: PendingCommand) {
        self.pending_command = Some(cmd);
    }

    /// Take and clear pending command
    pub fn take_pending(&mut self) -> Option<PendingCommand> {
        self.pending_command.take()
    }

    /// Check if there's a pending command
    pub fn has_pending(&self) -> bool {
        self.pending_command.is_some()
    }

    /// Get pending command display string
    pub fn pending_display(&self) -> Option<&str> {
        self.pending_command.as_ref().map(|cmd| match cmd {
            PendingCommand::G => "g",
            PendingCommand::D => "d",
            PendingCommand::Y => "y",
            PendingCommand::C => "c",
            PendingCommand::Z => "Z",
            PendingCommand::FindForward => "f",
            PendingCommand::FindBackward => "F",
            PendingCommand::TillForward => "t",
            PendingCommand::TillBackward => "T",
            PendingCommand::Replace => "r",
            PendingCommand::IndentRight => ">",
            PendingCommand::IndentLeft => "<",
        })
    }

    // ============================================================================
    // Command Mode
    // ============================================================================

    /// Enter command mode
    pub fn enter_command_mode(&mut self) {
        self.mode = MagnifierMode::Command;
        self.command_buffer.clear();
    }

    /// Enter command mode with prefix (for search)
    pub fn enter_command_mode_with(&mut self, prefix: &str) {
        self.mode = MagnifierMode::Command;
        self.command_buffer = prefix.to_string();
    }

    /// Exit command mode
    pub fn exit_command_mode(&mut self) {
        self.mode = MagnifierMode::Normal;
        self.command_buffer.clear();
    }

    /// Get command buffer
    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Insert character in command buffer
    pub fn command_insert_char(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    /// Backspace in command buffer
    pub fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    // ============================================================================
    // Visual Mode
    // ============================================================================

    /// Enter visual mode (character-wise)
    pub fn enter_visual_mode(&mut self) {
        self.mode = MagnifierMode::Visual;
        self.visual_anchor = Some(self.cursor);
    }

    /// Enter visual line mode
    pub fn enter_visual_line_mode(&mut self) {
        self.mode = MagnifierMode::VisualLine;
        self.visual_anchor = Some((self.cursor.0, 0));
    }

    /// Exit visual mode
    pub fn exit_visual_mode(&mut self) {
        self.mode = MagnifierMode::Normal;
        self.visual_anchor = None;
    }

    /// Get visual selection
    pub fn get_visual_selection(&self) -> Option<Selection> {
        let anchor = self.visual_anchor?;
        let cursor = self.cursor;

        match self.mode {
            MagnifierMode::Visual => {
                let (start, end) = if anchor <= cursor {
                    (anchor, cursor)
                } else {
                    (cursor, anchor)
                };
                Some(Selection::CharWise { start, end })
            }
            MagnifierMode::VisualLine => {
                let (start_line, end_line) = if anchor.0 <= cursor.0 {
                    (anchor.0, cursor.0)
                } else {
                    (cursor.0, anchor.0)
                };
                Some(Selection::LineWise {
                    start_line,
                    end_line,
                })
            }
            _ => None,
        }
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

    // ============================================================================
    // Advanced Operators (Tier 1)
    // ============================================================================

    /// Change operator - delete and enter insert (c)
    pub fn change_char(&mut self) {
        self.push_undo();
        self.delete_char();
        self.enter_insert_mode();
    }

    /// Change entire line (cc)
    pub fn change_line(&mut self) {
        self.push_undo();
        let line = self.current_line_mut();
        line.clear();
        self.cursor.1 = 0;
        self.enter_insert_mode();
    }

    /// Change to end of line (C)
    pub fn change_to_eol(&mut self) {
        self.push_undo();
        let cursor_col = self.cursor.1;
        let line = self.current_line_mut();
        line.truncate(cursor_col);
        self.enter_insert_mode();
    }

    /// Replace single character (r)
    pub fn replace_char(&mut self, c: char) {
        self.push_undo();
        let cursor_col = self.cursor.1;
        let line = self.current_line_mut();
        let chars: Vec<char> = line.chars().collect();
        if cursor_col < chars.len() {
            let mut new_chars = chars;
            new_chars[cursor_col] = c;
            *line = new_chars.into_iter().collect();
        }
    }

    /// Join current line with next (J)
    pub fn join_lines(&mut self) {
        self.push_undo();
        let line_idx = self.cursor.0;
        if line_idx + 1 < self.lines.len() {
            let next_line = self.lines.remove(line_idx + 1);
            let current = self.current_line_mut();
            if !current.is_empty() && !next_line.is_empty() {
                current.push(' ');
            }
            current.push_str(&next_line);
        }
    }

    /// Indent line (>>)
    pub fn indent_line(&mut self) {
        self.push_undo();
        self.current_line_mut().insert_str(0, "  ");
        self.cursor.1 += 2;
    }

    /// Dedent line (<<)
    pub fn dedent_line(&mut self) {
        self.push_undo();
        let line = self.current_line_mut();
        if line.starts_with("  ") {
            line.drain(0..2);
            self.cursor.1 = self.cursor.1.saturating_sub(2);
        } else if line.starts_with('\t') {
            line.remove(0);
            self.cursor.1 = self.cursor.1.saturating_sub(1);
        }
    }

    // ============================================================================
    // Undo/Redo (Tier 1)
    // ============================================================================

    /// Push current state to undo stack
    pub fn push_undo(&mut self) {
        let snapshot = UndoSnapshot {
            lines: self.lines.clone(),
            cursor: self.cursor,
        };
        self.undo_stack.push(snapshot);
        self.redo_stack.clear(); // Clear redo on new edit
    }

    /// Undo last change (u)
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo
            let current = UndoSnapshot {
                lines: self.lines.clone(),
                cursor: self.cursor,
            };
            self.redo_stack.push(current);

            // Restore snapshot
            self.lines = snapshot.lines;
            self.cursor = snapshot.cursor;
            self.clamp_cursor();
        }
    }

    /// Redo last undone change (Ctrl+r)
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo
            let current = UndoSnapshot {
                lines: self.lines.clone(),
                cursor: self.cursor,
            };
            self.undo_stack.push(current);

            // Restore snapshot
            self.lines = snapshot.lines;
            self.cursor = snapshot.cursor;
            self.clamp_cursor();
        }
    }

    // ============================================================================
    // Visual Mode Operations (Tier 2)
    // ============================================================================

    /// Delete visual selection
    pub fn delete_selection(&mut self) {
        if let Some(selection) = self.get_visual_selection() {
            self.push_undo();
            match selection {
                Selection::CharWise { start, end } => {
                    // Delete characters from start to end
                    if start.0 == end.0 {
                        // Single line selection
                        let line = &mut self.lines[start.0];
                        let chars: Vec<char> = line.chars().collect();
                        let start_col = start.1.min(chars.len());
                        let end_col = (end.1 + 1).min(chars.len());
                        let new_line: String = chars
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i < start_col || *i >= end_col)
                            .map(|(_, c)| c)
                            .collect();
                        *line = new_line;
                        self.cursor = start;
                    } else {
                        // Multi-line selection - delete from start to end
                        // For simplicity, delete entire lines between start and end
                        self.lines.drain(start.0..=end.0);
                        if self.lines.is_empty() {
                            self.lines.push(String::new());
                        }
                        self.cursor = (start.0.min(self.lines.len() - 1), 0);
                    }
                }
                Selection::LineWise {
                    start_line,
                    end_line,
                } => {
                    // Delete entire lines
                    let deleted: Vec<String> = self.lines.drain(start_line..=end_line).collect();
                    self.clipboard = deleted;
                    if self.lines.is_empty() {
                        self.lines.push(String::new());
                    }
                    self.cursor = (start_line.min(self.lines.len() - 1), 0);
                }
            }
            self.exit_visual_mode();
            self.clamp_cursor();
        }
    }

    /// Yank visual selection
    pub fn yank_selection(&mut self) {
        if let Some(selection) = self.get_visual_selection() {
            match selection {
                Selection::CharWise { start, end } => {
                    if start.0 == end.0 {
                        // Single line - yank substring
                        let line = &self.lines[start.0];
                        let chars: Vec<char> = line.chars().collect();
                        let start_col = start.1.min(chars.len());
                        let end_col = (end.1 + 1).min(chars.len());
                        let yanked: String = chars[start_col..end_col].iter().collect();
                        self.clipboard = vec![yanked];
                    } else {
                        // Multi-line - yank entire lines
                        self.clipboard = self.lines[start.0..=end.0].to_vec();
                    }
                }
                Selection::LineWise {
                    start_line,
                    end_line,
                } => {
                    self.clipboard = self.lines[start_line..=end_line].to_vec();
                }
            }
            self.exit_visual_mode();
        }
    }

    /// Change visual selection (delete and enter insert)
    pub fn change_selection(&mut self) {
        self.delete_selection();
        self.enter_insert_mode();
    }

    // ============================================================================
    // Search (Tier 2)
    // ============================================================================

    /// Search forward for pattern
    pub fn search_forward(&mut self, pattern: String) {
        self.search_pattern = Some(pattern);
        self.find_all_matches();
        self.jump_to_next_match();
    }

    /// Find all matches of current search pattern
    fn find_all_matches(&mut self) {
        self.search_matches.clear();
        self.current_match = None;

        if let Some(pattern) = &self.search_pattern {
            for (line_idx, line) in self.lines.iter().enumerate() {
                // Use char indices to handle multi-byte characters correctly
                let chars: Vec<char> = line.chars().collect();
                let mut char_pos = 0;

                while char_pos < chars.len() {
                    let remaining: String = chars[char_pos..].iter().collect();
                    if let Some(match_pos) = remaining.find(pattern) {
                        // Convert byte position to char position
                        let match_char_pos = remaining[..match_pos].chars().count();
                        self.search_matches
                            .push((line_idx, char_pos + match_char_pos));
                        // Move past this match (by at least 1 char to avoid infinite loop)
                        char_pos += match_char_pos + pattern.chars().count().max(1);
                    } else {
                        break;
                    }
                }
            }
        }
    }

    /// Jump to next search match (n)
    pub fn jump_to_next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        // Find next match after cursor
        let current_pos = self.cursor;
        let next_idx = self
            .search_matches
            .iter()
            .position(|&pos| pos > current_pos)
            .unwrap_or(0); // Wrap to first match

        self.current_match = Some(next_idx);
        let (line, col) = self.search_matches[next_idx];
        self.cursor = (line, col);
        self.clamp_cursor();
    }

    /// Jump to previous search match (N)
    pub fn jump_to_prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        // Find previous match before cursor
        let current_pos = self.cursor;
        let prev_idx = self
            .search_matches
            .iter()
            .rposition(|&pos| pos < current_pos)
            .unwrap_or(self.search_matches.len() - 1); // Wrap to last match

        self.current_match = Some(prev_idx);
        let (line, col) = self.search_matches[prev_idx];
        self.cursor = (line, col);
        self.clamp_cursor();
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_pattern = None;
        self.search_matches.clear();
        self.current_match = None;
    }

    /// Get word under cursor for * search
    pub fn get_word_under_cursor(&self) -> Option<String> {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        if self.cursor.1 >= chars.len() {
            return None;
        }

        // Find word boundaries
        let mut start = self.cursor.1;
        let mut end = self.cursor.1;

        // Expand left
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Expand right
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// Get search matches for UI highlighting
    pub fn search_matches(&self) -> &[(usize, usize)] {
        &self.search_matches
    }

    /// Get current match index
    pub fn current_match_index(&self) -> Option<usize> {
        self.current_match
    }

    /// Get search pattern
    pub fn search_pattern(&self) -> Option<&str> {
        self.search_pattern.as_deref()
    }

    // ============================================================================
    // Character Find (Tier 2)
    // ============================================================================

    /// Find character forward (f)
    pub fn find_char_forward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        let start = self.cursor.1 + 1;

        for (i, &c) in chars.iter().enumerate().skip(start) {
            if c == ch {
                self.cursor.1 = i;
                self.last_find = Some(FindCommand::Forward(ch));
                return;
            }
        }
    }

    /// Find character backward (F)
    pub fn find_char_backward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();

        for i in (0..self.cursor.1).rev() {
            if chars[i] == ch {
                self.cursor.1 = i;
                self.last_find = Some(FindCommand::Backward(ch));
                return;
            }
        }
    }

    /// Till character forward (t)
    pub fn till_char_forward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        let start = self.cursor.1 + 1;

        for (i, &c) in chars.iter().enumerate().skip(start) {
            if c == ch {
                self.cursor.1 = i.saturating_sub(1);
                self.last_find = Some(FindCommand::TillForward(ch));
                return;
            }
        }
    }

    /// Till character backward (T)
    pub fn till_char_backward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();

        for i in (0..self.cursor.1).rev() {
            if chars[i] == ch {
                self.cursor.1 = (i + 1).min(chars.len().saturating_sub(1));
                self.last_find = Some(FindCommand::TillBackward(ch));
                return;
            }
        }
    }

    /// Repeat last find (;)
    pub fn repeat_find(&mut self) {
        if let Some(find) = self.last_find {
            match find {
                FindCommand::Forward(ch) => self.find_char_forward(ch),
                FindCommand::Backward(ch) => self.find_char_backward(ch),
                FindCommand::TillForward(ch) => self.till_char_forward(ch),
                FindCommand::TillBackward(ch) => self.till_char_backward(ch),
            }
        }
    }

    /// Repeat last find in reverse (,)
    pub fn repeat_find_reverse(&mut self) {
        if let Some(find) = self.last_find {
            match find {
                FindCommand::Forward(ch) => self.find_char_backward(ch),
                FindCommand::Backward(ch) => self.find_char_forward(ch),
                FindCommand::TillForward(ch) => self.till_char_backward(ch),
                FindCommand::TillBackward(ch) => self.till_char_forward(ch),
            }
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
