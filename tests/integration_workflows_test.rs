use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, CsvData};
use std::path::PathBuf;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_test_csv() -> CsvData {
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

#[test]
fn test_complete_navigation_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // User workflow: Navigate to bottom-right, then back to top-left
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.selected_row(), Some(2));
    assert_eq!(app.ui.selected_col, 2);

    // gg - Go to first row (multi-key command)
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(app.selected_row(), Some(0));

    // 0 - Go to first column
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
    assert_eq!(app.ui.selected_col, 0);
}

#[test]
fn test_help_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Open help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.ui.show_cheatsheet);

    // Try to navigate (should be blocked)
    let initial_row = app.selected_row();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), initial_row);

    // Close help with Esc
    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(!app.ui.show_cheatsheet);

    // Navigation should work again
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_ne!(app.selected_row(), initial_row);
}

#[test]
fn test_quit_workflow_clean_state() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    assert!(!app.should_quit);

    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_quit_workflow_dirty_state() {
    let mut csv_data = create_test_csv();
    csv_data.is_dirty = true;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    assert!(!app.should_quit);

    // First quit attempt should warn
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(!app.should_quit);
    assert!(app.status_message.is_some());
}

#[test]
fn test_file_switching_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Start at file 0
    assert_eq!(app.current_file_index, 0);

    // Switch forward through all files
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

    // Switch backward
    let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    assert!(should_reload);
    assert_eq!(app.current_file_index, 2);
}

#[test]
fn test_help_and_quit_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Open help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.ui.show_cheatsheet);

    // Try to quit while help is open (should not work)
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(!app.should_quit); // q is blocked when help is shown

    // Close help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(!app.ui.show_cheatsheet);

    // Now quit should work
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_navigate_then_switch_file_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Navigate to a specific position
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.selected_row(), Some(2));
    assert_eq!(app.ui.selected_col, 2);

    // Switch file
    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert!(should_reload);
    assert_eq!(app.current_file_index, 1);
}

#[test]
fn test_rapid_key_sequence_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Simulate rapid user input
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    }

    // Should end at maximum position
    assert_eq!(app.selected_row(), Some(2));
    assert_eq!(app.ui.selected_col, 2);
}

#[test]
fn test_zigzag_navigation_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Zigzag pattern: down-right, down-right, down-right
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    assert_eq!(app.selected_row(), Some(2));
    assert_eq!(app.ui.selected_col, 2);
}

#[test]
fn test_help_toggle_multiple_times() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    for _ in 0..5 {
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.ui.show_cheatsheet);
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(!app.ui.show_cheatsheet);
    }
}

#[test]
fn test_boundary_navigation_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Try to go beyond boundaries multiple times
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    }
    assert_eq!(app.selected_row(), Some(0));
    assert_eq!(app.ui.selected_col, 0);

    // Go to opposite corner
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

    // Try to go beyond boundaries
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    }
    assert_eq!(app.selected_row(), Some(2));
    assert_eq!(app.ui.selected_col, 2);
}

#[test]
fn test_current_file_tracking() {
    let csv_data = create_test_csv();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files.clone(), 1, None, false, None);

    // Should start at file index 1
    assert_eq!(app.current_file(), &csv_files[1]);

    app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(app.current_file(), &csv_files[2]);

    app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    assert_eq!(app.current_file(), &csv_files[1]);
}

#[test]
fn test_status_message_lifecycle() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Initially no status message
    assert!(app.status_message.is_none());

    // Make data dirty and try to quit
    app.csv_data.is_dirty = true;
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();

    // Should have status message
    assert!(app.status_message.is_some());
}
