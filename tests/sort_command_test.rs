use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, ColIndex, Document, FileConfig, InputResult, RowIndex};
use std::path::PathBuf;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Helper to send command string to app
/// Handles deferred operations like SortDocument by executing them inline
fn send_command(app: &mut App, cmd: &str) {
    let _ = app.handle_key(key_event(KeyCode::Char(':')));
    for c in cmd.chars() {
        let _ = app.handle_key(key_event(KeyCode::Char(c)));
    }
    if let Ok(InputResult::SortDocument {
        col_indices,
        ascending,
        description,
    }) = app.handle_key(key_event(KeyCode::Enter))
    {
        app.document.sort_by_columns(&col_indices, ascending);
        let current_file = app.current_file().clone();
        app.session.mark_dirty(&current_file);
        let direction = if ascending { "ascending" } else { "descending" };
        app.status_message = Some(lazycsv::input::StatusMessage::from(format!(
            "Sorted by {} {}",
            description, direction
        )));
    }
}

/// Helper to create a test app with Name, Age, City columns
fn create_test_app() -> App {
    let doc = Document::new(
        vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
        vec![
            vec!["Charlie".to_string(), "25".to_string(), "NYC".to_string()],
            vec!["Alice".to_string(), "30".to_string(), "LA".to_string()],
            vec!["Bob".to_string(), "20".to_string(), "NYC".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    App::new(doc, files, 0, FileConfig::new())
}

// ===== :sort (ascending) =====

#[test]
fn test_sort_ascending_by_column_number() {
    let mut app = create_test_app();

    send_command(&mut app, "sort 1");

    // Column 1 is Name — sorted ascending: Alice, Bob, Charlie
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Charlie");
}

#[test]
fn test_sort_ascending_by_header_name() {
    let mut app = create_test_app();

    send_command(&mut app, "sort Name");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Charlie");
}

#[test]
fn test_sort_ascending_header_name_case_insensitive() {
    let mut app = create_test_app();

    send_command(&mut app, "sort name");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Charlie");
}

#[test]
fn test_sort_ascending_numeric_column() {
    let mut app = create_test_app();

    // Age column: 25, 30, 20 → sorted numerically: 20, 25, 30
    send_command(&mut app, "sort Age");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "20");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "25");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(1)), "30");
}

// ===== :sort! (descending) =====

#[test]
fn test_sort_descending_by_column_number() {
    let mut app = create_test_app();

    send_command(&mut app, "sort! 1");

    // Name descending: Charlie, Bob, Alice
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Charlie");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Alice");
}

#[test]
fn test_sort_descending_by_header_name() {
    let mut app = create_test_app();

    send_command(&mut app, "sort! Age");

    // Age descending numerically: 30, 25, 20
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "30");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "25");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(1)), "20");
}

// ===== Multi-column sort =====

#[test]
fn test_sort_multi_column() {
    let doc = Document::new(
        vec!["City".to_string(), "Name".to_string()],
        vec![
            vec!["NYC".to_string(), "Charlie".to_string()],
            vec!["LA".to_string(), "Alice".to_string()],
            vec!["NYC".to_string(), "Bob".to_string()],
            vec!["LA".to_string(), "Dave".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Sort by City, then Name
    send_command(&mut app, "sort City,Name");

    // LA comes first, then NYC; within each city, names are alphabetical
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "LA");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "LA");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "Dave");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "NYC");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(1)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(4), ColIndex::new(0)), "NYC");
    assert_eq!(app.document.cell(RowIndex::new(4), ColIndex::new(1)), "Charlie");
}

#[test]
fn test_sort_multi_column_by_number() {
    let doc = Document::new(
        vec!["City".to_string(), "Name".to_string()],
        vec![
            vec!["NYC".to_string(), "Charlie".to_string()],
            vec!["LA".to_string(), "Alice".to_string()],
            vec!["NYC".to_string(), "Bob".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Sort by column 1 (City), then column 2 (Name)
    send_command(&mut app, "sort 1,2");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "LA");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "NYC");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "NYC");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(1)), "Charlie");
}

// ===== Header stays fixed =====

#[test]
fn test_sort_preserves_header() {
    let mut app = create_test_app();

    send_command(&mut app, "sort 1");

    // Header row should remain unchanged
    assert_eq!(app.document.cell(RowIndex::new(0), ColIndex::new(0)), "Name");
    assert_eq!(app.document.cell(RowIndex::new(0), ColIndex::new(1)), "Age");
    assert_eq!(app.document.cell(RowIndex::new(0), ColIndex::new(2)), "City");
}

// ===== Dirty flag =====

#[test]
fn test_sort_marks_dirty() {
    let mut app = create_test_app();
    assert!(!app.document.is_dirty);

    send_command(&mut app, "sort 1");

    assert!(app.document.is_dirty);
}

// ===== Status messages =====

#[test]
fn test_sort_shows_success_message() {
    let mut app = create_test_app();

    send_command(&mut app, "sort Name");

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Sorted"));
    assert!(msg.contains("ascending"));
}

#[test]
fn test_sort_bang_shows_descending_message() {
    let mut app = create_test_app();

    send_command(&mut app, "sort! Name");

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Sorted"));
    assert!(msg.contains("descending"));
}

// ===== Error cases =====

#[test]
fn test_sort_no_arg_shows_usage() {
    let mut app = create_test_app();

    send_command(&mut app, "sort");

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Usage"));
}

#[test]
fn test_sort_column_out_of_range() {
    let mut app = create_test_app();

    send_command(&mut app, "sort 999");

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("out of range"));
}

#[test]
fn test_sort_column_zero_out_of_range() {
    let mut app = create_test_app();

    send_command(&mut app, "sort 0");

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("out of range"));
}

#[test]
fn test_sort_unknown_column_name() {
    let mut app = create_test_app();

    send_command(&mut app, "sort BadName");

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("not found"));
}

// ===== Numeric-aware sorting =====

#[test]
fn test_sort_numeric_not_lexicographic() {
    let doc = Document::new(
        vec!["Value".to_string()],
        vec![
            vec!["10".to_string()],
            vec!["2".to_string()],
            vec!["1".to_string()],
            vec!["20".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "sort 1");

    // Numeric sort: 1, 2, 10, 20 (not lexicographic: 1, 10, 2, 20)
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "1");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "2");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "10");
    assert_eq!(app.document.cell(RowIndex::new(4), ColIndex::new(0)), "20");
}

// ===== Edge cases =====

#[test]
fn test_sort_single_data_row_no_op() {
    let doc = Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "sort 1");

    // Should not crash, row stays the same
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "1");
}

#[test]
fn test_sort_empty_cells() {
    let doc = Document::new(
        vec!["Name".to_string()],
        vec![
            vec!["Bob".to_string()],
            vec!["".to_string()],
            vec!["Alice".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    send_command(&mut app, "sort 1");

    // Empty string sorts before other strings
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Bob");
}

// ===== Column letter specifiers =====

#[test]
fn test_sort_by_column_letter() {
    let mut app = create_test_app();

    // Column A is Name — sort ascending: Alice, Bob, Charlie
    send_command(&mut app, "sort A");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Charlie");
}

#[test]
fn test_sort_by_column_letter_lowercase() {
    let mut app = create_test_app();

    send_command(&mut app, "sort a");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Charlie");
}

#[test]
fn test_sort_by_multiple_column_letters() {
    let doc = Document::new(
        vec!["City".to_string(), "Name".to_string()],
        vec![
            vec!["NYC".to_string(), "Charlie".to_string()],
            vec!["LA".to_string(), "Alice".to_string()],
            vec!["NYC".to_string(), "Bob".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Sort by column A (City), then column B (Name)
    send_command(&mut app, "sort A,B");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "LA");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "Alice");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "NYC");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(1)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "NYC");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(1)), "Charlie");
}

#[test]
fn test_sort_descending_by_column_letter() {
    let mut app = create_test_app();

    send_command(&mut app, "sort! A");

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "Charlie");
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "Alice");
}

#[test]
fn test_sort_column_letter_out_of_range() {
    let mut app = create_test_app();

    // Only 3 columns (A, B, C) — ZZ is out of range
    send_command(&mut app, "sort ZZ");

    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("not found"));
}

// ===== No regression: :c column jump still works =====

#[test]
fn test_c_column_jump_not_broken() {
    let mut app = create_test_app();

    // :cB should still jump to column B (index 1)
    send_command(&mut app, "cB");

    assert_eq!(app.view_state.selected_column.get(), 1);
}
