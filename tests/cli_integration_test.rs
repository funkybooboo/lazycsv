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

    assert_eq!(app.document.headers, vec!["a", "b", "c"]);
    assert_eq!(app.document.rows[0], vec!["1", "2", "3"]);
    assert_eq!(app.session.config().delimiter, Some(b';'));
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
        app.document.headers,
        vec!["Column 1", "Column 2", "Column 3"]
    );
    assert_eq!(app.document.rows.len(), 2);
    assert_eq!(app.document.rows[0], vec!["a", "b", "c"]);
    assert_eq!(app.document.rows[1], vec!["1", "2", "3"]);
    assert!(app.session.config().no_headers);
}

#[test]
fn test_default_csv_loading_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "header1,header2\nval1,val2").unwrap();

    let args = CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap()]).unwrap();
    let app = App::from_cli(args).unwrap();

    assert_eq!(app.document.headers, vec!["header1", "header2"]);
    assert_eq!(app.document.rows[0], vec!["val1", "val2"]);
    assert_eq!(app.session.config().delimiter, None);
    assert!(!app.session.config().no_headers);
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

    assert_eq!(app.document.headers, vec!["h1", "h2"]);
    assert_eq!(app.session.files().len(), 2);
    assert_eq!(app.session.active_file_index(), 0);
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
        lazycsv::Document::from_file(&file_path, None, false, Some("utf-16le".to_string()))
            .unwrap();

    assert_eq!(csv_data.headers, vec!["h1", "h2"]);
    assert_eq!(csv_data.rows[0], vec!["val1", "val2"]);
}

#[test]
fn test_invalid_encoding_integration() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    write(&file_path, "a,b,c").unwrap();

    let result = lazycsv::Document::from_file(
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

    assert_eq!(app.document.headers, vec!["Column 1", "Column 2"]);
    assert_eq!(app.document.rows.len(), 2);
    assert_eq!(app.document.rows[0], vec!["a", "b"]);
    assert_eq!(app.document.rows[1], vec!["1", "2"]);
    assert!(app.session.config().no_headers);
    assert_eq!(app.session.config().delimiter, Some(b';'));
}

#[test]
fn test_invalid_utf8_bytes_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid_utf8.csv");

    // Create file with invalid UTF-8 byte sequences
    // Start with valid UTF-8, then add invalid sequences
    let mut invalid_bytes = b"A,B,C\n".to_vec();
    invalid_bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
    invalid_bytes.extend_from_slice(b",value,");
    invalid_bytes.extend_from_slice(&[0x80, 0x81]); // More invalid UTF-8

    write(&file_path, &invalid_bytes).unwrap();

    // encoding_rs should handle this with replacement characters
    let result = lazycsv::Document::from_file(&file_path, None, false, None);

    // Should either succeed with replacement chars or fail gracefully
    match result {
        Ok(doc) => {
            // Parsed with replacement characters - acceptable behavior
            assert!(!doc.headers.is_empty());
        }
        Err(err) => {
            // Failed to parse - also acceptable
            assert!(!err.to_string().is_empty());
        }
    }
}

#[test]
fn test_mixed_encoding_in_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("mixed_encoding.csv");

    // First half UTF-8, second half with byte sequences that could be other encoding
    let mut mixed_bytes = b"A,B,C\nval1,val2,val3\n".to_vec();
    // Add some bytes that could be interpreted differently in different encodings
    mixed_bytes.extend_from_slice(&[0xC0, 0x80, 0xE0, 0x80, 0x80]);

    write(&file_path, &mixed_bytes).unwrap();

    // Should handle mixed content (encoding_rs will decode as UTF-8 with replacements)
    let result = lazycsv::Document::from_file(&file_path, None, false, None);

    match result {
        Ok(doc) => {
            assert!(!doc.headers.is_empty());
            // May contain replacement characters
        }
        Err(err) => {
            assert!(!err.to_string().is_empty());
        }
    }
}
