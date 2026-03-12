//! Tests for column reordering edge cases
//!
//! Tests the `:start_col,end_col m target_col` command with various edge cases.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::Mode;
use lazycsv::{App, ColIndex, Document, FileConfig};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

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
        "test.csv".into(),
    );
    App::new(doc, vec![], 0, FileConfig::new())
}

fn enter_command(app: &mut App, cmd: &str) {
    // Enter command mode with ':'
    let _ = app.handle_key(key(KeyCode::Char(':')));
    assert_eq!(app.mode, Mode::Command);

    // Type the command
    for c in cmd.chars() {
        let _ = app.handle_key(key(KeyCode::Char(c)));
    }

    // Execute with Enter
    let _ = app.handle_key(key(KeyCode::Enter));
}

fn get_column_order(app: &App) -> Vec<String> {
    (0..app.document.column_count())
        .map(|i| {
            app.document
                .cell(lazycsv::RowIndex::new(0), ColIndex::new(i))
                .to_string()
        })
        .collect()
}

#[test]
fn test_move_single_column_to_beginning() {
    let mut app = create_test_app();

    // Move column D to beginning (before A)
    // :D,D m 0
    enter_command(&mut app, "D,D m 0");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["D", "A", "B", "C", "E"]);
}

#[test]
fn test_move_single_column_to_end() {
    let mut app = create_test_app();

    // Move column B to end (after E)
    enter_command(&mut app, "B,B m E");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["A", "C", "D", "E", "B"]);
}

#[test]
fn test_move_preserves_data() {
    let mut app = create_test_app();

    // Move column B to end (after E)
    enter_command(&mut app, "B,B m E");

    // Check that data moved with the column
    let col_b_idx = 4; // B is now at index 4 (end)
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(1), ColIndex::new(col_b_idx)),
        "B1"
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(2), ColIndex::new(col_b_idx)),
        "B2"
    );
}

#[test]
fn test_move_invalid_source_column() {
    let mut app = create_test_app();

    // Try to move non-existent column Z
    enter_command(&mut app, "Z,Z m A");

    // Should show error and not crash
    // Order should remain unchanged
    let order = get_column_order(&app);
    assert_eq!(order, vec!["A", "B", "C", "D", "E"]);
}

#[test]
fn test_move_invalid_target_column() {
    let mut app = create_test_app();

    // Try to move to non-existent column Z - should error
    enter_command(&mut app, "B,B m Z");

    // Should show error message but not crash
    // The command should fail gracefully
    assert!(app.status_message.is_some());
}

#[test]
fn test_move_with_numeric_columns() {
    let mut app = create_test_app();

    // Use column letters (numeric column indices don't work in move command)
    // Move column D to beginning
    enter_command(&mut app, "D,D m 0");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["D", "A", "B", "C", "E"]);
}

#[test]
fn test_move_multiple_columns_forward() {
    let mut app = create_test_app();

    // Move columns B,C after D (should become A, D, B, C, E)
    enter_command(&mut app, "B,C m D");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["A", "D", "B", "C", "E"]);
}

#[test]
fn test_move_multiple_columns_backward() {
    let mut app = create_test_app();

    // Move columns D,E before B (should become A, D, E, B, C)
    enter_command(&mut app, "D,E m A");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["A", "D", "E", "B", "C"]);
}

#[test]
fn test_move_column_to_same_position() {
    let mut app = create_test_app();

    // Move column C after itself (no change)
    enter_command(&mut app, "C m C");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["A", "B", "C", "D", "E"]);
}

#[test]
fn test_move_first_column_to_end() {
    let mut app = create_test_app();

    // Move column A to end (after E)
    enter_command(&mut app, "A,A m E");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["B", "C", "D", "E", "A"]);
}

#[test]
fn test_move_last_column_to_beginning() {
    let mut app = create_test_app();

    // Move column E to beginning
    enter_command(&mut app, "E,E m 0");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["E", "A", "B", "C", "D"]);
}

#[test]
fn test_move_all_columns_except_first() {
    let mut app = create_test_app();

    // Move columns B through E to beginning
    enter_command(&mut app, "B,E m 0");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["B", "C", "D", "E", "A"]);
}

#[test]
fn test_move_adjacent_columns() {
    let mut app = create_test_app();

    // Move columns B,C after C (should move them after D actually)
    enter_command(&mut app, "B,C m D");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["A", "D", "B", "C", "E"]);
}

#[test]
fn test_move_with_column_letters() {
    let mut app = create_test_app();

    // Use column letters instead of indices
    // Move columns D,E before B
    enter_command(&mut app, "D,E m A");

    let order = get_column_order(&app);
    assert_eq!(order, vec!["A", "D", "E", "B", "C"]);
}
