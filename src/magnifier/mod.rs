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
/// Most VimEditor methods are available through Deref/DerefMut.
#[derive(Debug, Clone)]
pub struct MagnifierState {
    /// Vim editor for all text editing operations
    editor: VimEditor,

    /// Original cell position in the CSV (for display)
    cell_position: (RowIndex, ColIndex),

    /// Original content (for dirty checking)
    original_content: String,
}

// Implement Deref to allow transparent access to VimEditor methods
impl std::ops::Deref for MagnifierState {
    type Target = VimEditor;

    fn deref(&self) -> &Self::Target {
        &self.editor
    }
}

// Implement DerefMut to allow transparent mutable access to VimEditor methods
impl std::ops::DerefMut for MagnifierState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.editor
    }
}

impl MagnifierState {
    /// Create a new magnifier state from cell content
    pub fn new(content: String, position: (RowIndex, ColIndex)) -> Self {
        let editor = VimEditor::new(content.clone());

        Self {
            editor,
            cell_position: position,
            original_content: content,
        }
    }

    // ============================================================================
    // Magnifier-Specific Methods
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

    /// Get cell position in CSV
    pub fn cell_position(&self) -> (RowIndex, ColIndex) {
        self.cell_position
    }

    /// Get visual selection (converts from VimEditor's Selection type)
    pub fn visual_selection(&self) -> Option<Selection> {
        self.editor.visual_selection().map(Selection::from)
    }

    // Note: All other VimEditor methods (mode(), cursor(), move_left(), etc.)
    // are available through Deref/DerefMut traits
}
