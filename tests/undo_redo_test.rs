//! Integration tests for CSV-level undo/redo (v0.10.0)
//!
//! Tests:
//! - u (undo) and Ctrl+r (redo) keybindings
//! - . (dot repeat) for cell edits and row operations
//! - :w preserves undo history
//! - File switch preserves per-file history

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

fn ctrl_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

// ============================================================================
// Undo cell edit via u
// ============================================================================

#[test]
fn test_u_undoes_cell_edit() {
    let mut app = create_test_app();
    // Select row 1, col 0 (first data row, "Alice")
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    let original = app.document.cell(RowIndex::new(1), ColIndex::new(0));
    assert_eq!(original, "Alice");

    // Edit the cell
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "Zara".into());
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Zara"
    );

    // Press u to undo
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
}

// ============================================================================
// Redo via Ctrl+r
// ============================================================================

#[test]
fn test_ctrl_r_redoes_cell_edit() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "Zara".into());

    // Undo
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );

    // Redo
    app.handle_key(ctrl_key(KeyCode::Char('r'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Zara"
    );
}

// ============================================================================
// Undo row delete (dd)
// ============================================================================

#[test]
fn test_u_undoes_row_delete() {
    let mut app = create_test_app();
    let orig_count = app.document.row_count();
    app.view_state.table_state.select(Some(2)); // row 2 = "Bob,200,B"

    // Press dd to delete row
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count - 1);

    // Undo
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count);
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
}

// ============================================================================
// Undo row insert (o)
// ============================================================================

#[test]
fn test_u_undoes_row_insert() {
    let mut app = create_test_app();
    let orig_count = app.document.row_count();
    app.view_state.table_state.select(Some(1));

    // Press o to insert row below
    app.handle_key(key(KeyCode::Char('o'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count + 1);

    // Escape back to normal mode
    app.handle_key(key(KeyCode::Esc)).unwrap();

    // Undo
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count);
}

// ============================================================================
// Undo column delete (,dd)
// ============================================================================

#[test]
fn test_u_undoes_column_delete() {
    let mut app = create_test_app();
    let orig_cols = app.document.column_count();
    app.view_state.selected_column = ColIndex::new(1); // "value" column

    // ,dd: press comma, then dd
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    assert_eq!(app.document.column_count(), orig_cols - 1);

    // Undo
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.column_count(), orig_cols);
    assert_eq!(app.document.header(ColIndex::new(1)), "value");
}

// ============================================================================
// Undo cell clear (Delete key)
// ============================================================================

#[test]
fn test_u_undoes_cell_clear() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );

    // Press Delete to clear cell
    app.handle_key(key(KeyCode::Delete)).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "");

    // Undo
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
}

// ============================================================================
// Dot repeat (.)
// ============================================================================

#[test]
fn test_dot_repeats_last_cell_edit() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    // Edit cell to "TEST"
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "TEST".into());
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "TEST"
    );

    // Move to another cell
    app.view_state.table_state.select(Some(2));

    // Press . to repeat
    app.handle_key(key(KeyCode::Char('.'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(2), ColIndex::new(0)),
        "TEST"
    );
}

#[test]
fn test_dot_with_no_previous_edit() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));

    // Press . with no prior edit
    app.handle_key(key(KeyCode::Char('.'))).unwrap();

    // Should show "No previous edit" message, no crash
    assert!(app.status_message.is_some());
}

// ============================================================================
// :w preserves undo history
// ============================================================================

#[test]
fn test_w_preserves_undo_history() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    // Make an edit
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "Edited".into());
    assert!(app.history.can_undo());

    // Save (:w)
    let _ = app.save_all_files();

    // History should still be available after save
    assert!(app.history.can_undo());

    // Undo should still work
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
}

// ============================================================================
// Multiple undo/redo cycle
// ============================================================================

#[test]
fn test_multiple_undo_redo_with_keys() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    // Edit 1
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "First".into());
    // Edit 2
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "Second".into());

    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Second"
    );

    // Undo twice
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "First"
    );
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );

    // Redo twice
    app.handle_key(ctrl_key(KeyCode::Char('r'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "First"
    );
    app.handle_key(ctrl_key(KeyCode::Char('r'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Second"
    );
}

// ============================================================================
// New edit clears redo stack
// ============================================================================

#[test]
fn test_new_edit_clears_redo() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "A".into());
    app.handle_key(key(KeyCode::Char('u'))).unwrap(); // undo
    assert!(app.history.can_redo());

    // New edit should clear redo
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "B".into());
    assert!(!app.history.can_redo());
}

// ============================================================================
// Undo at oldest shows message
// ============================================================================

#[test]
fn test_undo_at_oldest_shows_message() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));

    // No edits, undo should show message
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("oldest"));
}

#[test]
fn test_redo_at_newest_shows_message() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));

    // No undone edits, redo should show message
    app.handle_key(ctrl_key(KeyCode::Char('r'))).unwrap();
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("newest"));
}

// ============================================================================
// 5dd creates single undo step
// ============================================================================

#[test]
fn test_5dd_single_undo_step() {
    let mut app = create_test_app();
    let orig_count = app.document.row_count(); // 4 (header + 3 data)
    app.view_state.table_state.select(Some(1)); // first data row

    // Type 3dd (delete 3 data rows)
    app.handle_key(key(KeyCode::Char('3'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.document.row_count(), orig_count - 3);

    // Single undo restores all 3 rows
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count);
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
    assert_eq!(
        app.document.cell(RowIndex::new(3), ColIndex::new(0)),
        "Charlie"
    );
}

// ============================================================================
// Undo paste rows (p)
// ============================================================================

#[test]
fn test_undo_paste_rows_below() {
    let mut app = create_test_app();
    let orig_count = app.document.row_count();
    app.view_state.table_state.select(Some(1));

    // yy then p: yank row then paste below
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count + 1);

    // Undo should remove pasted row
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count);
}

#[test]
fn test_undo_paste_rows_above() {
    let mut app = create_test_app();
    let orig_count = app.document.row_count();
    app.view_state.table_state.select(Some(2));

    // yy then P: yank row then paste above
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    app.handle_key(key(KeyCode::Char('P'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count + 1);

    // Undo should remove pasted row
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count);
}

// ============================================================================
// Interleaved edits and deletes
// ============================================================================

#[test]
fn test_interleaved_cell_edit_and_row_delete_undo() {
    let mut app = create_test_app();
    let orig_count = app.document.row_count();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    // Edit cell
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "Edited".into());
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Edited"
    );

    // Delete row 2
    app.view_state.table_state.select(Some(2));
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count - 1);

    // Undo delete
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count);

    // Undo cell edit
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
}

// ============================================================================
// Undo all the way back to original state
// ============================================================================

#[test]
fn test_undo_all_restores_original() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    let orig = app.document.cell(RowIndex::new(1), ColIndex::new(0));

    // Make 5 edits
    for i in 0..5 {
        app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), format!("v{}", i));
    }

    // Undo all 5
    for _ in 0..5 {
        app.handle_key(key(KeyCode::Char('u'))).unwrap();
    }

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), orig);

    // Further undo does nothing
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), orig);
}

// ============================================================================
// Redo after partial undo, then new edit clears redo
// ============================================================================

#[test]
fn test_partial_undo_then_new_edit() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "A".into());
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "B".into());
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "C".into());

    // Undo twice (back to "A")
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "A");

    // New edit — should clear redo stack
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "D".into());
    assert!(!app.history.can_redo());

    // Can't redo to B or C anymore
    app.handle_key(ctrl_key(KeyCode::Char('r'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "D");
}

// ============================================================================
// Undo across different columns
// ============================================================================

#[test]
fn test_undo_edits_across_columns() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));

    // Edit col 0
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(0), "X".into());
    // Edit col 1
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(1), "Y".into());

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "X");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "Y");

    // Undo col 1 edit
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "X");
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "100");

    // Undo col 0 edit
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(
        app.document.cell(RowIndex::new(1), ColIndex::new(0)),
        "Alice"
    );
}

// ============================================================================
// Undo column insert (,o)
// ============================================================================

#[test]
fn test_undo_column_insert() {
    let mut app = create_test_app();
    let orig_cols = app.document.column_count();
    app.view_state.selected_column = ColIndex::new(0);

    // ,o: insert column after
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('o'))).unwrap();
    assert_eq!(app.document.column_count(), orig_cols + 1);

    // Undo
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.column_count(), orig_cols);
}

// ============================================================================
// Dot repeat row delete
// ============================================================================

#[test]
fn test_dot_repeats_row_delete() {
    let mut app = create_test_app();
    let orig_count = app.document.row_count();
    app.view_state.table_state.select(Some(1));

    // dd to delete a row
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count - 1);

    // . to repeat (delete another row)
    app.handle_key(key(KeyCode::Char('.'))).unwrap();
    assert_eq!(app.document.row_count(), orig_count - 2);
}

// ============================================================================
// Insert mode edit then undo
// ============================================================================

#[test]
fn test_insert_mode_edit_then_undo() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(1);

    // Enter insert mode with 's' (substitute — clears cell first)
    app.handle_key(key(KeyCode::Char('s'))).unwrap();

    // Type some text
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('9'))).unwrap();
    app.handle_key(key(KeyCode::Char('9'))).unwrap();

    // Commit with Tab and exit insert mode
    app.handle_key(key(KeyCode::Tab)).unwrap();
    app.handle_key(key(KeyCode::Esc)).unwrap();

    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "999");

    // Undo in normal mode
    app.handle_key(key(KeyCode::Char('u'))).unwrap();
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(1)), "100");
}
