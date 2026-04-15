//! Visual mode selection tracking.

use crate::domain::position::{ColIndex, RowIndex};

/// Visual mode selection anchor and type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualSelection {
    /// Starting position of the selection
    pub anchor: (RowIndex, ColIndex),
    /// Current cursor position (end of selection)
    pub cursor: (RowIndex, ColIndex),
    /// Type of visual selection
    pub mode: VisualMode,
}

/// Type of visual selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualMode {
    /// Rectangular block selection
    Block,
    /// Whole row selection
    Line,
    /// Whole column selection
    Column,
}

impl VisualSelection {
    /// Create a new visual selection starting at the given position
    pub fn new(row: RowIndex, col: ColIndex, mode: VisualMode) -> Self {
        Self {
            anchor: (row, col),
            cursor: (row, col),
            mode,
        }
    }

    /// Update the cursor position
    pub fn update_cursor(&mut self, row: RowIndex, col: ColIndex) {
        self.cursor = (row, col);
    }

    /// Get the selection bounds as (start_row, end_row, start_col, end_col)
    /// Returns normalized bounds (start <= end)
    pub fn bounds(&self) -> (RowIndex, RowIndex, ColIndex, ColIndex) {
        let (start_row, end_row) = if self.anchor.0 <= self.cursor.0 {
            (self.anchor.0, self.cursor.0)
        } else {
            (self.cursor.0, self.anchor.0)
        };

        let (start_col, end_col) = if self.anchor.1 <= self.cursor.1 {
            (self.anchor.1, self.cursor.1)
        } else {
            (self.cursor.1, self.anchor.1)
        };

        (start_row, end_row, start_col, end_col)
    }
}
