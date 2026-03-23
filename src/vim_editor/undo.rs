//! Undo/redo history management

use super::VimEditor;

/// Undo snapshot for state restoration
///
/// Stores a complete snapshot of the document and cursor position for undo/redo operations.
/// We use full snapshots instead of deltas for simplicity and correctness.
#[derive(Debug, Clone)]
pub struct UndoSnapshot {
    pub lines: Vec<String>,
    pub cursor: (usize, usize),
}

impl UndoSnapshot {
    /// Create a new undo snapshot
    pub fn new(lines: Vec<String>, cursor: (usize, usize)) -> Self {
        Self { lines, cursor }
    }
}

impl VimEditor {
    // ============================================================================
    // Undo/Redo Operations
    // ============================================================================

    /// Push current state to undo stack
    ///
    /// Should be called before any editing operation to allow undoing it.
    /// Clears the redo stack since we're creating a new timeline.
    pub fn push_undo(&mut self) {
        let snapshot = UndoSnapshot {
            lines: self.lines.clone(),
            cursor: self.cursor,
        };

        // Limit undo history depth
        if self.undo_stack.len() >= self.undo_limit {
            self.undo_stack.pop_front();
        }

        self.undo_stack.push_back(snapshot);
        self.redo_stack.clear(); // Clear redo on new edit
    }

    /// Undo last change (u)
    ///
    /// Restores the previous state from undo stack and moves current state to redo stack.
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop_back() {
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
    ///
    /// Restores the next state from redo stack and moves current state to undo stack.
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo
            let current = UndoSnapshot {
                lines: self.lines.clone(),
                cursor: self.cursor,
            };
            self.undo_stack.push_back(current);

            // Restore snapshot
            self.lines = snapshot.lines;
            self.cursor = snapshot.cursor;
            self.clamp_cursor();
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
