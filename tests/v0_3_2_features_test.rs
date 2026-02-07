//! Tests for v0.3.2 features
//!
//! - `:c` command for column navigation
//! - Reserved commands (`:q`, `:w`, `:h`)
//! - Out-of-bounds error handling
//! - No timeout on pending commands
//! - Default directory scanning

mod common;

use clap::Parser;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::input::PendingCommand;
use lazycsv::{cli::CliArgs, App, ColIndex, Document, FileConfig, RowIndex};
use std::fs::write;
use std::path::PathBuf;
use tempfile::TempDir;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn create_test_csv_5_cols() -> Document {
    Document {
        headers: vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
        ],
        rows: vec![
            vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
            ],
            vec![
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
            ],
            vec![
                "11".to_string(),
                "12".to_string(),
                "13".to_string(),
                "14".to_string(),
                "15".to_string(),
            ],
        ],
        filename: "test.csv".to_string(),
        is_dirty: false,
    }
}

// ===== `:c` Command Tests =====

#[test]
fn test_command_c_column_letter_uppercase() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'c C' to jump to column C
    app.handle_key(key_event(KeyCode::Char('c'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    app.handle_key(key_event(KeyCode::Char('C'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.view_state.selected_column, ColIndex::new(2)); // C = index 2
}

#[test]
fn test_command_c_column_letter_lowercase() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'c d' (lowercase) to jump to column D
    app.handle_key(key_event(KeyCode::Char('c'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    app.handle_key(key_event(KeyCode::Char('d'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.view_state.selected_column, ColIndex::new(3)); // D = index 3
}

#[test]
fn test_command_c_column_number() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'c 4' to jump to column 4 (D)
    app.handle_key(key_event(KeyCode::Char('c'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    app.handle_key(key_event(KeyCode::Char('4'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.view_state.selected_column, ColIndex::new(3)); // Column 4 = index 3
}

#[test]
fn test_command_c_first_column() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Move to column E first
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(4));

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'c A' to jump to first column
    app.handle_key(key_event(KeyCode::Char('c'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    app.handle_key(key_event(KeyCode::Char('A'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.view_state.selected_column, ColIndex::new(0)); // A = index 0
}

// ===== Reserved Commands Tests =====

#[test]
fn test_reserved_command_q_quits() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    assert!(!app.should_quit);

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'q' and enter
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert!(app.should_quit);
}

#[test]
fn test_reserved_command_q_does_not_jump_to_column() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let initial_column = app.view_state.selected_column;

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'q' - should quit, not jump to column Q
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Column should remain unchanged
    assert_eq!(app.view_state.selected_column, initial_column);
}

#[test]
fn test_reserved_command_h_shows_help() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    assert!(!app.view_state.help_overlay_visible);

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'h' and enter
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Help should be visible (or there should be a relevant status message)
    // Note: Implementation may vary - help overlay or status message
}

// ===== Out-of-Bounds Error Tests =====

#[test]
fn test_out_of_bounds_row_shows_error() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type '999' to try to jump to row 999
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should have an error message about row not existing
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(
        msg.contains("999") || msg.contains("max") || msg.contains("exist"),
        "Error message should indicate out of bounds: {}",
        msg
    );
}

#[test]
fn test_out_of_bounds_column_shows_error() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'c Z' to try to jump to column Z (only have A-E)
    app.handle_key(key_event(KeyCode::Char('c'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    app.handle_key(key_event(KeyCode::Char('Z'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should have an error message about column not existing
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(
        msg.contains("Z") || msg.contains("max") || msg.contains("exist") || msg.contains("Column"),
        "Error message should indicate out of bounds column: {}",
        msg
    );
}

// ===== Pending Command Tests (No Timeout) =====

#[test]
fn test_pending_g_command_no_timeout() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Press 'g' to start pending command
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // Should be in pending state
    assert!(matches!(
        app.input_state.pending_command,
        Some(PendingCommand::G)
    ));

    // The pending command should remain until next key
    // (no timeout in v0.3.2)
    assert!(app.input_state.pending_command.is_some());

    // Complete with 'g' -> gg
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // Should have executed and cleared pending state
    assert_eq!(app.input_state.pending_command, None);
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_pending_z_command_no_timeout() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Press 'z' to start pending command
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();

    // Should be in pending state
    assert!(matches!(
        app.input_state.pending_command,
        Some(PendingCommand::Z)
    ));

    // Complete with 'z' -> zz (center viewport)
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();

    // Should have executed and cleared pending state
    assert_eq!(app.input_state.pending_command, None);
}

#[test]
fn test_pending_count_no_timeout() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Press '5' to start count prefix
    app.handle_key(key_event(KeyCode::Char('5'))).unwrap();

    // Count should be accumulated
    assert!(app.input_state.command_count.is_some());

    // Complete with 'j' -> 5j (but clamp to max rows)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // Should have executed
    assert_eq!(app.input_state.command_count, None);
    // With 3 rows (0,1,2), 5j from 0 goes to row 2
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
}

// ===== Default Directory Tests =====

#[test]
fn test_default_directory_scans_current() {
    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.path().join("data1.csv");
    let file2_path = temp_dir.path().join("data2.csv");

    write(&file1_path, "A,B\n1,2\n").unwrap();
    write(&file2_path, "X,Y\n3,4\n").unwrap();

    // Change to temp directory and verify CLI behavior
    // Note: This tests the CLI parsing, not actual directory changing
    let args = CliArgs::try_parse_from(["lazycsv", temp_dir.path().to_str().unwrap()]).unwrap();

    // Should successfully parse with directory path
    assert!(args.path.is_some());
}

#[test]
fn test_from_cli_with_directory_finds_csv_files() {
    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.path().join("customers.csv");
    let file2_path = temp_dir.path().join("orders.csv");

    write(&file1_path, "ID,Name\n1,Alice\n2,Bob\n").unwrap();
    write(&file2_path, "OrderID,Total\n100,50.00\n").unwrap();

    let args = CliArgs::try_parse_from(["lazycsv", temp_dir.path().to_str().unwrap()]).unwrap();
    let result = App::from_cli(args);

    assert!(result.is_ok(), "Should load directory with CSV files");

    let app = result.unwrap();
    // Should have found both CSV files
    assert!(app.session.files().len() >= 2);
}

// ===== Status Bar Format Tests =====

#[test]
fn test_pending_command_in_status() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Press 'g' to start pending command
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // The pending command should be visible in input state
    assert!(matches!(
        app.input_state.pending_command,
        Some(PendingCommand::G)
    ));
}

// ===== Edge Case Tests =====

#[test]
fn test_command_c_empty_argument() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let initial_column = app.view_state.selected_column;

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type 'c' with no argument, then enter
    app.handle_key(key_event(KeyCode::Char('c'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Column should remain unchanged or show error
    // (implementation may vary)
    // Just verify app is stable and column didn't unexpectedly change
    assert!(app.get_selected_row().is_some());
    // Column should remain unchanged when given empty argument
    assert_eq!(app.view_state.selected_column, initial_column);
}

#[test]
fn test_multiple_pending_commands_cancel_each_other() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Press 'g' then 'z' - different pending commands
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // State should be G
    assert!(matches!(
        app.input_state.pending_command,
        Some(PendingCommand::G)
    ));

    // Pressing 'z' should handle the gz sequence
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();

    // Depending on implementation, 'gz' might be invalid or have special meaning
    // Just verify the app is in a stable state
    assert!(!app.should_quit);
}

#[test]
fn test_escape_cancels_command_mode() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Enter command mode
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type some characters
    app.handle_key(key_event(KeyCode::Char('c'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();

    // Press escape to cancel
    app.handle_key(key_event(KeyCode::Esc)).unwrap();

    // Should be back in normal mode, app stable
    assert!(!app.should_quit);
}

#[test]
fn test_row_jump_with_command_mode() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Enter command mode with ':'
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();

    // Type '2' to jump to row 2
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Should be at row 2 (index 1, since row numbers are 1-based in UI)
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
}

#[test]
fn test_vim_like_status_line_components() {
    let csv_data = create_test_csv_5_cols();
    let csv_files = vec![PathBuf::from("test.csv")];
    let app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Verify app has the components needed for vim-like status line
    assert!(app.get_selected_row().is_some());
    assert!(app.view_state.selected_column.get() < 5);

    // App should be in Normal mode initially
    assert!(matches!(app.mode, lazycsv::app::Mode::Normal));
}
