use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, ColIndex, Document, FileConfig, InputResult, RowIndex};
use std::path::PathBuf;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_test_csv() -> Document {
    Document {
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
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // User workflow: Navigate to bottom-right, then back to top-left
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // gg - Go to first row (multi-key command)
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(0)));

    // 0 - Go to first column
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_help_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Open help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.view_state.help_overlay_visible);

    // Try to navigate (should be blocked)
    let initial_row = app.get_selected_row();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.get_selected_row(), initial_row);

    // Close help with Esc
    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(!app.view_state.help_overlay_visible);

    // Navigation should work again
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_ne!(app.get_selected_row(), initial_row);
}

#[test]
fn test_quit_workflow_clean_state() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    assert!(!app.should_quit);

    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_quit_workflow_dirty_state() {
    let mut csv_data = create_test_csv();
    csv_data.is_dirty = true;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

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
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Start at file 0
    assert_eq!(app.session.active_file_index(), 0);

    // Switch forward through all files
    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(should_reload, InputResult::ReloadFile);
    assert_eq!(app.session.active_file_index(), 1);

    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(should_reload, InputResult::ReloadFile);
    assert_eq!(app.session.active_file_index(), 2);

    // Wrap around to first file
    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(should_reload, InputResult::ReloadFile);
    assert_eq!(app.session.active_file_index(), 0);

    // Switch backward
    let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    assert_eq!(should_reload, InputResult::ReloadFile);
    assert_eq!(app.session.active_file_index(), 2);
}

#[test]
fn test_help_and_quit_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Open help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.view_state.help_overlay_visible);

    // Try to quit while help is open (should not work)
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(!app.should_quit); // q is blocked when help is shown

    // Close help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(!app.view_state.help_overlay_visible);

    // Now quit should work
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_navigate_then_switch_file_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Navigate to a specific position
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Switch file
    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(should_reload, InputResult::ReloadFile);
    assert_eq!(app.session.active_file_index(), 1);
}

#[test]
fn test_rapid_key_sequence_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Simulate rapid user input
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    }

    // Should end at maximum position
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_zigzag_navigation_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Zigzag pattern: down-right, down-right, down-right
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_help_toggle_multiple_times() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    for _ in 0..5 {
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.view_state.help_overlay_visible);
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(!app.view_state.help_overlay_visible);
    }
}

#[test]
fn test_boundary_navigation_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Try to go beyond boundaries multiple times
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    }
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(0)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Go to opposite corner
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

    // Try to go beyond boundaries
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    }
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_current_file_tracking() {
    let csv_data = create_test_csv();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files.clone(), 1, FileConfig::new());

    // Should start at file index 1
    assert_eq!(app.get_current_file(), &csv_files[1]);

    app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(app.get_current_file(), &csv_files[2]);

    app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    assert_eq!(app.get_current_file(), &csv_files[1]);
}

#[test]
fn test_status_message_lifecycle() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Initially no status message
    assert!(app.status_message.is_none());

    // Make data dirty and try to quit
    app.document.is_dirty = true;
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();

    // Should have status message
    assert!(app.status_message.is_some());
}

// ===== Priority 3: Integration Workflow Tests =====

#[test]
fn test_complete_user_session_workflow() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Simulate realistic user session
    // 1. Navigate around
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    // 2. Go to specific location with gg
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(0)));

    // 3. Use count prefix
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));

    // 4. Navigate to end
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

    // 5. Toggle help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.view_state.help_overlay_visible);
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(!app.view_state.help_overlay_visible);

    // 6. Switch files
    app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(app.session.active_file_index(), 1);

    // 7. Navigate in new file
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // All state should be consistent
    assert_eq!(app.input_state.pending_command, None);
    assert_eq!(app.input_state.command_count, None);
}

#[test]
fn test_rapid_file_switching_10_times() {
    let csv_data = create_test_csv();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files.clone(), 0, FileConfig::new());

    // Rapidly switch between files
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    }

    // Should wrap around correctly (10 % 3 = 1)
    assert_eq!(app.session.active_file_index(), 1);

    // Try backward switches
    for _ in 0..10 {
        app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    }

    // Should wrap correctly backward
    assert_eq!(app.session.active_file_index(), 0);
}

#[test]
fn test_help_spam_100_toggles() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Spam help toggle 100 times
    for _ in 0..100 {
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    }

    // Should end in closed state (100 is even)
    assert!(!app.view_state.help_overlay_visible);

    // One more to open
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.view_state.help_overlay_visible);
}

#[test]
fn test_all_navigation_keys_in_sequence() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Test all navigation keys work
    let keys = vec![
        'h', 'j', 'k', 'l', // hjkl
        '0', '$', // First/last column
        'G', // Last row
    ];

    for key in keys {
        app.handle_key(key_event(KeyCode::Char(key))).unwrap();
    }

    // Execute gg (multi-key command)
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // All should complete without panic
    assert!(app.get_selected_row().is_some());
}

#[test]
fn test_mixed_operations_with_count_prefixes() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Mix count prefixes with various commands
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // 2j

    app.handle_key(key_event(KeyCode::Char('3'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap(); // 3l (will clamp)

    app.handle_key(key_event(KeyCode::Char('1'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap(); // 1G (go to row 1)

    // State should be consistent
    assert_eq!(app.input_state.command_count, None);
}

#[test]
fn test_error_recovery_from_invalid_sequence() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Try invalid sequence: g followed by invalid char
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap(); // Invalid

    // Should recover cleanly
    assert_eq!(app.input_state.pending_command, None);

    // Next command should work normally
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
}

#[test]
fn test_navigation_state_preserved_across_help() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Navigate to specific position
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    let row_before = app.get_selected_row();
    let col_before = app.view_state.selected_column;

    // Open and close help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();

    // Position should be preserved
    assert_eq!(app.get_selected_row(), row_before);
    assert_eq!(app.view_state.selected_column, col_before);
}

#[test]
fn test_count_prefix_with_file_switching() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Build count prefix
    app.handle_key(key_event(KeyCode::Char('5'))).unwrap();

    // Switch file (count should be cleared or not apply to file switching)
    app.handle_key(key_event(KeyCode::Char(']'))).unwrap();

    // State should be valid
    assert_eq!(app.session.active_file_index(), 1);
}
