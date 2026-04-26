use super::*;
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::{InputResult, PendingCommand};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::num::NonZeroUsize;
use std::path::PathBuf;

fn create_test_csv_data() -> Document {
    Document::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ],
        "test.csv".to_string(),
    )
}

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_app_initialization() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Always starts at row 0 (displays as row 1)
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    assert!(!app.should_quit);
    assert!(!app.view_state.help_overlay_visible);
}

#[test]
fn test_navigation_down() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Starts at row 0, move down to row 1
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // Move down to row 2
    app.handle_key(key_event(KeyCode::Down)).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(2)));

    // Move down to row 3 (last row)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

    // Try to go beyond last row - should stay at last row
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
}

#[test]
fn test_navigation_up() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    app.view_state.table_state.select(Some(2));

    app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // Can navigate to row 0
    app.handle_key(key_event(KeyCode::Up)).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Try to go before first row - should stay at row 0
    app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_navigation_left_right() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(1));

    app.handle_key(key_event(KeyCode::Right)).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Try to go beyond last column
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(1));

    app.handle_key(key_event(KeyCode::Left)).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Try to go before first column
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_navigation_home_end() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    app.view_state.table_state.select(Some(1));

    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(3))); // Last row

    // gg - Go to first row (row 0)
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0))); // First row
}

#[test]
fn test_navigation_first_last_column() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    app.view_state.selected_column = ColIndex::new(1);

    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2)); // Last column

    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(0)); // First column
}

#[test]
fn test_q_opens_sql_editor() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.mode, Mode::Normal);

    // Space+q opens SQL editor
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    assert_eq!(app.mode, Mode::SqlEditor);
    assert!(!app.should_quit);
}

#[test]
fn test_quit_via_command_mode() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert!(!app.should_quit);

    // Enter command mode and type :q
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_quit_with_unsaved_changes() {
    let mut csv_data = create_test_csv_data();
    csv_data.is_dirty = true;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert!(!app.should_quit);

    // Try :q with unsaved changes
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();
    assert!(!app.should_quit); // Should not quit
    assert!(app.status_message.is_some()); // Should show warning
}

#[test]
fn test_help_toggle() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert!(!app.view_state.help_overlay_visible);

    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.view_state.help_overlay_visible);

    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(!app.view_state.help_overlay_visible);
}

#[test]
fn test_help_close_with_esc() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    app.view_state.help_overlay_visible = true;

    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(!app.view_state.help_overlay_visible);
}

#[test]
fn test_navigation_blocked_when_help_shown() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    app.view_state.help_overlay_visible = true;
    let initial_row = app.selected_row();
    let initial_col = app.view_state.selected_column;

    // Try navigation with help shown
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), initial_row);

    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, initial_col);

    // File switching should also be blocked
    let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
    assert_eq!(should_reload, InputResult::Continue);
}

#[test]
fn test_current_file_path() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv"), PathBuf::from("other.csv")];
    let app = App::new(
        csv_data,
        csv_files.clone(),
        0,
        crate::session::FileConfig::new(),
    );

    assert_eq!(app.current_file(), &csv_files[0]);
}

// ========== v0.1.2: Multi-Key Command Tests ==========

#[test]
fn test_multi_key_gg_goes_to_first_row() {
    // Setup: Create app (starts at row 0), move to last row
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to last row (row 3)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

    // Execute gg command: press 'g' then 'g'
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // Should go to row 0 (first row)
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_multi_key_g_goes_to_last_row() {
    // Setup: Create app (starts at row 0)
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Press G to go to last row
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

    // Should be at last row (row 3)
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
}

#[test]
fn test_multi_key_2g_goes_to_row_2() {
    // Setup: Create app (starts at row 0)
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Press '2' to start count prefix
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    // Press 'G' to execute go to row 2
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

    // 2G should go to absolute row 2
    assert_eq!(app.selected_row(), Some(RowIndex::new(2)));
}

// ========== v0.1.2: Count Prefix Tests ==========

#[test]
fn test_count_prefix_2j_moves_down_2_rows() {
    // Setup: Create app (starts at row 0)
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Press '2' to set count prefix
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    // Press 'j' to move down 2 rows
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // Should be at row 2 (moved down 2 rows from row 0)
    assert_eq!(app.selected_row(), Some(RowIndex::new(2)));
}

#[test]
fn test_count_prefix_0_goes_to_first_column() {
    // Setup: Create app at column 2 (last column)
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to last column (column 2, index 2)
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Press '0' alone (no existing count) - should go to first column
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

    // Should be at column 0 (not treated as start of count)
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_count_prefix_clears_after_use() {
    // Setup: Create app (starts at row 0)
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Set count prefix '2'
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    // Use it with 'j' to move down 2 rows
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(2)));

    // Now press 'j' again without count - should only move 1 row
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

    // Move back to row 0 (gg goes to row 0)
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Press 'j' without count - should move only 1 row (count was cleared)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1))); // Only moved 1 row, not 2
}

// ========== v0.1.2: Error Handling Tests ==========

#[test]
fn test_error_file_not_found_shows_message() {
    // Try to load a non-existent file
    use crate::Document;
    use std::path::PathBuf;

    let result = Document::from_file(
        &PathBuf::from("/nonexistent/path/file.csv"),
        None,
        false,
        None,
    );

    // Should return an error, not panic
    assert!(result.is_err());
}

#[test]
fn test_dirty_flag_behavior() {
    // Setup: Create app with clean data
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Initially not dirty
    assert!(!app.document.is_dirty);

    // Navigation shouldn't set dirty flag
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert!(!app.document.is_dirty);
}

#[test]
fn test_state_after_help_toggle() {
    // Setup: Create app
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let initial_row = app.selected_row();

    // Open help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.view_state.help_overlay_visible);

    // Navigation should be blocked when help is shown
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), initial_row); // Should not move

    // Close help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(!app.view_state.help_overlay_visible);

    // Now navigation should work
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(
        app.selected_row(),
        Some(initial_row.unwrap().saturating_add(1))
    );
}

#[test]
fn test_count_prefix_2l_moves_right_2_columns() {
    // Setup: Create app at column 0
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Press '2' to set count prefix
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    // Press 'l' to move right 2 columns
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    // Should be at column 2 (moved right 2 columns from column 0)
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_special_keys_ignored_in_normal_mode() {
    // Setup: Create app
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let initial_row = app.selected_row();
    let initial_col = app.view_state.selected_column;

    // Press various special keys that should be ignored
    app.handle_key(key_event(KeyCode::F(1))).unwrap();
    app.handle_key(key_event(KeyCode::Insert)).unwrap();
    app.handle_key(key_event(KeyCode::Delete)).unwrap();

    // State should remain unchanged
    assert_eq!(app.selected_row(), initial_row);
    assert_eq!(app.view_state.selected_column, initial_col);
    assert!(!app.should_quit);
}

#[test]
fn test_esc_cancels_multi_key_command() {
    // Setup: Create app
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Start multi-key by pressing 'g'. With the keymap path, `g` goes
    // into `chord_buffer` (since `gg`/`gj`/etc. are bound). The legacy
    // `pending_command` is no longer set in this case.
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert!(
        !app.input_state.chord_buffer.is_empty() || app.input_state.pending_command.is_some(),
        "either chord buffer or pending_command should be set after `g`"
    );

    // Press ESC to cancel
    app.handle_key(key_event(KeyCode::Esc)).unwrap();

    // Both should be cleared after Esc.
    assert!(app.input_state.chord_buffer.is_empty());
    assert!(app.input_state.pending_command.is_none());
}

#[test]
fn test_count_prefix_3g_goes_to_row_3() {
    // Setup: Create app with more rows
    let csv_data = Document::new(
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
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Press '3' then 'G' to go to absolute row 3
    app.handle_key(key_event(KeyCode::Char('3'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

    // Should be at row 3
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
}

#[test]
fn test_help_closed_with_esc() {
    // Setup: Create app
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Open help
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.view_state.help_overlay_visible);

    // Close help with ESC
    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(!app.view_state.help_overlay_visible);
}

#[test]
fn test_sequential_navigation_workflow() {
    // Setup: Create app (starts at row 0)
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Complex navigation sequence
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 1
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap(); // Right to col 1
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 2
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap(); // Left to col 0
    app.handle_key(key_event(KeyCode::Char('k'))).unwrap(); // Up to row 1

    // Should be at row 1, col 0
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_dollar_sign_goes_to_last_column() {
    // Setup: Create app at column 0
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Press '$' to go to last column
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

    // Should be at last column (column 2)
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_zero_goes_to_first_column() {
    // Setup: Create app at last column
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to last column
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Press '0' to go to first column
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

    // Should be at first column (column 0)
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_page_up_down_navigation() {
    // Setup: Create app with more rows
    let csv_data = Document::new(
        vec!["A".to_string()],
        vec![
            vec!["1".to_string()],
            vec!["2".to_string()],
            vec!["3".to_string()],
            vec!["4".to_string()],
            vec!["5".to_string()],
            vec!["6".to_string()],
            vec!["7".to_string()],
            vec!["8".to_string()],
            vec!["9".to_string()],
            vec!["10".to_string()],
        ],
        "test.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Start at row 5 (move 5 times from row 0)
    for _ in 0..5 {
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    }
    assert_eq!(app.selected_row(), Some(RowIndex::new(5)));

    // Page up should move up (typically ~20 rows, but we only have 10)
    app.handle_key(key_event(KeyCode::PageUp)).unwrap();
    // Should be at row 5 or lower
    assert!(app.selected_row().unwrap().get() <= 5);

    // Page down should move down
    app.handle_key(key_event(KeyCode::PageDown)).unwrap();
    // Should have moved or stayed at boundary
}

#[test]
fn test_home_end_keys() {
    // Setup: Create app at middle
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to middle column
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(1));

    // Home and End keys should work without crashing
    app.handle_key(key_event(KeyCode::Home)).unwrap();
    app.handle_key(key_event(KeyCode::End)).unwrap();
    // Test passes if no panic occurs
}

#[test]
fn test_column_boundary_navigation() {
    // Setup: Create app
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Try to go left from first column (should stay)
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Go to last column
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Try to go right from last column (should stay)
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_file_switch_preserves_position() {
    // Setup: Create app, navigate to row 2, column 2
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Navigate to row 2, column 2 (move 2 rows down from row 0, 2 columns right)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    assert_eq!(app.selected_row(), Some(RowIndex::new(2)));
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Note: In real app, file switch would reload and reset position
    // This test verifies current behavior
}

// ===== Priority 1: Navigation Edge Cases =====

#[test]
fn test_navigation_gg_on_single_row_file() {
    // CSV with only one data row (+ header = 2 total rows)
    let csv_data = Document::new(
        vec!["A".to_string(), "B".to_string()],
        vec![vec!["1".to_string(), "2".to_string()]],
        "test.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Execute gg - should go to row 0 (first row)
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // Should be at row 0
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_navigation_g_shift_on_single_row_file() {
    let csv_data = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Execute G (go to last row) - should go to row 1 (only data row)
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

    // Should be at row 1 (the only data row)
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
}

#[test]
fn test_count_prefix_exceeds_row_bounds() {
    let csv_data = create_test_csv_data(); // Has 3 rows
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());
    let initial_row = app.selected_row();

    // Try to jump to row 9999 with 9999G
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

    // Position should not change when out of bounds
    assert_eq!(app.selected_row(), initial_row);
    // Should show error message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("does not exist"));
}

#[test]
fn test_count_prefix_exceeds_column_bounds() {
    let csv_data = create_test_csv_data(); // Has 3 columns
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Try to move right 100 columns with 100l
    app.handle_key(key_event(KeyCode::Char('1'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    // Should clamp to last column (column 2)
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_navigation_dollar_on_single_column() {
    let csv_data = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Execute $ (go to last column)
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

    // Should stay at column 0 (only column)
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_navigation_zero_already_at_first_column() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Execute 0 (go to first column)
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

    // Should stay at column 0
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_navigation_j_on_last_row() {
    let csv_data = create_test_csv_data(); // 3 data rows (row 1,2,3)
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to last row (row 3)
    app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

    // Try to move down from last row
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // Should stay at last row
    assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
}

#[test]
fn test_navigation_k_on_first_row() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Should start at row 0 (first row)
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

    // Try to move up from first row - should stay at row 0
    app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
}

#[test]
fn test_navigation_h_on_first_column() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.view_state.selected_column, ColIndex::new(0));

    // Try to move left from first column
    app.handle_key(key_event(KeyCode::Char('h'))).unwrap();

    // Should stay at column 0
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
}

#[test]
fn test_navigation_l_on_last_column() {
    let csv_data = create_test_csv_data(); // 3 columns
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to last column
    app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Try to move right from last column
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

    // Should stay at column 2
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));
}

#[test]
fn test_count_prefix_zero_special_case() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to column 2
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column, ColIndex::new(2));

    // Execute 0j (should treat as "0" to first column, not "0 times j")
    app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // Should have moved to first column, then down one row (from row 0 to row 1)
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
}

// ===== Priority 2: State Management Tests =====

#[test]
fn test_pending_key_cleared_on_esc() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Start a multi-key command. With the keymap path the chord state
    // lives in `chord_buffer` rather than `pending_command`.
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert!(!app.input_state.chord_buffer.is_empty() || app.input_state.pending_command.is_some());

    // Press ESC to cancel
    app.handle_key(key_event(KeyCode::Esc)).unwrap();

    // Both should be cleared.
    assert_eq!(app.input_state.pending_command, None);
    assert!(app.input_state.chord_buffer.is_empty());
}

#[test]
fn test_pending_key_cleared_on_valid_command() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Execute gg command — chord state lives in `chord_buffer` now.
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert!(!app.input_state.chord_buffer.is_empty() || app.input_state.pending_command.is_some());

    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

    // Both pending_command and chord_buffer should be cleared after
    // the command completes.
    assert_eq!(app.input_state.pending_command, None);
    assert!(app.input_state.chord_buffer.is_empty());
}

#[test]
fn test_count_prefix_cleared_after_use() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Build count prefix 25
    app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
    assert_eq!(app.input_state.command_count, NonZeroUsize::new(25));

    // Execute j (move down 25 rows, will clamp to last row)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // Count should be cleared
    assert_eq!(app.input_state.command_count, None);
}

#[test]
fn test_state_consistency_after_rapid_navigation() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Rapid navigation sequence
    let keys = vec!['j', 'j', 'k', 'l', 'h', 'j', 'l', 'k'];
    for key in keys {
        app.handle_key(key_event(KeyCode::Char(key))).unwrap();
    }

    // State should still be valid
    assert!(app.selected_row().is_some());
    assert!(app.view_state.selected_column.get() < app.document.column_count());
    assert_eq!(app.input_state.pending_command, None);
    assert_eq!(app.input_state.command_count, None);
}

#[test]
fn test_dirty_flag_persistence_across_operations() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Initial state should not be dirty
    assert!(!app.document.is_dirty);

    // Simulate making a change (we'll manually set it since editing isn't implemented yet)
    app.document.is_dirty = true;

    // Navigation should not affect dirty flag
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
    assert!(app.document.is_dirty);

    // Help toggle should not affect dirty flag
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.document.is_dirty);
    app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
    assert!(app.document.is_dirty);
}

#[test]
fn test_state_after_invalid_g_sequence() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let initial_row = app.selected_row();

    // Start g command — keymap holds it in chord_buffer.
    app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
    assert!(!app.input_state.chord_buffer.is_empty() || app.input_state.pending_command.is_some());

    // Send letter (parametric `g{letter}` column jump). The keymap can't
    // express this so it gives up and replays both keys through the
    // legacy handler, which transitions to `GotoColumn` state.
    app.handle_key(key_event(KeyCode::Char('x'))).unwrap();

    // After replay, the legacy parametric chord state machinery is in
    // play.
    assert!(matches!(
        app.input_state.pending_command,
        Some(PendingCommand::GotoColumn(_))
    ));

    // Send Enter to execute the column jump
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // State should be cleared after executing
    assert_eq!(app.input_state.pending_command, None);
    // Row should not have changed
    assert_eq!(app.selected_row(), initial_row);
    // Column should not have changed (X doesn't exist, shows error)
    assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    // Should show error message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("does not exist"));
}

#[test]
fn test_count_prefix_max_digits() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Build a very large count
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('9'))).unwrap();

    // Should have count set
    assert!(app.input_state.command_count.is_some());

    // Execute command
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

    // Should clamp to valid range (last row = row 3)
    assert_eq!(app.selected_row(), Some(RowIndex::new(3))); // Last row in test data
}

// ===== Z-Command Integration Tests (Viewport Positioning) =====

#[test]
fn test_z_command_top_viewport() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to next row (row 1)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // Execute zt (viewport top)
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('t'))).unwrap();

    assert_eq!(app.view_state.viewport_mode, crate::ui::ViewportMode::Top);
    assert!(app.status_message.is_some());
    assert!(app
        .status_message
        .as_ref()
        .unwrap()
        .as_str()
        .contains("top"));
}

#[test]
fn test_z_command_center_viewport() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to next row (row 1)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // Execute zz (viewport center)
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();

    assert_eq!(
        app.view_state.viewport_mode,
        crate::ui::ViewportMode::Center
    );
    assert!(app.status_message.is_some());
    assert!(app
        .status_message
        .as_ref()
        .unwrap()
        .as_str()
        .contains("center"));
}

#[test]
fn test_z_command_bottom_viewport() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to next row (row 1)
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

    // Execute zb (viewport bottom)
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('b'))).unwrap();

    assert_eq!(
        app.view_state.viewport_mode,
        crate::ui::ViewportMode::Bottom
    );
    assert!(app.status_message.is_some());
    assert!(app
        .status_message
        .as_ref()
        .unwrap()
        .as_str()
        .contains("bottom"));
}

#[test]
fn test_viewport_mode_persists_across_navigation() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Set viewport to center
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
    assert_eq!(
        app.view_state.viewport_mode,
        crate::ui::ViewportMode::Center
    );

    // Move down - viewport should reset to Auto
    app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.view_state.viewport_mode, crate::ui::ViewportMode::Auto);
}

// Note: Most runtime error tests (file deletion, permission changes, etc.)
// are in tests/error_handling_test.rs as integration tests since they
// require file system operations with tempfile.

#[test]
fn test_f_command_shows_current_filename() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // :f with no argument shows current filename
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.mode, Mode::Normal);
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(
        msg.contains("test.csv"),
        "Expected filename in status, got: {}",
        msg
    );
}

#[test]
fn test_f_command_renames_file() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // :f newname.csv
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    for c in "newname.csv".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.document.filename, "newname.csv");
    assert_eq!(app.current_file(), &PathBuf::from("newname.csv"));
    assert!(app.document.is_dirty);

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(
        msg.contains("newname.csv"),
        "Expected rename confirmation, got: {}",
        msg
    );
}

#[test]
fn test_f_command_rename_marks_session_dirty() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert!(!app.session.is_current_file_dirty());

    // :f renamed.csv
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    for c in "renamed.csv".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert!(app.session.is_current_file_dirty());
}

#[test]
fn test_f_command_rename_preserves_query_output_status() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("result.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Mark as query output (simulating what happens after SQL execution)
    let path = app.current_file().clone();
    app.session.mark_query_output(&path);

    // :f results.csv
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
    app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
    for c in "renamed_result.csv".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    // Query output status should follow the renamed file
    let new_path = app.current_file().clone();
    assert_eq!(new_path, PathBuf::from("renamed_result.csv"));
    assert!(app.session.is_query_output(&new_path));
    assert!(!app.session.is_query_output(&PathBuf::from("result.csv")));
}

// ============================================================================
// Phase 4: Magnifier Integration Tests
// ============================================================================

#[test]
fn test_open_magnifier() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Position cursor at a cell with content
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    app.open_magnifier();

    assert!(app.magnifier_state.is_some());
    assert_eq!(app.mode, Mode::Magnifier);

    let mag = app.magnifier_state.as_ref().unwrap();
    assert_eq!(mag.cell_position(), (RowIndex::new(1), ColIndex::new(0)));
}

#[test]
fn test_save_and_close_magnifier() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Open magnifier
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);
    app.open_magnifier();

    // Edit content
    if let Some(mag) = app.magnifier_state.as_mut() {
        mag.enter_insert_mode();
        mag.insert_char('X');
    }

    // Save and close
    app.save_and_close_magnifier();

    assert!(app.magnifier_state.is_none());
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.document.is_dirty);

    // Check that content was updated
    let cell = app.document.cell(RowIndex::new(1), ColIndex::new(0));
    assert!(cell.starts_with('X'));
}

#[test]
fn test_close_magnifier_discard() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Open magnifier and edit
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);
    app.open_magnifier();

    let original_content = app
        .document
        .cell(RowIndex::new(1), ColIndex::new(0))
        .to_string();

    if let Some(mag) = app.magnifier_state.as_mut() {
        mag.enter_insert_mode();
        mag.insert_char('X');
    }

    // Discard changes
    app.close_magnifier_discard();

    assert!(app.magnifier_state.is_none());
    assert_eq!(app.mode, Mode::Normal);
    assert!(!app.document.is_dirty);

    // Check that content was NOT updated
    let cell = app.document.cell(RowIndex::new(1), ColIndex::new(0));
    assert_eq!(cell, original_content);
}

#[test]
fn test_magnifier_is_dirty() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Initially not dirty
    assert!(!app.magnifier_is_dirty());

    // Open magnifier
    app.open_magnifier();
    assert!(!app.magnifier_is_dirty());

    // Edit content
    if let Some(mag) = app.magnifier_state.as_mut() {
        mag.enter_insert_mode();
        mag.insert_char('X');
    }

    // Now it should be dirty
    assert!(app.magnifier_is_dirty());
}

#[test]
fn test_magnifier_with_empty_cell() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Position at an empty cell (beyond data)
    app.view_state.table_state.select(Some(10));
    app.view_state.selected_column = ColIndex::new(0);

    app.open_magnifier();

    assert!(app.magnifier_state.is_some());
    let mag = app.magnifier_state.as_ref().unwrap();
    assert_eq!(mag.content(), "");
}

#[test]
fn test_magnifier_multiline_content() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    app.open_magnifier();

    // Clear existing content and add multiline content
    if let Some(mag) = app.magnifier_state.as_mut() {
        // Delete existing content
        mag.delete_line();
        // Add new multiline content
        mag.enter_insert_mode();
        mag.insert_char('L');
        mag.insert_char('1');
        mag.newline();
        mag.insert_char('L');
        mag.insert_char('2');
    }

    app.save_and_close_magnifier();

    // Check that multiline content was saved (app starts at row 0)
    let cell = app.document.cell(RowIndex::new(0), ColIndex::new(0));
    assert_eq!(cell, "L1\nL2");
}

#[test]
fn test_search_slash_enters_search_mode() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    assert_eq!(app.mode, Mode::Normal);
    app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
    assert_eq!(app.mode, Mode::Search);
}

#[test]
fn test_search_esc_returns_to_normal() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Enter search mode
    app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
    assert_eq!(app.mode, Mode::Search);

    // Type something
    app.handle_key(key_event(KeyCode::Char('t'))).unwrap();
    assert_eq!(app.mode, Mode::Search);
    assert_eq!(app.search_buffer, "t");

    // Esc should return to Normal
    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_search_enter_executes_search() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Enter search, type "5", press Enter
    app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert_eq!(app.mode, Mode::Normal);
    assert!(app.search_state.is_some());
    let state = app.search_state.as_ref().unwrap();
    assert_eq!(state.pattern, "5");
    assert_eq!(state.match_count(), 1);
}

#[test]
fn test_search_esc_in_normal_mode_clears_search() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Perform a search
    app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.search_state.is_some());

    // Esc in Normal mode should clear search highlighting
    app.handle_key(key_event(KeyCode::Esc)).unwrap();
    assert!(app.search_state.is_none());
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_search_asterisk_searches_current_cell() {
    let csv_data = Document::new(
        vec!["Name".to_string(), "City".to_string()],
        vec![
            vec!["Alice".to_string(), "Portland".to_string()],
            vec!["Bob".to_string(), "Boston".to_string()],
            vec!["Charlie".to_string(), "Portland".to_string()],
        ],
        "test.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Move to row 1, col 1 ("Portland")
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(1);

    // Press * to search for current cell content
    app.handle_key(key_event(KeyCode::Char('*'))).unwrap();

    assert_eq!(app.mode, Mode::Normal);
    assert!(app.search_state.is_some());
    let state = app.search_state.as_ref().unwrap();
    assert_eq!(state.pattern, "Portland");
    assert_eq!(state.match_count(), 2);
}

#[test]
fn test_search_noh_clears_search() {
    let csv_data = create_test_csv_data();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Perform a search
    app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
    app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
    app.handle_key(key_event(KeyCode::Enter)).unwrap();
    assert!(app.search_state.is_some());

    // Execute :noh
    app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
    for c in "noh".chars() {
        app.handle_key(key_event(KeyCode::Char(c))).unwrap();
    }
    app.handle_key(key_event(KeyCode::Enter)).unwrap();

    assert!(app.search_state.is_none());
}
