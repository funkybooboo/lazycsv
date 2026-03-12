/// Tests for header navigation commands (gh and gd)
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::domain::position::RowIndex;
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

fn create_test_app_with_data() -> (App, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(
        &temp_dir,
        "test.csv",
        "Name,Age,City\nAlice,30,NYC\nBob,25,LA\nCharlie,35,SF\n",
    );

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let app = App::new(doc, files, 0, FileConfig::new());

    (app, temp_dir)
}

#[test]
fn test_gh_moves_to_header_row() {
    let (mut app, _temp) = create_test_app_with_data();

    // Start at row 2 (data row)
    app.view_state.table_state.select(Some(2));
    assert_eq!(app.selected_row(), Some(RowIndex::new(2)));

    // Press 'g' then 'h' to go to header
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();

    // Should be at row 0 (header)
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(0)),
        "gh should move to row 0 (header)"
    );

    // Status message should confirm
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("header"))
        .unwrap_or(false));
}

#[test]
fn test_gh_requires_header_mode_on() {
    let (mut app, _temp) = create_test_app_with_data();

    // Turn header mode OFF
    app.document.toggle_header_mode();
    assert!(!app.document.header_mode);

    // Start at row 2
    app.view_state.table_state.select(Some(2));

    // Try gh command
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();

    // Should stay at row 2
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(2)),
        "gh should not move when header_mode is OFF"
    );

    // Should show error message
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("OFF") || m.as_str().contains(":ht"))
        .unwrap_or(false));
}

#[test]
fn test_gh_on_empty_document() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "empty.csv", "");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Empty CSV still has 1 row (empty header), so gh should work
    // Try gh on empty document
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();

    // Should move to row 0 successfully
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("header"))
        .unwrap_or(false));
}

#[test]
fn test_gd_moves_to_first_data_row() {
    let (mut app, _temp) = create_test_app_with_data();

    // Start at row 3
    app.view_state.table_state.select(Some(3));
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

    // Press 'g' then 'd' to go to first data row
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    // Should be at row 1 (first data row)
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(1)),
        "gd should move to row 1 (first data row)"
    );

    // Status message should confirm
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("data"))
        .unwrap_or(false));
}

#[test]
fn test_gd_on_header_only_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "headers_only.csv", "Name,Age,City\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Document has only header row (row 0), no data rows
    assert_eq!(app.document.row_count(), 1);

    // Try gd command
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    // Should show error message
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("No data"))
        .unwrap_or(false));
}

#[test]
fn test_gh_then_insert_edits_header() {
    let (mut app, _temp) = create_test_app_with_data();

    // Start at row 2
    app.view_state.table_state.select(Some(2));

    // Go to header with gh
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Enter Insert mode
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    assert_eq!(app.mode, lazycsv::app::Mode::Insert);

    // Type some text
    app.handle_key(key_event(KeyCode::Char('F'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('u'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    // Commit with Enter
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Header should be updated
    assert_eq!(
        app.document
            .cell(RowIndex::new(0), app.view_state.selected_column),
        "NameFull"
    );
}

#[test]
fn test_gh_gd_round_trip() {
    let (mut app, _temp) = create_test_app_with_data();

    // Start at row 3
    app.view_state.table_state.select(Some(3));

    // Go to header
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Go to first data row
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // Go back to header
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_gh_preserves_column() {
    let (mut app, _temp) = create_test_app_with_data();

    // Move to row 2, column 2
    app.view_state.table_state.select(Some(2));
    app.view_state.selected_column = lazycsv::domain::position::ColIndex::new(2);

    let original_col = app.view_state.selected_column;

    // Go to header
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();

    // Column should be preserved
    assert_eq!(
        app.view_state.selected_column, original_col,
        "gh should preserve current column"
    );
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_gd_preserves_column() {
    let (mut app, _temp) = create_test_app_with_data();

    // Move to row 3, column 1
    app.view_state.table_state.select(Some(3));
    app.view_state.selected_column = lazycsv::domain::position::ColIndex::new(1);

    let original_col = app.view_state.selected_column;

    // Go to first data row
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();

    // Column should be preserved
    assert_eq!(
        app.view_state.selected_column, original_col,
        "gd should preserve current column"
    );
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
}

#[test]
fn test_gh_vs_gg_with_header_mode_on() {
    let (mut app, _temp) = create_test_app_with_data();

    // Ensure header mode is ON (default)
    assert!(app.document.header_mode);

    // Start at row 2
    app.view_state.table_state.select(Some(2));

    // gg should go to row 1 (first data row) when header_mode is ON
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(1)),
        "gg goes to row 1 when header_mode ON"
    );

    // Move back to row 2
    app.view_state.table_state.select(Some(2));

    // gh should go to row 0 (header row) regardless
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(0)),
        "gh always goes to row 0"
    );
}

#[test]
fn test_gh_vs_gg_with_header_mode_off() {
    let (mut app, _temp) = create_test_app_with_data();

    // Turn header mode OFF
    app.document.toggle_header_mode();
    assert!(!app.document.header_mode);

    // Start at row 2
    app.view_state.table_state.select(Some(2));

    // gg should go to row 0 when header_mode is OFF
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(0)),
        "gg goes to row 0 when header_mode OFF"
    );

    // Move back to row 2
    app.view_state.table_state.select(Some(2));

    // gh should NOT work and show error
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(2)),
        "gh should not move when header_mode OFF"
    );
}

#[test]
fn test_gd_works_regardless_of_header_mode() {
    let (mut app, _temp) = create_test_app_with_data();

    // Test with header mode ON
    assert!(app.document.header_mode);
    app.view_state.table_state.select(Some(3));
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // Turn header mode OFF
    app.document.toggle_header_mode();
    assert!(!app.document.header_mode);

    // Test gd still works
    app.view_state.table_state.select(Some(3));
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(RowIndex::new(1)),
        "gd should work regardless of header_mode"
    );
}
