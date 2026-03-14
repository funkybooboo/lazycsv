use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, ColIndex, Document, FileConfig};
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

fn create_test_doc() -> Document {
    Document::new(
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
        ],
        vec![
            vec![
                "A1".to_string(),
                "B1".to_string(),
                "C1".to_string(),
                "D1".to_string(),
                "E1".to_string(),
            ],
            vec![
                "A2".to_string(),
                "B2".to_string(),
                "C2".to_string(),
                "D2".to_string(),
                "E2".to_string(),
            ],
            vec![
                "A3".to_string(),
                "B3".to_string(),
                "C3".to_string(),
                "D3".to_string(),
                "E3".to_string(),
            ],
        ],
        "test.csv".to_string(),
    )
}

#[test]
fn test_delete_column_range_b_to_d() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Verify initial state: 5 columns
    assert_eq!(app.document.column_count(), 5);

    // Execute :B,Dd to delete columns B, C, D
    send_command(&mut app, "B,Dd");

    // Should have 2 columns left: A and E
    assert_eq!(app.document.column_count(), 2);

    // Check headers (row 0)
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "E"]);

    // Check data row 1 (row index 1)
    assert_eq!(app.document.get_rows_range(1, 2)[0], vec!["A1", "E1"]);
}

#[test]
fn test_yank_column_range_b_to_d() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :B,Dy to yank columns B, C, D
    send_command(&mut app, "B,Dy");

    // Columns should NOT be deleted (still have 5)
    assert_eq!(app.document.column_count(), 5);

    // Should show success message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("3") && msg.contains("column"));
}

#[test]
fn test_delete_single_column_c() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :C,Cd to delete just column C
    send_command(&mut app, "C,Cd");

    // Should have 4 columns left
    assert_eq!(app.document.column_count(), 4);

    // Check headers: A, B, D, E
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "B", "D", "E"]);
}

#[test]
fn test_delete_all_columns_a_to_e() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :A,Ed to delete all columns
    send_command(&mut app, "A,Ed");

    // Should have 0 columns
    assert_eq!(app.document.column_count(), 0);
}

#[test]
fn test_column_range_invalid_start_after_end() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :D,Bd (invalid: D > B)
    send_command(&mut app, "D,Bd");

    // Should show error message
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Invalid") || msg.contains("start"));

    // No columns should be deleted
    assert_eq!(app.document.column_count(), 5);
}

#[test]
fn test_column_range_out_of_bounds() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :D,Zd (Z doesn't exist)
    send_command(&mut app, "D,Zd");

    // Should delete D through E (clamp to max)
    assert_eq!(app.document.column_count(), 3);

    // Check headers: A, B, C
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "B", "C"]);
}

#[test]
fn test_column_range_both_out_of_bounds() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :X,Zd (both don't exist)
    send_command(&mut app, "X,Zd");

    // Should show error or delete nothing
    assert_eq!(app.document.column_count(), 5);
}

#[test]
fn test_column_range_multi_letter_columns() {
    // Create document with 30 columns (A-Z, AA-AD)
    let headers: Vec<String> = (0..30)
        .map(|i| {
            if i < 26 {
                ((b'A' + i) as char).to_string()
            } else {
                format!("A{}", (b'A' + (i - 26)) as char)
            }
        })
        .collect();

    let data_row: Vec<String> = (1..=30).map(|i| format!("val{}", i)).collect();

    let doc = Document::new(headers, vec![data_row.clone()], "test.csv".to_string());
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Delete columns Z to AB (indices 25-27)
    send_command(&mut app, "Z,ABd");

    // Should have 27 columns left (30 - 3)
    assert_eq!(app.document.column_count(), 27);
}

#[test]
fn test_column_range_cursor_adjustment_after_delete() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Move cursor to column D (index 3)
    app.view_state.selected_column = ColIndex::new(3);

    // Delete columns B,C,D
    send_command(&mut app, "B,Dd");

    // Cursor should be adjusted (column D no longer exists)
    // Should be at column 1 (E, which is now at index 1)
    assert!(app.view_state.selected_column.get() < app.document.column_count());
}

#[test]
fn test_incomplete_column_range_shows_error() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // Execute :B,D without operation (incomplete)
    send_command(&mut app, "B,D");

    // Should show error message about incomplete command
    assert!(app.status_message.is_some());
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Incomplete") || msg.contains("Unknown"));

    // No columns should be deleted
    assert_eq!(app.document.column_count(), 5);
}

// ===== Move column tests =====

#[test]
fn test_move_columns_d_e_after_a() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :D,E m A → A D E B C
    send_command(&mut app, "D,E m A");

    assert_eq!(app.document.column_count(), 5);
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "D", "E", "B", "C"]);
}

#[test]
fn test_move_column_to_end() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :A,A m E → B C D E A
    send_command(&mut app, "A,A m E");

    assert_eq!(app.document.column_count(), 5);
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["B", "C", "D", "E", "A"]);
}

#[test]
fn test_move_column_to_beginning() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :E,E m 0 → E A B C D
    send_command(&mut app, "E,E m 0");

    assert_eq!(app.document.column_count(), 5);
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["E", "A", "B", "C", "D"]);
}

#[test]
fn test_move_range_to_middle() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :D,E m B → A B D E C
    send_command(&mut app, "D,E m B");

    assert_eq!(app.document.column_count(), 5);
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "B", "D", "E", "C"]);
}

#[test]
fn test_move_noop_already_in_place() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :B,D m A → columns B-D are already after A, so no-op
    send_command(&mut app, "B,D m A");

    assert_eq!(app.document.column_count(), 5);
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "B", "C", "D", "E"]);
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("already in position"));
}

#[test]
fn test_move_invalid_range_start_after_end() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :D,B m A → invalid (D > B)
    send_command(&mut app, "D,B m A");

    assert_eq!(app.document.column_count(), 5);
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("Invalid") || msg.contains("start"));
}

#[test]
fn test_move_dest_inside_source_is_noop() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :B,D m C → destination C is inside source range B-D, no-op
    send_command(&mut app, "B,D m C");

    assert_eq!(app.document.column_count(), 5);
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "B", "C", "D", "E"]);
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("already in position"));
}

#[test]
fn test_move_cursor_follows_moved_columns() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :D,E m A → A D E B C, cursor should be at index 1 (first moved column D)
    send_command(&mut app, "D,E m A");

    assert_eq!(app.view_state.selected_column.get(), 1);
}

#[test]
fn test_move_sets_dirty() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    assert!(!app.document.is_dirty);

    send_command(&mut app, "D,E m A");

    assert!(app.document.is_dirty);
}

#[test]
fn test_move_preserves_data_rows() {
    let doc = create_test_doc();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, FileConfig::new());

    // :D,E m A → A D E B C (with data)
    send_command(&mut app, "D,E m A");

    // Headers
    assert_eq!(app.document.get_rows_range(0, 1)[0], vec!["A", "D", "E", "B", "C"]);
    // Data row 1
    assert_eq!(app.document.get_rows_range(1, 2)[0], vec!["A1", "D1", "E1", "B1", "C1"]);
    // Data row 2
    assert_eq!(app.document.get_rows_range(2, 3)[0], vec!["A2", "D2", "E2", "B2", "C2"]);
    // Data row 3
    assert_eq!(app.document.get_rows_range(3, 4)[0], vec!["A3", "D3", "E3", "B3", "C3"]);
}
