//! Integration tests for dual clipboard (row buffer / column buffer)
//!
//! Tests:
//! - P (paste row above)
//! - Comma leader entry (,)
//! - ,yy (yank column)
//! - ,dd (delete column)
//! - ,p (paste column right)
//! - ,P (paste column left)
//! - ,o (insert empty column right)
//! - ,O (insert empty column left)
//! - Cross-buffer isolation (row ops don't affect column buffer and vice versa)
//! - Count prefixes: 5dd, 5yy, multi-row paste
//! - cc (clear row + enter insert mode)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::Mode;
use lazycsv::{App, Document, FileConfig};
use std::path::PathBuf;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// 5-column, 3-data-row document:
///   Row 0 (header): A  B  C  D  E
///   Row 1:          A1 B1 C1 D1 E1
///   Row 2:          A2 B2 C2 D2 E2
///   Row 3:          A3 B3 C3 D3 E3
///
/// App starts at row 1, column 0.
fn create_test_app() -> App {
    let doc = Document::new(
        vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
        vec![
            vec![
                "A1".into(),
                "B1".into(),
                "C1".into(),
                "D1".into(),
                "E1".into(),
            ],
            vec![
                "A2".into(),
                "B2".into(),
                "C2".into(),
                "D2".into(),
                "E2".into(),
            ],
            vec![
                "A3".into(),
                "B3".into(),
                "C3".into(),
                "D3".into(),
                "E3".into(),
            ],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    App::new(doc, files, 0, FileConfig::new())
}

// ============================================================================
// P — paste row above
// ============================================================================

#[test]
fn test_capital_p_pastes_row_above() {
    let mut app = create_test_app();

    // Yank current row (row 1: A1 B1 C1 D1 E1)
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Move down to row 2
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.get_selected_row().unwrap().get(), 2);

    let initial_row_count = app.document.row_count();

    // Paste above
    app.handle_key(key(KeyCode::Char('P'))).unwrap();

    // Row count increased
    assert_eq!(app.document.row_count(), initial_row_count + 1);
    // Selection stays at row 2 (the pasted row)
    assert_eq!(app.get_selected_row().unwrap().get(), 2);
    // The pasted row should contain the yanked content
    assert_eq!(app.document.rows[2], vec!["A1", "B1", "C1", "D1", "E1"]);
    // The original row 2 (A2...) is now at row 3
    assert_eq!(app.document.rows[3], vec!["A2", "B2", "C2", "D2", "E2"]);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Pasted"))
        .unwrap_or(false));
}

#[test]
fn test_capital_p_without_clipboard_shows_error() {
    let mut app = create_test_app();
    assert!(app.clipboard.row_buffer_empty());

    app.handle_key(key(KeyCode::Char('P'))).unwrap();

    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Nothing to paste"))
        .unwrap_or(false));
}

// ============================================================================
// ,yy — yank column
// ============================================================================

#[test]
fn test_comma_yy_yanks_current_column() {
    let mut app = create_test_app();

    // Selected column is 0 (A). Column 0: [A, A1, A2, A3]
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    assert!(!app.clipboard.column_buffer_empty());
    assert_eq!(
        app.clipboard.as_column(),
        Some(vec!["A".into(), "A1".into(), "A2".into(), "A3".into()])
    );
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("column") && m.as_str().contains("yanked"))
        .unwrap_or(false));
    // Document unchanged
    assert!(!app.document.is_dirty);
    assert_eq!(app.document.column_count(), 5);
}

#[test]
fn test_comma_yy_on_different_column() {
    let mut app = create_test_app();

    // Move to column 2 (C)
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column.get(), 2);

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    assert_eq!(
        app.clipboard.as_column(),
        Some(vec!["C".into(), "C1".into(), "C2".into(), "C3".into()])
    );
}

// ============================================================================
// ,dd — delete column
// ============================================================================

#[test]
fn test_comma_dd_deletes_current_column() {
    let mut app = create_test_app();

    // Delete column 0 (A)
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.column_count(), 4);
    // Column A removed — first column is now B
    assert_eq!(app.document.rows[0], vec!["B", "C", "D", "E"]);
    assert_eq!(app.document.rows[1], vec!["B1", "C1", "D1", "E1"]);
    assert!(app.document.is_dirty);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("column") && m.as_str().contains("deleted"))
        .unwrap_or(false));
}

#[test]
fn test_comma_dd_stores_in_column_buffer() {
    let mut app = create_test_app();

    // Delete column 0
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert!(!app.clipboard.column_buffer_empty());
    assert_eq!(
        app.clipboard.as_column(),
        Some(vec!["A".into(), "A1".into(), "A2".into(), "A3".into()])
    );
    // Row buffer should be unaffected
    assert!(app.clipboard.row_buffer_empty());
}

#[test]
fn test_comma_dd_adjusts_selection_at_last_column() {
    let mut app = create_test_app();

    // Move to last column (E, index 4)
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
    }
    assert_eq!(app.view_state.selected_column.get(), 4);

    // Delete last column
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.column_count(), 4);
    // Selection should adjust to new last column (index 3)
    assert_eq!(app.view_state.selected_column.get(), 3);
}

// ============================================================================
// ,p — paste column right
// ============================================================================

#[test]
fn test_comma_p_pastes_column_right() {
    let mut app = create_test_app();

    // Yank column 0 (A)
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Paste right — column should be inserted at index 1
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert_eq!(app.document.column_count(), 6);
    // Column 1 should be a copy of column A
    assert_eq!(app.document.rows[0][1], "A");
    assert_eq!(app.document.rows[1][1], "A1");
    // Selection moves to the inserted column
    assert_eq!(app.view_state.selected_column.get(), 1);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Pasted"))
        .unwrap_or(false));
}

#[test]
fn test_comma_p_without_column_buffer_shows_error() {
    let mut app = create_test_app();
    assert!(app.clipboard.column_buffer_empty());

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Nothing to paste"))
        .unwrap_or(false));
}

// ============================================================================
// ,P — paste column left
// ============================================================================

#[test]
fn test_comma_capital_p_pastes_column_left() {
    let mut app = create_test_app();

    // Move to column 2 (C)
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();

    // Yank column C
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Paste left — column inserted at index 2 (current position)
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('P'))).unwrap();

    assert_eq!(app.document.column_count(), 6);
    // Column at index 2 should be the pasted copy of C
    assert_eq!(app.document.rows[0][2], "C");
    assert_eq!(app.document.rows[1][2], "C1");
    // Original C is now at index 3
    assert_eq!(app.document.rows[0][3], "C");
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Pasted"))
        .unwrap_or(false));
}

#[test]
fn test_comma_capital_p_without_column_buffer_shows_error() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('P'))).unwrap();

    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Nothing to paste"))
        .unwrap_or(false));
}

// ============================================================================
// ,o — insert empty column right
// ============================================================================

#[test]
fn test_comma_o_inserts_empty_column_right() {
    let mut app = create_test_app();
    assert_eq!(app.view_state.selected_column.get(), 0);

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('o'))).unwrap();

    assert_eq!(app.document.column_count(), 6);
    // New column inserted at index 1; original B is now at index 2
    assert_eq!(app.document.rows[0][2], "B");
    // The new column data rows should be empty
    assert_eq!(app.document.rows[1][1], "");
    assert_eq!(app.document.rows[2][1], "");
    // Selection moves to the new column
    assert_eq!(app.view_state.selected_column.get(), 1);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("empty column"))
        .unwrap_or(false));
}

// ============================================================================
// ,O — insert empty column left
// ============================================================================

#[test]
fn test_comma_capital_o_inserts_empty_column_left() {
    let mut app = create_test_app();

    // Move to column 2 (C)
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column.get(), 2);

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('O'))).unwrap();

    assert_eq!(app.document.column_count(), 6);
    // Empty column inserted at index 2; original C pushed to index 3
    assert_eq!(app.document.rows[0][3], "C");
    assert_eq!(app.document.rows[1][2], "");
    // Selection stays at index 2 (the new empty column)
    assert_eq!(app.view_state.selected_column.get(), 2);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("empty column"))
        .unwrap_or(false));
}

// ============================================================================
// Cross-buffer isolation
// ============================================================================

#[test]
fn test_yy_then_comma_p_shows_nothing_to_paste() {
    let mut app = create_test_app();

    // Yank a row (goes to row buffer)
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    assert!(!app.clipboard.row_buffer_empty());
    assert!(app.clipboard.column_buffer_empty());

    // Try ,p — column buffer is empty, should fail
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Nothing to paste"))
        .unwrap_or(false));
    // Document unchanged
    assert_eq!(app.document.column_count(), 5);
}

#[test]
fn test_comma_yy_then_p_shows_nothing_to_paste() {
    let mut app = create_test_app();

    // Yank a column (goes to column buffer)
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    assert!(!app.clipboard.column_buffer_empty());
    assert!(app.clipboard.row_buffer_empty());

    // Try p — row buffer is empty, should fail
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Nothing to paste"))
        .unwrap_or(false));
    // Document unchanged
    assert_eq!(app.document.row_count(), 4);
}

#[test]
fn test_comma_yy_then_capital_p_row_shows_nothing() {
    let mut app = create_test_app();

    // Yank a column
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Try P (paste row above) — row buffer empty
    app.handle_key(key(KeyCode::Char('P'))).unwrap();

    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Nothing to paste"))
        .unwrap_or(false));
}

#[test]
fn test_both_buffers_independent_after_yank() {
    let mut app = create_test_app();

    // Yank a row
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Yank a column — should NOT erase the row buffer
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    assert!(!app.clipboard.row_buffer_empty());
    assert!(!app.clipboard.column_buffer_empty());

    // Row paste should still work
    let initial_rows = app.document.row_count();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();
    assert_eq!(app.document.row_count(), initial_rows + 1);

    // Column paste should still work
    let initial_cols = app.document.column_count();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();
    assert_eq!(app.document.column_count(), initial_cols + 1);
}

#[test]
fn test_dd_stores_in_row_buffer_not_column() {
    let mut app = create_test_app();

    // Delete row with dd
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert!(!app.clipboard.row_buffer_empty());
    assert!(app.clipboard.column_buffer_empty());
}

#[test]
fn test_comma_dd_stores_in_column_buffer_not_row() {
    let mut app = create_test_app();

    // Delete column with ,dd
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert!(!app.clipboard.column_buffer_empty());
    assert!(app.clipboard.row_buffer_empty());
}

// ============================================================================
// Comma leader cancellation and unknown sequences
// ============================================================================

#[test]
fn test_comma_cancelled_with_esc() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    assert!(app.input_state.pending_command.is_some());

    app.handle_key(key(KeyCode::Esc)).unwrap();

    // Pending command cleared
    assert!(app.input_state.pending_command.is_none());
}

#[test]
fn test_comma_d_cancelled_with_esc() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    // Now in CommaD state — cancel
    app.handle_key(key(KeyCode::Esc)).unwrap();

    assert!(app.input_state.pending_command.is_none());
}

#[test]
fn test_comma_unknown_key_shows_error() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('x'))).unwrap();

    // Should show unknown command message
    assert!(app.status_message.is_some());
    assert!(app.input_state.pending_command.is_none());
}

// ============================================================================
// Delete then paste round-trip
// ============================================================================

#[test]
fn test_comma_dd_then_comma_p_round_trip() {
    let mut app = create_test_app();

    // Delete column B (index 1)
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column.get(), 1);

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.column_count(), 4);
    // Deleted column B is in column buffer
    assert_eq!(
        app.clipboard.as_column(),
        Some(vec!["B".into(), "B1".into(), "B2".into(), "B3".into()])
    );

    // Paste it back with ,p (right of current column)
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert_eq!(app.document.column_count(), 5);
    // The pasted column should contain B data
    let paste_col = app.view_state.selected_column.get();
    assert_eq!(app.document.rows[0][paste_col], "B");
    assert_eq!(app.document.rows[1][paste_col], "B1");
}

#[test]
fn test_dd_then_capital_p_round_trip() {
    let mut app = create_test_app();

    // Get row 1 content before delete
    let row1: Vec<String> = app.document.rows[1].clone();

    // Delete row 1 with dd
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    assert_eq!(app.document.row_count(), 3);

    // Paste above with P
    app.handle_key(key(KeyCode::Char('P'))).unwrap();
    assert_eq!(app.document.row_count(), 4);

    // Pasted row should match
    let pasted_idx = app.get_selected_row().unwrap().get();
    assert_eq!(app.document.rows[pasted_idx], row1);
}

// ============================================================================
// Helper: 10-data-row document for count prefix tests
// ============================================================================

/// 3-column, 10-data-row document:
///   Row 0 (header): H1  H2  H3
///   Row 1:          R1A R1B R1C
///   Row 2:          R2A R2B R2C
///   ...
///   Row 10:         R10A R10B R10C
///
/// App starts at row 1, column 0.
fn create_large_test_app() -> App {
    let headers = vec!["H1".into(), "H2".into(), "H3".into()];
    let data_rows: Vec<Vec<String>> = (1..=10)
        .map(|i| vec![format!("R{}A", i), format!("R{}B", i), format!("R{}C", i)])
        .collect();
    let doc = Document::new(headers, data_rows, "test.csv".to_string());
    let files = vec![std::path::PathBuf::from("test.csv")];
    App::new(doc, files, 0, FileConfig::new())
}

// ============================================================================
// Count prefix: dd
// ============================================================================

#[test]
fn test_5dd_deletes_5_rows() {
    let mut app = create_large_test_app();
    assert_eq!(app.document.row_count(), 11); // header + 10 data rows
    assert_eq!(app.get_selected_row().unwrap().get(), 1);

    // Type 5dd
    app.handle_key(key(KeyCode::Char('5'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.row_count(), 6); // header + 5 remaining
                                             // Row 1 should now be R6A (previously row 6)
    assert_eq!(app.document.rows[1][0], "R6A");
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("5 row(s) deleted"))
        .unwrap_or(false));
}

#[test]
fn test_dd_count_clamps_to_available_rows() {
    let mut app = create_large_test_app();
    // Move to row 9 (second-to-last data row)
    for _ in 0..8 {
        app.handle_key(key(KeyCode::Char('j'))).unwrap();
    }
    assert_eq!(app.get_selected_row().unwrap().get(), 9);

    // Type 99dd — should delete rows 9 and 10 (only 2 available), not panic
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.row_count(), 9); // header + 8 remaining
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("2 row(s) deleted"))
        .unwrap_or(false));
}

#[test]
fn test_5dd_stores_in_row_buffer_as_region() {
    let mut app = create_large_test_app();

    // 5dd from row 1
    app.handle_key(key(KeyCode::Char('5'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    let region = app.clipboard.as_region().expect("region should exist");
    assert_eq!(region.len(), 5);
    assert_eq!(region[0][0], "R1A");
    assert_eq!(region[4][0], "R5A");
    // Column buffer should be unaffected
    assert!(app.clipboard.column_buffer_empty());
}

// ============================================================================
// Count prefix: yy
// ============================================================================

#[test]
fn test_5yy_yanks_5_rows() {
    let mut app = create_large_test_app();

    // 5yy from row 1
    app.handle_key(key(KeyCode::Char('5'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Document unchanged
    assert_eq!(app.document.row_count(), 11);
    assert!(!app.document.is_dirty);

    let region = app.clipboard.as_region().expect("region should exist");
    assert_eq!(region.len(), 5);
    assert_eq!(region[0][0], "R1A");
    assert_eq!(region[4][0], "R5A");
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("5 row(s) yanked"))
        .unwrap_or(false));
}

#[test]
fn test_yy_count_clamps_to_available_rows() {
    let mut app = create_large_test_app();
    // Move to row 9
    for _ in 0..8 {
        app.handle_key(key(KeyCode::Char('j'))).unwrap();
    }
    assert_eq!(app.get_selected_row().unwrap().get(), 9);

    // 99yy — should yank rows 9 and 10 only
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    let region = app.clipboard.as_region().expect("region should exist");
    assert_eq!(region.len(), 2);
    assert_eq!(region[0][0], "R9A");
    assert_eq!(region[1][0], "R10A");
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("2 row(s) yanked"))
        .unwrap_or(false));
}

// ============================================================================
// Multi-row paste
// ============================================================================

#[test]
fn test_5yy_then_p_pastes_5_rows() {
    let mut app = create_large_test_app();

    // 5yy from row 1
    app.handle_key(key(KeyCode::Char('5'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Move to row 8
    for _ in 0..7 {
        app.handle_key(key(KeyCode::Char('j'))).unwrap();
    }
    assert_eq!(app.get_selected_row().unwrap().get(), 8);

    // p — paste below row 8
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert_eq!(app.document.row_count(), 16); // 11 + 5 pasted
                                              // Rows 9-13 should be R1A..R5A
    assert_eq!(app.document.rows[9][0], "R1A");
    assert_eq!(app.document.rows[13][0], "R5A");
    // Selection should be on last pasted row (13)
    assert_eq!(app.get_selected_row().unwrap().get(), 13);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Pasted 5 row(s)"))
        .unwrap_or(false));
}

#[test]
fn test_5yy_then_capital_p_pastes_5_rows_above() {
    let mut app = create_large_test_app();

    // 5yy from row 1
    app.handle_key(key(KeyCode::Char('5'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Move to row 8
    for _ in 0..7 {
        app.handle_key(key(KeyCode::Char('j'))).unwrap();
    }
    assert_eq!(app.get_selected_row().unwrap().get(), 8);

    // P — paste above row 8
    app.handle_key(key(KeyCode::Char('P'))).unwrap();

    assert_eq!(app.document.row_count(), 16); // 11 + 5 pasted
                                              // Rows 8-12 should be R1A..R5A
    assert_eq!(app.document.rows[8][0], "R1A");
    assert_eq!(app.document.rows[12][0], "R5A");
    // Selection stays at row 8 (first pasted row)
    assert_eq!(app.get_selected_row().unwrap().get(), 8);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Pasted 5 row(s)"))
        .unwrap_or(false));
}

// ============================================================================
// cc — clear row + enter insert mode
// ============================================================================

#[test]
fn test_cc_clears_row_enters_insert() {
    let mut app = create_test_app();
    assert_eq!(app.get_selected_row().unwrap().get(), 1);

    // Verify row 1 has content
    assert_eq!(app.document.rows[1], vec!["A1", "B1", "C1", "D1", "E1"]);

    // cc
    app.handle_key(key(KeyCode::Char('c'))).unwrap();
    app.handle_key(key(KeyCode::Char('c'))).unwrap();

    // All cells cleared
    assert_eq!(app.document.rows[1], vec!["", "", "", "", ""]);
    // Should be in insert mode
    assert_eq!(app.mode, Mode::Insert);
    // Cursor at first column
    assert_eq!(app.view_state.selected_column.get(), 0);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Row cleared"))
        .unwrap_or(false));
}

#[test]
fn test_cc_preserves_row_count() {
    let mut app = create_test_app();
    let initial_count = app.document.row_count();

    // cc
    app.handle_key(key(KeyCode::Char('c'))).unwrap();
    app.handle_key(key(KeyCode::Char('c'))).unwrap();

    // Row count unchanged (cc clears, doesn't delete)
    assert_eq!(app.document.row_count(), initial_count);
}

#[test]
fn test_pending_c_cancelled_with_esc() {
    let mut app = create_test_app();

    // c then Esc
    app.handle_key(key(KeyCode::Char('c'))).unwrap();
    assert!(app.input_state.pending_command.is_some());

    app.handle_key(key(KeyCode::Esc)).unwrap();
    assert!(app.input_state.pending_command.is_none());
    // Should still be in Normal mode
    assert_eq!(app.mode, Mode::Normal);
}

// ============================================================================
// Count prefix: ,dd (delete multiple columns)
// ============================================================================

#[test]
fn test_3_comma_dd_deletes_3_columns() {
    let mut app = create_test_app();
    assert_eq!(app.document.column_count(), 5); // A B C D E
    assert_eq!(app.view_state.selected_column.get(), 0);

    // Type 3,dd — delete columns A, B, C
    app.handle_key(key(KeyCode::Char('3'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.column_count(), 2); // D E remain
    assert_eq!(app.document.rows[0][0], "D");
    assert_eq!(app.document.rows[0][1], "E");
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("3 column(s) deleted"))
        .unwrap_or(false));
}

#[test]
fn test_comma_dd_count_clamps_to_available_columns() {
    let mut app = create_test_app();
    // Move to column 3 (D)
    for _ in 0..3 {
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
    }
    assert_eq!(app.view_state.selected_column.get(), 3);

    // 99,dd — should delete columns D and E (only 2 available), not panic
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.column_count(), 3); // A B C remain
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("2 column(s) deleted"))
        .unwrap_or(false));
}

#[test]
fn test_3_comma_dd_stores_in_column_buffer_as_region() {
    let mut app = create_test_app();

    // 3,dd from column 0 — deletes A, B, C
    app.handle_key(key(KeyCode::Char('3'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    let columns = app.clipboard.as_columns().expect("columns should exist");
    assert_eq!(columns.len(), 3);
    // First column: A, A1, A2, A3
    assert_eq!(columns[0], vec!["A", "A1", "A2", "A3"]);
    // Third column: C, C1, C2, C3
    assert_eq!(columns[2], vec!["C", "C1", "C2", "C3"]);
    // Row buffer should be unaffected
    assert!(app.clipboard.row_buffer_empty());
}

#[test]
fn test_comma_dd_adjusts_selection_when_count_removes_last_columns() {
    let mut app = create_test_app();
    // Move to column 2 (C)
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column.get(), 2);

    // 5,dd — deletes C, D, E (3 columns since only 3 remain from col 2)
    app.handle_key(key(KeyCode::Char('5'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.column_count(), 2); // A B remain
                                                // Selection should adjust to new last column (index 1)
    assert_eq!(app.view_state.selected_column.get(), 1);
}

// ============================================================================
// Count prefix: ,yy (yank multiple columns)
// ============================================================================

#[test]
fn test_3_comma_yy_yanks_3_columns() {
    let mut app = create_test_app();

    // 3,yy from column 0 — yanks A, B, C
    app.handle_key(key(KeyCode::Char('3'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Document unchanged
    assert_eq!(app.document.column_count(), 5);
    assert!(!app.document.is_dirty);

    let columns = app.clipboard.as_columns().expect("columns should exist");
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0], vec!["A", "A1", "A2", "A3"]);
    assert_eq!(columns[1], vec!["B", "B1", "B2", "B3"]);
    assert_eq!(columns[2], vec!["C", "C1", "C2", "C3"]);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("3 column(s) yanked"))
        .unwrap_or(false));
}

#[test]
fn test_comma_yy_count_clamps_to_available_columns() {
    let mut app = create_test_app();
    // Move to column 4 (E, last column)
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
    }
    assert_eq!(app.view_state.selected_column.get(), 4);

    // 99,yy — should yank only column E
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    let columns = app.clipboard.as_columns().expect("columns should exist");
    assert_eq!(columns.len(), 1);
    assert_eq!(columns[0], vec!["E", "E1", "E2", "E3"]);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("1 column(s) yanked"))
        .unwrap_or(false));
}

// ============================================================================
// Multi-column paste: ,p and ,P
// ============================================================================

#[test]
fn test_3_comma_yy_then_comma_p_pastes_3_columns() {
    let mut app = create_test_app();

    // 3,yy from column 0 — yanks A, B, C
    app.handle_key(key(KeyCode::Char('3'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Move to column 4 (E)
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
    }
    assert_eq!(app.view_state.selected_column.get(), 4);

    // ,p — paste right of E
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert_eq!(app.document.column_count(), 8); // 5 + 3 pasted
                                                // Columns 5, 6, 7 should be A, B, C copies
    assert_eq!(app.document.rows[0][5], "A");
    assert_eq!(app.document.rows[0][6], "B");
    assert_eq!(app.document.rows[0][7], "C");
    assert_eq!(app.document.rows[1][5], "A1");
    // Selection should be on first pasted column (5)
    assert_eq!(app.view_state.selected_column.get(), 5);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Pasted 3 column(s)"))
        .unwrap_or(false));
}

#[test]
fn test_3_comma_yy_then_comma_capital_p_pastes_3_columns_left() {
    let mut app = create_test_app();

    // 3,yy from column 0 — yanks A, B, C
    app.handle_key(key(KeyCode::Char('3'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Move to column 4 (E)
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
    }
    assert_eq!(app.view_state.selected_column.get(), 4);

    // ,P — paste left of E
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('P'))).unwrap();

    assert_eq!(app.document.column_count(), 8); // 5 + 3 pasted
                                                // Columns 4, 5, 6 should be A, B, C copies; original E pushed to 7
    assert_eq!(app.document.rows[0][4], "A");
    assert_eq!(app.document.rows[0][5], "B");
    assert_eq!(app.document.rows[0][6], "C");
    assert_eq!(app.document.rows[0][7], "E");
    // Selection stays at column 4 (first pasted)
    assert_eq!(app.view_state.selected_column.get(), 4);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("Pasted 3 column(s)"))
        .unwrap_or(false));
}

#[test]
fn test_3_comma_dd_then_comma_p_round_trip() {
    let mut app = create_test_app();

    // Move to column 1 (B)
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    assert_eq!(app.view_state.selected_column.get(), 1);

    // 3,dd — delete B, C, D
    app.handle_key(key(KeyCode::Char('3'))).unwrap();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.column_count(), 2); // A E remain

    // ,p — paste right of current column
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    assert_eq!(app.document.column_count(), 5); // A E B C D
                                                // Verify B, C, D are pasted back
    let paste_start = app.view_state.selected_column.get();
    assert_eq!(app.document.rows[0][paste_start], "B");
    assert_eq!(app.document.rows[1][paste_start], "B1");
    assert_eq!(app.document.rows[0][paste_start + 1], "C");
    assert_eq!(app.document.rows[0][paste_start + 2], "D");
}
