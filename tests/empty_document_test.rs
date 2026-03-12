/// Tests for edge cases with empty documents
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
fn test_empty_file_0_bytes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "empty.csv", "");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();

    // Empty file should have:
    // - 1 row (empty header row)
    // - 0 columns
    println!(
        "Empty file: row_count={}, column_count={}",
        doc.row_count(),
        doc.column_count()
    );
    assert_eq!(
        doc.row_count(),
        1,
        "Empty file should have 1 row (empty header)"
    );
    assert_eq!(doc.column_count(), 0, "Empty file should have 0 columns");
}

#[test]
fn test_header_only_file_no_data() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "headers_only.csv", "Name,Age,City\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();

    println!(
        "Header-only file: row_count={}, column_count={}",
        doc.row_count(),
        doc.column_count()
    );
    assert_eq!(
        doc.row_count(),
        1,
        "Header-only file should have 1 row (header)"
    );
    assert_eq!(
        doc.column_count(),
        3,
        "Header-only file should have 3 columns"
    );
    assert_eq!(doc.cell(RowIndex::new(0), ColIndex::new(0)), "Name");
    assert_eq!(doc.cell(RowIndex::new(0), ColIndex::new(1)), "Age");
    assert_eq!(doc.cell(RowIndex::new(0), ColIndex::new(2)), "City");
}

#[test]
fn test_app_new_with_empty_document_0_cols() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "empty.csv", "");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let app = App::new(doc, files, 0, FileConfig::new());

    // With 0 columns, cursor should not be selectable
    println!("App with 0-col doc: selected_row={:?}", app.selected_row());

    // Currently this will select Some(0), but ideally should be None or handle gracefully
    // For now, just document the current behavior
    assert!(
        app.selected_row().is_some(),
        "Currently selects row 0 even with 0 columns"
    );
}

#[test]
fn test_app_new_with_header_only_document() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "headers.csv", "A,B,C\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let app = App::new(doc, files, 0, FileConfig::new());

    // With header mode ON and only 1 row (the header), cursor should be on row 0
    // because there's no data row to move to
    println!(
        "App with header-only doc: selected_row={:?}, header_mode={}",
        app.selected_row(),
        app.document.header_mode
    );

    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(0)),
        "With only header row, cursor should be on row 0"
    );
}

#[test]
fn test_single_row_single_column() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "single.csv", "Header\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    assert_eq!(doc.row_count(), 1);
    assert_eq!(doc.column_count(), 1);
    assert_eq!(doc.cell(RowIndex::new(0), ColIndex::new(0)), "Header");

    let files = vec![file_path.clone()];
    let app = App::new(doc, files, 0, FileConfig::new());
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_navigation_with_header_only_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "headers.csv", "A,B,C\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Try to move down - should stay on row 0 (no data rows)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(0)),
        "j on header-only file should stay on row 0"
    );

    // Try gg - should stay on row 0
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(0)),
        "gg on header-only file should go to row 0 (only row available)"
    );
}

#[test]
fn test_delete_last_data_row_moves_to_header() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "single_data.csv", "A,B\n1,2\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Start on row 1 (first data row)
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
    assert_eq!(app.document.row_count(), 2); // header + 1 data row

    // Delete the only data row with dd
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    // Should now have only header row
    assert_eq!(
        app.document.row_count(),
        1,
        "After deleting last data row, only header remains"
    );

    // Cursor should move to row 0 (header row)
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(0)),
        "After deleting last data row, cursor should move to header (row 0)"
    );
}
