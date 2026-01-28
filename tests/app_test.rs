use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, CsvData};
use std::path::PathBuf;

fn create_test_csv_data() -> CsvData {
    CsvData {
        headers: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        rows: vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ],
        filename: "test.csv".to_string(),
        is_dirty: false,
    }
}

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_app_initialization() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let app = App::new(csv_data, csv_files, 0);

    assert_eq!(app.selected_row(), Some(0));
    assert_eq!(app.selected_col, 0);
    assert!(!app.should_quit);
    assert!(!app.show_cheatsheet);
}

#[test]
fn test_navigation_down() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(1));

    app.handle_key(key_event(KeyCode::Down)).unwrap();
    assert_eq!(app.selected_row(), Some(2));

    // Try to go beyond last row - should stay at last row
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(2));
}

#[test]
fn test_navigation_up() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    app.table_state.select(Some(2));

    app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
    assert_eq!(app.selected_row(), Some(1));

    app.handle_key(key_event(KeyCode::Up)).unwrap();
    assert_eq!(app.selected_row(), Some(0));

    // Try to go before first row - should stay at first row
    app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
    assert_eq!(app.selected_row(), Some(0));
}

#[test]
fn test_navigation_left_right() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    assert_eq!(app.selected_col, 0);

    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.selected_col, 1);

    app.handle_key(key_event(KeyCode::Right)).unwrap();
    assert_eq!(app.selected_col, 2);

    // Try to go beyond last column
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.selected_col, 2);

    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.selected_col, 1);

    app.handle_key(key_event(KeyCode::Left)).unwrap();
    assert_eq!(app.selected_col, 0);

    // Try to go before first column
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.selected_col, 0);
}

#[test]
fn test_navigation_home_end() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    app.table_state.select(Some(1));

    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    assert_eq!(app.selected_row(), Some(2)); // Last row

    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(app.selected_row(), Some(0)); // First row
}

#[test]
fn test_navigation_first_last_column() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    app.selected_col = 1;

    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.selected_col, 2); // Last column

    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
    assert_eq!(app.selected_col, 0); // First column
}

#[test]
fn test_vim_word_navigation() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    app.handle_key(key_event(KeyCode::Char('w'))).unwrap();
    assert_eq!(app.selected_col, 1);

    app.handle_key(key_event(KeyCode::Char('w'))).unwrap();
    assert_eq!(app.selected_col, 2);

    app.handle_key(key_event(KeyCode::Char('b'))).unwrap();
    assert_eq!(app.selected_col, 1);

    app.handle_key(key_event(KeyCode::Char('b'))).unwrap();
    assert_eq!(app.selected_col, 0);
}

#[test]
fn test_quit_functionality() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    assert!(!app.should_quit);

    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_quit_with_unsaved_changes() {
    let mut csv_data = create_test_csv_data();
    csv_data.is_dirty = true;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    assert!(!app.should_quit);

    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(!app.should_quit); // Should not quit
    assert!(app.status_message.is_some()); // Should show warning
}

#[test]
fn test_help_toggle() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    assert!(!app.show_cheatsheet);

    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.show_cheatsheet);

    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(!app.show_cheatsheet);
}

#[test]
fn test_help_close_with_esc() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    app.show_cheatsheet = true;

    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(!app.show_cheatsheet);
}

#[test]
fn test_file_switching_next() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 0);

    assert_eq!(app.current_file_index, 0);

    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert!(should_reload);
    assert_eq!(app.current_file_index, 1);

    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert!(should_reload);
    assert_eq!(app.current_file_index, 2);

    // Wrap around to first file
    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert!(should_reload);
    assert_eq!(app.current_file_index, 0);
}

#[test]
fn test_file_switching_previous() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 0);

    assert_eq!(app.current_file_index, 0);

    let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    assert!(should_reload);
    assert_eq!(app.current_file_index, 2); // Wrap to last file

    let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    assert!(should_reload);
    assert_eq!(app.current_file_index, 1);
}

#[test]
fn test_no_file_switching_with_single_file() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("file1.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert!(!should_reload); // Should not reload with single file
}

#[test]
fn test_navigation_blocked_when_help_shown() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    app.show_cheatsheet = true;
    let initial_row = app.selected_row();
    let initial_col = app.selected_col;

    // Try navigation with help shown
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), initial_row);

    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.selected_col, initial_col);

    // File switching should also be blocked
    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert!(!should_reload);
}

#[test]
fn test_current_file_path() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv"), PathBuf::from("other.csv")];
    let app = App::new(csv_data, csv_files.clone(), 0);

    assert_eq!(app.current_file(), &csv_files[0]);
}
