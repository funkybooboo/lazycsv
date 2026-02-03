//! Common test utilities and helper functions

#![allow(dead_code)] // Test utilities may not all be used yet

use lazycsv::Document;
use std::fs::write;
use tempfile::NamedTempFile;

/// Create a large CSV Document programmatically for testing
pub fn create_large_csv(rows: usize, cols: usize) -> Document {
    let headers = (0..cols).map(|i| format!("Col{}", i)).collect();
    let rows_data = (0..rows)
        .map(|r| (0..cols).map(|c| format!("R{}C{}", r, c)).collect())
        .collect();
    Document {
        headers,
        rows: rows_data,
        filename: "large.csv".to_string(),
        is_dirty: false,
    }
}

/// Create a temporary CSV file with the given content
pub fn create_temp_csv_file(content: &str) -> NamedTempFile {
    let file = NamedTempFile::new().expect("Failed to create temp file");
    write(file.path(), content).expect("Failed to write to temp file");
    file
}

/// Create a CSV Document for testing with specified dimensions
pub fn create_test_document(rows: usize, cols: usize) -> Document {
    create_large_csv(rows, cols)
}

/// Create a CSV file with alternating empty and filled cells
pub fn create_alternating_csv() -> NamedTempFile {
    create_temp_csv_file("A,B,C,D,E\nval1,,val2,,val3\n,,val4,,\n")
}

/// Create a CSV file with all empty cells in data rows
pub fn create_all_empty_csv() -> NamedTempFile {
    create_temp_csv_file("A,B,C\n,,\n,,\n")
}

/// Create a CSV file with malformed quotes
pub fn create_malformed_quotes_csv() -> NamedTempFile {
    create_temp_csv_file("A,B,C\nvalue1,\"unclosed,value3\n")
}

/// Create a CSV file with inconsistent column counts
pub fn create_inconsistent_columns_csv() -> NamedTempFile {
    create_temp_csv_file("A,B,C\n1,2,3\n4,5,6,7,8\n")
}

/// Create a CSV file with control characters
pub fn create_control_chars_csv() -> NamedTempFile {
    create_temp_csv_file("A,B,C\n\tvalue1\t,\rvalue2\r,value3\n")
}

/// Create a very wide CSV file (100+ columns)
pub fn create_wide_csv(cols: usize) -> NamedTempFile {
    let headers = (0..cols)
        .map(|i| format!("Col{}", i))
        .collect::<Vec<_>>()
        .join(",");
    let row = (0..cols)
        .map(|i| format!("val{}", i))
        .collect::<Vec<_>>()
        .join(",");
    let content = format!("{}\n{}\n", headers, row);
    create_temp_csv_file(&content)
}

/// Generate a large CSV file content as a String (for performance testing)
pub fn generate_large_csv_content(rows: usize, cols: usize) -> String {
    let headers = (0..cols)
        .map(|i| format!("Col{}", i))
        .collect::<Vec<_>>()
        .join(",");

    let mut content = headers + "\n";
    for r in 0..rows {
        let row = (0..cols)
            .map(|c| format!("R{}C{}", r, c))
            .collect::<Vec<_>>()
            .join(",");
        content.push_str(&row);
        content.push('\n');
    }
    content
}

/// Create a temporary CSV file with large content
pub fn create_large_temp_csv(rows: usize, cols: usize) -> NamedTempFile {
    let content = generate_large_csv_content(rows, cols);
    create_temp_csv_file(&content)
}
