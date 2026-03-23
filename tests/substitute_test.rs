//! Integration tests for CSV substitute commands (v0.16.0)
//!
//! Tests: s/old/new/, %s/old/new/g, row ranges, column ranges, regex, undo

use std::io::Write;
use tempfile::NamedTempFile;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::session::FileConfig;
use lazycsv::{App, ColIndex, Document, RowIndex};

fn create_test_app() -> App {
    let csv = "name,value,category\nAlice,100,A\nBob,200,B\nCharlie,300,C\n";
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

fn type_command(app: &mut App, cmd: &str) {
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    for c in cmd.chars() {
        app.handle_key(key(KeyCode::Char(c))).unwrap();
    }
    app.handle_key(key(KeyCode::Enter)).unwrap();
}

// ============================================================================
// %s — replace in all cells
// ============================================================================

#[test]
fn test_percent_s_replaces_all_cells() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/Alice/Zara/");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Zara");
    // Other cells unchanged
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
}

#[test]
fn test_percent_s_global_flag() {
    let mut app = create_test_app();

    // Set a cell with repeated pattern
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "aaa".into());

    type_command(&mut app, "%s/a/X/g");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "XXX");
}

#[test]
fn test_percent_s_without_global_replaces_first_only() {
    let mut app = create_test_app();

    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "aaa".into());

    type_command(&mut app, "%s/a/X/");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Xaa");
}

#[test]
fn test_percent_s_undo() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/Alice/Zara/");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Zara");

    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
}

// ============================================================================
// s/ — replace in current cell
// ============================================================================

#[test]
fn test_s_replaces_current_cell() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    type_command(&mut app, "s/Alice/Zara/");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Zara");
    // Other rows unchanged
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
}

// ============================================================================
// Row range: 1,2s/old/new/g
// ============================================================================

#[test]
fn test_row_range_substitute() {
    let mut app = create_test_app();

    // Replace "0" with "9" in rows 1-2 only
    type_command(&mut app, "1,2s/0/9/g");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "199");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "299");
    // Row 3 unchanged
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(1)), "300");
}

#[test]
fn test_row_range_substitute_undo() {
    let mut app = create_test_app();

    type_command(&mut app, "1,2s/0/9/g");
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "100");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "200");
}

// ============================================================================
// Column range: A,Bs/old/new/g
// ============================================================================

#[test]
fn test_column_range_substitute() {
    let mut app = create_test_app();

    // Replace "B" in columns A-B (name and value columns)
    type_command(&mut app, "A,Bs/Bob/Robert/");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Robert");
    // Column C unchanged
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(2)), "B");
}

// ============================================================================
// Regex patterns
// ============================================================================

#[test]
fn test_regex_pattern() {
    let mut app = create_test_app();

    // Replace all digits with X
    type_command(&mut app, "%s/\\d+/X/g");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "X");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "X");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(1)), "X");
}

#[test]
fn test_regex_word_boundary() {
    let mut app = create_test_app();

    // Only replace whole word "A" in category column, not in "Alice"
    type_command(&mut app, "%s/^A$/Z/");
    // "Alice" should NOT be changed (not an exact match)
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    // Category "A" should be changed
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(2)), "Z");
}

// ============================================================================
// Case-insensitive flag
// ============================================================================

#[test]
fn test_case_insensitive_flag() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/alice/REPLACED/i");
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "REPLACED"
    );
}

#[test]
fn test_case_sensitive_by_default() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/alice/REPLACED/");
    // "Alice" has capital A, should NOT match without /i
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
}

// ============================================================================
// Empty replacement (delete)
// ============================================================================

#[test]
fn test_empty_replacement_deletes() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/0//g");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "1");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "2");
}

// ============================================================================
// Pattern not found
// ============================================================================

#[test]
fn test_pattern_not_found() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/NONEXISTENT/X/g");
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("not found"));
}

// ============================================================================
// Invalid regex
// ============================================================================

#[test]
fn test_invalid_regex_shows_error() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/[invalid/X/g");
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Invalid pattern"));
}

// ============================================================================
// Alternate delimiter
// ============================================================================

#[test]
fn test_alternate_delimiter() {
    let mut app = create_test_app();

    type_command(&mut app, "%s|Alice|Zara|");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Zara");
}

// ============================================================================
// Status message shows count
// ============================================================================

#[test]
fn test_status_shows_replacement_count() {
    let mut app = create_test_app();

    type_command(&mut app, "%s/0/X/g");
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("replacement"));
    assert!(msg.contains("cell"));
}
