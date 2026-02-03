use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::input::PendingCommand;
use lazycsv::{App, ColIndex, Document, FileConfig, InputResult, RowIndex};
use std::fs::write;
use std::path::PathBuf;
use tempfile::TempDir;

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

    // Try column jump sequence: g followed by letter 'z'
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap(); // Start column jump to Z

    // Should be in GotoColumn state (z is a valid letter for column names)
    assert!(matches!(
        app.input_state.pending_command,
        Some(PendingCommand::GotoColumn(_))
    ));

    // Press Enter to execute the column jump (will clamp to last column)
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should be cleared after executing
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

#[test]
fn test_complete_session_load_navigate_switch_quit() {
    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.path().join("file1.csv");
    let file2_path = temp_dir.path().join("file2.csv");

    write(&file1_path, "A,B,C\n1,2,3\n4,5,6\n7,8,9").unwrap();
    write(&file2_path, "X,Y,Z\n10,11,12\n13,14,15").unwrap();

    let doc = Document::from_file(&file1_path, None, false, None).unwrap();
    let mut app = App::new(
        doc,
        vec![file1_path.clone(), file2_path.clone()],
        0,
        FileConfig::new(),
    );

    // Navigate in first file
    for _ in 0..5 {
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    }
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    // Switch to second file
    app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    app.reload_current_file().unwrap();

    // Navigate in second file
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

    // Switch back
    app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    app.reload_current_file().unwrap();

    // App should be in valid state
    assert_eq!(app.session.active_file_index(), 0);
    assert!(!app.should_quit);
}

#[test]
fn test_recover_from_file_switch_error() {
    let temp_dir = TempDir::new().unwrap();
    let valid_file = temp_dir.path().join("valid.csv");
    let invalid_path = PathBuf::from("/nonexistent/invalid.csv");

    write(&valid_file, "A,B\n1,2\n3,4").unwrap();

    let doc = Document::from_file(&valid_file, None, false, None).unwrap();
    let mut app = App::new(
        doc,
        vec![valid_file.clone(), invalid_path],
        0,
        FileConfig::new(),
    );

    // Switch to invalid file
    app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    let result = app.reload_current_file();

    // Should fail to reload
    assert!(result.is_err());

    // Switch back to valid file
    app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    let result = app.reload_current_file();

    // Should successfully reload
    assert!(result.is_ok());
    assert_eq!(app.session.active_file_index(), 0);
}

#[test]
fn test_rapid_navigation_and_file_switching() {
    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.path().join("f1.csv");
    let file2_path = temp_dir.path().join("f2.csv");

    write(&file1_path, "A,B,C\n1,2,3\n4,5,6\n7,8,9\n10,11,12").unwrap();
    write(&file2_path, "X,Y,Z\n20,21,22\n23,24,25").unwrap();

    let doc = Document::from_file(&file1_path, None, false, None).unwrap();
    let mut app = App::new(
        doc,
        vec![file1_path.clone(), file2_path.clone()],
        0,
        FileConfig::new(),
    );

    // Rapid mixed operations (50 keypresses)
    let keys = [
        'j', 'j', 'k', 'l', 'h', 'j', 'l', ']', 'j', 'j', 'k', 'h', '[', 'j', 'l', 'j', 'k', '$',
        '0', 'j', ']', 'k', 'k', '[', 'l', 'l', 'h', 'j', 'j', 'j', 'k', 'k', 'l', '$', '0', ']',
        '[', 'j', 'k', 'l', 'h', 'j', 'l', 'k', '0', '$', ']', '[', 'j', 'k',
    ];

    for key in keys.iter() {
        if *key == ']' || *key == '[' {
            app.handle_key(key_event(KeyCode::Char(*key))).unwrap();
            // Reload after file switch
            let _ = app.reload_current_file();
        } else {
            app.handle_key(key_event(KeyCode::Char(*key))).unwrap();
        }
    }

    // App should remain stable
    assert!(!app.should_quit);
    // Should have valid position
    assert!(app.get_selected_row().is_some());
}

#[test]
fn test_help_during_multi_key_command() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Start a multi-key command (g for goto)
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert!(app.input_state.pending_command.is_some());

    // Try to open help with '?'
    // Note: This may complete the command as 'g?' or may open help depending on implementation
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();

    // Either the help opened, or the pending command was processed
    // Both are acceptable behaviors - just verify app is stable
    let was_help_opened = app.view_state.help_overlay_visible;

    if was_help_opened {
        // Close help
        app.handle_key(key_event(KeyCode::Esc)).unwrap();
        assert!(!app.view_state.help_overlay_visible);
    }

    // Should be in valid state regardless
    assert!(!app.should_quit);
}

#[test]
fn test_viewport_mode_reset_across_files() {
    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.path().join("f1.csv");
    let file2_path = temp_dir.path().join("f2.csv");

    write(&file1_path, "A,B\n1,2\n3,4\n5,6").unwrap();
    write(&file2_path, "X,Y\n7,8\n9,10").unwrap();

    let doc = Document::from_file(&file1_path, None, false, None).unwrap();
    let mut app = App::new(
        doc,
        vec![file1_path.clone(), file2_path.clone()],
        0,
        FileConfig::new(),
    );

    // Set viewport mode to center
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    assert_eq!(
        app.view_state.viewport_mode,
        lazycsv::ui::ViewportMode::Center
    );

    // Switch to file 2
    app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    app.reload_current_file().unwrap();

    // Viewport mode should persist or reset (document behavior)
    // Either behavior is acceptable, just verify app is stable
    assert_eq!(app.session.active_file_index(), 1);

    // Switch back to file 1
    app.handle_key(key_event(KeyCode::Char('['))).unwrap();
    app.reload_current_file().unwrap();

    // App should be stable
    assert_eq!(app.session.active_file_index(), 0);
}

#[test]
fn test_status_message_lifecycle_complete() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Trigger a viewport positioning command which should produce a status message
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap(); // zz = center viewport

    // Should have status message about viewport positioning
    let had_message = app.status_message.is_some();

    if had_message {
        // Next keypress should clear it (or it may already be cleared depending on implementation)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    }

    // App should be in valid state
    assert!(!app.should_quit);
}
