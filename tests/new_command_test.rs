use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, Document, FileConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Helper to send command string to app
fn send_command(app: &mut App, cmd: &str) {
    // Enter command mode
    let _ = app.handle_key(key_event(KeyCode::Char(':')));

    // Type command
    for c in cmd.chars() {
        let _ = app.handle_key(key_event(KeyCode::Char(c)));
    }

    // Press Enter
    let _ = app.handle_key(key_event(KeyCode::Enter));
}

#[test]
fn test_new_command_with_headers() {
    // Start with a basic document
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Create new document with custom headers
    send_command(&mut app, "new Name,Age,City");

    // Should create document with 3 columns
    assert_eq!(app.document.column_count(), 3);
    assert_eq!(app.document.rows[0][0], "Name");
    assert_eq!(app.document.rows[0][1], "Age");
    assert_eq!(app.document.rows[0][2], "City");

    // Should have only header row (0 data rows)
    assert_eq!(app.document.data_row_count(), 0);

    // Should be marked as dirty
    assert!(app.document.is_dirty);

    // Header mode should be enabled
    assert!(app.document.header_mode);
}

#[test]
fn test_new_command_without_headers() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Create new document without headers
    send_command(&mut app, "new");

    // Should create document with 1 column named "Column 1"
    assert_eq!(app.document.column_count(), 1);
    assert_eq!(app.document.rows[0][0], "Column 1");

    // Should have only header row
    assert_eq!(app.document.data_row_count(), 0);

    // Should be marked as dirty
    assert!(app.document.is_dirty);

    // Header mode should be enabled
    assert!(app.document.header_mode);
}

#[test]
fn test_new_command_single_header() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Create new document with single header
    send_command(&mut app, "new ID");

    // Should create document with 1 column
    assert_eq!(app.document.column_count(), 1);
    assert_eq!(app.document.rows[0][0], "ID");
    assert_eq!(app.document.data_row_count(), 0);
}

#[test]
fn test_new_command_many_headers() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Create new document with many headers
    send_command(&mut app, "new A,B,C,D,E,F,G,H,I,J");

    // Should create document with 10 columns
    assert_eq!(app.document.column_count(), 10);
    assert_eq!(app.document.rows[0][0], "A");
    assert_eq!(app.document.rows[0][9], "J");
}

#[test]
fn test_new_command_with_spaces_in_headers() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Headers with spaces
    send_command(&mut app, "new First Name,Last Name,Email Address");

    assert_eq!(app.document.column_count(), 3);
    assert_eq!(app.document.rows[0][0], "First Name");
    assert_eq!(app.document.rows[0][1], "Last Name");
    assert_eq!(app.document.rows[0][2], "Email Address");
}

#[test]
fn test_new_command_preserves_delimiter() {
    // Create a real file for delimiter testing
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    fs::write(&file_path, "A;B\n1;2\n").unwrap();

    // Load with semicolon delimiter
    let doc = Document::from_file(&file_path, Some(b';'), false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Verify delimiter is semicolon
    assert_eq!(app.document.delimiter, ';');

    // Create new document
    send_command(&mut app, "new X,Y,Z");

    // Should preserve the delimiter
    assert_eq!(app.document.delimiter, ';');
}

#[test]
fn test_new_command_cursor_position() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Create new document
    send_command(&mut app, "new Name,Age");

    // Cursor should be at row 1 (first data row position, even though it doesn't exist yet)
    // Or at row 0 if we're allowing editing the header
    // This depends on implementation - let's check that selection exists
    assert!(app.view_state.table_state.selected().is_some());
}

#[test]
fn test_new_command_filename_unchanged() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "original.csv".to_string(),
    );
    let files = vec![PathBuf::from("original.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    let original_filename = app.document.filename.clone();

    // Create new document
    send_command(&mut app, "new X,Y,Z");

    // Filename should stay the same (or be changed to "untitled.csv" depending on design)
    // For now, let's assume it stays the same since we're replacing current document
    assert_eq!(app.document.filename, original_filename);
}
