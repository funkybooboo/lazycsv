//! Type-safe position types for CSV table navigation.
//!
//! This module provides newtype wrappers for row and column indices to prevent
//! accidental mixing of row/column coordinates at compile time.

use std::num::NonZeroUsize;

/// Newtype wrapper for row indices to prevent confusion with column indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowIndex(usize);

impl RowIndex {
    /// Create a new RowIndex from a usize value
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Get the underlying usize value
    pub const fn get(self) -> usize {
        self.0
    }

    /// Add to the row index, saturating at usize::MAX
    pub fn saturating_add(self, rhs: usize) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    /// Subtract from the row index, saturating at 0
    pub fn saturating_sub(self, rhs: usize) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    /// Convert to 1-based line number (for display)
    pub fn to_line_number(self) -> NonZeroUsize {
        NonZeroUsize::new(self.0 + 1).unwrap()
    }
}

impl From<usize> for RowIndex {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<RowIndex> for usize {
    fn from(index: RowIndex) -> Self {
        index.get()
    }
}

/// Newtype wrapper for column indices to prevent confusion with row indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColIndex(usize);

impl ColIndex {
    /// Create a new ColIndex from a usize value
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Get the underlying usize value
    pub const fn get(self) -> usize {
        self.0
    }

    /// Add to the column index, saturating at usize::MAX
    pub fn saturating_add(self, rhs: usize) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    /// Subtract from the column index, saturating at 0
    pub fn saturating_sub(self, rhs: usize) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    /// Convert to 1-based column number (for display)
    pub fn to_column_number(self) -> NonZeroUsize {
        NonZeroUsize::new(self.0 + 1).unwrap()
    }
}

impl From<usize> for ColIndex {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<ColIndex> for usize {
    fn from(index: ColIndex) -> Self {
        index.get()
    }
}

/// Position in the CSV table (row and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: RowIndex,
    pub col: ColIndex,
}

impl Position {
    /// Create a new position
    pub const fn new(row: RowIndex, col: ColIndex) -> Self {
        Self { row, col }
    }

    /// Create a position from raw usize values
    pub const fn from_raw(row: usize, col: usize) -> Self {
        Self {
            row: RowIndex::new(row),
            col: ColIndex::new(col),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_index_new() {
        let row = RowIndex::new(5);
        assert_eq!(row.get(), 5);
    }

    #[test]
    fn test_row_index_saturating_add() {
        let row = RowIndex::new(5);
        assert_eq!(row.saturating_add(3).get(), 8);

        let max_row = RowIndex::new(usize::MAX);
        assert_eq!(max_row.saturating_add(1).get(), usize::MAX);
    }

    #[test]
    fn test_row_index_saturating_sub() {
        let row = RowIndex::new(5);
        assert_eq!(row.saturating_sub(3).get(), 2);

        let zero_row = RowIndex::new(0);
        assert_eq!(zero_row.saturating_sub(1).get(), 0);
    }

    #[test]
    fn test_row_index_to_line_number() {
        let row = RowIndex::new(0);
        assert_eq!(row.to_line_number().get(), 1);

        let row = RowIndex::new(99);
        assert_eq!(row.to_line_number().get(), 100);
    }

    #[test]
    fn test_row_index_from_usize() {
        let row: RowIndex = 10.into();
        assert_eq!(row.get(), 10);
    }

    #[test]
    fn test_col_index_new() {
        let col = ColIndex::new(3);
        assert_eq!(col.get(), 3);
    }

    #[test]
    fn test_col_index_saturating_add() {
        let col = ColIndex::new(2);
        assert_eq!(col.saturating_add(5).get(), 7);

        let max_col = ColIndex::new(usize::MAX);
        assert_eq!(max_col.saturating_add(1).get(), usize::MAX);
    }

    #[test]
    fn test_col_index_saturating_sub() {
        let col = ColIndex::new(7);
        assert_eq!(col.saturating_sub(4).get(), 3);

        let zero_col = ColIndex::new(0);
        assert_eq!(zero_col.saturating_sub(1).get(), 0);
    }

    #[test]
    fn test_col_index_from_usize() {
        let col: ColIndex = 5.into();
        assert_eq!(col.get(), 5);
    }

    #[test]
    fn test_position_new() {
        let pos = Position::new(RowIndex::new(10), ColIndex::new(5));
        assert_eq!(pos.row.get(), 10);
        assert_eq!(pos.col.get(), 5);
    }

    #[test]
    fn test_position_from_raw() {
        let pos = Position::from_raw(10, 5);
        assert_eq!(pos.row.get(), 10);
        assert_eq!(pos.col.get(), 5);
    }

    // Type safety test - this should not compile if we try to mix row and col
    #[test]
    fn test_type_safety() {
        let row = RowIndex::new(5);
        let col = ColIndex::new(10);

        // These are different types and can't be compared directly
        // This test just verifies they can be created separately
        assert_eq!(row.get(), 5);
        assert_eq!(col.get(), 10);
    }

    // ==========================================
    // Type Safety Verification (Compile-Time)
    // ==========================================
    //
    // The following code demonstrates type safety at compile time.
    // These examples will NOT compile, which is the desired behavior:
    //
    // Example 1: Cannot pass ColIndex where RowIndex is expected
    // ```compile_fail
    // let col = ColIndex::new(5);
    // let pos = Position::new(col, col); // ERROR: expected RowIndex, found ColIndex
    // ```
    //
    // Example 2: Cannot pass RowIndex where ColIndex is expected
    // ```compile_fail
    // let row = RowIndex::new(10);
    // let pos = Position::new(row, row); // ERROR: expected ColIndex, found RowIndex
    // ```
    //
    // Example 3: Cannot accidentally use wrong index in get_cell()
    // ```compile_fail
    // let row = RowIndex::new(5);
    // let col = ColIndex::new(10);
    // let cell = document.get_cell(col, row); // ERROR: arguments in wrong order
    // ```
    //
    // These compile-time checks prevent an entire class of bugs where
    // row and column indices could be accidentally swapped. Before the
    // introduction of type-safe indices in Phase 1, this was a common
    // source of subtle bugs that only manifested at runtime.
}
