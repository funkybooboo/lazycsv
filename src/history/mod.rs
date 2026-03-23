//! CSV-level undo/redo history.
//!
//! Tracks document mutations as reversible commands. Each mutation
//! (cell edit, row insert/delete, column insert/delete) is recorded
//! so it can be undone with `u` and redone with `Ctrl+r`.
//!
//! Design:
//! - Command-based (stores deltas, not full snapshots) for efficiency
//! - Compound operations (e.g. 5dd) are a single undo step
//! - Per-file history preserved across file switches
//! - `:w` does NOT clear undo history

use crate::csv::document::Document;
use crate::domain::position::{ColIndex, RowIndex};
use std::collections::VecDeque;

/// A reversible edit command that can be undone/redone.
#[derive(Debug, Clone)]
pub enum EditCommand {
    /// Cell value changed: (row, col, old_value, new_value)
    SetCell {
        row: RowIndex,
        col: ColIndex,
        old_value: String,
        new_value: String,
    },

    /// Row inserted at index
    InsertRow {
        at: RowIndex,
    },

    /// Row deleted: (index, deleted_data)
    DeleteRow {
        at: RowIndex,
        data: Vec<String>,
    },

    /// Multiple rows deleted: (start, deleted_data)
    DeleteRows {
        start: RowIndex,
        data: Vec<Vec<String>>,
    },

    /// Column inserted at index with data
    InsertColumn {
        at: ColIndex,
        data: Vec<String>,
    },

    /// Column deleted: (index, header, column_data)
    DeleteColumn {
        at: ColIndex,
        data: Vec<String>,
    },

    /// Multiple columns deleted: (start, headers+data per column)
    DeleteColumns {
        start: ColIndex,
        data: Vec<Vec<String>>,
    },

    /// Compound operation (multiple commands as a single undo step)
    Compound(Vec<EditCommand>),
}

impl EditCommand {
    /// Apply this command's undo (reverse the edit) to the document.
    pub fn undo(&self, doc: &mut Document) {
        match self {
            EditCommand::SetCell {
                row,
                col,
                old_value,
                ..
            } => {
                doc.set_cell(*row, *col, old_value.clone());
            }
            EditCommand::InsertRow { at } => {
                doc.delete_row(*at);
            }
            EditCommand::DeleteRow { at, data } => {
                doc.insert_row(*at);
                for (col_idx, value) in data.iter().enumerate() {
                    if !value.is_empty() {
                        doc.set_cell(*at, ColIndex::new(col_idx), value.clone());
                    }
                }
            }
            EditCommand::DeleteRows { start, data } => {
                // Re-insert rows in order
                for (i, row_data) in data.iter().enumerate() {
                    let row_idx = RowIndex::new(start.get() + i);
                    doc.insert_row(row_idx);
                    for (col_idx, value) in row_data.iter().enumerate() {
                        if !value.is_empty() {
                            doc.set_cell(row_idx, ColIndex::new(col_idx), value.clone());
                        }
                    }
                }
            }
            EditCommand::InsertColumn { at, .. } => {
                doc.delete_column(*at);
            }
            EditCommand::DeleteColumn { at, data } => {
                doc.insert_column(*at, data.clone());
            }
            EditCommand::DeleteColumns { start, data } => {
                // Re-insert columns left to right
                for (i, col_data) in data.iter().enumerate() {
                    let col_idx = ColIndex::new(start.get() + i);
                    doc.insert_column(col_idx, col_data.clone());
                }
            }
            EditCommand::Compound(commands) => {
                // Undo in reverse order
                for cmd in commands.iter().rev() {
                    cmd.undo(doc);
                }
            }
        }
    }

    /// Apply this command's redo (re-apply the edit) to the document.
    pub fn redo(&self, doc: &mut Document) {
        match self {
            EditCommand::SetCell {
                row,
                col,
                new_value,
                ..
            } => {
                doc.set_cell(*row, *col, new_value.clone());
            }
            EditCommand::InsertRow { at } => {
                doc.insert_row(*at);
            }
            EditCommand::DeleteRow { at, .. } => {
                doc.delete_row(*at);
            }
            EditCommand::DeleteRows { start, data } => {
                // Delete from end to start so indices stay valid
                let end = RowIndex::new(start.get() + data.len() - 1);
                doc.delete_rows(*start, end);
            }
            EditCommand::InsertColumn { at, .. } => {
                doc.delete_column(*at);
            }
            EditCommand::DeleteColumn { at, data } => {
                // delete_column needs the column to exist; we insert then remove for redo
                // Actually simpler: just delete the column at that index
                // But the column was already deleted... for redo we need to delete again
                // The column was restored by undo, so we just delete it again
                doc.delete_column(*at);
                let _ = data; // data only needed for undo
            }
            EditCommand::DeleteColumns { start, data } => {
                let end = ColIndex::new(start.get() + data.len() - 1);
                doc.delete_columns(*start, end);
            }
            EditCommand::Compound(commands) => {
                // Redo in forward order
                for cmd in commands {
                    cmd.redo(doc);
                }
            }
        }
    }
}

/// Undo/redo history for a single document.
#[derive(Debug, Clone)]
pub struct History {
    undo_stack: VecDeque<EditCommand>,
    redo_stack: Vec<EditCommand>,
    max_size: usize,
}

impl History {
    /// Create a new history with the given max undo depth.
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Record a new edit command. Clears the redo stack.
    pub fn push(&mut self, command: EditCommand) {
        if self.undo_stack.len() >= self.max_size {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(command);
        self.redo_stack.clear();
    }

    /// Undo the last command. Returns the command that was undone (for redo).
    pub fn undo(&mut self, doc: &mut Document) -> bool {
        if let Some(cmd) = self.undo_stack.pop_back() {
            cmd.undo(doc);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    /// Redo the last undone command.
    pub fn redo(&mut self, doc: &mut Document) -> bool {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.redo(doc);
            self.undo_stack.push_back(cmd);
            true
        } else {
            false
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Number of undo steps available.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Document::new puts headers as row 0, so data rows start at index 1.
    // row_count() returns total rows including header.
    // cell(RowIndex(1), ...) = first data row.

    fn test_doc() -> Document {
        Document::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                vec!["1".into(), "2".into(), "3".into()],
                vec!["4".into(), "5".into(), "6".into()],
                vec!["7".into(), "8".into(), "9".into()],
            ],
            "test.csv".into(),
        )
    }

    #[test]
    fn test_undo_set_cell() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        let old = doc
            .set_cell(RowIndex::new(1), ColIndex::new(0), "X".into())
            .unwrap();
        history.push(EditCommand::SetCell {
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            old_value: old,
            new_value: "X".into(),
        });

        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "X");

        history.undo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "1");
    }

    #[test]
    fn test_redo_set_cell() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        let old = doc
            .set_cell(RowIndex::new(1), ColIndex::new(0), "X".into())
            .unwrap();
        history.push(EditCommand::SetCell {
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            old_value: old,
            new_value: "X".into(),
        });

        history.undo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "1");

        history.redo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "X");
    }

    #[test]
    fn test_undo_insert_row() {
        let mut doc = test_doc();
        let mut history = History::new(100);
        let orig_count = doc.row_count();

        doc.insert_row(RowIndex::new(2));
        history.push(EditCommand::InsertRow {
            at: RowIndex::new(2),
        });
        assert_eq!(doc.row_count(), orig_count + 1);

        history.undo(&mut doc);
        assert_eq!(doc.row_count(), orig_count);
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "4");
    }

    #[test]
    fn test_undo_delete_row() {
        let mut doc = test_doc();
        let mut history = History::new(100);
        let orig_count = doc.row_count();

        // Delete row 2 = ["4","5","6"]
        let data = doc.delete_row(RowIndex::new(2)).unwrap();
        assert_eq!(data, vec!["4", "5", "6"]);
        history.push(EditCommand::DeleteRow {
            at: RowIndex::new(2),
            data,
        });
        assert_eq!(doc.row_count(), orig_count - 1);
        // Row 2 is now ["7","8","9"]
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "7");

        history.undo(&mut doc);
        assert_eq!(doc.row_count(), orig_count);
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "4");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(1)), "5");
    }

    #[test]
    fn test_undo_delete_rows() {
        let mut doc = test_doc();
        let mut history = History::new(100);
        let orig_count = doc.row_count();

        // Delete data rows 1 and 2 (indices 1-2)
        let data = doc.delete_rows(RowIndex::new(1), RowIndex::new(2));
        history.push(EditCommand::DeleteRows {
            start: RowIndex::new(1),
            data,
        });
        assert_eq!(doc.row_count(), orig_count - 2);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "7");

        history.undo(&mut doc);
        assert_eq!(doc.row_count(), orig_count);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "1");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "4");
    }

    #[test]
    fn test_undo_delete_column() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        let data = doc.delete_column(ColIndex::new(1));
        history.push(EditCommand::DeleteColumn {
            at: ColIndex::new(1),
            data,
        });
        assert_eq!(doc.column_count(), 2);

        history.undo(&mut doc);
        assert_eq!(doc.column_count(), 3);
        assert_eq!(doc.header(ColIndex::new(1)), "B");
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(1)), "2");
    }

    #[test]
    fn test_undo_insert_column() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        let data = vec!["New".into(), "a".into(), "b".into(), "c".into()];
        doc.insert_column(ColIndex::new(1), data.clone());
        history.push(EditCommand::InsertColumn {
            at: ColIndex::new(1),
            data,
        });
        assert_eq!(doc.column_count(), 4);

        history.undo(&mut doc);
        assert_eq!(doc.column_count(), 3);
        assert_eq!(doc.header(ColIndex::new(1)), "B");
    }

    #[test]
    fn test_undo_limit_respected() {
        let mut doc = test_doc();
        let mut history = History::new(3);

        for i in 0..5 {
            let old = doc
                .set_cell(
                    RowIndex::new(1),
                    ColIndex::new(0),
                    format!("v{}", i),
                )
                .unwrap();
            history.push(EditCommand::SetCell {
                row: RowIndex::new(1),
                col: ColIndex::new(0),
                old_value: old,
                new_value: format!("v{}", i),
            });
        }

        assert_eq!(history.undo_count(), 3);

        // Can only undo 3 times
        assert!(history.undo(&mut doc));
        assert!(history.undo(&mut doc));
        assert!(history.undo(&mut doc));
        assert!(!history.undo(&mut doc));
    }

    #[test]
    fn test_new_command_clears_redo() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        let old = doc
            .set_cell(RowIndex::new(1), ColIndex::new(0), "X".into())
            .unwrap();
        history.push(EditCommand::SetCell {
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            old_value: old,
            new_value: "X".into(),
        });

        history.undo(&mut doc);
        assert!(history.can_redo());

        // New edit should clear redo
        let old2 = doc
            .set_cell(RowIndex::new(1), ColIndex::new(0), "Y".into())
            .unwrap();
        history.push(EditCommand::SetCell {
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            old_value: old2,
            new_value: "Y".into(),
        });
        assert!(!history.can_redo());
    }

    #[test]
    fn test_multiple_undo_redo_cycle() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        // Edit 1
        let old1 = doc
            .set_cell(RowIndex::new(1), ColIndex::new(0), "A".into())
            .unwrap();
        history.push(EditCommand::SetCell {
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            old_value: old1,
            new_value: "A".into(),
        });

        // Edit 2
        let old2 = doc
            .set_cell(RowIndex::new(1), ColIndex::new(0), "B".into())
            .unwrap();
        history.push(EditCommand::SetCell {
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            old_value: old2,
            new_value: "B".into(),
        });

        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "B");

        // Undo twice
        history.undo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "A");
        history.undo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "1");

        // Redo twice
        history.redo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "A");
        history.redo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "B");
    }

    #[test]
    fn test_compound_undo() {
        let mut doc = test_doc();
        let mut history = History::new(100);
        let orig_count = doc.row_count();

        // Simulate 3dd: delete all 3 data rows as one step
        let data = doc.delete_rows(RowIndex::new(1), RowIndex::new(3));
        history.push(EditCommand::DeleteRows {
            start: RowIndex::new(1),
            data,
        });
        assert_eq!(doc.row_count(), orig_count - 3);

        history.undo(&mut doc);
        assert_eq!(doc.row_count(), orig_count);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "1");
        assert_eq!(doc.cell(RowIndex::new(3), ColIndex::new(2)), "9");
    }

    #[test]
    fn test_empty_history() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert!(!history.undo(&mut doc));
        assert!(!history.redo(&mut doc));
    }

    #[test]
    fn test_undo_preserves_other_cells() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        let old = doc
            .set_cell(RowIndex::new(1), ColIndex::new(0), "X".into())
            .unwrap();
        history.push(EditCommand::SetCell {
            row: RowIndex::new(1),
            col: ColIndex::new(0),
            old_value: old,
            new_value: "X".into(),
        });

        // Other cells unchanged
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(1)), "2");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "4");

        history.undo(&mut doc);
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "1");
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(1)), "2");
    }

    #[test]
    fn test_delete_columns_undo() {
        let mut doc = test_doc();
        let mut history = History::new(100);

        let data = doc.delete_columns(ColIndex::new(0), ColIndex::new(1));
        history.push(EditCommand::DeleteColumns {
            start: ColIndex::new(0),
            data,
        });
        assert_eq!(doc.column_count(), 1);
        assert_eq!(doc.header(ColIndex::new(0)), "C");

        history.undo(&mut doc);
        assert_eq!(doc.column_count(), 3);
        assert_eq!(doc.header(ColIndex::new(0)), "A");
        assert_eq!(doc.header(ColIndex::new(1)), "B");
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "1");
    }

    #[test]
    fn test_redo_delete_row() {
        let mut doc = test_doc();
        let mut history = History::new(100);
        let orig_count = doc.row_count();

        // Delete row 2 = ["4","5","6"]
        let data = doc.delete_row(RowIndex::new(2)).unwrap();
        history.push(EditCommand::DeleteRow {
            at: RowIndex::new(2),
            data,
        });
        assert_eq!(doc.row_count(), orig_count - 1);

        history.undo(&mut doc);
        assert_eq!(doc.row_count(), orig_count);

        history.redo(&mut doc);
        assert_eq!(doc.row_count(), orig_count - 1);
        // Row 2 is now ["7","8","9"] again
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "7");
    }
}
