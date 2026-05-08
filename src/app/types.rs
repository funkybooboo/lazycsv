//! Core application types — Mode, EditBuffer, FileOperation.

use std::path::PathBuf;

/// Application modes (vim-style modal editing)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Default mode for navigation and commands
    Normal,
    /// Quick single-cell editing (entered via i, a, s)
    Insert,
    /// Full vim editor for cell content (entered via Enter)
    Magnifier,
    /// Visual Block mode - rectangular cell selection (entered via v)
    VisualBlock,
    /// Visual Line mode - whole row selection (entered via V)
    VisualLine,
    /// Visual Column mode - whole column selection (entered via ,v)
    VisualColumn,
    /// Execute commands (entered via :)
    Command,
    /// File list picker (entered via :files)
    FileList,
    /// SQL query editor (entered via q)
    SqlEditor,
    /// Search input (entered via /)
    Search,
    /// Prompt for file operations (rename, move, copy, create)
    FileOperationPrompt,
    /// Theme selector modal (entered via :theme or T)
    ThemeSelector,
}

/// Edit buffer for cell editing
#[derive(Debug, Clone, Default)]
pub struct EditBuffer {
    /// Current content being edited
    pub content: String,
    /// Cursor position within content
    pub cursor: usize,
    /// Original content (for cancel/undo)
    pub original: String,
}

/// File operation being prompted for
#[derive(Debug, Clone, PartialEq)]
pub enum FileOperation {
    Rename(PathBuf), // Original path
    Delete(PathBuf), // Path to delete
    Move(PathBuf),   // Source path
    Copy(PathBuf),   // Source path
    Create,          // New file in current directory
}
