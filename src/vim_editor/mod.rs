//! Reusable vim-style text editor engine
//!
//! This module provides a standalone vim editor that can be embedded in different contexts
//! (e.g., Magnifier mode for cell editing, SQL editor for query editing).
//!
//! ## Features
//!
//! - Modal editing (Normal, Insert, Visual, VisualLine, Command)
//! - Full vim motions (hjkl, w/b/e, 0/$, gg/G, f/t, etc.)
//! - Vim operators (x, dd, yy, p, d{motion}, y{motion}, c{motion})
//! - Visual selection (character-wise and line-wise)
//! - Search (/, n, N, *)
//! - Unlimited undo/redo
//! - Ex commands (:w, :q, :wq, :q!, :noh)
//!
//! ## Usage
//!
//! ```rust
//! use lazycsv::vim_editor::VimEditor;
//!
//! let mut editor = VimEditor::new("Hello\nWorld".to_string());
//!
//! // Vim motion commands
//! editor.move_down();         // j - move down one line
//! editor.move_to_line_end();  // $ - move to end of line
//!
//! assert_eq!(editor.cursor(), (1, 4)); // Line 1, column 4 (last char of "World")
//! assert_eq!(editor.content(), "Hello\nWorld");
//! ```

pub mod clipboard;
pub mod commands;
pub mod modes;
pub mod motions;
pub mod operators;
pub mod search;
pub mod undo;
pub mod visual;

pub use modes::{FindCommand, PendingCommand, Selection, VimMode};

use std::collections::VecDeque;

/// Core vim editor state
///
/// This struct contains all the state needed for vim-style editing,
/// independent of any specific context (cell editing, SQL editing, etc.).
#[derive(Debug, Clone)]
pub struct VimEditor {
    /// Text buffer as vector of lines
    lines: Vec<String>,

    /// Current vim mode
    mode: VimMode,

    /// Cursor position (line, column) - 0-indexed, char positions
    /// Line: 0 to lines.len()-1
    /// Column: 0 to line.len() (can be at end for insert)
    cursor: (usize, usize),

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

    /// Undo history stack (limited to 1000 entries)
    undo_stack: VecDeque<undo::UndoSnapshot>,

    /// Redo history stack
    redo_stack: Vec<undo::UndoSnapshot>,

    /// Search pattern
    search_pattern: Option<String>,

    /// Search match positions (line, col)
    search_matches: Vec<(usize, usize)>,

    /// Current match index
    current_match: Option<usize>,

    /// Last find command for ; and ,
    last_find: Option<FindCommand>,

    /// Last executed ex command (for embedding context to handle)
    last_ex_command: Option<commands::ExCommand>,
}

impl VimEditor {
    /// Maximum undo history depth
    const MAX_UNDO_HISTORY: usize = 1000;

    /// Create a new vim editor with the given content
    ///
    /// # Arguments
    ///
    /// * `content` - Initial text content (may contain newlines)
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::vim_editor::VimEditor;
    ///
    /// let editor = VimEditor::new("Hello\nWorld".to_string());
    /// assert_eq!(editor.line_count(), 2);
    /// ```
    pub fn new(content: String) -> Self {
        // Split content into lines, preserving empty lines
        let lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(String::from).collect()
        };

        Self {
            lines,
            mode: VimMode::Normal,
            cursor: (0, 0),
            clipboard: Vec::new(),
            count_prefix: None,
            pending_command: None,
            command_buffer: String::new(),
            visual_anchor: None,
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
            search_pattern: None,
            search_matches: Vec::new(),
            current_match: None,
            last_find: None,
            last_ex_command: None,
        }
    }

    /// Get the current content as a single string with newlines
    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    /// Get the current vim mode
    pub fn mode(&self) -> VimMode {
        self.mode
    }

    /// Get the current cursor position (line, column)
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Get the number of lines in the document
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get a specific line by index
    pub fn line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    /// Get all lines as a slice
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Set cursor position (for testing)
    pub fn set_cursor_for_test(&mut self, line: usize, col: usize) {
        self.cursor = (line, col);
        self.clamp_cursor();
    }

    /// Clamp cursor to valid position
    ///
    /// Ensures cursor is within document bounds and respects mode-specific constraints.
    fn clamp_cursor(&mut self) {
        // Ensure line is valid
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        let line_count = self.lines.len();
        self.cursor.0 = self.cursor.0.min(line_count - 1);

        // Clamp column to line length
        let line = &self.lines[self.cursor.0];
        let line_len = line.chars().count();

        self.cursor.1 = match self.mode {
            VimMode::Insert => {
                // Insert mode: cursor can be at end (after last char)
                self.cursor.1.min(line_len)
            }
            _ => {
                // Normal/Visual modes: cursor must be on a character
                if line_len == 0 {
                    0
                } else {
                    self.cursor.1.min(line_len - 1)
                }
            }
        };
    }

    /// Get count prefix and reset it
    pub fn take_count(&mut self) -> usize {
        self.count_prefix.take().unwrap_or(1)
    }

    /// Set count prefix
    pub fn set_count_prefix(&mut self, count: usize) {
        self.count_prefix = Some(count);
    }

    /// Get pending command
    pub fn pending_command(&self) -> Option<PendingCommand> {
        self.pending_command
    }

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

    /// Get display string for pending command
    pub fn pending_display(&self) -> Option<&str> {
        match self.pending_command {
            Some(PendingCommand::G) => Some("g"),
            Some(PendingCommand::D) => Some("d"),
            Some(PendingCommand::Y) => Some("y"),
            Some(PendingCommand::C) => Some("c"),
            Some(PendingCommand::Z) => Some("Z"),
            Some(PendingCommand::FindForward) => Some("f"),
            Some(PendingCommand::FindBackward) => Some("F"),
            Some(PendingCommand::TillForward) => Some("t"),
            Some(PendingCommand::TillBackward) => Some("T"),
            Some(PendingCommand::Replace) => Some("r"),
            Some(PendingCommand::IndentRight) => Some(">"),
            Some(PendingCommand::IndentLeft) => Some("<"),
            None => None,
        }
    }

    // ============================================================================
    // Internal Helpers
    // ============================================================================

    /// Get current line (where cursor is)
    pub(crate) fn current_line(&self) -> &str {
        self.lines
            .get(self.cursor.0)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get mutable reference to current line
    pub(crate) fn current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor.0]
    }

    // ============================================================================
    // Mode Transitions
    // ============================================================================

    /// Enter insert mode
    pub fn enter_insert_mode(&mut self) {
        self.mode = VimMode::Insert;
    }

    /// Exit insert mode (return to normal mode)
    pub fn exit_insert_mode(&mut self) {
        self.mode = VimMode::Normal;
        self.clamp_cursor();
    }

    // ============================================================================
    // High-Level Key Handling
    // ============================================================================

    /// Handle a keyboard input event
    ///
    /// This is a high-level method that dispatches KeyEvents to appropriate vim commands.
    /// Returns true if the key was handled, false otherwise.
    ///
    /// This method is typically used by embedding contexts (SQL editor, Magnifier)
    /// to delegate vim editing to the VimEditor.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match self.mode {
            VimMode::Normal | VimMode::Visual | VimMode::VisualLine => {
                self.handle_normal_mode_key(key)
            }
            VimMode::Insert => self.handle_insert_mode_key(key),
            VimMode::Command => self.handle_command_mode_key(key),
        }
    }

    /// Check if an ex command was executed and return it
    ///
    /// Returns Some(command) if an ex command was just executed (e.g., "w", "q", "wq")
    /// This allows the embedding context to handle commands like :w (save) or :q (quit)
    pub fn check_ex_command(&mut self) -> Option<String> {
        self.last_ex_command.take().map(|cmd| match cmd {
            commands::ExCommand::Write => "w".to_string(),
            commands::ExCommand::Quit => "q".to_string(),
            commands::ExCommand::WriteQuit => "wq".to_string(),
            commands::ExCommand::ForceQuit => "q!".to_string(),
            commands::ExCommand::NoHighlight => "noh".to_string(),
            commands::ExCommand::Unknown(s) => s,
        })
    }

    // ============================================================================
    // Internal Key Handlers
    // ============================================================================

    /// Handle key in normal/visual modes
    fn handle_normal_mode_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // Esc: Exit visual mode or clear pending
            (KeyCode::Esc, _) => {
                if self.mode == VimMode::Visual || self.mode == VimMode::VisualLine {
                    self.exit_visual_mode();
                } else {
                    self.count_prefix = None;
                    self.pending_command = None;
                }
                true
            }

            // i: Enter insert mode
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.push_undo(); // Save state before entering insert
                self.enter_insert_mode();
                true
            }

            // a: Append (insert after cursor)
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                self.push_undo(); // Save state before entering insert
                self.move_right();
                self.enter_insert_mode();
                true
            }

            // A: Append at end of line
            (KeyCode::Char('A'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.push_undo(); // Save state before entering insert
                self.move_to_line_end();
                self.enter_insert_mode();
                // In insert mode, move after last char
                self.cursor.1 += 1;
                true
            }

            // o: Open line below
            (KeyCode::Char('o'), KeyModifiers::NONE) => {
                self.push_undo(); // Save state before modification
                self.insert_line_below();
                true
            }

            // O: Open line above
            (KeyCode::Char('O'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.push_undo(); // Save state before modification
                self.insert_line_above();
                true
            }

            // Navigation: hjkl
            (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, _) => {
                let count = self.take_count();
                for _ in 0..count {
                    self.move_left();
                }
                true
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
                let count = self.take_count();
                for _ in 0..count {
                    self.move_down();
                }
                true
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
                let count = self.take_count();
                for _ in 0..count {
                    self.move_up();
                }
                true
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, _) => {
                let count = self.take_count();
                for _ in 0..count {
                    self.move_right();
                }
                true
            }

            // Word motions
            (KeyCode::Char('w'), KeyModifiers::NONE) => {
                let count = self.take_count();
                for _ in 0..count {
                    self.move_next_word();
                }
                true
            }
            (KeyCode::Char('b'), KeyModifiers::NONE) => {
                let count = self.take_count();
                for _ in 0..count {
                    self.move_prev_word();
                }
                true
            }
            (KeyCode::Char('e'), KeyModifiers::NONE) => {
                let count = self.take_count();
                for _ in 0..count {
                    self.move_end_word();
                }
                true
            }

            // Line motions
            (KeyCode::Char('0'), KeyModifiers::NONE) => {
                self.move_to_line_start();
                true
            }
            (KeyCode::Char('$'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.move_to_line_end();
                true
            }

            // File motions
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                if self.has_pending() && self.pending_command() == Some(PendingCommand::G) {
                    // gg: Go to first line
                    self.take_pending();
                    self.move_to_first_line();
                } else {
                    self.set_pending(PendingCommand::G);
                }
                true
            }
            (KeyCode::Char('G'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.move_to_last_line();
                true
            }

            // Delete operations
            (KeyCode::Char('x'), KeyModifiers::NONE) => {
                self.push_undo(); // Save before delete
                self.delete_char();
                true
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if self.mode == VimMode::Visual || self.mode == VimMode::VisualLine {
                    // Delete visual selection
                    self.push_undo(); // Save before delete
                    self.delete_selection();
                    self.exit_visual_mode();
                } else if self.has_pending() && self.pending_command() == Some(PendingCommand::D) {
                    // dd: Delete line(s) with count
                    self.take_pending();
                    self.push_undo(); // Save before delete
                    let count = self.take_count();
                    for _ in 0..count {
                        self.delete_line();
                    }
                } else {
                    self.set_pending(PendingCommand::D);
                }
                true
            }
            // D: Delete to end of line
            (KeyCode::Char('D'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.push_undo(); // Save before delete
                self.delete_to_eol();
                true
            }

            // Yank operations
            (KeyCode::Char('y'), KeyModifiers::NONE) => {
                if self.mode == VimMode::Visual || self.mode == VimMode::VisualLine {
                    // Yank visual selection
                    self.yank_selection();
                    self.exit_visual_mode();
                } else if self.has_pending() && self.pending_command() == Some(PendingCommand::Y) {
                    // yy: Yank line
                    self.take_pending();
                    self.yank_line();
                } else {
                    self.set_pending(PendingCommand::Y);
                }
                true
            }

            // Paste
            (KeyCode::Char('p'), KeyModifiers::NONE) => {
                self.paste_below();
                true
            }
            (KeyCode::Char('P'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.paste_above();
                true
            }

            // Undo/Redo
            (KeyCode::Char('u'), KeyModifiers::NONE) => {
                self.undo();
                true
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.redo();
                true
            }

            // Visual mode
            (KeyCode::Char('v'), KeyModifiers::NONE) => {
                self.enter_visual_mode();
                true
            }
            (KeyCode::Char('V'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.enter_visual_line_mode();
                true
            }

            // Search
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                // Future: Implement search prompt UI for '/' command
                // Note: n/N/* search commands work using external search results
                false
            }
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                self.jump_to_next_match();
                true
            }
            (KeyCode::Char('N'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.jump_to_prev_match();
                true
            }
            (KeyCode::Char('*'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.search_word_under_cursor();
                true
            }

            // Ex command mode
            (KeyCode::Char(':'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.mode = VimMode::Command;
                self.command_buffer.clear();
                true
            }

            // Numbers for count prefix
            (KeyCode::Char(c), KeyModifiers::NONE) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).expect("digit validated by is_ascii_digit") as usize;
                if let Some(count) = self.count_prefix {
                    self.count_prefix = Some(count * 10 + digit);
                } else if digit > 0 {
                    self.count_prefix = Some(digit);
                }
                true
            }

            _ => false,
        }
    }

    /// Handle key in insert mode
    fn handle_insert_mode_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // Esc: Exit insert mode
            (KeyCode::Esc, _) => {
                self.exit_insert_mode();
                true
            }

            // Enter: Insert newline
            (KeyCode::Enter, _) => {
                self.newline();
                true
            }

            // Backspace: Delete before cursor
            (KeyCode::Backspace, _) => {
                self.delete_char_before();
                true
            }

            // Regular character input
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.insert_char(c);
                true
            }

            // Arrow keys for navigation in insert mode
            (KeyCode::Left, _) => {
                self.move_left();
                true
            }
            (KeyCode::Right, _) => {
                self.move_right();
                true
            }
            (KeyCode::Up, _) => {
                self.move_up();
                true
            }
            (KeyCode::Down, _) => {
                self.move_down();
                true
            }

            _ => false,
        }
    }

    /// Handle key in command mode
    fn handle_command_mode_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // Esc: Exit command mode
            (KeyCode::Esc, _) => {
                self.exit_command_mode();
                true
            }

            // Enter: Execute command
            (KeyCode::Enter, _) => {
                let cmd = self.parse_command();
                match cmd {
                    commands::ExCommand::NoHighlight => {
                        self.clear_search();
                    }
                    _ => {
                        // Store for embedding context to handle via check_ex_command()
                        self.last_ex_command = Some(cmd);
                    }
                }
                self.exit_command_mode();
                true
            }

            // Backspace: Delete character from command buffer
            (KeyCode::Backspace, _) => {
                self.command_backspace();
                true
            }

            // Character input
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.command_insert_char(c);
                true
            }

            _ => false,
        }
    }
}
