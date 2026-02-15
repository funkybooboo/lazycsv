use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::Mode;
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
fn test_files_command_enters_file_list_mode() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :files command
    send_command(&mut app, "files");

    // Should enter FileList mode
    assert_eq!(app.mode, Mode::FileList);

    // Cursor should start at position 0
    assert_eq!(app.view_state.file_list_selected, 0);
}

#[test]
fn test_file_list_mode_escape_cancels() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");
    assert_eq!(app.mode, Mode::FileList);

    // Press Escape
    let _ = app.handle_key(key_event(KeyCode::Esc));

    // Should return to Normal mode
    assert_eq!(app.mode, Mode::Normal);

    // Filter should be cleared
    assert_eq!(app.input_state.file_filter_buffer, "");
}

#[test]
fn test_file_list_cursor_navigation_down() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "file3.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");
    assert_eq!(app.view_state.file_list_selected, 0);

    // Press 'j' to move cursor down
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Press 'j' again
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 2);

    // Pressing 'j' at the end should not move beyond last file
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 2);
}

#[test]
fn test_file_list_cursor_navigation_up() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "file3.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");

    // Move to bottom first
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 2);

    // Press 'k' to move cursor up
    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Press 'k' again
    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.view_state.file_list_selected, 0);

    // Pressing 'k' at the top should not move beyond first file
    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.view_state.file_list_selected, 0);
}

#[test]
fn test_file_list_arrow_keys_navigation() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "file3.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");

    // Use Down arrow
    let _ = app.handle_key(key_event(KeyCode::Down));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Use Down arrow again
    let _ = app.handle_key(key_event(KeyCode::Down));
    assert_eq!(app.view_state.file_list_selected, 2);

    // Use Up arrow
    let _ = app.handle_key(key_event(KeyCode::Up));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Use Up arrow again
    let _ = app.handle_key(key_event(KeyCode::Up));
    assert_eq!(app.view_state.file_list_selected, 0);
}

#[test]
fn test_file_list_enter_selects_cursor_position() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "file3.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Currently on file 1 (index 0)
    assert_eq!(app.session.active_file_index(), 0);

    send_command(&mut app, "files");

    // Move cursor to file 2
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Press Enter to select
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // Should switch to file 2 (index 1) and return to Normal mode
    assert_eq!(app.session.active_file_index(), 1);
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_file_list_filter_by_name() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "customers.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "orders.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "products.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");
    assert_eq!(app.mode, Mode::FileList);

    // Type "ord" to filter
    let _ = app.handle_key(key_event(KeyCode::Char('o')));
    let _ = app.handle_key(key_event(KeyCode::Char('r')));
    let _ = app.handle_key(key_event(KeyCode::Char('d')));

    // Filter buffer should contain "ord"
    assert_eq!(app.input_state.file_filter_buffer, "ord");

    // Cursor should reset to 0 when filtering
    assert_eq!(app.view_state.file_list_selected, 0);

    // Press Enter to select first (and only) match
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // Should switch to orders.csv (index 1)
    assert_eq!(app.session.active_file_index(), 1);
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_file_list_filter_with_cursor_navigation() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "customers.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "customer_orders.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "products.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");

    // Type "cust" to filter - should match 2 files
    let _ = app.handle_key(key_event(KeyCode::Char('c')));
    let _ = app.handle_key(key_event(KeyCode::Char('u')));
    let _ = app.handle_key(key_event(KeyCode::Char('s')));
    let _ = app.handle_key(key_event(KeyCode::Char('t')));

    // Cursor should be at 0
    assert_eq!(app.view_state.file_list_selected, 0);

    // Press 'j' to move to second match
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Press Enter to select customer_orders.csv
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // Should switch to customer_orders.csv (index 1)
    assert_eq!(app.session.active_file_index(), 1);
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_file_list_filter_backspace() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");

    // Type some characters
    let _ = app.handle_key(key_event(KeyCode::Char('a')));
    let _ = app.handle_key(key_event(KeyCode::Char('b')));
    let _ = app.handle_key(key_event(KeyCode::Char('c')));
    assert_eq!(app.input_state.file_filter_buffer, "abc");

    // Backspace twice
    let _ = app.handle_key(key_event(KeyCode::Backspace));
    let _ = app.handle_key(key_event(KeyCode::Backspace));
    assert_eq!(app.input_state.file_filter_buffer, "a");

    // Cursor should reset to 0 after backspace
    assert_eq!(app.view_state.file_list_selected, 0);
}

#[test]
fn test_file_list_filter_no_match() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "customers.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "orders.csv", "X,Y\n3,4\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");

    // Type filter that doesn't match
    let _ = app.handle_key(key_event(KeyCode::Char('x')));
    let _ = app.handle_key(key_event(KeyCode::Char('y')));
    let _ = app.handle_key(key_event(KeyCode::Char('z')));

    // Press Enter
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // Should show error and stay in FileList mode
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("No matching"));
    assert_eq!(app.mode, Mode::FileList);
}

#[test]
fn test_file_list_single_file_navigation() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");

    // Cursor should be at 0
    assert_eq!(app.view_state.file_list_selected, 0);

    // Pressing j or k shouldn't crash with single file
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 0);

    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.view_state.file_list_selected, 0);

    // Press Enter to select
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // Should stay on file 1 and return to Normal mode
    assert_eq!(app.session.active_file_index(), 0);
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_file_list_cursor_resets_on_filter_change() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "apple.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "banana.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "cherry.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "files");

    // Move cursor to position 2
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 2);

    // Type a filter character - cursor should reset to 0
    let _ = app.handle_key(key_event(KeyCode::Char('b')));
    assert_eq!(app.view_state.file_list_selected, 0);
    assert_eq!(app.input_state.file_filter_buffer, "b");
}
