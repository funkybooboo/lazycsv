//! Formula-related methods on App.

use super::App;
use crate::domain::position::{ColIndex, RowIndex};

impl App {
    /// Commit a cell value, detecting and storing formulas.
    /// If `content` starts with '=', tries to parse it as a formula.
    /// The document always stores the computed value; the formula is kept separately.
    pub fn commit_cell_value(&mut self, row: RowIndex, col: ColIndex, content: String) {
        // Record old value for undo before mutation
        let old_value = self.document.cell(row, col);

        if let Some(formula) = crate::formula::parse_formula(&content) {
            // Evaluate using document cell values
            let computed = formula.evaluate(&|r, c| self.document.storage.get_cell(r, c));
            let new_value = computed.clone();
            self.formula_store
                .set(row.get(), col.get(), content, formula);
            self.document.set_cell(row, col, computed);
            self.history.push(crate::history::EditCommand::SetCell {
                row,
                col,
                old_value,
                new_value,
            });
        } else {
            // Not a formula — remove any existing formula and store raw value
            let new_value = content.clone();
            self.formula_store.remove(row.get(), col.get());
            self.document.set_cell(row, col, content);
            self.history.push(crate::history::EditCommand::SetCell {
                row,
                col,
                old_value,
                new_value,
            });
        }

        // Store for dot-repeat (use row=0, col=0 as placeholder — dot applies at cursor)
        self.last_edit = Some(crate::history::EditCommand::SetCell {
            row,
            col,
            old_value: String::new(), // placeholder
            new_value: self.document.cell(row, col),
        });

        self.document.is_dirty = true;
        let file_path = self.current_file().clone();
        self.session.mark_dirty(&file_path);
        self.last_edit_position = Some((row, col));

        // Re-evaluate any formulas that reference this cell
        self.re_evaluate_formulas_referencing(row.get(), col.get());
    }

    /// Re-evaluate all formulas that reference the given cell.
    fn re_evaluate_formulas_referencing(&mut self, changed_row: usize, changed_col: usize) {
        let dependents = self
            .formula_store
            .cells_referencing(changed_row, changed_col);
        for (r, c) in dependents {
            if let Some(formula) = self.formula_store.get_formula(r, c).cloned() {
                let computed =
                    formula.evaluate(&|row, col| self.document.storage.get_cell(row, col));
                self.document.storage.set_cell(r, c, computed);
            }
        }
    }

    /// Get the display value for a cell — the formula text if it has a formula, otherwise the raw value.
    /// Used when entering edit mode or showing in the formula bar.
    pub fn cell_formula_or_value(&self, row: RowIndex, col: ColIndex) -> String {
        if let Some(raw) = self.formula_store.get_raw(row.get(), col.get()) {
            raw.to_string()
        } else {
            self.document.cell(row, col).to_string()
        }
    }
}
