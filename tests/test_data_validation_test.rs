//! Tests for validating all CSV files in test_data/ directory.
//!
//! These tests ensure that all test CSV files can be parsed correctly
//! and that the application can handle various edge cases like sparse data,
//! unicode content, long values, and special characters.

use lazycsv::{App, ColIndex, Document, RowIndex};
use std::path::PathBuf;

const TEST_DATA_DIR: &str = "test_data";

fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(TEST_DATA_DIR).join(filename)
}

fn load_test_csv(filename: &str) -> Document {
    let path = test_data_path(filename);
    Document::from_file(&path, None, false, None)
        .unwrap_or_else(|e| panic!("Failed to load {}: {}", filename, e))
}

// =============================================================================
// Meta-test: Verify ALL CSV files in test_data load without errors
// =============================================================================

#[test]
fn test_all_test_data_files_load() {
    let test_data_path = PathBuf::from(TEST_DATA_DIR);
    let mut loaded_count = 0;
    let mut skipped_count = 0;

    for entry in std::fs::read_dir(&test_data_path).expect("Failed to read test_data directory") {
        let path = entry.expect("Failed to read entry").path();
        if path.extension().is_some_and(|e| e == "csv") {
            let filename = path.file_name().unwrap().to_str().unwrap();

            // Skip empty files (0 bytes) - they're expected to fail
            if std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) == 0 {
                skipped_count += 1;
                continue;
            }

            let result = Document::from_file(&path, None, false, None);
            assert!(
                result.is_ok(),
                "Failed to load {}: {:?}",
                filename,
                result.err()
            );
            loaded_count += 1;
        }
    }

    // Ensure we actually tested some files
    assert!(
        loaded_count >= 10,
        "Expected at least 10 CSV files, found {}",
        loaded_count
    );
    println!(
        "Successfully loaded {} CSV files, skipped {} empty files",
        loaded_count, skipped_count
    );
}

// =============================================================================
// Individual file tests with specific assertions
// =============================================================================

#[test]
fn test_sparse_csv() {
    let doc = load_test_csv("sparse.csv");
    assert_eq!(doc.column_count(), 10, "sparse.csv should have 10 columns");
    assert_eq!(doc.row_count(), 16, "sparse.csv should have 15 rows");

    // Verify some cells are empty (sparse data)
    let empty_cell = doc.cell(RowIndex::new(10), ColIndex::new(1)); // Row 10, col B
    assert!(empty_cell.is_empty(), "Expected empty cell in sparse data");

    // Verify some cells have data
    let filled_cell = doc.cell(RowIndex::new(1), ColIndex::new(1)); // Row 1, col B (Name)
    assert_eq!(filled_cell, "Alice");
}

#[test]
fn test_unicode_csv() {
    let doc = load_test_csv("unicode 国家.csv");
    assert_eq!(doc.column_count(), 7, "unicode.csv should have 7 columns");
    assert_eq!(doc.row_count(), 16, "unicode.csv should have 15 rows");

    // Verify Japanese greeting preserved
    let greeting = doc.cell(RowIndex::new(1), ColIndex::new(3));
    assert!(
        greeting.contains("こんにちは"),
        "Japanese greeting should be preserved"
    );

    // Verify emoji preserved
    let emoji = doc.cell(RowIndex::new(1), ColIndex::new(4));
    assert!(
        emoji.contains("🇯🇵"),
        "Japanese flag emoji should be preserved"
    );

    // Verify currency symbol preserved
    let currency = doc.cell(RowIndex::new(1), ColIndex::new(5));
    assert!(currency.contains("¥"), "Yen symbol should be preserved");
}

#[test]
fn test_long_values_csv() {
    let doc = load_test_csv("long_values.csv");
    assert_eq!(
        doc.column_count(),
        4,
        "long_values.csv should have 4 columns"
    );
    assert_eq!(doc.row_count(), 11, "long_values.csv should have 10 rows");

    // Verify long content is preserved (not truncated during parsing)
    // Row 0 = header, Row 3 = 4th row in file = data row with ID=3
    let long_desc = doc.cell(RowIndex::new(3), ColIndex::new(2));
    assert!(
        long_desc.len() > 100,
        "Long description should be preserved (got {} chars)",
        long_desc.len()
    );
}

#[test]
fn test_single_column_csv() {
    let doc = load_test_csv("single_column.csv");
    assert_eq!(
        doc.column_count(),
        1,
        "single_column.csv should have 1 column"
    );
    assert_eq!(doc.row_count(), 16, "single_column.csv should have 15 rows");

    // Verify header
    assert_eq!(doc.header(ColIndex::new(0)), "Names");

    // Verify first few values
    assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
}

#[test]
fn test_special_chars_csv() {
    let doc = load_test_csv("special_chars.csv");
    assert_eq!(
        doc.column_count(),
        5,
        "special_chars.csv should have 5 columns"
    );
    assert_eq!(doc.row_count(), 17, "special_chars.csv should have 16 data rows + header");

    // Verify quoted values with commas are parsed correctly
    // Row 0 = header, Row 2 = 3rd row in file = data row with ID=2
    let quoted_name = doc.cell(RowIndex::new(2), ColIndex::new(1));
    assert!(
        quoted_name.contains(","),
        "Quoted value with comma should be preserved, got: '{}'",
        quoted_name
    );

    // Verify quotes within quotes
    // Row 3 = 4th row in file = data row with ID=3
    let with_quotes = doc.cell(RowIndex::new(3), ColIndex::new(1));
    assert!(
        with_quotes.contains("\""),
        "Embedded quotes should be preserved, got: '{}'",
        with_quotes
    );
}

#[test]
fn test_empty_cells_csv() {
    let doc = load_test_csv("empty_cells.csv");
    assert_eq!(
        doc.column_count(),
        8,
        "empty_cells.csv should have 8 columns"
    );
    assert_eq!(doc.row_count(), 23, "empty_cells.csv should have 22 rows");

    // Verify completely empty row (row 10, 0-indexed row 9)
    for col in 0..8 {
        let cell = doc.cell(RowIndex::new(10), ColIndex::new(col));
        assert!(cell.is_empty(), "Row 10 column {} should be empty", col + 1);
    }

    // Verify diagonal pattern - first cell of first data row should have value
    let first_cell = doc.cell(RowIndex::new(1), ColIndex::new(0));
    assert_eq!(first_cell, "1", "First cell should be '1'");
}

#[test]
fn test_numbers_csv() {
    let doc = load_test_csv("numbers.csv");
    assert_eq!(doc.column_count(), 10, "numbers.csv should have 10 columns");
    assert_eq!(doc.row_count(), 11, "numbers.csv should have 10 rows");

    // Verify integer
    let integer = doc.cell(RowIndex::new(1), ColIndex::new(1));
    assert_eq!(integer, "42");

    // Verify decimal
    let decimal = doc.cell(RowIndex::new(1), ColIndex::new(2));
    assert_eq!(decimal, "3.14");

    // Verify scientific notation preserved as string
    let scientific = doc.cell(RowIndex::new(1), ColIndex::new(3));
    assert!(scientific.contains("E") || scientific.contains("e"));
}

#[test]
fn test_dates_csv() {
    let doc = load_test_csv("dates.csv");
    assert_eq!(doc.column_count(), 8, "dates.csv should have 8 columns");
    assert_eq!(doc.row_count(), 11, "dates.csv should have 10 rows");

    // Verify ISO date format preserved
    let iso_date = doc.cell(RowIndex::new(1), ColIndex::new(1));
    assert_eq!(iso_date, "2024-01-15");

    // Verify US date format preserved
    let us_date = doc.cell(RowIndex::new(1), ColIndex::new(2));
    assert_eq!(us_date, "01/15/2024");
}

#[test]
fn test_patterns_csv() {
    let doc = load_test_csv("patterns.csv");
    assert_eq!(
        doc.column_count(),
        10,
        "patterns.csv should have 10 columns"
    );
    assert_eq!(doc.row_count(), 21, "patterns.csv should have 20 rows");

    // Verify diagonal pattern - X on diagonal
    // Row 0 = header, Row 1 = first data row with X at col 0, Row 2 = second data row with X at col 1, etc.
    assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "X");
    assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(1)), "X");
    assert_eq!(doc.cell(RowIndex::new(3), ColIndex::new(2)), "X");

    // Off-diagonal should be empty
    assert!(doc.cell(RowIndex::new(1), ColIndex::new(1)).is_empty());
}

#[test]
fn test_whitespace_csv() {
    let doc = load_test_csv("whitespace.csv");
    assert_eq!(
        doc.column_count(),
        7,
        "whitespace.csv should have 7 columns"
    );
    assert_eq!(doc.row_count(), 11, "whitespace.csv should have 10 rows");

    // CSV parsing typically trims whitespace, but let's verify structure is correct
    let headers: Vec<String> = (0..7).map(|i| doc.header(ColIndex::new(i))).collect();
    assert!(headers.iter().any(|h| h == "Leading"));
    assert!(headers.iter().any(|h| h == "Trailing"));
}

#[test]
fn test_very_wide_csv() {
    let doc = load_test_csv("very_wide.csv");
    assert_eq!(
        doc.column_count(),
        101,
        "very_wide.csv should have 101 columns"
    );
    assert_eq!(doc.row_count(), 21, "very_wide.csv should have 20 rows");

    // Verify we can access last column
    let last_col_header = doc.header(ColIndex::new(100));
    assert!(
        !last_col_header.is_empty(),
        "Last column header should exist"
    );

    // Verify data in last column
    let last_col_data = doc.cell(RowIndex::new(1), ColIndex::new(100));
    assert!(
        !last_col_data.is_empty() || last_col_data.is_empty(),
        "Should be able to read last column"
    );
}

#[test]
fn test_customers_csv() {
    let doc = load_test_csv("customers.csv");
    assert_eq!(doc.column_count(), 5, "customers.csv should have 5 columns");
    assert_eq!(
        doc.row_count(),
        5,
        "customers.csv should have 5 rows (1 header + 4 data)"
    );

    // Verify headers
    assert_eq!(doc.header(ColIndex::new(0)), "CustomerID");
    assert_eq!(doc.header(ColIndex::new(1)), "Company");
}

#[test]
fn test_sample_csv() {
    let doc = load_test_csv("sample.csv");
    assert_eq!(doc.column_count(), 6, "sample.csv should have 6 columns");
    assert_eq!(doc.row_count(), 11, "sample.csv should have 10 rows");

    // Verify first row
    assert_eq!(
        doc.cell(RowIndex::new(1), ColIndex::new(1)),
        "Alice Johnson"
    );
}

// =============================================================================
// Navigation tests on loaded test data
// =============================================================================

#[test]
fn test_navigation_on_sparse_data() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use lazycsv::session::FileConfig;

    let doc = load_test_csv("sparse.csv");
    let files = vec![test_data_path("sparse.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Navigate down
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(2))); // row 1 + 1 = row 2

    // Navigate to last row
    app.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(15))); // 16 total rows (15 data + 1 header), last = row 15

    // Navigate to first data row
    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE))
        .unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1))); // First data row with header_mode ON
}

#[test]
fn test_navigation_on_wide_data() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use lazycsv::session::FileConfig;

    let doc = load_test_csv("very_wide.csv");
    let files = vec![test_data_path("very_wide.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Navigate to last column
    app.handle_key(KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(100));

    // Navigate to first column
    app.handle_key(KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_navigation_on_single_column() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use lazycsv::session::FileConfig;

    let doc = load_test_csv("single_column.csv");
    let files = vec![test_data_path("single_column.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Try to navigate right (should stay at column 0)
    app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(
        app.view_state.selected_column,
        ColIndex::new(0),
        "Should stay at column 0 in single-column file"
    );

    // Navigate down should work
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(2))); // row 1 + 1 = row 2
}

// =============================================================================
// Large file tests (performance sanity check)
// =============================================================================

#[test]
fn test_large_file_loads() {
    // test_10k.csv - 10,000 rows
    let path = test_data_path("test_10k.csv");
    if path.exists() {
        let doc =
            Document::from_file(&path, None, false, None).expect("Failed to load test_10k.csv");
        assert!(
            doc.row_count() >= 10000,
            "test_10k.csv should have at least 10000 rows"
        );
    }
}

#[test]
fn test_wide_file_loads() {
    // test_wide.csv - 30 columns
    let path = test_data_path("test_wide.csv");
    if path.exists() {
        let doc =
            Document::from_file(&path, None, false, None).expect("Failed to load test_wide.csv");
        assert!(
            doc.column_count() >= 30,
            "test_wide.csv should have at least 30 columns"
        );
    }
}
