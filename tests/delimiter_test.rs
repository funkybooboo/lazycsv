use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, ColIndex, Document, FileConfig, RowIndex};
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

/// Helper to create a test CSV file
fn create_test_csv(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    fs::write(&file_path, content).unwrap();
    (temp_dir, file_path)
}

#[test]
fn test_delim_command_changes_delimiter() {
    // Create CSV with semicolon delimiter
    let (_temp, file_path) = create_test_csv("Name;Age;City\nAlice;30;NYC\nBob;25;LA");

    // Load with comma delimiter (will parse incorrectly)
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Initially, it should parse as 1 column (no commas found)
    assert_eq!(app.document.column_count(), 1);
    assert_eq!(
        app.document.cell(RowIndex::new(0), ColIndex::new(0)),
        "Name;Age;City"
    );

    // Change delimiter to semicolon
    send_command(&mut app, "delim ;");

    // Delimiter should be updated AND file reloaded
    assert_eq!(app.document.delimiter, ';');

    // Now should have 3 columns parsed correctly
    assert_eq!(app.document.column_count(), 3);
    assert_eq!(
        app.document.cell(RowIndex::new(0), ColIndex::new(0)),
        "Name"
    );
    assert_eq!(app.document.cell(RowIndex::new(0), ColIndex::new(1)), "Age");
    assert_eq!(
        app.document.cell(RowIndex::new(0), ColIndex::new(2)),
        "City"
    );

    // All rows (header + data) should be parsed correctly
    assert_eq!(app.document.row_count(), 3);
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "30");
}

#[test]
fn test_delim_command_invalid_multichar() {
    let (_temp, file_path) = create_test_csv("A,B,C\n1,2,3");
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "delim abc");

    // Should show error message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("single character") || msg.contains("Delimiter must"));
}

#[test]
fn test_delim_command_no_argument() {
    let (_temp, file_path) = create_test_csv("A,B,C\n1,2,3");
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "delim");

    // Should show usage message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Usage") || msg.contains("delim"));
}

#[test]
fn test_delim_persists_in_session() {
    let (_temp, file_path) = create_test_csv("A,B,C\n1,2,3");
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Set delimiter
    send_command(&mut app, "delim |");
    assert_eq!(app.document.delimiter, '|');

    // Delimiter should be tracked in session
    let current_file = app.current_file().clone();
    assert_eq!(app.session.delimiter(&current_file), '|');
}

#[test]
fn test_delim_per_file_tracking() {
    // Create two CSV files
    let (_temp1, file1) = create_test_csv("A,B,C\n1,2,3");
    let (_temp2, file2) = create_test_csv("X;Y;Z\n4;5;6");

    // Create app with multiple files
    let doc1 = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc1, files, 0, FileConfig::new());

    // Set delimiter for file1
    send_command(&mut app, "delim |");
    assert_eq!(app.document.delimiter, '|');

    // Switch to file2 (implementation pending)
    // Verify file1's delimiter is still tracked
    assert_eq!(app.session.delimiter(&file1), '|');
    assert_eq!(app.session.delimiter(&file2), ','); // Default
}

#[test]
fn test_delim_common_delimiters() {
    let (_temp, file_path) = create_test_csv("A,B,C\n1,2,3");
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Test semicolon
    send_command(&mut app, "delim ;");
    assert_eq!(app.document.delimiter, ';');

    // Test tab (need to figure out how to input this)
    // For now, skip tab test - will require special handling

    // Test pipe
    send_command(&mut app, "delim |");
    assert_eq!(app.document.delimiter, '|');

    // Test colon
    send_command(&mut app, "delim :");
    assert_eq!(app.document.delimiter, ':');
}
