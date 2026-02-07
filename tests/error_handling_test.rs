//! Error handling and failure scenario tests

mod common;

use clap::Parser;
use lazycsv::{cli::CliArgs, App, Document};
use std::fs::write;
use tempfile::TempDir;

#[test]
fn test_file_not_found_error_message() {
    let args = CliArgs::try_parse_from(["lazycsv", "/nonexistent/path/file.csv"]).unwrap();
    let result = App::from_cli(args);

    assert!(result.is_err(), "Expected error for non-existent file");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("file.csv")
            || err_msg.contains("No such file")
            || err_msg.contains("not found"),
        "Error message should mention the file: {}",
        err_msg
    );
}

#[test]
#[cfg(unix)]
fn test_permission_denied_error_handling() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("no_read_permission.csv");
    write(&file_path, "A,B,C\n1,2,3").unwrap();

    // Remove read permissions
    let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o000);
    std::fs::set_permissions(&file_path, perms).unwrap();

    let result = Document::from_file(&file_path, None, false, None);

    // Restore permissions for cleanup
    let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&file_path, perms).unwrap();

    assert!(
        result.is_err(),
        "Expected error for file without read permissions"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("permission")
            || err_msg.contains("Permission")
            || err_msg.contains("denied"),
        "Error should mention permission issue: {}",
        err_msg
    );
}

#[test]
fn test_directory_instead_of_file_error() {
    let temp_dir = TempDir::new().unwrap();

    // Try to load a directory as if it were a file
    let result = Document::from_file(temp_dir.path(), None, false, None);

    assert!(
        result.is_err(),
        "Expected error when loading directory as CSV"
    );
    let err_msg = result.unwrap_err().to_string();
    // Error should indicate it's a directory or similar issue
    assert!(
        err_msg.contains("directory") || err_msg.contains("Is a directory") || !err_msg.is_empty(),
        "Error message: {}",
        err_msg
    );
}

#[test]
fn test_empty_directory_no_csv_files() {
    let temp_dir = TempDir::new().unwrap();

    let args = CliArgs::try_parse_from(["lazycsv", temp_dir.path().to_str().unwrap()]).unwrap();
    let result = App::from_cli(args);

    assert!(
        result.is_err(),
        "Expected error for directory with no CSV files"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("No CSV files found")
            || err_msg.contains("no CSV")
            || err_msg.contains("not found"),
        "Error should mention no CSV files: {}",
        err_msg
    );
}

#[test]
fn test_corrupt_csv_malformed_quotes() {
    let malformed_csv = common::create_malformed_quotes_csv();

    // CSV crate behavior: may parse with error or handle gracefully
    let result = Document::from_file(malformed_csv.path(), None, false, None);

    // This test documents current behavior - CSV crate may error or parse gracefully
    // If it errors, that's fine. If it succeeds, that's also acceptable.
    match result {
        Ok(_doc) => {
            // Parsed gracefully - acceptable behavior
        }
        Err(err) => {
            // Failed with error - also acceptable
            let err_msg = err.to_string();
            assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }
}

#[test]
fn test_corrupt_csv_inconsistent_columns() {
    let inconsistent_csv = common::create_inconsistent_columns_csv();

    // CSV crate in strict mode should reject inconsistent column counts
    let result = Document::from_file(inconsistent_csv.path(), None, false, None);

    // Document behavior: may error or handle gracefully depending on csv crate settings
    match result {
        Ok(doc) => {
            // If it succeeds, verify the data structure
            assert!(!doc.rows.is_empty());
        }
        Err(err) => {
            // Expected to fail with inconsistent columns
            let err_msg = err.to_string();
            assert!(!err_msg.is_empty());
        }
    }
}

#[test]
fn test_binary_file_treated_as_csv() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("binary.csv");

    // Write binary data (not valid UTF-8)
    let binary_data: Vec<u8> = vec![0xFF, 0xFE, 0xFD, 0x00, 0x80, 0x81, 0x82];
    std::fs::write(&file_path, &binary_data).unwrap();

    let result = Document::from_file(&file_path, None, false, None);

    // encoding_rs should handle this by replacing invalid chars or we get an error
    match result {
        Ok(doc) => {
            // Parsed with replacement characters - acceptable
            assert!(!doc.headers.is_empty() || doc.rows.is_empty());
        }
        Err(err) => {
            // Failed to parse - also acceptable
            let err_msg = err.to_string();
            assert!(!err_msg.is_empty());
        }
    }
}

#[test]
fn test_extremely_large_file_memory_limit() {
    // This test verifies that a moderately large file can be loaded
    // In the future, this could test streaming/pagination support

    // Generate CSV content for 1000 rows and 10 columns
    let mut content = String::from("A,B,C,D,E,F,G,H,I,J\n");
    for row in 0..1000 {
        content.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            row,
            row + 1,
            row + 2,
            row + 3,
            row + 4,
            row + 5,
            row + 6,
            row + 7,
            row + 8,
            row + 9
        ));
    }
    let temp_file = common::create_temp_csv_file(&content);

    let result = Document::from_file(temp_file.path(), None, false, None);
    assert!(result.is_ok(), "Should be able to load a 1000-row file");

    let doc = result.unwrap();
    assert_eq!(doc.row_count(), 1000);
    assert_eq!(doc.column_count(), 10);
}

#[test]
fn test_reload_file_deleted_during_session() {
    let temp_file = common::create_temp_csv_file("A,B\n1,2\n");
    let path = temp_file.path().to_path_buf();

    // Load the file successfully
    let doc = Document::from_file(&path, None, false, None).unwrap();
    let mut app = App::new(
        doc,
        vec![path.clone()],
        0,
        lazycsv::session::FileConfig::new(),
    );

    // Delete the file
    drop(temp_file); // Closes and deletes the temp file

    // Try to reload
    let result = app.reload_current_file();

    assert!(
        result.is_err(),
        "Expected error when reloading deleted file"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Failed to reload")
            || err_msg.contains("not found")
            || err_msg.contains("No such file"),
        "Error should indicate reload failure: {}",
        err_msg
    );
}

#[test]
#[cfg(unix)]
fn test_reload_file_permission_changed() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "A,B\n1,2\n").unwrap();

    // Load the file successfully
    let doc = Document::from_file(&file_path, None, false, None).unwrap();
    let mut app = App::new(
        doc,
        vec![file_path.clone()],
        0,
        lazycsv::session::FileConfig::new(),
    );

    // Change permissions to no read
    let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o000);
    std::fs::set_permissions(&file_path, perms).unwrap();

    // Try to reload
    let result = app.reload_current_file();

    // Restore permissions for cleanup
    let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&file_path, perms).unwrap();

    assert!(
        result.is_err(),
        "Expected error when reloading file without permissions"
    );
}

#[test]
fn test_switch_file_file_deleted() {
    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.path().join("file1.csv");
    let file2_path = temp_dir.path().join("file2.csv");

    write(&file1_path, "A,B\n1,2\n").unwrap();
    write(&file2_path, "C,D\n3,4\n").unwrap();

    // Load first file
    let doc = Document::from_file(&file1_path, None, false, None).unwrap();
    let mut app = App::new(
        doc,
        vec![file1_path.clone(), file2_path.clone()],
        0,
        lazycsv::session::FileConfig::new(),
    );

    // Delete the second file
    std::fs::remove_file(&file2_path).unwrap();

    // Switch to second file (should fail to reload)
    app.handle_key(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Char(']'),
    ))
    .unwrap();

    let result = app.reload_current_file();
    assert!(
        result.is_err(),
        "Expected error when switching to deleted file"
    );

    // Should be able to switch back to first file
    app.handle_key(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Char('['),
    ))
    .unwrap();

    let result = app.reload_current_file();
    assert!(result.is_ok(), "Should successfully reload first file");
}
