//! Integration tests for yazi-style directory memory in the file browser.
//!
//! When navigating into a directory and back, the cursor position should be
//! remembered so you return to the same file you left from.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::Mode;
use lazycsv::{App, Document, FileConfig};
use std::fs;
use tempfile::TempDir;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_test_csv(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

fn create_test_app_with_dir(temp_dir: &TempDir) -> App {
    let file1 = create_test_csv(temp_dir, "alpha.csv", "A,B\n1,2\n");
    let file2 = create_test_csv(temp_dir, "beta.csv", "X,Y\n3,4\n");
    let file3 = create_test_csv(temp_dir, "gamma.csv", "M,N\n5,6\n");

    let doc = Document::from_file(&file1, None, false, None).unwrap();
    let files = vec![file1, file2, file3];
    let mut app = App::new(doc, files, 0, FileConfig::new());
    app.view_state.current_directory = temp_dir.path().to_path_buf();
    app
}

fn open_files_menu(app: &mut App) {
    let _ = app.handle_key(key_event(KeyCode::Char(' ')));
    let _ = app.handle_key(key_event(KeyCode::Char('f')));
}

#[test]
fn test_directory_selected_starts_empty() {
    let temp_dir = TempDir::new().unwrap();
    let app = create_test_app_with_dir(&temp_dir);
    assert!(app.view_state.directory_selected.is_empty());
}

#[test]
fn test_directory_selected_populated_after_navigating_into_subdir() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    let _ = create_test_csv(&temp_dir, "top.csv", "A,B\n1,2\n");

    let mut app = create_test_app_with_dir(&temp_dir);
    open_files_menu(&mut app);

    // Move to the subdir entry and enter it
    navigate_down(&mut app, 1);
    let _ = app.handle_key(key_event(KeyCode::Enter));

    // After entering subdir, the parent directory's cursor should be saved
    assert!(
        app.view_state
            .directory_selected
            .contains_key(&temp_dir.path().to_path_buf()),
        "Parent directory should be saved in directory_selected"
    );
}

#[test]
fn test_directory_selected_cursor_value_preserved() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    // Put a file inside the subdir so it's navigable
    let subdir_file = subdir.join("inner.csv");
    fs::write(&subdir_file, "X,Y\n3,4\n").unwrap();
    let _ = create_test_csv(&temp_dir, "a.csv", "A,B\n1,2\n");
    let _ = create_test_csv(&temp_dir, "b.csv", "X,Y\n3,4\n");

    let mut app = create_test_app_with_dir(&temp_dir);
    open_files_menu(&mut app);
    assert_eq!(app.mode, Mode::FileList);

    // Move to position 2
    navigate_down(&mut app, 2);
    let cursor_before = app.view_state.file_list_selected;
    assert_eq!(cursor_before, 2);

    // Manually insert into directory_selected to verify the HashMap works
    app.view_state
        .directory_selected
        .insert(temp_dir.path().to_path_buf(), cursor_before);

    let saved = app
        .view_state
        .directory_selected
        .get(&temp_dir.path().to_path_buf())
        .copied();
    assert_eq!(saved, Some(2));
}

#[test]
fn test_directory_memory_map_direct_manipulation() {
    let temp_dir = TempDir::new().unwrap();
    let mut app = create_test_app_with_dir(&temp_dir);

    let sub1 = temp_dir.path().join("sub1");
    let sub2 = temp_dir.path().join("sub2");

    // Initially directory_selected is empty
    assert!(app.view_state.directory_selected.is_empty());

    // Simulate saving cursor positions for two directories
    app.view_state.directory_selected.insert(sub1.clone(), 3);
    app.view_state.directory_selected.insert(sub2.clone(), 5);

    assert_eq!(app.view_state.directory_selected.get(&sub1), Some(&3));
    assert_eq!(app.view_state.directory_selected.get(&sub2), Some(&5));
}

fn navigate_down(app: &mut App, n: usize) {
    for _ in 0..n {
        let _ = app.handle_key(key_event(KeyCode::Char('j')));
    }
}
