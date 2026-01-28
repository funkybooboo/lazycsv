use lazycsv::cli;
use std::fs::File;
use tempfile::TempDir;

#[test]
fn test_parse_args_no_arguments() {
    let args = vec!["lazycsv".to_string()];
    let result = cli::parse_args(&args);
    // Should use current directory when no args provided
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string_lossy(), ".");
}

#[test]
fn test_parse_args_with_valid_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    File::create(&file_path).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        file_path.to_string_lossy().to_string(),
    ];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), file_path);
}

#[test]
fn test_parse_args_file_not_found() {
    let args = vec![
        "lazycsv".to_string(),
        "/nonexistent/file.csv".to_string(),
    ];

    let result = cli::parse_args(&args);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Path not found"));
}

#[test]
fn test_parse_args_path_is_directory() {
    let temp_dir = TempDir::new().unwrap();

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    let result = cli::parse_args(&args);
    // Directories are now valid - main.rs will scan for CSV files
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), temp_dir.path());
}

#[test]
fn test_parse_args_relative_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.csv");
    let file = File::create(&file_path).unwrap();
    drop(file); // Ensure file is flushed

    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let args = vec!["lazycsv".to_string(), "data.csv".to_string()];

    let result = cli::parse_args(&args);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
}

#[test]
fn test_parse_args_absolute_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    File::create(&file_path).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        file_path.to_string_lossy().to_string(),
    ];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert!(result.unwrap().is_absolute());
}

#[test]
fn test_parse_args_with_spaces_in_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file with spaces.csv");
    File::create(&file_path).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        file_path.to_string_lossy().to_string(),
    ];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());
}

#[test]
fn test_parse_args_too_many_arguments() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    File::create(&file_path).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        file_path.to_string_lossy().to_string(),
        "extra".to_string(),
    ];

    // Should still work, extra args are ignored
    let result = cli::parse_args(&args);
    assert!(result.is_ok());
}

#[test]
fn test_parse_args_current_directory_dot() {
    let args = vec!["lazycsv".to_string(), ".".to_string()];
    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string_lossy(), ".");
}

#[test]
fn test_parse_args_parent_directory() {
    let args = vec!["lazycsv".to_string(), "..".to_string()];
    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string_lossy(), "..");
}

#[test]
fn test_parse_args_relative_directory() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    std::fs::create_dir(&sub_dir).unwrap();

    let args = vec![
        "lazycsv".to_string(),
        sub_dir.to_string_lossy().to_string(),
    ];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), sub_dir);
}

#[test]
fn test_parse_args_absolute_directory() {
    let temp_dir = TempDir::new().unwrap();

    let args = vec![
        "lazycsv".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    ];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());
    assert!(result.unwrap().is_absolute());
}

#[test]
fn test_parse_args_nonexistent_directory() {
    let args = vec![
        "lazycsv".to_string(),
        "/nonexistent/directory".to_string(),
    ];

    let result = cli::parse_args(&args);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Path not found"));
}

#[test]
fn test_parse_args_directory_with_trailing_slash() {
    let temp_dir = TempDir::new().unwrap();
    let dir_with_slash = format!("{}/", temp_dir.path().to_string_lossy());

    let args = vec!["lazycsv".to_string(), dir_with_slash];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());
}

#[test]
fn test_parse_args_subdirectory_relative_path() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create subdirectory
    std::fs::create_dir("subdir").unwrap();

    let args = vec!["lazycsv".to_string(), "subdir".to_string()];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_parse_args_nested_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create nested subdirectory
    std::fs::create_dir("sub1").unwrap();
    std::fs::create_dir("sub1/sub2").unwrap();

    let args = vec!["lazycsv".to_string(), "sub1/sub2".to_string()];

    let result = cli::parse_args(&args);
    assert!(result.is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}
