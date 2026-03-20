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
pub mod key_handler;
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
                key_handler::handle_key(self, key).was_handled()
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

            // Enter: Execute command or search
            (KeyCode::Enter, _) => {
                if self.command_buffer.starts_with('/') {
                    // Search forward
                    let pattern = self.command_buffer[1..].to_string();
                    self.mode = VimMode::Normal;
                    self.command_buffer.clear();
                    if !pattern.is_empty() {
                        self.search_forward(pattern);
                    }
                } else if self.command_buffer.starts_with('%') {
                    // Substitution: %s/old/new/g
                    let cmd_str = self.command_buffer.clone();
                    self.mode = VimMode::Normal;
                    self.command_buffer.clear();
                    self.execute_substitution(&cmd_str);
                } else {
                    let cmd = self.parse_command();
                    match cmd {
                        commands::ExCommand::NoHighlight => {
                            self.clear_search();
                        }
                        _ => {
                            self.last_ex_command = Some(cmd);
                        }
                    }
                    self.exit_command_mode();
                }
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

    /// Execute a substitution command (:%s/old/new/g or %s/old/new/g)
    pub fn execute_substitution(&mut self, cmd: &str) {
        // Parse %s/old/new/g or %s/old/new/
        let cmd = cmd.strip_prefix('%').unwrap_or(cmd);
        let cmd = cmd.strip_prefix('s').unwrap_or(cmd);

        if cmd.is_empty() {
            return;
        }

        let delim = cmd.chars().next().unwrap();
        let parts: Vec<&str> = cmd[delim.len_utf8()..].split(delim).collect();
        if parts.len() < 2 {
            return;
        }

        let pattern = parts[0];
        let replacement = parts[1];
        let global = parts.get(2).map(|s| s.contains('g')).unwrap_or(false);

        if pattern.is_empty() {
            return;
        }

        self.push_undo();

        for line in &mut self.lines {
            if global {
                *line = line.replace(pattern, replacement);
            } else {
                // Replace first occurrence only
                if let Some(pos) = line.find(pattern) {
                    let mut new_line = String::with_capacity(line.len());
                    new_line.push_str(&line[..pos]);
                    new_line.push_str(replacement);
                    new_line.push_str(&line[pos + pattern.len()..]);
                    *line = new_line;
                }
            }
        }
    }
}
