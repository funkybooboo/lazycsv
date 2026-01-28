use lazycsv::{cli, file_scanner, CsvData};
use std::fs::File;
use std::io::Write as IoWrite;
use tempfile::TempDir;

// Integration tests for directory handling in main workflow

#[test]
fn test_directory_with_single_csv() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    let mut file = File::create(&csv_path).unwrap();
    writeln!(file, "Name,Age").unwrap();
    writeln!(file, "Alice,30").unwrap();

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    // Parse args to get directory
    let path = cli::parse_args(&args).unwrap();
    assert!(path.is_dir());

    // Scan directory for CSV files
    let csv_files = file_scanner::scan_directory(&path).unwrap();
    assert_eq!(csv_files.len(), 1);

    // Load the first CSV file
    let csv_data = CsvData::from_file(&csv_files[0]).unwrap();
    assert_eq!(csv_data.row_count(), 1);
    assert_eq!(csv_data.column_count(), 2);
}

#[test]
fn test_directory_with_multiple_csvs() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple CSV files
    for name in &["zebra.csv", "apple.csv", "mango.csv"] {
        let csv_path = temp_dir.path().join(name);
        let mut file = File::create(&csv_path).unwrap();
        writeln!(file, "Col1,Col2").unwrap();
        writeln!(file, "Data,Value").unwrap();
    }

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    // Should find all 3 CSV files, sorted alphabetically
    assert_eq!(csv_files.len(), 3);
    assert_eq!(csv_files[0].file_name().unwrap(), "apple.csv");
    assert_eq!(csv_files[1].file_name().unwrap(), "mango.csv");
    assert_eq!(csv_files[2].file_name().unwrap(), "zebra.csv");

    // Should load first file (apple.csv)
    let csv_data = CsvData::from_file(&csv_files[0]).unwrap();
    assert_eq!(csv_data.filename, "apple.csv");
}

#[test]
fn test_directory_with_no_csvs() {
    let temp_dir = TempDir::new().unwrap();

    // Create non-CSV files
    File::create(temp_dir.path().join("data.txt")).unwrap();
    File::create(temp_dir.path().join("config.json")).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    // Should find no CSV files
    assert_eq!(csv_files.len(), 0);
}

#[test]
fn test_directory_empty() {
    let temp_dir = TempDir::new().unwrap();

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    // Empty directory should return empty list
    assert_eq!(csv_files.len(), 0);
}

#[test]
fn test_current_directory_dot() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create CSV in current directory
    let mut file = File::create("test.csv").unwrap();
    writeln!(file, "A,B,C").unwrap();
    writeln!(file, "1,2,3").unwrap();

    let args = vec!["lazycsv".to_string(), ".".to_string()];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    assert_eq!(csv_files.len(), 1);

    let csv_data = CsvData::from_file(&csv_files[0]).unwrap();
    assert_eq!(csv_data.row_count(), 1);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_no_args_defaults_to_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create CSV in current directory
    let mut file = File::create("data.csv").unwrap();
    writeln!(file, "Name").unwrap();
    writeln!(file, "Test").unwrap();

    let args = vec!["lazycsv".to_string()];

    let path = cli::parse_args(&args).unwrap();
    assert_eq!(path.to_string_lossy(), ".");

    let csv_files = file_scanner::scan_directory(&path).unwrap();
    assert_eq!(csv_files.len(), 1);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_subdirectory_path() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();

    // Create CSV in subdirectory
    let mut file = File::create(sub_dir.join("data.csv")).unwrap();
    writeln!(file, "X,Y").unwrap();
    writeln!(file, "1,2").unwrap();

    let args = vec![
        "lazycsv".to_string(),
        sub_dir.to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    assert_eq!(csv_files.len(), 1);
    assert_eq!(csv_files[0].file_name().unwrap(), "data.csv");
}

#[test]
fn test_directory_only_scans_immediate_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create CSV in root
    File::create(temp_dir.path().join("root.csv")).unwrap();

    // Create CSV in subdirectory (should not be included)
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();
    File::create(sub_dir.join("nested.csv")).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    // Should only find root.csv, not nested.csv
    assert_eq!(csv_files.len(), 1);
    assert_eq!(csv_files[0].file_name().unwrap(), "root.csv");
}

#[test]
fn test_file_path_still_works() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("specific.csv");
    let mut file = File::create(&csv_path).unwrap();
    writeln!(file, "Header").unwrap();
    writeln!(file, "Value").unwrap();

    // Also create other CSVs in the directory
    File::create(temp_dir.path().join("other.csv")).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        csv_path.to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    assert!(path.is_file());

    // When given a file, scan_directory_for_csvs should scan its parent directory
    let csv_files = file_scanner::scan_directory_for_csvs(&path).unwrap();

    // Should find both CSV files in the directory
    assert_eq!(csv_files.len(), 2);
}

#[test]
fn test_relative_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("myfile.csv");

    let mut file = File::create(&csv_path).unwrap();
    writeln!(file, "A,B").unwrap();
    writeln!(file, "1,2").unwrap();
    drop(file); // Ensure file is flushed

    let args = vec![
        "lazycsv".to_string(),
        csv_path.to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    assert!(path.is_file());

    let csv_files = file_scanner::scan_directory_for_csvs(&path).unwrap();
    assert_eq!(csv_files.len(), 1);
}

#[test]
fn test_directory_with_mixed_extensions() {
    let temp_dir = TempDir::new().unwrap();

    File::create(temp_dir.path().join("data.csv")).unwrap();
    File::create(temp_dir.path().join("data.CSV")).unwrap(); // Different case
    File::create(temp_dir.path().join("data.txt")).unwrap();
    File::create(temp_dir.path().join("data.tsv")).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    // Should only include lowercase .csv files
    assert!(csv_files.len() >= 1);
    assert!(csv_files
        .iter()
        .all(|p| p.extension().unwrap() == "csv"));
}

#[test]
fn test_parent_directory_double_dot() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();

    // Create CSV in parent directory
    let mut file = File::create(temp_dir.path().join("parent.csv")).unwrap();
    writeln!(file, "Data").unwrap();
    drop(file); // Ensure file is flushed

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&sub_dir).unwrap();

    let args = vec!["lazycsv".to_string(), "..".to_string()];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    // Cleanup before assertions to avoid leaving directory changed on failure
    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(csv_files.len(), 1);
    assert_eq!(csv_files[0].file_name().unwrap(), "parent.csv");
}

#[test]
fn test_directory_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let special_dir = temp_dir.path().join("dir with spaces");
    std::fs::create_dir(&special_dir).unwrap();

    let mut file = File::create(special_dir.join("data.csv")).unwrap();
    writeln!(file, "Col1").unwrap();

    let args = vec![
        "lazycsv".to_string(),
        special_dir.to_string_lossy().to_string(),
    ];

    let path = cli::parse_args(&args).unwrap();
    let csv_files = file_scanner::scan_directory(&path).unwrap();

    assert_eq!(csv_files.len(), 1);
}
