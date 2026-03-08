//! Property-based tests for type-safe position types.
//!
//! These tests use proptest to verify mathematical properties and invariants
//! of RowIndex, ColIndex, and Position types across a wide range of inputs.

#[cfg(test)]
mod tests {
    use crate::domain::position::{ColIndex, Position, RowIndex};
    use proptest::prelude::*;

    // Strategy for generating valid usize values (avoiding MAX to prevent overflow issues)
    fn valid_usize() -> impl Strategy<Value = usize> {
        0..=(usize::MAX / 2)
    }

    // Strategy for generating small valid usize (for testing arithmetic without overflow)
    fn small_usize() -> impl Strategy<Value = usize> {
        0..10_000usize
    }

    // ==========================================
    // RowIndex Property Tests
    // ==========================================

    proptest! {
        /// RowIndex creation and retrieval is reversible
        #[test]
        fn row_index_new_get_reversible(value in valid_usize()) {
            let row = RowIndex::new(value);
            prop_assert_eq!(row.get(), value);
        }

        /// RowIndex From<usize> and Into<usize> are reversible
        #[test]
        fn row_index_from_into_reversible(value in valid_usize()) {
            let row: RowIndex = value.into();
            let back: usize = row.into();
            prop_assert_eq!(back, value);
        }

        /// saturating_add never overflows and respects saturation at usize::MAX
        #[test]
        fn row_index_saturating_add_never_overflows(a in valid_usize(), b in valid_usize()) {
            let row = RowIndex::new(a);
            let result = row.saturating_add(b);

            // Result should be saturation or exact sum
            let expected = a.saturating_add(b);
            prop_assert_eq!(result.get(), expected);
        }

        /// saturating_add is associative: (a + b) + c == a + (b + c)
        #[test]
        fn row_index_saturating_add_associative(a in small_usize(), b in small_usize(), c in small_usize()) {
            let row_a = RowIndex::new(a);
            let left = row_a.saturating_add(b).saturating_add(c);

            let row_a2 = RowIndex::new(a);
            let right = row_a2.saturating_add(b.saturating_add(c));

            prop_assert_eq!(left.get(), right.get());
        }

        /// saturating_add identity: a + 0 == a
        #[test]
        fn row_index_saturating_add_identity(a in valid_usize()) {
            let row = RowIndex::new(a);
            let result = row.saturating_add(0);
            prop_assert_eq!(result.get(), a);
        }

        /// saturating_sub never underflows and respects saturation at 0
        #[test]
        fn row_index_saturating_sub_never_underflows(a in valid_usize(), b in valid_usize()) {
            let row = RowIndex::new(a);
            let result = row.saturating_sub(b);

            // Result should be saturation or exact difference
            let expected = a.saturating_sub(b);
            prop_assert_eq!(result.get(), expected);
        }

        /// saturating_sub identity: a - 0 == a
        #[test]
        fn row_index_saturating_sub_identity(a in valid_usize()) {
            let row = RowIndex::new(a);
            let result = row.saturating_sub(0);
            prop_assert_eq!(result.get(), a);
        }

        /// to_line_number always produces valid NonZeroUsize (never panics)
        #[test]
        fn row_index_to_line_number_never_panics(value in valid_usize()) {
            let row = RowIndex::new(value);
            let line_num = row.to_line_number();

            // Line number should be value + 1 (1-indexed)
            prop_assert_eq!(line_num.get(), value + 1);
        }

        /// RowIndex ordering is consistent with underlying usize
        #[test]
        fn row_index_ordering_consistent(a in valid_usize(), b in valid_usize()) {
            let row_a = RowIndex::new(a);
            let row_b = RowIndex::new(b);

            prop_assert_eq!(row_a < row_b, a < b);
            prop_assert_eq!(row_a <= row_b, a <= b);
            prop_assert_eq!(row_a > row_b, a > b);
            prop_assert_eq!(row_a >= row_b, a >= b);
            prop_assert_eq!(row_a == row_b, a == b);
        }
    }

    // ==========================================
    // ColIndex Property Tests
    // ==========================================

    proptest! {
        /// ColIndex creation and retrieval is reversible
        #[test]
        fn col_index_new_get_reversible(value in valid_usize()) {
            let col = ColIndex::new(value);
            prop_assert_eq!(col.get(), value);
        }

        /// ColIndex From<usize> and Into<usize> are reversible
        #[test]
        fn col_index_from_into_reversible(value in valid_usize()) {
            let col: ColIndex = value.into();
            let back: usize = col.into();
            prop_assert_eq!(back, value);
        }

        /// saturating_add never overflows and respects saturation at usize::MAX
        #[test]
        fn col_index_saturating_add_never_overflows(a in valid_usize(), b in valid_usize()) {
            let col = ColIndex::new(a);
            let result = col.saturating_add(b);

            let expected = a.saturating_add(b);
            prop_assert_eq!(result.get(), expected);
        }

        /// saturating_add is associative: (a + b) + c == a + (b + c)
        #[test]
        fn col_index_saturating_add_associative(a in small_usize(), b in small_usize(), c in small_usize()) {
            let col_a = ColIndex::new(a);
            let left = col_a.saturating_add(b).saturating_add(c);

            let col_a2 = ColIndex::new(a);
            let right = col_a2.saturating_add(b.saturating_add(c));

            prop_assert_eq!(left.get(), right.get());
        }

        /// saturating_add identity: a + 0 == a
        #[test]
        fn col_index_saturating_add_identity(a in valid_usize()) {
            let col = ColIndex::new(a);
            let result = col.saturating_add(0);
            prop_assert_eq!(result.get(), a);
        }

        /// saturating_sub never underflows and respects saturation at 0
        #[test]
        fn col_index_saturating_sub_never_underflows(a in valid_usize(), b in valid_usize()) {
            let col = ColIndex::new(a);
            let result = col.saturating_sub(b);

            let expected = a.saturating_sub(b);
            prop_assert_eq!(result.get(), expected);
        }

        /// saturating_sub identity: a - 0 == a
        #[test]
        fn col_index_saturating_sub_identity(a in valid_usize()) {
            let col = ColIndex::new(a);
            let result = col.saturating_sub(0);
            prop_assert_eq!(result.get(), a);
        }

        /// to_column_number always produces valid NonZeroUsize (never panics)
        #[test]
        fn col_index_to_column_number_never_panics(value in valid_usize()) {
            let col = ColIndex::new(value);
            let col_num = col.to_column_number();

            // Column number should be value + 1 (1-indexed)
            prop_assert_eq!(col_num.get(), value + 1);
        }

        /// ColIndex ordering is consistent with underlying usize
        #[test]
        fn col_index_ordering_consistent(a in valid_usize(), b in valid_usize()) {
            let col_a = ColIndex::new(a);
            let col_b = ColIndex::new(b);

            prop_assert_eq!(col_a < col_b, a < b);
            prop_assert_eq!(col_a <= col_b, a <= b);
            prop_assert_eq!(col_a > col_b, a > b);
            prop_assert_eq!(col_a >= col_b, a >= b);
            prop_assert_eq!(col_a == col_b, a == b);
        }
    }

    // ==========================================
    // Position Property Tests
    // ==========================================

    proptest! {
        /// Position::new creates position with correct row and col
        #[test]
        fn position_new_stores_values_correctly(row in valid_usize(), col in valid_usize()) {
            let pos = Position::new(RowIndex::new(row), ColIndex::new(col));
            prop_assert_eq!(pos.row.get(), row);
            prop_assert_eq!(pos.col.get(), col);
        }

        /// Position::from_raw creates position with correct row and col
        #[test]
        fn position_from_raw_stores_values_correctly(row in valid_usize(), col in valid_usize()) {
            let pos = Position::from_raw(row, col);
            prop_assert_eq!(pos.row.get(), row);
            prop_assert_eq!(pos.col.get(), col);
        }

        /// Position equality is based on both row and col
        #[test]
        fn position_equality(row1 in valid_usize(), col1 in valid_usize(),
                           row2 in valid_usize(), col2 in valid_usize()) {
            let pos1 = Position::from_raw(row1, col1);
            let pos2 = Position::from_raw(row2, col2);

            let expected_equal = (row1 == row2) && (col1 == col2);
            prop_assert_eq!(pos1 == pos2, expected_equal);
        }
    }

    // ==========================================
    // Cross-Type Property Tests
    // ==========================================

    proptest! {
        /// RowIndex and ColIndex are distinct types (compile-time check via usage)
        #[test]
        fn row_and_col_are_distinct_types(row_val in valid_usize(), col_val in valid_usize()) {
            let row = RowIndex::new(row_val);
            let col = ColIndex::new(col_val);

            // Can create Position with row and col
            let _pos = Position::new(row, col);

            // This test primarily verifies compile-time type safety
            // At runtime, we just verify the values are stored correctly
            prop_assert_eq!(row.get(), row_val);
            prop_assert_eq!(col.get(), col_val);
        }

        /// Arithmetic operations on RowIndex don't affect ColIndex (independence)
        #[test]
        fn row_col_arithmetic_independence(row in small_usize(), col in small_usize(),
                                          add_val in small_usize()) {
            let pos = Position::from_raw(row, col);

            // Perform arithmetic on row
            let new_row = pos.row.saturating_add(add_val);

            // Original col should be unchanged
            prop_assert_eq!(pos.col.get(), col);
            prop_assert_eq!(new_row.get(), row.saturating_add(add_val));
        }
    }

    // ==========================================
    // Boundary Condition Tests
    // ==========================================

    #[test]
    fn row_index_at_max_value() {
        let row = RowIndex::new(usize::MAX);

        // Adding should saturate
        let result = row.saturating_add(1);
        assert_eq!(result.get(), usize::MAX);

        // Subtracting should work
        let result = row.saturating_sub(1);
        assert_eq!(result.get(), usize::MAX - 1);
    }

    #[test]
    fn row_index_at_zero() {
        let row = RowIndex::new(0);

        // Subtracting should saturate at 0
        let result = row.saturating_sub(1);
        assert_eq!(result.get(), 0);

        // Adding should work
        let result = row.saturating_add(1);
        assert_eq!(result.get(), 1);
    }

    #[test]
    fn col_index_at_max_value() {
        let col = ColIndex::new(usize::MAX);

        // Adding should saturate
        let result = col.saturating_add(1);
        assert_eq!(result.get(), usize::MAX);

        // Subtracting should work
        let result = col.saturating_sub(1);
        assert_eq!(result.get(), usize::MAX - 1);
    }

    #[test]
    fn col_index_at_zero() {
        let col = ColIndex::new(0);

        // Subtracting should saturate at 0
        let result = col.saturating_sub(1);
        assert_eq!(result.get(), 0);

        // Adding should work
        let result = col.saturating_add(1);
        assert_eq!(result.get(), 1);
    }

    #[test]
    fn line_number_never_zero() {
        // Even row 0 produces line number 1
        let row = RowIndex::new(0);
        let line = row.to_line_number();
        assert_eq!(line.get(), 1);

        // Row at reasonable max value works fine
        let large_row = RowIndex::new(usize::MAX - 1);
        let large_line = large_row.to_line_number();
        assert_eq!(large_line.get(), usize::MAX);

        // Note: RowIndex::new(usize::MAX).to_line_number() will panic
        // in debug mode due to overflow. This is acceptable as:
        // 1. usize::MAX rows is beyond any realistic CSV file size
        // 2. The panic documents a logic error (CSV shouldn't have usize::MAX rows)
        // 3. Production code should validate row counts before creating indices
    }

    #[test]
    fn column_number_never_zero() {
        // Even col 0 produces column number 1
        let col = ColIndex::new(0);
        let col_num = col.to_column_number();
        assert_eq!(col_num.get(), 1);
    }
}
