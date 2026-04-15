//! Magnifier mode methods on App.

use super::{App, Mode};
use crate::domain::position::RowIndex;

impl App {
    /// Open magnifier for the current cell
    pub fn open_magnifier(&mut self) {
        let row = self
            .view_state
            .table_state
            .selected()
            .map(RowIndex::new)
            .unwrap_or(RowIndex::new(0));
        let col = self.view_state.selected_column;

        // Get cell content — show formula text if this cell has a formula
        let cell_content = self.cell_formula_or_value(row, col);

        // Create magnifier state
        self.magnifier_state = Some(crate::magnifier::MagnifierState::new(
            cell_content,
            (row, col),
        ));

        // Switch to magnifier mode
        self.mode = Mode::Magnifier;
    }

    /// Save magnifier content to cell (keep magnifier open)
    pub fn save_magnifier_content(&mut self) {
        if let Some(mag) = &self.magnifier_state {
            let content = mag.content();
            let (row, col) = mag.cell_position();

            // Use commit_cell_value to handle formula detection
            self.commit_cell_value(row, col, content.clone());

            // Update magnifier's original content so it's no longer dirty
            if let Some(mag) = &mut self.magnifier_state {
                mag.mark_clean_with_content(content);
            }
        }
    }

    /// Save magnifier content to cell and close magnifier
    pub fn save_and_close_magnifier(&mut self) {
        if let Some(mag) = self.magnifier_state.take() {
            let content = mag.content();
            let (row, col) = mag.cell_position();

            // Use commit_cell_value to handle formula detection
            self.commit_cell_value(row, col, content);

            // Return to normal mode
            self.mode = Mode::Normal;
        }
    }

    /// Close magnifier without saving changes
    pub fn close_magnifier_discard(&mut self) {
        self.magnifier_state = None;
        self.mode = Mode::Normal;
    }

    /// Check if magnifier has unsaved changes
    pub fn magnifier_is_dirty(&self) -> bool {
        self.magnifier_state
            .as_ref()
            .map(|m| m.is_dirty())
            .unwrap_or(false)
    }

    /// Get mutable reference to magnifier state (for input handling)
    pub fn magnifier_state_mut(&mut self) -> Option<&mut crate::magnifier::MagnifierState> {
        self.magnifier_state.as_mut()
    }
}
