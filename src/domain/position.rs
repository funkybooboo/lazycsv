//! Type-safe position types for CSV table navigation.
//!
//! This module provides newtype wrappers for row and column indices to prevent
//! accidental mixing of row/column coordinates at compile time.
//!
//! # Why Newtypes?
//!
//! Before v0.2.0, LazyCSV used plain `usize` values for both rows and columns.
//! This led to subtle bugs where row and column coordinates could be accidentally
//! swapped at function boundaries. The compiler couldn't catch these errors because
//! both were the same type.
//!
//! By using distinct newtype wrappers ([`RowIndex`] and [`ColIndex`]), we get
//! compile-time guarantees that:
//! - Row indices can't be used where column indices are expected
//! - Column indices can't be used where row indices are expected
//! - Type errors are caught at compile time, not runtime
//!
//! # Design Decisions
//!
//! ## Saturation Arithmetic
//!
//! All arithmetic operations use saturation semantics rather than panicking or
//! wrapping. This makes the API safer for user-driven navigation where bounds
//! are checked separately:
//!
//! ```
//! use lazycsv::domain::position::RowIndex;
//!
//! let row = RowIndex::new(5);
//! let result = row.saturating_add(3); // 8
//! assert_eq!(result.get(), 8);
//!
//! let at_zero = RowIndex::new(0);
//! let still_zero = at_zero.saturating_sub(1); // Saturates at 0
//! assert_eq!(still_zero.get(), 0);
//!
//! let at_max = RowIndex::new(usize::MAX);
//! let still_max = at_max.saturating_add(1); // Saturates at MAX
//! assert_eq!(still_max.get(), usize::MAX);
//! ```
//!
//! ## 1-Based Display Numbering
//!
//! Internally, rows and columns are 0-indexed (like arrays). For display to users,
//! we provide conversion to 1-based numbering using [`NonZeroUsize`]:
//!
//! ```
//! use lazycsv::domain::position::RowIndex;
//!
//! let row = RowIndex::new(0); // Internal: row 0
//! let line = row.to_line_number(); // Display: line 1
//! assert_eq!(line.get(), 1);
//!
//! let row_99 = RowIndex::new(99); // Internal: row 99
//! let line_100 = row_99.to_line_number(); // Display: line 100
//! assert_eq!(line_100.get(), 100);
//! ```
//!
//! # Examples
//!
//! ## Creating and Using RowIndex
//!
//! ```
//! use lazycsv::domain::position::RowIndex;
//!
//! // Create from usize
//! let row = RowIndex::new(10);
//! assert_eq!(row.get(), 10);
//!
//! // Or use From/Into
//! let row: RowIndex = 15.into();
//! assert_eq!(row.get(), 15);
//!
//! // Navigate with saturation
//! let next = row.saturating_add(1);
//! let prev = row.saturating_sub(1);
//! assert_eq!(next.get(), 16);
//! assert_eq!(prev.get(), 14);
//! ```
//!
//! ## Creating and Using ColIndex
//!
//! ```
//! use lazycsv::domain::position::ColIndex;
//!
//! // Create column index
//! let col = ColIndex::new(5);
//! assert_eq!(col.get(), 5);
//!
//! // Navigate columns
//! let next_col = col.saturating_add(1);
//! assert_eq!(next_col.get(), 6);
//! ```
//!
//! ## Using Position
//!
//! ```
//! use lazycsv::domain::position::{Position, RowIndex, ColIndex};
//!
//! // Create position from indices
//! let row = RowIndex::new(10);
//! let col = ColIndex::new(5);
//! let pos = Position::new(row, col);
//!
//! assert_eq!(pos.row.get(), 10);
//! assert_eq!(pos.col.get(), 5);
//!
//! // Or create from raw values
//! let pos2 = Position::from_raw(10, 5);
//! assert_eq!(pos, pos2);
//! ```
//!
//! ## Type Safety in Action
//!
//! ```compile_fail
//! use lazycsv::domain::position::{Position, RowIndex, ColIndex};
//!
//! let row = RowIndex::new(10);
//! let col = ColIndex::new(5);
//!
//! // This won't compile - arguments in wrong order!
//! let pos = Position::new(col, row);
//! // Error: expected RowIndex, found ColIndex
//! ```

use std::num::NonZeroUsize;

/// Newtype wrapper for row indices to prevent confusion with column indices.
///
/// # Examples
///
/// ```
/// use lazycsv::domain::position::RowIndex;
///
/// // Create a row index
/// let row = RowIndex::new(5);
/// assert_eq!(row.get(), 5);
///
/// // Arithmetic with saturation
/// let next = row.saturating_add(1);
/// let prev = row.saturating_sub(1);
/// assert_eq!(next.get(), 6);
/// assert_eq!(prev.get(), 4);
///
/// // Convert to 1-based line number for display
/// let line_number = row.to_line_number();
/// assert_eq!(line_number.get(), 6); // Row 5 is line 6 (1-indexed)
/// ```
///
/// # Type Safety
///
/// RowIndex cannot be used where ColIndex is expected:
///
/// ```compile_fail
/// use lazycsv::domain::position::{RowIndex, ColIndex, Position};
///
/// let row = RowIndex::new(5);
/// let pos = Position::new(row, row); // Error: expected ColIndex, found RowIndex
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowIndex(usize);

impl RowIndex {
    /// Create a new RowIndex from a usize value.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::domain::position::RowIndex;
    ///
    /// let row = RowIndex::new(10);
    /// assert_eq!(row.get(), 10);
    /// ```
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Get the underlying usize value
    pub const fn get(self) -> usize {
        self.0
    }

    /// Add to the row index, saturating at usize::MAX.
    ///
    /// Returns a new RowIndex with the result. If the addition would overflow,
    /// returns RowIndex(usize::MAX) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::domain::position::RowIndex;
    ///
    /// let row = RowIndex::new(5);
    /// let next = row.saturating_add(3);
    /// assert_eq!(next.get(), 8);
    ///
    /// // Saturation at MAX
    /// let at_max = RowIndex::new(usize::MAX);
    /// let still_max = at_max.saturating_add(1);
    /// assert_eq!(still_max.get(), usize::MAX);
    /// ```
    pub fn saturating_add(self, rhs: usize) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    /// Subtract from the row index, saturating at 0.
    ///
    /// Returns a new RowIndex with the result. If the subtraction would underflow,
    /// returns RowIndex(0) instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::domain::position::RowIndex;
    ///
    /// let row = RowIndex::new(10);
    /// let prev = row.saturating_sub(3);
    /// assert_eq!(prev.get(), 7);
    ///
    /// // Saturation at 0
    /// let at_zero = RowIndex::new(0);
    /// let still_zero = at_zero.saturating_sub(1);
    /// assert_eq!(still_zero.get(), 0);
    /// ```
    pub fn saturating_sub(self, rhs: usize) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    /// Convert to 1-based line number (for display to users).
    ///
    /// Internally, rows are 0-indexed (like arrays). For display purposes,
    /// we convert to 1-based line numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::domain::position::RowIndex;
    ///
    /// let row = RowIndex::new(0); // First row internally
    /// assert_eq!(row.to_line_number().get(), 1); // Line 1 for display
    ///
    /// let row = RowIndex::new(99); // Row 99 internally
    /// assert_eq!(row.to_line_number().get(), 100); // Line 100 for display
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the row index is `usize::MAX`, as adding 1 would overflow.
    /// This is acceptable as no CSV file can realistically have `usize::MAX` rows.
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

/// Newtype wrapper for column indices to prevent confusion with row indices.
///
/// # Examples
///
/// ```
/// use lazycsv::domain::position::ColIndex;
///
/// // Create a column index
/// let col = ColIndex::new(3);
/// assert_eq!(col.get(), 3);
///
/// // Arithmetic with saturation
/// let next = col.saturating_add(1);
/// let prev = col.saturating_sub(1);
/// assert_eq!(next.get(), 4);
/// assert_eq!(prev.get(), 2);
///
/// // Convert to 1-based column number for display
/// let col_number = col.to_column_number();
/// assert_eq!(col_number.get(), 4); // Column 3 is column 4 (1-indexed)
/// ```
///
/// # Type Safety
///
/// ColIndex cannot be used where RowIndex is expected:
///
/// ```compile_fail
/// use lazycsv::domain::position::{RowIndex, ColIndex, Position};
///
/// let col = ColIndex::new(5);
/// let pos = Position::new(col, col); // Error: expected RowIndex, found ColIndex
/// ```
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

    /// Convert to 1-based column number (for display to users).
    ///
    /// Internally, columns are 0-indexed (like arrays). For display purposes,
    /// we convert to 1-based column numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::domain::position::ColIndex;
    ///
    /// let col = ColIndex::new(0); // First column internally (column A)
    /// assert_eq!(col.to_column_number().get(), 1); // Column 1 for display
    ///
    /// let col = ColIndex::new(25); // Column 25 internally (column Z)
    /// assert_eq!(col.to_column_number().get(), 26); // Column 26 for display
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the column index is `usize::MAX`, as adding 1 would overflow.
    /// This is acceptable as no CSV file can realistically have `usize::MAX` columns.
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

/// Position in the CSV table (row and column).
///
/// Represents a specific cell location using type-safe [`RowIndex`] and [`ColIndex`].
///
/// # Examples
///
/// ```
/// use lazycsv::domain::position::{Position, RowIndex, ColIndex};
///
/// // Create from type-safe indices
/// let row = RowIndex::new(10);
/// let col = ColIndex::new(5);
/// let pos = Position::new(row, col);
///
/// assert_eq!(pos.row.get(), 10);
/// assert_eq!(pos.col.get(), 5);
///
/// // Create from raw usize values
/// let pos2 = Position::from_raw(10, 5);
/// assert_eq!(pos, pos2);
/// ```
///
/// # Type Safety
///
/// Position enforces correct argument order at compile time:
///
/// ```compile_fail
/// use lazycsv::domain::position::{Position, RowIndex, ColIndex};
///
/// let row = RowIndex::new(10);
/// let col = ColIndex::new(5);
///
/// // This won't compile - arguments in wrong order!
/// let pos = Position::new(col, row);
/// // Error: expected RowIndex, found ColIndex
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: RowIndex,
    pub col: ColIndex,
}

impl Position {
    /// Create a new position from type-safe indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::domain::position::{Position, RowIndex, ColIndex};
    ///
    /// let row = RowIndex::new(5);
    /// let col = ColIndex::new(10);
    /// let pos = Position::new(row, col);
    ///
    /// assert_eq!(pos.row.get(), 5);
    /// assert_eq!(pos.col.get(), 10);
    /// ```
    pub const fn new(row: RowIndex, col: ColIndex) -> Self {
        Self { row, col }
    }

    /// Create a position from raw usize values.
    ///
    /// Convenience method for creating a position without explicitly
    /// constructing RowIndex and ColIndex.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazycsv::domain::position::Position;
    ///
    /// let pos = Position::from_raw(5, 10);
    /// assert_eq!(pos.row.get(), 5);
    /// assert_eq!(pos.col.get(), 10);
    /// ```
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
