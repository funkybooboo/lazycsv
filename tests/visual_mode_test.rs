//! Integration tests for visual mode (Block, Line, Column)
//!
//! Tests:
//! - Visual mode entry (v, V, ,v)
//! - Visual mode movement (hjkl, arrows)
//! - Visual operations (d, y, p)
//! - Visual mode exit (Esc, after operation)
//! - gv (reselect last selection)
//! - Edge cases and boundary conditions

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::Mode;
use lazycsv::{App, ColIndex, Document, FileConfig, RowIndex};
use std::path::PathBuf;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Create test app with 5 columns, 5 rows (including header)
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
            vec![
                "A4".into(),
                "B4".into(),
                "C4".into(),
                "D4".into(),
                "E4".into(),
            ],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    App::new(doc, files, 0, FileConfig::new())
}

// ============================================================================
// Visual Block Mode (v) - Entry and Movement
// ============================================================================

#[test]
fn test_v_enters_visual_block_mode() {
    let mut app = create_test_app();
    assert_eq!(app.mode, Mode::Normal);

    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    assert_eq!(app.mode, Mode::VisualBlock);
    assert!(app.visual_selection.is_some());
}

#[test]
fn test_visual_block_movement_extends_selection() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();

    // Move right
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    let sel = app.visual_selection.as_ref().unwrap();
    let (_, _, start_col, end_col) = sel.bounds();
    assert_eq!(end_col.get() - start_col.get(), 1);

    // Move down
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    let sel = app.visual_selection.as_ref().unwrap();
    let (start_row, end_row, _, _) = sel.bounds();
    assert_eq!(end_row.get() - start_row.get(), 1);
}

#[test]
fn test_visual_block_arrow_keys_work() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();

    app.handle_key(key(KeyCode::Right)).unwrap();
    app.handle_key(key(KeyCode::Down)).unwrap();

    let sel = app.visual_selection.as_ref().unwrap();
    let (start_row, end_row, start_col, end_col) = sel.bounds();
    assert!(end_row > start_row);
    assert!(end_col > start_col);
}

#[test]
fn test_visual_block_esc_exits() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    assert_eq!(app.mode, Mode::VisualBlock);

    app.handle_key(key(KeyCode::Esc)).unwrap();
    assert_eq!(app.mode, Mode::Normal);
    assert!(app.visual_selection.is_none());
}

// ============================================================================
// Visual Line Mode (V) - Entry and Movement
// ============================================================================

#[test]
fn test_shift_v_enters_visual_line_mode() {
    let mut app = create_test_app();
    assert_eq!(app.mode, Mode::Normal);

    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    assert_eq!(app.mode, Mode::VisualLine);
    assert!(app.visual_selection.is_some());
}

#[test]
fn test_visual_line_j_extends_selection() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char('V'))).unwrap();

    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    let sel = app.visual_selection.as_ref().unwrap();
    let (start_row, end_row, _, _) = sel.bounds();
    assert_eq!(end_row.get() - start_row.get(), 1);
}

#[test]
fn test_visual_line_k_extends_selection_upward() {
    let mut app = create_test_app();
    // Move to row 3 first
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('k'))).unwrap();

    let sel = app.visual_selection.as_ref().unwrap();
    let (start_row, end_row, _, _) = sel.bounds();
    assert_eq!(end_row.get() - start_row.get(), 1);
}

#[test]
fn test_visual_line_esc_exits() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    assert_eq!(app.mode, Mode::VisualLine);

    app.handle_key(key(KeyCode::Esc)).unwrap();
    assert_eq!(app.mode, Mode::Normal);
}

// ============================================================================
// Visual Column Mode (,v) - Entry and Movement
// ============================================================================

#[test]
fn test_comma_v_enters_visual_column_mode() {
    let mut app = create_test_app();
    assert_eq!(app.mode, Mode::Normal);

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    assert_eq!(app.mode, Mode::VisualColumn);
    assert!(app.visual_selection.is_some());
}

#[test]
fn test_visual_column_l_extends_selection() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();

    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    let sel = app.visual_selection.as_ref().unwrap();
    let (_, _, start_col, end_col) = sel.bounds();
    assert_eq!(end_col.get() - start_col.get(), 1);
}

#[test]
fn test_visual_column_h_extends_selection_leftward() {
    let mut app = create_test_app();
    // Move to column 2 first
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('h'))).unwrap();

    let sel = app.visual_selection.as_ref().unwrap();
    let (_, _, start_col, end_col) = sel.bounds();
    assert_eq!(end_col.get() - start_col.get(), 1);
}

#[test]
fn test_visual_column_esc_exits() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    assert_eq!(app.mode, Mode::VisualColumn);

    app.handle_key(key(KeyCode::Esc)).unwrap();
    assert_eq!(app.mode, Mode::Normal);
}

// ============================================================================
// Visual Block - Delete Operation
// ============================================================================

#[test]
fn test_visual_block_d_deletes_cells() {
    let mut app = create_test_app();

    // Select 2x2 block starting at A1
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    let initial_rows = app.document.row_count();
    let initial_cols = app.document.column_count();

    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    // Structure preserved (rows and columns not deleted)
    assert_eq!(app.document.row_count(), initial_rows);
    assert_eq!(app.document.column_count(), initial_cols);

    // Cells are cleared
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(1), lazycsv::ColIndex::new(0)),
        ""
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(1), lazycsv::ColIndex::new(1)),
        ""
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(2), lazycsv::ColIndex::new(0)),
        ""
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(2), lazycsv::ColIndex::new(1)),
        ""
    );

    // Adjacent cells unchanged
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(1), lazycsv::ColIndex::new(2)),
        "C1"
    );

    // Back to normal mode
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_visual_block_d_stores_in_region_buffer() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    let region = app.clipboard.region().expect("region should exist");
    assert_eq!(region.len(), 2); // 2 rows
    assert_eq!(region[0].len(), 2); // 2 columns
    assert_eq!(region[0][0], "A1");
    assert_eq!(region[0][1], "B1");
    assert_eq!(region[1][0], "A2");
    assert_eq!(region[1][1], "B2");
}

// ============================================================================
// Visual Line - Delete Operation
// ============================================================================

#[test]
fn test_visual_line_d_deletes_rows() {
    let mut app = create_test_app();

    // Select rows 1-2
    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    let initial_rows = app.document.row_count();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    // 2 rows deleted
    assert_eq!(app.document.row_count(), initial_rows - 2);

    // First data row is now what was row 3
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "A3");

    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_visual_line_d_stores_in_row_buffer() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    let rows = app.clipboard.rows().expect("rows should exist");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], "A1");
    assert_eq!(rows[1][0], "A2");
}

// ============================================================================
// Visual Column - Delete Operation
// ============================================================================

#[test]
fn test_visual_column_d_deletes_columns() {
    let mut app = create_test_app();

    // Select columns 0-1 (A-B)
    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();

    let initial_cols = app.document.column_count();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    // 2 columns deleted
    assert_eq!(app.document.column_count(), initial_cols - 2);

    // First column is now what was column C
    assert_eq!(app.document.header(lazycsv::ColIndex::new(0)), "C");

    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_visual_column_d_stores_in_column_buffer() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    let cols = app.clipboard.columns().expect("columns should exist");
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0][0], "A"); // Header
    assert_eq!(cols[0][1], "A1"); // First data row
}

// ============================================================================
// Visual Block - Yank Operation
// ============================================================================

#[test]
fn test_visual_block_y_yanks_region() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    let initial_rows = app.document.row_count();
    let initial_cols = app.document.column_count();

    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Document unchanged
    assert_eq!(app.document.row_count(), initial_rows);
    assert_eq!(app.document.column_count(), initial_cols);
    assert!(!app.document.is_dirty);

    // Region in clipboard
    let region = app.clipboard.region().expect("region should exist");
    assert_eq!(region.len(), 2);
    assert_eq!(region[0][0], "A1");

    assert_eq!(app.mode, Mode::Normal);
}

// ============================================================================
// Visual Line - Yank Operation
// ============================================================================

#[test]
fn test_visual_line_y_yanks_rows() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    let initial_rows = app.document.row_count();

    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Document unchanged
    assert_eq!(app.document.row_count(), initial_rows);
    assert!(!app.document.is_dirty);

    // Rows in clipboard
    let rows = app.clipboard.rows().expect("rows should exist");
    assert_eq!(rows.len(), 2);

    assert_eq!(app.mode, Mode::Normal);
}

// ============================================================================
// Visual Column - Yank Operation
// ============================================================================

#[test]
fn test_visual_column_y_yanks_columns() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();

    let initial_cols = app.document.column_count();

    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Document unchanged
    assert_eq!(app.document.column_count(), initial_cols);
    assert!(!app.document.is_dirty);

    // Columns in clipboard
    let cols = app.clipboard.columns().expect("columns should exist");
    assert_eq!(cols.len(), 2);

    assert_eq!(app.mode, Mode::Normal);
}

// ============================================================================
// Visual Block - Paste Operation
// ============================================================================

#[test]
fn test_visual_block_p_pastes_over_selection() {
    let mut app = create_test_app();

    // Yank a 2x2 region from A1
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Move to C3
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();

    // Select and paste
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    // Check pasted content
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(3), lazycsv::ColIndex::new(2)),
        "A1"
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(3), lazycsv::ColIndex::new(3)),
        "B1"
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(4), lazycsv::ColIndex::new(2)),
        "A2"
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(4), lazycsv::ColIndex::new(3)),
        "B2"
    );
}

// ============================================================================
// Visual Line - Paste Operation
// ============================================================================

#[test]
fn test_visual_line_p_replaces_rows() {
    let mut app = create_test_app();

    // Yank row 1
    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Move to row 3
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    // Select and paste over row 3
    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('p'))).unwrap();

    // Row 3 should now contain row 1's content
    assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "A1");
}

// ============================================================================
// gv - Reselect Last Selection
// ============================================================================

#[test]
fn test_gv_reselects_visual_block() {
    let mut app = create_test_app();

    // Make a selection and delete
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    assert_eq!(app.mode, Mode::Normal);

    // Reselect
    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();

    assert_eq!(app.mode, Mode::VisualBlock);
    let sel = app.visual_selection.as_ref().unwrap();
    let (start_row, end_row, start_col, end_col) = sel.bounds();
    assert_eq!(end_row.get() - start_row.get(), 1);
    assert_eq!(end_col.get() - start_col.get(), 1);
}

#[test]
fn test_gv_reselects_visual_line() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Reselect
    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();

    assert_eq!(app.mode, Mode::VisualLine);
}

#[test]
fn test_gv_reselects_visual_column() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();

    // Reselect
    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();

    assert_eq!(app.mode, Mode::VisualColumn);
}

#[test]
fn test_gv_without_previous_selection_shows_error() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('g'))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();

    assert_eq!(app.mode, Mode::Normal);
    assert!(app
        .status_message
        .as_ref()
        .map(|m| m.as_str().contains("No previous"))
        .unwrap_or(false));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_visual_block_single_cell_selection() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    // Don't move, just delete
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    // Single cell cleared
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(1), lazycsv::ColIndex::new(0)),
        ""
    );
    assert_eq!(
        app.document
            .cell(lazycsv::RowIndex::new(1), lazycsv::ColIndex::new(1)),
        "B1"
    );
}

#[test]
fn test_visual_line_single_row() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    // Single row deleted
    let initial_rows = 5;
    assert_eq!(app.document.row_count(), initial_rows - 1);
}

#[test]
fn test_visual_column_single_column() {
    let mut app = create_test_app();

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();

    // Single column deleted
    let initial_cols = 5;
    assert_eq!(app.document.column_count(), initial_cols - 1);
}

#[test]
fn test_visual_block_at_boundary() {
    let mut app = create_test_app();

    // Move to last cell
    for _ in 0..3 {
        app.handle_key(key(KeyCode::Char('j'))).unwrap();
    }
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Char('l'))).unwrap();
    }

    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    // Try to move beyond boundary
    app.handle_key(key(KeyCode::Char('l'))).unwrap();
    app.handle_key(key(KeyCode::Char('j'))).unwrap();

    // Should stay at last cell
    let sel = app.visual_selection.as_ref().unwrap();
    let (_, _, _, end_col) = sel.bounds();
    assert_eq!(end_col.get(), 4);
}

#[test]
fn test_visual_modes_save_to_last_selection() {
    let mut app = create_test_app();

    // All three modes should save
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('d'))).unwrap();
    assert!(app.last_visual_selection.is_some());

    app.handle_key(key(KeyCode::Char('V'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    assert!(app.last_visual_selection.is_some());

    app.handle_key(key(KeyCode::Char(','))).unwrap();
    app.handle_key(key(KeyCode::Char('v'))).unwrap();
    app.handle_key(key(KeyCode::Char('y'))).unwrap();
    assert!(app.last_visual_selection.is_some());
}
