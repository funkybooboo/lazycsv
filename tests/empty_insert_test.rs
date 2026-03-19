/// Tests for inserting rows on empty/minimal documents
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::domain::position::{ColIndex, RowIndex};
use lazycsv::{App, Document, FileConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_test_csv(temp_dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let file_path = temp_dir.path().join(name);
    fs::write(&file_path, content).unwrap();
    file_path
}

#[test]
fn test_o_on_empty_0_column_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "empty.csv", "");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    assert_eq!(app.document.row_count(), 1);
    assert_eq!(app.document.column_count(), 0);

    // Try to press 'o' to insert a row
    // This should work, but will create a row with 0 columns
    app.handle_key(key_event(KeyCode::Char('o'))).unwrap();

    // Should have inserted a new row
    assert_eq!(
        app.document.row_count(),
        2,
        "o should insert a row even with 0 columns"
    );

    // Cursor should move to the new row
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(1)),
        "After o, cursor should be on new row"
    );
}

#[test]
fn test_o_on_header_only_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "headers.csv", "Name,Age,City\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    assert_eq!(app.document.row_count(), 1); // Only header
    assert_eq!(app.document.column_count(), 3);
    assert_eq!(app.selected_row(), Some(RowIndex::new(0))); // On row 0

    // Press 'o' to insert first data row
    app.handle_key(key_event(KeyCode::Char('o'))).unwrap();

    // Should have 2 rows now (header + 1 data)
    assert_eq!(app.document.row_count(), 2);

    // Cursor should be on row 1 (first data row)
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // New row should have correct number of columns (all empty)
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(2)), "");
}
