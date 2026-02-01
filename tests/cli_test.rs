use clap::Parser;
use lazycsv::cli::CliArgs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_cli_default_args() {
    let args = CliArgs::try_parse_from(["lazycsv"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.path, None);
    assert_eq!(args.delimiter, None);
    assert!(!args.no_headers);
    assert_eq!(args.encoding, None);
}

#[test]
fn test_cli_with_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    std::fs::File::create(&file_path).unwrap();

    let args = CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap()]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.path, Some(file_path));
}

#[test]
fn test_cli_with_delimiter() {
    let args = CliArgs::try_parse_from(["lazycsv", "--delimiter", ";"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.delimiter, Some(b';'));
}

#[test]
fn test_cli_with_delimiter_short() {
    let args = CliArgs::try_parse_from(["lazycsv", "-d", ";"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.delimiter, Some(b';'));
}

#[test]
fn test_cli_invalid_delimiter() {
    let args = CliArgs::try_parse_from(["lazycsv", "--delimiter", "abc"]);
    assert!(args.is_err());
}

#[test]
fn test_cli_with_no_headers() {
    let args = CliArgs::try_parse_from(["lazycsv", "--no-headers"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(args.no_headers);
}

#[test]
fn test_cli_all_args_combined() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.csv");
    std::fs::File::create(&file_path).unwrap();

    let args = CliArgs::try_parse_from([
        "lazycsv",
        file_path.to_str().unwrap(),
        "--delimiter",
        ",",
        "--no-headers",
        "--encoding",
        "utf-8",
    ]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.path, Some(file_path));
    assert_eq!(args.delimiter, Some(b','));
    assert!(args.no_headers);
    assert_eq!(args.encoding, Some("utf-8".to_string()));
}

#[test]
fn test_cli_path_not_found_is_ok_for_parser() {
    let args = CliArgs::try_parse_from(["lazycsv", "/non/existent/path.csv"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.path, Some(PathBuf::from("/non/existent/path.csv")));
}

#[test]
fn test_cli_with_encoding() {
    let args = CliArgs::try_parse_from(["lazycsv", "--encoding", "utf-16le"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.encoding, Some("utf-16le".to_string()));
}

#[test]
fn test_cli_with_encoding_short() {
    let args = CliArgs::try_parse_from(["lazycsv", "-e", "latin1"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.encoding, Some("latin1".to_string()));
}
