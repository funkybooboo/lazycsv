//! Tests for file persistence (v0.4.1)
//!
//! - :w - Save current file
//! - :W - Save all dirty files
//! - :wq - Save current and quit
//! - :Wq - Save all and quit
//! - :q - Quit (blocks if dirty)
//! - :q! - Force quit (discards changes)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::domain::position::{ColIndex, RowIndex};
use lazycsv::{App, Document, FileConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_test_csv(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_w_saves_current_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "test.csv", "A,B,C\n1,2,3\n");

    // Load file
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Make a change
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap(); // Enter insert mode
    app.handle_key(key_event(KeyCode::Char('X'))).unwrap(); // Type X
    app.handle_key(key_event(KeyCode::Enter)).unwrap(); // Commit edit

    assert!(app.document.is_dirty);

    // Save with :w
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('w'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // File should be saved
    assert!(!app.document.is_dirty);
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("X"));
}

#[test]
fn test_wq_saves_and_quits() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "test.csv", "A,B,C\n1,2,3\n");

    // Load file
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Make a change
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('Y'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert!(app.document.is_dirty);
    assert!(!app.should_quit);

    // Save and quit with :wq
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('w'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should have quit and saved
    assert!(app.should_quit);
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("Y"));
}

#[test]
fn test_q_blocks_if_dirty() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "test.csv", "A,B,C\n1,2,3\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Make a change
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('Z'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert!(app.document.is_dirty);

    // Try to quit with :q
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should NOT have quit
    assert!(!app.should_quit);
    assert!(app.status_message.is_some());
}

#[test]
fn test_q_succeeds_if_clean() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "test.csv", "A,B,C\n1,2,3\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    assert!(!app.document.is_dirty);

    // Quit with :q
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should have quit
    assert!(app.should_quit);
}

#[test]
fn test_q_bang_discards_changes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "test.csv", "A,B,C\n1,2,3\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Make a change
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('W'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert!(app.document.is_dirty);

    // Force quit with :q!
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('!'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should have quit
    assert!(app.should_quit);

    // File should NOT contain the change
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(!content.contains("W"));
}

#[test]
#[allow(non_snake_case)]
fn test_W_saves_all_dirty_files() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Edit file1 - use 's' to replace cell content
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('Q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Verify the edit worked
    assert!(app.document.is_dirty);
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Q"
    );

    // Cache current doc and mark as dirty (this is what file switching would do)
    app.session
        .cache_document(app.current_file().clone(), app.document.clone());
    app.session.mark_dirty(&file1);

    // Switch to file2
    app.session.next_file();
    app.document = Document::from_file(&file2, None, false, None).unwrap();
    // Reset view state after loading new doc
    app.view_state.table_state.select(Some(1));

    // Edit file2 - use 's' to replace cell content
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('R'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert!(app.document.is_dirty);

    // Save all with :W (capital W)
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('W'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Both files should be saved
    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();
    assert!(content1.contains("Q"), "file1 should contain Q");
    assert!(content2.contains("R"), "file2 should contain R");
}

#[test]
fn test_csv_writer_escapes_quotes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "test.csv", "Name\nAlice\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Edit to add quotes
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('"'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('H'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('"'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Save
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('w'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Check that quotes are escaped
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("\"\"Hi\"\""));
}

#[test]
fn test_csv_writer_escapes_commas() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_csv(&temp_dir, "test.csv", "Name\nAlice\n");

    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let files = vec![file_path.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Use 's' to substitute (replace cell), then type "Last, First"
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap(); // Substitute (clear and edit)
    app.handle_key(key_event(KeyCode::Char('L'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('a'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('t'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(','))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    app.handle_key(key_event(KeyCode::Char('F'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('i'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('r'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('s'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('t'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Save
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('w'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Check that cell is quoted (contains comma)
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(
        content.contains("\"Last, First\""),
        "Content was: {}",
        content
    );
}
