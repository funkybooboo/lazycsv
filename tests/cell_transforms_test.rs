//! Integration tests for cell transforms (v0.14.0)
//!
//! Tests keybindings and commands:
//! - ~ (toggle case)
//! - g~ (title case), g. (toggle boolean), gj/gk (swap rows)
//! - :upper, :lower, :title, :trim commands
//! - Undo support for all transforms

use std::io::Write;
use tempfile::NamedTempFile;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::session::FileConfig;
use lazycsv::{App, ColIndex, Document, RowIndex};

fn create_test_app() -> App {
    let csv = "name,value,active\nalice,100,true\nBOB,200,false\nCharlie,300,yes\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv.as_bytes()).unwrap();
    let path = temp_file.path().to_path_buf();
    temp_file.keep().unwrap();

    let csv_data = Document::from_file(&path, None, false, None).unwrap();
    let file_config = FileConfig::with_options(None, false, None);
    App::new(csv_data, vec![path], 0, file_config)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

// ============================================================================
// ~ Toggle case
// ============================================================================

#[test]
fn test_tilde_toggles_lower_to_upper() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "alice"
    );

    app.handle_key(key(KeyCode::Char('~'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "ALICE"
    );
}

#[test]
fn test_tilde_toggles_upper_to_lower() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(2));
    app.view_state.selected_column = ColIndex::new(0);
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "BOB");

    app.handle_key(key(KeyCode::Char('~'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "bob");
}

#[test]
fn test_tilde_undo() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    app.handle_key(key(KeyCode::Char('~'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "ALICE"
    );

    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "alice"
    );
}

// ============================================================================
// g~ Title case
// ============================================================================

#[test]
fn test_g_tilde_title_case() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    // g then ~
    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('~'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
}

#[test]
fn test_g_tilde_title_case_all_caps() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(2));
    app.view_state.selected_column = ColIndex::new(0);

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('~'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
}

// ============================================================================
// g. Toggle boolean
// ============================================================================

#[test]
fn test_g_dot_toggles_true_to_false() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(2); // "true"

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('.'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(2)),
        "false"
    );
}

#[test]
fn test_g_dot_toggles_false_to_true() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(2));
    app.view_state.selected_column = ColIndex::new(2); // "false"

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('.'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(2), ColIndex::new(2)),
        "true"
    );
}

#[test]
fn test_g_dot_toggles_yes_to_no() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(3));
    app.view_state.selected_column = ColIndex::new(2); // "yes"

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('.'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(2)), "no");
}

#[test]
fn test_g_dot_non_boolean_shows_message() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0); // "alice" — not boolean

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('.'))).unwrap();

    // Cell unchanged
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "alice"
    );
    // Status message shown
    assert!(app.status_message.is_some());
}

#[test]
fn test_g_dot_undo() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(2);

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('.'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(2)),
        "false"
    );

    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(2)),
        "true"
    );
}

// ============================================================================
// gj/gk Swap rows
// ============================================================================

#[test]
fn test_gj_swaps_row_down() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1)); // "alice"

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    // alice and BOB should be swapped
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "BOB");
    assert_eq!(
        app.document.cell(RowIndex::new(2), ColIndex::new(0)),
        "alice"
    );
    // Cursor follows the moved row
    assert_eq!(app.view_state.table_state.selected(), Some(2));
}

#[test]
fn test_gk_swaps_row_up() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(2)); // "BOB"

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('k'))).unwrap();

    // BOB and alice should be swapped
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "BOB");
    assert_eq!(
        app.document.cell(RowIndex::new(2), ColIndex::new(0)),
        "alice"
    );
    // Cursor follows the moved row
    assert_eq!(app.view_state.table_state.selected(), Some(1));
}

#[test]
fn test_gj_undo() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "BOB");

    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "alice"
    );
}

#[test]
fn test_gk_at_first_data_row_does_nothing() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1)); // first data row

    let orig = app.document.cell(RowIndex::new(1), ColIndex::new(0));

    // gk should not swap with header
    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('k'))).unwrap();

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), orig);
}

// ============================================================================
// :upper, :lower, :title, :trim commands
// ============================================================================

fn type_command(app: &mut App, cmd: &str) {
    // Enter command mode
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    // Type command
    for c in cmd.chars() {
        app.handle_key(key(KeyCode::Char(c))).unwrap();
    }
    // Execute
    app.handle_key(key(KeyCode::Enter)).unwrap();
}

#[test]
fn test_upper_command() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    type_command(&mut app, "upper");
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "ALICE"
    );
}

#[test]
fn test_lower_command() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(2));
    app.view_state.selected_column = ColIndex::new(0);

    type_command(&mut app, "lower");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "bob");
}

#[test]
fn test_title_command() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    type_command(&mut app, "title");
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
}

#[test]
fn test_trim_command() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    // First set a value with whitespace
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "  alice  ".into());
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "  alice  "
    );

    type_command(&mut app, "trim");
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "alice"
    );
}

#[test]
fn test_upper_command_undo() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    type_command(&mut app, "upper");
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "ALICE"
    );

    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "alice"
    );
}
