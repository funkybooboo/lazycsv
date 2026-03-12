//! Integration tests for type-safe position types with CSV documents.
//!
//! These tests verify that RowIndex, ColIndex, and Position types compose
//! correctly with the csv::Document type and handle realistic navigation scenarios.

use lazycsv::csv::Document;
use lazycsv::domain::position::{ColIndex, Position, RowIndex};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test CSV file
fn create_test_csv(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.csv");
    fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

#[test]
fn test_row_index_with_document_navigation() {
    let content = "Name,Age,City\nAlice,30,NYC\nBob,25,LA\nCarol,35,SF\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Test row index navigation within document bounds
    let first_data_row = RowIndex::new(1); // Row 1 = first data row (after header)
    let last_data_row = RowIndex::new(3); // Row 3 = last data row

    assert!(first_data_row.get() < doc.row_count());
    assert!(last_data_row.get() < doc.row_count());

    // Test moving forward
    let next = first_data_row.saturating_add(1);
    assert_eq!(next.get(), 2);
    assert!(next.get() < doc.row_count());

    // Test moving backward
    let prev = last_data_row.saturating_sub(1);
    assert_eq!(prev.get(), 2);
}

#[test]
fn test_col_index_with_document_navigation() {
    let content = "A,B,C,D,E\n1,2,3,4,5\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Test column index navigation within document bounds
    let first_col = ColIndex::new(0); // Column A
    let last_col = ColIndex::new(4); // Column E

    assert!(first_col.get() < doc.column_count());
    assert!(last_col.get() < doc.column_count());

    // Test moving right
    let next = first_col.saturating_add(1);
    assert_eq!(next.get(), 1); // Column B
    assert!(next.get() < doc.column_count());

    // Test moving left
    let prev = last_col.saturating_sub(1);
    assert_eq!(prev.get(), 3); // Column D
}

#[test]
fn test_position_with_document_cell_access() {
    let content = "Name,Age,City\nAlice,30,NYC\nBob,25,LA\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Create positions for specific cells
    let header_pos = Position::from_raw(0, 0); // "Name" header
    let alice_name = Position::from_raw(1, 0); // "Alice"
    let alice_age = Position::from_raw(1, 1); // "30"
    let bob_city = Position::from_raw(2, 2); // "LA"

    // Verify positions are within bounds
    assert!(header_pos.row.get() < doc.row_count());
    assert!(header_pos.col.get() < doc.column_count());

    assert!(alice_name.row.get() < doc.row_count());
    assert!(alice_name.col.get() < doc.column_count());

    assert!(bob_city.row.get() < doc.row_count());
    assert!(bob_city.col.get() < doc.column_count());

    // Access cells using type-safe positions
    let name_header = doc.cell(header_pos.row, header_pos.col);
    assert_eq!(name_header, "Name");

    let alice = doc.cell(alice_name.row, alice_name.col);
    assert_eq!(alice, "Alice");

    let age = doc.cell(alice_age.row, alice_age.col);
    assert_eq!(age, "30");

    let city = doc.cell(bob_city.row, bob_city.col);
    assert_eq!(city, "LA");
}

#[test]
fn test_type_conversion_with_document_boundary() {
    let content = "A,B,C\n1,2,3\n4,5,6\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Convert document dimensions to indices
    let max_row: RowIndex = (doc.row_count() - 1).into();
    let max_col: ColIndex = (doc.column_count() - 1).into();

    assert_eq!(max_row.get(), 2); // 3 rows total, max index = 2
    assert_eq!(max_col.get(), 2); // 3 cols total, max index = 2

    // Convert back to usize
    let row_count: usize = max_row.into();
    let col_count: usize = max_col.into();

    assert_eq!(row_count, 2);
    assert_eq!(col_count, 2);
}

#[test]
fn test_row_index_boundary_navigation_with_document() {
    let content = "A\n1\n2\n3\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Test navigation at document boundaries
    let first_row = RowIndex::new(0);
    let last_row = RowIndex::new(doc.row_count() - 1);

    // Try to go before first row (should saturate at 0)
    let before_first = first_row.saturating_sub(1);
    assert_eq!(before_first.get(), 0);

    // Try to go past last row (should saturate at last + 1, but not overflow)
    let past_last = last_row.saturating_add(1);
    assert_eq!(past_last.get(), doc.row_count());

    // Verify saturation doesn't cause out-of-bounds access
    assert!(before_first.get() < doc.row_count());
    // past_last is intentionally out of bounds for testing boundary behavior
}

#[test]
fn test_col_index_boundary_navigation_with_document() {
    let content = "A,B,C\n1,2,3\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Test navigation at document boundaries
    let first_col = ColIndex::new(0);
    let last_col = ColIndex::new(doc.column_count() - 1);

    // Try to go before first column (should saturate at 0)
    let before_first = first_col.saturating_sub(1);
    assert_eq!(before_first.get(), 0);

    // Try to go past last column (should saturate at last + 1)
    let past_last = last_col.saturating_add(1);
    assert_eq!(past_last.get(), doc.column_count());

    // Verify saturation doesn't cause out-of-bounds access
    assert!(before_first.get() < doc.column_count());
}

#[test]
fn test_position_equality_with_document_cells() {
    let content = "A,B\n1,2\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Create two positions pointing to same cell
    let pos1 = Position::from_raw(1, 0);
    let pos2 = Position::new(RowIndex::new(1), ColIndex::new(0));

    // Verify they're equal
    assert_eq!(pos1, pos2);

    // Verify they access the same cell
    let cell1 = doc.cell(pos1.row, pos1.col);
    let cell2 = doc.cell(pos2.row, pos2.col);
    assert_eq!(cell1, cell2);
    assert_eq!(cell1, "1");
}

#[test]
fn test_large_document_navigation() {
    // Create a large document with 1000 rows and 50 columns
    let mut content = String::new();

    // Headers
    for col in 0..50 {
        if col > 0 {
            content.push(',');
        }
        content.push_str(&format!("Col{}", col));
    }
    content.push('\n');

    // Data rows
    for row in 0..1000 {
        for col in 0..50 {
            if col > 0 {
                content.push(',');
            }
            content.push_str(&format!("R{}C{}", row, col));
        }
        content.push('\n');
    }

    let (_temp_dir, file_path) = create_test_csv(&content);
    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Test navigation in large document
    // Row 500 in document = header (row 0) + 500 data rows, so internal row 500
    let start = Position::from_raw(500, 25); // Middle of document
    assert!(start.row.get() < doc.row_count());
    assert!(start.col.get() < doc.column_count());

    // Jump forward by 100 rows
    let forward = start.row.saturating_add(100);
    assert_eq!(forward.get(), 600);
    assert!(forward.get() < doc.row_count());

    // Jump right by 20 columns
    let right = start.col.saturating_add(20);
    assert_eq!(right.get(), 45);
    assert!(right.get() < doc.column_count());

    // Access cell at new position
    // Row 600 = data row 599 (because row 0 is header)
    let new_pos = Position::new(forward, right);
    let cell = doc.cell(new_pos.row, new_pos.col);
    assert_eq!(cell, "R599C45");
}

#[test]
fn test_empty_document_with_header() {
    let content = "Name,Age,City\n"; // Header only, no data
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    // Document should have 1 row (header) and 3 columns
    assert_eq!(doc.row_count(), 1);
    assert_eq!(doc.column_count(), 3);

    // Header row position
    let header_pos = Position::from_raw(0, 0);
    assert!(header_pos.row.get() < doc.row_count());
    assert!(header_pos.col.get() < doc.column_count());

    // First data row would be out of bounds
    let first_data = RowIndex::new(1);
    assert!(first_data.get() >= doc.row_count());
}

#[test]
fn test_type_safety_prevents_row_col_confusion() {
    let content = "A,B,C\n1,2,3\n";
    let (_temp_dir, file_path) = create_test_csv(content);

    let doc = Document::from_file(&file_path, None, false, None).expect("Failed to load document");

    let row = RowIndex::new(1);
    let col = ColIndex::new(2);

    // Correct order: row, then col
    let cell = doc.cell(row, col);
    assert_eq!(cell, "3");

    // The following would not compile (type safety at work):
    // let cell = doc.cell(col, row); // ERROR: expected RowIndex, found ColIndex

    // This test verifies compile-time type safety
    // We can't test the negative case (wrong order) because it won't compile!
}

#[test]
fn test_position_display_numbering() {
    // Verify 0-indexed internal representation vs 1-indexed display
    let row = RowIndex::new(0);
    let col = ColIndex::new(0);

    // Internal: 0-indexed
    assert_eq!(row.get(), 0);
    assert_eq!(col.get(), 0);

    // Display: 1-indexed
    assert_eq!(row.to_line_number().get(), 1);
    assert_eq!(col.to_column_number().get(), 1);

    // Row 99 (internal) = Line 100 (display)
    let row_99 = RowIndex::new(99);
    assert_eq!(row_99.to_line_number().get(), 100);
}
