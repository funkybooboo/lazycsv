use clap::Parser;
use lazycsv::{cli::CliArgs, App};
use std::fs::write;
use tempfile::TempDir;

#[test]
fn test_delimiter_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "a;b;c\n1;2;3").unwrap();

    let args =
        CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap(), "--delimiter", ";"])
            .unwrap();

    let app = App::from_cli(args).unwrap();

    assert_eq!(app.csv_data.headers, vec!["a", "b", "c"]);
    assert_eq!(app.csv_data.rows[0], vec!["1", "2", "3"]);
    assert_eq!(app.delimiter, Some(b';'));
}

#[test]
fn test_no_headers_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "a,b,c\n1,2,3").unwrap();

    let args =
        CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap(), "--no-headers"]).unwrap();

    let app = App::from_cli(args).unwrap();

    assert_eq!(
        app.csv_data.headers,
        vec!["Column 1", "Column 2", "Column 3"]
    );
    assert_eq!(app.csv_data.rows.len(), 2);
    assert_eq!(app.csv_data.rows[0], vec!["a", "b", "c"]);
    assert_eq!(app.csv_data.rows[1], vec!["1", "2", "3"]);
    assert!(app.no_headers);
}

#[test]
fn test_default_csv_loading_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "header1,header2\nval1,val2").unwrap();

    let args = CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap()]).unwrap();
    let app = App::from_cli(args).unwrap();

    assert_eq!(app.csv_data.headers, vec!["header1", "header2"]);
    assert_eq!(app.csv_data.rows[0], vec!["val1", "val2"]);
    assert_eq!(app.delimiter, None);
    assert!(!app.no_headers);
}

#[test]
fn test_directory_path_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("a_test.csv");
    write(&file_path, "h1,h2\n1,2").unwrap();
    let _other_file = temp_dir.path().join("b_test.csv");
    write(_other_file, "h3,h4\n3,4").unwrap();

    let args = CliArgs::try_parse_from(["lazycsv", temp_dir.path().to_str().unwrap()]).unwrap();
    let app = App::from_cli(args).unwrap();

    assert_eq!(app.csv_data.headers, vec!["h1", "h2"]);
    assert_eq!(app.csv_files.len(), 2);
    assert_eq!(app.current_file_index, 0);
}

#[test]
fn test_encoding_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("utf16.csv");

    // Create UTF-16LE encoded bytes manually
    // "h1,h2\nval1,val2" in UTF-16LE
    let encoded_bytes: Vec<u8> = vec![
        104, 0, // 'h'
        49, 0, // '1'
        44, 0, // ','
        104, 0, // 'h'
        50, 0, // '2'
        10, 0, // '\n'
        118, 0, // 'v'
        97, 0, // 'a'
        108, 0, // 'l'
        49, 0, // '1'
        44, 0, // ','
        118, 0, // 'v'
        97, 0, // 'a'
        108, 0, // 'l'
        50, 0, // '2'
    ];

    write(&file_path, &encoded_bytes).unwrap();

    let csv_data =
        lazycsv::CsvData::from_file(&file_path, None, false, Some("utf-16le".to_string())).unwrap();

    assert_eq!(csv_data.headers, vec!["h1", "h2"]);
    assert_eq!(csv_data.rows[0], vec!["val1", "val2"]);
}

#[test]
fn test_invalid_encoding_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "a,b,c").unwrap();

    let result = lazycsv::CsvData::from_file(
        &file_path,
        None,
        false,
        Some("invalid-encoding".to_string()),
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported encoding"));
}

#[test]
fn test_delimiter_and_no_headers_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "a;b\n1;2").unwrap();

    let args = CliArgs::try_parse_from([
        "lazycsv",
        file_path.to_str().unwrap(),
        "--delimiter",
        ";",
        "--no-headers",
    ])
    .unwrap();

    let app = App::from_cli(args).unwrap();

    assert_eq!(app.csv_data.headers, vec!["Column 1", "Column 2"]);
    assert_eq!(app.csv_data.rows.len(), 2);
    assert_eq!(app.csv_data.rows[0], vec!["a", "b"]);
    assert_eq!(app.csv_data.rows[1], vec!["1", "2"]);
    assert!(app.no_headers);
    assert_eq!(app.delimiter, Some(b';'));
}
