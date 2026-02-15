use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, Document, FileConfig};
use std::path::PathBuf;

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
fn test_5g_jumps_to_row_5() {
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string()],
        vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "4".to_string()],
            vec!["5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string()],
            vec!["9".to_string(), "10".to_string()],
            vec!["11".to_string(), "12".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Start at row 1
    assert_eq!(app.view_state.table_state.selected(), Some(1));

    // Type 5g to jump to row 5
    let _ = app.handle_key(key_event(KeyCode::Char('5')));
    let _ = app.handle_key(key_event(KeyCode::Char('g')));

    // Should jump to row 5
    assert_eq!(app.view_state.table_state.selected(), Some(5));
}

#[test]
fn test_bg_removed_use_c_command_instead() {
    // This test verifies that Bg syntax is no longer supported
    // Users should use :cB instead
    let doc = Document::new(
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ],
        vec![vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Start at column A (0)
    assert_eq!(app.view_state.selected_column.get(), 0);

    // Type B - should NOT start a jump sequence anymore
    let _ = app.handle_key(key_event(KeyCode::Char('B')));

    // B alone should not set pending command
    assert!(app.input_state.pending_command.is_none());

    // Use :cB instead
    send_command(&mut app, "cB");
    assert_eq!(app.view_state.selected_column.get(), 1);
}

#[test]
fn test_cell_jump_removed_use_c_command_and_row_jump() {
    // B1g syntax is removed. Use :cB then 1g instead
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
            vec!["10".to_string(), "11".to_string(), "12".to_string()],
            vec!["13".to_string(), "14".to_string(), "15".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Start at row 1, column A
    assert_eq!(app.view_state.table_state.selected(), Some(1));
    assert_eq!(app.view_state.selected_column.get(), 0);

    // Jump to cell B1 using :cB then stay at row 1
    send_command(&mut app, "cB");
    assert_eq!(app.view_state.table_state.selected(), Some(1));
    assert_eq!(app.view_state.selected_column.get(), 1);
}

#[test]
fn test_cell_jump_c3_removed() {
    // C3g syntax is removed. Use :cC and 3g separately
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
            vec!["10".to_string(), "11".to_string(), "12".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Jump to cell C3 using :cC and 3g
    send_command(&mut app, "cC");
    let _ = app.handle_key(key_event(KeyCode::Char('3')));
    let _ = app.handle_key(key_event(KeyCode::Char('g')));

    // Should be at row 3, column C (2)
    assert_eq!(app.view_state.table_state.selected(), Some(3));
    assert_eq!(app.view_state.selected_column.get(), 2);
}

#[test]
fn test_c_command_works_with_and_without_space() {
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Test :c B (with space) - should work
    send_command(&mut app, "c B");
    assert_eq!(app.view_state.selected_column.get(), 1);

    // Test :cC (without space) - should also work
    send_command(&mut app, "cC");
    assert_eq!(app.view_state.selected_column.get(), 2);
}

#[test]
fn test_old_colon_number_navigation_removed() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![
            vec!["1".to_string()],
            vec!["2".to_string()],
            vec!["3".to_string()],
            vec!["4".to_string()],
            vec!["5".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Try old :5 command (should not work as navigation)
    send_command(&mut app, "5");

    // Should either show error or do nothing (not jump to row 5)
    // The cursor should still be at the original position
    assert_eq!(app.view_state.table_state.selected(), Some(1));
}

#[test]
fn test_esc_clears_count_prefix() {
    // Test that Esc clears count prefix (not JumpTarget anymore)
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Start typing a count: 5
    let _ = app.handle_key(key_event(KeyCode::Char('5')));

    // Should have count prefix
    assert!(app.input_state.command_count.is_some());

    // Press Esc to cancel
    let _ = app.handle_key(key_event(KeyCode::Esc));

    // Count should be cleared
    assert!(app.input_state.command_count.is_none());
}

#[test]
fn test_multiple_digit_row_jump() {
    let doc = Document::new(
        vec!["A".to_string()],
        (1..=20)
            .map(|i| vec![i.to_string()])
            .collect::<Vec<Vec<String>>>(),
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Type 15g to jump to row 15
    let _ = app.handle_key(key_event(KeyCode::Char('1')));
    let _ = app.handle_key(key_event(KeyCode::Char('5')));
    let _ = app.handle_key(key_event(KeyCode::Char('g')));

    // Should jump to row 15
    assert_eq!(app.view_state.table_state.selected(), Some(15));
}

#[test]
fn test_letter_column_jump_removed_use_c_command() {
    // Zg syntax is removed. Use :cZ instead
    // Create 30 columns (A-Z, then AA-AD)
    let headers: Vec<String> = (0..30)
        .map(|i| {
            if i < 26 {
                ((b'A' + i) as char).to_string()
            } else {
                format!("A{}", (b'A' + (i - 26)) as char)
            }
        })
        .collect();

    let doc = Document::new(
        headers,
        vec![vec!["1".to_string(); 30]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Start at column 0
    assert_eq!(app.view_state.selected_column.get(), 0);

    // Use :cZ to jump to column Z
    send_command(&mut app, "cZ");

    // Should jump to column Z (last single-letter column, index 25)
    assert_eq!(app.view_state.selected_column.get(), 25);
}

// ========== Tests for :c command (column jump with colon command) ==========

#[test]
fn test_c_command_jumps_to_column_a() {
    let doc = Document::new(
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ],
        vec![vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Start at column B (1)
    app.view_state.selected_column = lazycsv::ColIndex::new(1);

    // Use :cA to jump to column A
    send_command(&mut app, "cA");

    // Should be at column A (index 0)
    assert_eq!(app.view_state.selected_column.get(), 0);
}

#[test]
fn test_c_command_case_insensitive() {
    let doc = Document::new(
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ],
        vec![vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Try lowercase :cb
    send_command(&mut app, "cb");

    // Should jump to column B (index 1)
    assert_eq!(app.view_state.selected_column.get(), 1);

    // Try mixed case :Cc
    send_command(&mut app, "Cc");

    // Should jump to column C (index 2)
    assert_eq!(app.view_state.selected_column.get(), 2);
}

#[test]
fn test_c_command_multi_letter_column() {
    // Create 30 columns (A-Z, then AA-AD)
    let headers: Vec<String> = (0..30)
        .map(|i| {
            if i < 26 {
                ((b'A' + i) as char).to_string()
            } else {
                format!("A{}", (b'A' + (i - 26)) as char)
            }
        })
        .collect();

    let doc = Document::new(
        headers,
        vec![vec!["1".to_string(); 30]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Jump to column AA (index 26)
    send_command(&mut app, "cAA");

    // Should jump to column AA
    assert_eq!(app.view_state.selected_column.get(), 26);

    // Jump to column AB (index 27)
    send_command(&mut app, "cAB");

    // Should jump to column AB
    assert_eq!(app.view_state.selected_column.get(), 27);
}

#[test]
fn test_c_command_numeric_input() {
    let doc = Document::new(
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ],
        vec![vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Jump to column 1 (A)
    send_command(&mut app, "c1");

    // Should jump to column A (index 0)
    assert_eq!(app.view_state.selected_column.get(), 0);

    // Jump to column 3 (C)
    send_command(&mut app, "c3");

    // Should jump to column C (index 2)
    assert_eq!(app.view_state.selected_column.get(), 2);
}

#[test]
fn test_c_command_out_of_bounds_error() {
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Try to jump to column Z (doesn't exist)
    send_command(&mut app, "cZ");

    // Should show error message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("does not exist") || msg.contains("max"));

    // Cursor should remain at original position (column A)
    assert_eq!(app.view_state.selected_column.get(), 0);
}

#[test]
fn test_c_command_invalid_input_error() {
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Try invalid input with special characters
    send_command(&mut app, "c@#$");

    // Should show error message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Invalid") || msg.contains("Usage"));
}

#[test]
fn test_c_command_no_argument_shows_usage() {
    let doc = Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Try :c without argument
    send_command(&mut app, "c");

    // Should show usage message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Usage"));
}

#[test]
fn test_c_command_solves_reserved_letter_conflicts() {
    // Test that :cA, :cI, :cO, :cG all work (letters reserved for other commands)
    let doc = Document::new(
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
            "F".to_string(),
            "G".to_string(),
            "H".to_string(),
            "I".to_string(),
            "J".to_string(),
            "K".to_string(),
            "L".to_string(),
            "M".to_string(),
            "N".to_string(),
            "O".to_string(),
        ],
        vec![vec!["1".to_string(); 15]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Test :cA (A is reserved for Append mode)
    send_command(&mut app, "cA");
    assert_eq!(app.view_state.selected_column.get(), 0);

    // Test :cI (I is reserved for Insert mode)
    send_command(&mut app, "cI");
    assert_eq!(app.view_state.selected_column.get(), 8);

    // Test :cO (O is reserved for insert row above)
    send_command(&mut app, "cO");
    assert_eq!(app.view_state.selected_column.get(), 14);

    // Test :cG (G is reserved for goto last row)
    send_command(&mut app, "cG");
    assert_eq!(app.view_state.selected_column.get(), 6);
}
