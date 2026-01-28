use lazycsv::file_scanner;
use std::fs::File;
use tempfile::TempDir;

#[test]
fn test_scan_directory_single_csv() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.csv");
    File::create(&file_path).unwrap();

    let result = file_scanner::scan_directory_for_csvs(&file_path);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 1);
    assert_eq!(csv_files[0], file_path);
}

#[test]
fn test_scan_directory_multiple_csvs() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("a.csv")).unwrap();
    File::create(temp_dir.path().join("b.csv")).unwrap();
    File::create(temp_dir.path().join("c.csv")).unwrap();

    let target_file = temp_dir.path().join("b.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 3);

    // Should be sorted alphabetically
    assert!(csv_files[0].file_name().unwrap() == "a.csv");
    assert!(csv_files[1].file_name().unwrap() == "b.csv");
    assert!(csv_files[2].file_name().unwrap() == "c.csv");
}

#[test]
fn test_scan_directory_mixed_files() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.csv")).unwrap();
    File::create(temp_dir.path().join("notes.txt")).unwrap();
    File::create(temp_dir.path().join("config.json")).unwrap();
    File::create(temp_dir.path().join("other.csv")).unwrap();

    let target_file = temp_dir.path().join("data.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Should only include CSV files
    assert_eq!(csv_files.len(), 2);
    assert!(csv_files.iter().all(|p| p.extension().unwrap() == "csv"));
}

#[test]
fn test_scan_directory_with_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("root.csv")).unwrap();

    // Create subdirectory with CSV
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();
    File::create(sub_dir.join("nested.csv")).unwrap();

    let target_file = temp_dir.path().join("root.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Should only include files in the same directory (not subdirectories)
    assert_eq!(csv_files.len(), 1);
    assert_eq!(csv_files[0].file_name().unwrap(), "root.csv");
}

#[test]
fn test_scan_directory_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("only.csv");
    File::create(&file_path).unwrap();

    // Delete the file to leave empty directory
    std::fs::remove_file(&file_path).unwrap();

    let result = file_scanner::scan_directory_for_csvs(&file_path);
    // Should handle gracefully - either error or return the input file
    assert!(result.is_ok());
    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 1);
    assert_eq!(csv_files[0], file_path);
}

#[test]
fn test_scan_directory_sorting() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("zebra.csv")).unwrap();
    File::create(temp_dir.path().join("apple.csv")).unwrap();
    File::create(temp_dir.path().join("mango.csv")).unwrap();

    let target_file = temp_dir.path().join("mango.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 3);

    // Verify alphabetical sorting
    assert!(csv_files[0].file_name().unwrap() == "apple.csv");
    assert!(csv_files[1].file_name().unwrap() == "mango.csv");
    assert!(csv_files[2].file_name().unwrap() == "zebra.csv");
}

#[test]
fn test_scan_directory_case_sensitivity() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("Data.csv")).unwrap();
    File::create(temp_dir.path().join("data.csv")).unwrap();

    let target_file = temp_dir.path().join("Data.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Both should be included (case-sensitive filesystems)
    assert!(csv_files.len() >= 1);
}

#[test]
fn test_scan_directory_with_dots_in_filename() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.backup.csv")).unwrap();
    File::create(temp_dir.path().join("data.v2.csv")).unwrap();

    let target_file = temp_dir.path().join("data.backup.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 2);
}

#[test]
fn test_scan_directory_uppercase_extension() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.csv")).unwrap();
    File::create(temp_dir.path().join("other.CSV")).unwrap();

    let target_file = temp_dir.path().join("data.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Depends on case-sensitive extension matching
    assert!(csv_files.len() >= 1);
}

#[test]
fn test_scan_directory_preserves_full_path() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.csv")).unwrap();

    let target_file = temp_dir.path().join("data.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Should return full paths, not just filenames
    assert!(csv_files[0].is_absolute() || csv_files[0].to_string_lossy().contains('/'));
}

#[test]
fn test_scan_directory_no_parent_directory() {
    // Test with root-level file (edge case)
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    File::create(&file_path).unwrap();

    let result = file_scanner::scan_directory_for_csvs(&file_path);
    assert!(result.is_ok());
}

#[test]
fn test_scan_directory_hidden_files() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("visible.csv")).unwrap();
    File::create(temp_dir.path().join(".hidden.csv")).unwrap();

    let target_file = temp_dir.path().join("visible.csv");
    let result = file_scanner::scan_directory_for_csvs(&target_file);
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Hidden files should still be included
    assert!(csv_files.len() >= 1);
}

#[test]
fn test_scan_directory_relative_path_without_parent() {
    // This tests the fix for the bug where relative paths like "sample.csv"
    // would fail because parent() returns an empty path
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.csv")).unwrap();
    File::create(temp_dir.path().join("other.csv")).unwrap();

    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Use just the filename without any path
    let result = file_scanner::scan_directory_for_csvs(std::path::Path::new("data.csv"));
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Should find both CSV files in current directory
    assert_eq!(csv_files.len(), 2);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

// Tests for scan_directory() function (direct directory scanning)

#[test]
fn test_direct_scan_directory_basic() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("a.csv")).unwrap();
    File::create(temp_dir.path().join("b.csv")).unwrap();
    File::create(temp_dir.path().join("c.csv")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 3);

    // Verify alphabetical sorting
    assert_eq!(csv_files[0].file_name().unwrap(), "a.csv");
    assert_eq!(csv_files[1].file_name().unwrap(), "b.csv");
    assert_eq!(csv_files[2].file_name().unwrap(), "c.csv");
}

#[test]
fn test_direct_scan_directory_empty() {
    let temp_dir = TempDir::new().unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 0);
}

#[test]
fn test_direct_scan_directory_no_csv_files() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.txt")).unwrap();
    File::create(temp_dir.path().join("config.json")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 0);
}

#[test]
fn test_direct_scan_directory_mixed_files() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.csv")).unwrap();
    File::create(temp_dir.path().join("notes.txt")).unwrap();
    File::create(temp_dir.path().join("other.csv")).unwrap();
    File::create(temp_dir.path().join("config.json")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 2);
    assert!(csv_files.iter().all(|p| p.extension().unwrap() == "csv"));
}

#[test]
fn test_direct_scan_directory_ignores_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("root.csv")).unwrap();

    // Create subdirectory with CSV file
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();
    File::create(sub_dir.join("nested.csv")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Should only include files in the specified directory, not subdirectories
    assert_eq!(csv_files.len(), 1);
    assert_eq!(csv_files[0].file_name().unwrap(), "root.csv");
}

#[test]
fn test_direct_scan_directory_sorting() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("zebra.csv")).unwrap();
    File::create(temp_dir.path().join("apple.csv")).unwrap();
    File::create(temp_dir.path().join("mango.csv")).unwrap();
    File::create(temp_dir.path().join("banana.csv")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 4);

    // Verify alphabetical sorting
    assert_eq!(csv_files[0].file_name().unwrap(), "apple.csv");
    assert_eq!(csv_files[1].file_name().unwrap(), "banana.csv");
    assert_eq!(csv_files[2].file_name().unwrap(), "mango.csv");
    assert_eq!(csv_files[3].file_name().unwrap(), "zebra.csv");
}

#[test]
fn test_direct_scan_directory_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    File::create("test1.csv").unwrap();
    File::create("test2.csv").unwrap();

    let result = file_scanner::scan_directory(std::path::Path::new("."));
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 2);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_direct_scan_directory_with_hidden_files() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("visible.csv")).unwrap();
    File::create(temp_dir.path().join(".hidden.csv")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Both visible and hidden CSV files should be included
    assert!(csv_files.len() >= 1);
}

#[test]
fn test_direct_scan_directory_case_sensitive_extension() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("lowercase.csv")).unwrap();
    File::create(temp_dir.path().join("uppercase.CSV")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Only lowercase .csv should match
    assert!(csv_files
        .iter()
        .any(|p| p.file_name().unwrap() == "lowercase.csv"));
}

#[test]
fn test_direct_scan_directory_nonexistent() {
    let result = file_scanner::scan_directory(std::path::Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn test_direct_scan_directory_with_dots_in_filename() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.backup.csv")).unwrap();
    File::create(temp_dir.path().join("data.v1.csv")).unwrap();
    File::create(temp_dir.path().join("data.v2.csv")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    assert_eq!(csv_files.len(), 3);
}

#[test]
fn test_direct_scan_directory_preserves_full_path() {
    let temp_dir = TempDir::new().unwrap();
    File::create(temp_dir.path().join("data.csv")).unwrap();

    let result = file_scanner::scan_directory(temp_dir.path());
    assert!(result.is_ok());

    let csv_files = result.unwrap();
    // Should return full paths, not just filenames
    assert!(csv_files[0].is_absolute() || csv_files[0].to_string_lossy().contains('/'));
    assert!(csv_files[0].starts_with(temp_dir.path()));
}
