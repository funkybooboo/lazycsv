//! Tests for triple clipboard isolation
//!
//! Verifies that the three clipboard buffers (row, column, region) remain independent
//! and cannot be cross-pasted between different visual modes.

use lazycsv::{App, ColIndex, Document, FileConfig};

fn create_test_app() -> App {
    let doc = Document::new(
        vec!["Name".into(), "Age".into(), "City".into()],
        vec![
            vec!["Alice".into(), "30".into(), "NYC".into()],
            vec!["Bob".into(), "25".into(), "LA".into()],
            vec!["Charlie".into(), "35".into(), "SF".into()],
        ],
        "test.csv".into(),
    );
    App::new(doc, vec![], 0, FileConfig::new())
}

#[test]
fn test_row_buffer_isolated_from_column_buffer() {
    let mut app = create_test_app();

    // Yank a row (Alice's row) with yy
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);
    app.clipboard.yank_rows(vec![vec![
        "Alice".to_string(),
        "30".to_string(),
        "NYC".to_string(),
    ]]);

    // Try to paste with ,p (column paste)
    // Should find nothing in column buffer
    assert!(app.clipboard.get_columns().is_none());

    // Row buffer should still have data
    assert!(app.clipboard.get_rows().is_some());
    assert_eq!(app.clipboard.get_rows().unwrap().len(), 1);
}

#[test]
fn test_column_buffer_isolated_from_row_buffer() {
    let mut app = create_test_app();

    // Yank a column (Name column) with ,yy
    app.view_state.table_state.select(Some(0));
    app.view_state.selected_column = ColIndex::new(0);
    app.clipboard.yank_columns(vec![vec![
        "Name".to_string(),
        "Alice".to_string(),
        "Bob".to_string(),
        "Charlie".to_string(),
    ]]);

    // Try to paste with p (row paste)
    // Should find nothing in row buffer
    assert!(app.clipboard.get_rows().is_none());

    // Column buffer should still have data
    assert!(app.clipboard.get_columns().is_some());
    assert_eq!(app.clipboard.get_columns().unwrap().len(), 1);
}

#[test]
fn test_region_buffer_isolated_from_row_buffer() {
    let mut app = create_test_app();

    // Yank a rectangular region (2x2 block)
    app.clipboard.yank_region(vec![
        vec!["Alice".to_string(), "30".to_string()],
        vec!["Bob".to_string(), "25".to_string()],
    ]);

    // Try to paste with p (row paste)
    // Should find nothing in row buffer
    assert!(app.clipboard.get_rows().is_none());

    // Region buffer should still have data
    assert!(app.clipboard.get_region().is_some());
    assert_eq!(app.clipboard.get_region().unwrap().len(), 2);
}

#[test]
fn test_region_buffer_isolated_from_column_buffer() {
    let mut app = create_test_app();

    // Yank a rectangular region
    app.clipboard.yank_region(vec![
        vec!["Alice".to_string(), "30".to_string()],
        vec!["Bob".to_string(), "25".to_string()],
    ]);

    // Try to paste with ,p (column paste)
    // Should find nothing in column buffer
    assert!(app.clipboard.get_columns().is_none());

    // Region buffer should still have data
    assert!(app.clipboard.get_region().is_some());
}

#[test]
fn test_yank_row_clears_row_buffer_only() {
    let mut app = create_test_app();

    // Set up all three buffers
    app.clipboard.yank_rows(vec![vec!["old_row".to_string()]]);
    app.clipboard
        .yank_columns(vec![vec!["col1".to_string(), "col2".to_string()]]);
    app.clipboard
        .yank_region(vec![vec!["reg1".to_string(), "reg2".to_string()]]);

    // Verify all buffers have data
    assert!(app.clipboard.get_rows().is_some());
    assert!(app.clipboard.get_columns().is_some());
    assert!(app.clipboard.get_region().is_some());

    // Yank a new row
    app.clipboard.yank_rows(vec![vec![
        "Alice".to_string(),
        "30".to_string(),
        "NYC".to_string(),
    ]]);

    // Row buffer should have new data
    assert_eq!(app.clipboard.get_rows().unwrap()[0][0], "Alice");

    // Column and region buffers should still have old data
    assert_eq!(app.clipboard.get_columns().unwrap()[0][0], "col1");
    assert_eq!(app.clipboard.get_region().unwrap()[0][0], "reg1");
}

#[test]
fn test_yank_column_clears_column_buffer_only() {
    let mut app = create_test_app();

    // Set up all three buffers
    app.clipboard
        .yank_rows(vec![vec!["row1".to_string(), "row2".to_string()]]);
    app.clipboard
        .yank_columns(vec![vec!["old_col".to_string()]]);
    app.clipboard
        .yank_region(vec![vec!["reg1".to_string(), "reg2".to_string()]]);

    // Yank a new column
    app.clipboard.yank_columns(vec![vec![
        "Name".to_string(),
        "Alice".to_string(),
        "Bob".to_string(),
    ]]);

    // Column buffer should have new data
    assert_eq!(app.clipboard.get_columns().unwrap()[0][0], "Name");

    // Row and region buffers should still have old data
    assert_eq!(app.clipboard.get_rows().unwrap()[0][0], "row1");
    assert_eq!(app.clipboard.get_region().unwrap()[0][0], "reg1");
}

#[test]
fn test_yank_region_clears_region_buffer_only() {
    let mut app = create_test_app();

    // Set up all three buffers
    app.clipboard
        .yank_rows(vec![vec!["row1".to_string(), "row2".to_string()]]);
    app.clipboard
        .yank_columns(vec![vec!["col1".to_string(), "col2".to_string()]]);
    app.clipboard.yank_region(vec![vec!["old_reg".to_string()]]);

    // Yank a new region
    app.clipboard.yank_region(vec![
        vec!["Alice".to_string(), "30".to_string()],
        vec!["Bob".to_string(), "25".to_string()],
    ]);

    // Region buffer should have new data
    assert_eq!(app.clipboard.get_region().unwrap()[0][0], "Alice");

    // Row and column buffers should still have old data
    assert_eq!(app.clipboard.get_rows().unwrap()[0][0], "row1");
    assert_eq!(app.clipboard.get_columns().unwrap()[0][0], "col1");
}

#[test]
fn test_no_transpose_row_to_column() {
    let mut app = create_test_app();

    // Yank a row (horizontal data: ["Alice", "30", "NYC"])
    app.clipboard.yank_rows(vec![vec![
        "Alice".to_string(),
        "30".to_string(),
        "NYC".to_string(),
    ]]);

    // Column buffer should be empty (no transpose)
    assert!(app.clipboard.get_columns().is_none());
}

#[test]
fn test_no_transpose_column_to_row() {
    let mut app = create_test_app();

    // Yank a column (vertical data: ["Name", "Alice", "Bob", "Charlie"])
    app.clipboard.yank_columns(vec![vec![
        "Name".to_string(),
        "Alice".to_string(),
        "Bob".to_string(),
        "Charlie".to_string(),
    ]]);

    // Row buffer should be empty (no transpose)
    assert!(app.clipboard.get_rows().is_none());
}

#[test]
fn test_multiple_columns_stay_in_column_buffer() {
    let mut app = create_test_app();

    // Yank multiple columns (Name and Age)
    app.clipboard.yank_columns(vec![
        vec![
            "Name".to_string(),
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ],
        vec![
            "Age".to_string(),
            "30".to_string(),
            "25".to_string(),
            "35".to_string(),
        ],
    ]);

    // Should be in column buffer
    assert!(app.clipboard.get_columns().is_some());
    assert_eq!(app.clipboard.get_columns().unwrap().len(), 2);

    // Should NOT be in row or region buffers
    assert!(app.clipboard.get_rows().is_none());
    assert!(app.clipboard.get_region().is_none());
}

#[test]
fn test_multiple_rows_stay_in_row_buffer() {
    let mut app = create_test_app();

    // Yank multiple rows
    app.clipboard.yank_rows(vec![
        vec!["Alice".to_string(), "30".to_string(), "NYC".to_string()],
        vec!["Bob".to_string(), "25".to_string(), "LA".to_string()],
    ]);

    // Should be in row buffer
    assert!(app.clipboard.get_rows().is_some());
    assert_eq!(app.clipboard.get_rows().unwrap().len(), 2);

    // Should NOT be in column or region buffers
    assert!(app.clipboard.get_columns().is_none());
    assert!(app.clipboard.get_region().is_none());
}
