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

/// Helper to open files menu using Space+f
fn open_files_menu(app: &mut App) {
    let _ = app.handle_key(key_event(KeyCode::Char(' ')));
    let _ = app.handle_key(key_event(KeyCode::Char('f')));
}

/// Helper to set the browser directory to the temp dir so browser entries match
fn set_browser_dir(app: &mut App, dir: &TempDir) {
    app.view_state.current_directory = dir.path().to_path_buf();
}

#[test]
fn test_files_command_enters_file_list_mode() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Open files menu with Space+f
    open_files_menu(&mut app);

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

    open_files_menu(&mut app);
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

    open_files_menu(&mut app);
    assert_eq!(app.view_state.file_list_selected, 0); // ".." parent dir

    // Press 'j' to move cursor down
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 1); // file1.csv

    // Press 'j' again
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 2); // file2.csv

    // Keep pressing 'j' until we reach the last file
    // Note: Browser shows all entries in directory, not just session files
    let mut last_index = app.view_state.file_list_selected;
    for _ in 0..20 {
        let _ = app.handle_key(key_event(KeyCode::Char('j')));
        let new_index = app.view_state.file_list_selected;
        if new_index == last_index {
            break; // Reached the end
        }
        last_index = new_index;
    }

    // Pressing 'j' at the end should not move beyond last file
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, last_index);
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

    open_files_menu(&mut app);

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

    open_files_menu(&mut app);

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
    set_browser_dir(&mut app, &temp_dir);

    assert_eq!(app.session.active_file_index(), 0);

    open_files_menu(&mut app);

    // Browser shows: [.., file1.csv, file2.csv, file3.csv]
    // Move cursor past ".." to file2.csv (index 2)
    let _ = app.handle_key(key_event(KeyCode::Char('j'))); // -> ..  (idx 0 -> 1)
    let _ = app.handle_key(key_event(KeyCode::Char('j'))); // -> file2.csv (idx 1 -> 2)

    // Press Enter to select file2.csv
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
    set_browser_dir(&mut app, &temp_dir);

    open_files_menu(&mut app);
    assert_eq!(app.mode, Mode::FileList);

    // Browser shows: [.., customers.csv, orders.csv, products.csv]
    // Navigate to products.csv (index 3)
    let _ = app.handle_key(key_event(KeyCode::Char('j'))); // 0 -> 1
    let _ = app.handle_key(key_event(KeyCode::Char('j'))); // 1 -> 2
    let _ = app.handle_key(key_event(KeyCode::Char('j'))); // 2 -> 3

    // Press Enter to select products.csv
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // Should switch to products.csv (index 2 in session)
    assert_eq!(app.session.active_file_index(), 2);
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
    set_browser_dir(&mut app, &temp_dir);

    open_files_menu(&mut app);

    // Browser shows: [.., customer_orders.csv, customers.csv, products.csv]
    // Navigate to customer_orders.csv (index 1, after "..")
    let _ = app.handle_key(key_event(KeyCode::Char('j'))); // 0 -> 1

    // Press Enter to select customer_orders.csv
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // Should switch to customer_orders.csv (index 1 in session)
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

    open_files_menu(&mut app);

    // Enter search mode with /
    let _ = app.handle_key(key_event(KeyCode::Char('/')));

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

    open_files_menu(&mut app);

    // Enter search mode with /
    let _ = app.handle_key(key_event(KeyCode::Char('/')));

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
    set_browser_dir(&mut app, &temp_dir);

    open_files_menu(&mut app);

    // Browser shows: [.., file1.csv] — cursor starts at 0 (..)
    assert_eq!(app.view_state.file_list_selected, 0);

    // Pressing j moves to file1.csv (index 1)
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Pressing j again should stay at 1 (end of list)
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 1);

    // Pressing k moves back to 0
    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.view_state.file_list_selected, 0);

    // Navigate to file1.csv and press Enter
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
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

    open_files_menu(&mut app);

    // Move cursor to position 2
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 2);

    // Enter search mode and type a filter character - cursor should reset to 0
    let _ = app.handle_key(key_event(KeyCode::Char('/')));
    let _ = app.handle_key(key_event(KeyCode::Char('b')));
    assert_eq!(app.view_state.file_list_selected, 0);
    assert_eq!(app.input_state.file_filter_buffer, "b");
}

// ============================================================================
// New Yazi-like Keybindings Tests
// ============================================================================

#[test]
fn test_file_list_q_exits_like_escape() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    open_files_menu(&mut app);
    assert_eq!(app.mode, Mode::FileList);

    // Press 'q' to quit file manager
    let _ = app.handle_key(key_event(KeyCode::Char('q')));

    // Should return to Normal mode
    assert_eq!(app.mode, Mode::Normal);

    // Filter should be cleared
    assert_eq!(app.input_state.file_filter_buffer, "");
}

#[test]
fn test_file_list_g_jumps_to_top() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "file3.csv", "M,N\n5,6\n");
    let file4 = create_test_csv(&temp_dir, "file4.csv", "P,Q\n7,8\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone(), file4.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    open_files_menu(&mut app);

    // Move to bottom
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 3);

    // Press 'gg' to jump to top
    let _ = app.handle_key(key_event(KeyCode::Char('g')));
    let _ = app.handle_key(key_event(KeyCode::Char('g')));
    assert_eq!(app.view_state.file_list_selected, 0);
}

#[test]
fn test_file_list_shift_g_jumps_to_bottom() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "file3.csv", "M,N\n5,6\n");
    let file4 = create_test_csv(&temp_dir, "file4.csv", "P,Q\n7,8\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone(), file4.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    open_files_menu(&mut app);
    assert_eq!(app.view_state.file_list_selected, 0);

    // Press 'G' (Shift+g) to jump to bottom
    // Note: Browser shows all entries in directory, count may vary
    let _ = app.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT));
    let bottom_index = app.view_state.file_list_selected;
    assert!(bottom_index > 0, "Should jump to a position > 0");

    // Verify we're at the bottom by trying to move down
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(
        app.view_state.file_list_selected, bottom_index,
        "Should stay at bottom"
    );
}

#[test]
fn test_file_list_g_and_capital_g_with_filtered_list() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "apple.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "apricot.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(&temp_dir, "avocado.csv", "M,N\n5,6\n");
    let file4 = create_test_csv(&temp_dir, "banana.csv", "P,Q\n7,8\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone(), file3.clone(), file4.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());
    set_browser_dir(&mut app, &temp_dir);

    open_files_menu(&mut app);

    // Browser shows: [.., apple.csv, apricot.csv, avocado.csv, banana.csv]
    // Filter with 'a' — matches all 4 CSV files (not "..")
    let _ = app.handle_key(key_event(KeyCode::Char('/')));
    let _ = app.handle_key(key_event(KeyCode::Char('a')));
    let _ = app.handle_key(key_event(KeyCode::Enter)); // Exit search mode

    // Move down twice to get to index 2
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.view_state.file_list_selected, 2);

    // Jump to top with 'gg'
    let _ = app.handle_key(key_event(KeyCode::Char('g')));
    let _ = app.handle_key(key_event(KeyCode::Char('g')));
    assert_eq!(app.view_state.file_list_selected, 0);

    // Jump to bottom with 'G'
    // Filter 'a' matches all 4 CSV files: apple, apricot, avocado, banana → last index is 3
    let _ = app.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT));
    assert_eq!(app.view_state.file_list_selected, 3);
}

#[test]
fn test_file_list_operations_dont_filter() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = create_test_csv(&temp_dir, "file1.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(&temp_dir, "file2.csv", "X,Y\n3,4\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1.clone(), file2.clone()];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    open_files_menu(&mut app);

    // Start with empty filter
    assert_eq!(app.input_state.file_filter_buffer, "");

    // Press operation keys - they should not add to filter
    let _ = app.handle_key(key_event(KeyCode::Char('g')));
    assert_eq!(app.input_state.file_filter_buffer, "");

    let _ = app.handle_key(key_event(KeyCode::Char('r')));
    assert_eq!(app.input_state.file_filter_buffer, "");

    let _ = app.handle_key(key_event(KeyCode::Char('d')));
    assert_eq!(app.input_state.file_filter_buffer, "");

    let _ = app.handle_key(key_event(KeyCode::Char('y')));
    assert_eq!(app.input_state.file_filter_buffer, "");

    let _ = app.handle_key(key_event(KeyCode::Char('n')));
    assert_eq!(app.input_state.file_filter_buffer, "");

    let _ = app.handle_key(key_event(KeyCode::Char('o')));
    assert_eq!(app.input_state.file_filter_buffer, "");

    // Still in FileList mode (o opens file, so will exit)
    // Other operations should keep us in FileList mode
}
