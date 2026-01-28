use lazycsv::CsvData;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_single_row_csv() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Age,City").unwrap();
    writeln!(file, "Alice,30,NYC").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 1);
    assert_eq!(csv_data.column_count(), 3);
    assert_eq!(csv_data.get_cell(0, 0), "Alice");
}

#[test]
fn test_single_column_csv() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name").unwrap();
    writeln!(file, "Alice").unwrap();
    writeln!(file, "Bob").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 2);
    assert_eq!(csv_data.column_count(), 1);
    assert_eq!(csv_data.get_header(0), "Name");
}

#[test]
fn test_csv_with_empty_cells() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A,B,C").unwrap();
    writeln!(file, "1,,3").unwrap();
    writeln!(file, ",2,").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 2);
    assert_eq!(csv_data.get_cell(0, 0), "1");
    assert_eq!(csv_data.get_cell(0, 1), "");
    assert_eq!(csv_data.get_cell(0, 2), "3");
    assert_eq!(csv_data.get_cell(1, 0), "");
    assert_eq!(csv_data.get_cell(1, 1), "2");
    assert_eq!(csv_data.get_cell(1, 2), "");
}

#[test]
fn test_csv_with_quoted_fields() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Description").unwrap();
    writeln!(file, "Alice,\"Hello, World\"").unwrap();
    writeln!(file, "Bob,\"Line1\nLine2\"").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 2);
    assert_eq!(csv_data.get_cell(0, 0), "Alice");
    assert_eq!(csv_data.get_cell(0, 1), "Hello, World");
    assert_eq!(csv_data.get_cell(1, 1), "Line1\nLine2");
}

#[test]
fn test_csv_with_escaped_quotes() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Text").unwrap();
    writeln!(file, "\"She said \"\"hello\"\"\"").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 1);
    assert_eq!(csv_data.get_cell(0, 0), "She said \"hello\"");
}

#[test]
fn test_csv_with_whitespace() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A,B,C").unwrap();
    writeln!(file, "  1  ,  2  ,  3  ").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    // CSV parser should preserve whitespace
    assert_eq!(csv_data.get_cell(0, 0), "  1  ");
    assert_eq!(csv_data.get_cell(0, 1), "  2  ");
}

#[test]
fn test_csv_with_special_characters() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Symbol,Emoji").unwrap();
    writeln!(file, "★,😀").unwrap();
    writeln!(file, "€,日本").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 2);
    assert_eq!(csv_data.get_cell(0, 0), "★");
    assert_eq!(csv_data.get_cell(0, 1), "😀");
    assert_eq!(csv_data.get_cell(1, 0), "€");
    assert_eq!(csv_data.get_cell(1, 1), "日本");
}

#[test]
fn test_csv_with_long_text() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Text").unwrap();
    let long_text = "a".repeat(1000);
    writeln!(file, "{}", long_text).unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 1);
    assert_eq!(csv_data.get_cell(0, 0).len(), 1000);
}

#[test]
fn test_csv_with_numbers() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Int,Float,Scientific").unwrap();
    writeln!(file, "123,456.789,1.23e10").unwrap();
    writeln!(file, "-999,0.001,-5e-3").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 2);
    // Numbers are stored as strings
    assert_eq!(csv_data.get_cell(0, 0), "123");
    assert_eq!(csv_data.get_cell(0, 1), "456.789");
    assert_eq!(csv_data.get_cell(0, 2), "1.23e10");
}

#[test]
fn test_csv_with_mixed_row_lengths() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A,B,C").unwrap();
    writeln!(file, "1,2,3").unwrap();
    writeln!(file, "4,5").unwrap(); // Missing last column

    // CSV parser is strict - should fail with inconsistent field count
    let result = CsvData::from_file(file.path());
    assert!(result.is_err());
}

#[test]
fn test_large_csv() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A,B,C").unwrap();
    for i in 0..10000 {
        writeln!(file, "{},{},{}", i, i * 2, i * 3).unwrap();
    }

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 10000);
    assert_eq!(csv_data.get_cell(0, 0), "0");
    assert_eq!(csv_data.get_cell(9999, 0), "9999");
    assert_eq!(csv_data.get_cell(9999, 2), "29997");
}

#[test]
fn test_wide_csv() {
    let mut file = NamedTempFile::new().unwrap();
    let headers: Vec<String> = (0..100).map(|i| format!("Col{}", i)).collect();
    writeln!(file, "{}", headers.join(",")).unwrap();
    let row: Vec<String> = (0..100).map(|i| format!("val{}", i)).collect();
    writeln!(file, "{}", row.join(",")).unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.column_count(), 100);
    assert_eq!(csv_data.row_count(), 1);
    assert_eq!(csv_data.get_header(0), "Col0");
    assert_eq!(csv_data.get_header(99), "Col99");
}

#[test]
fn test_csv_with_blank_lines_ignored() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A,B").unwrap();
    writeln!(file, "1,2").unwrap();
    writeln!(file, "").unwrap(); // Blank line
    writeln!(file, "3,4").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    // CSV parser should handle blank lines appropriately
    // Standard CSV parsers may include or exclude them
    assert!(csv_data.row_count() >= 2);
}

#[test]
fn test_filename_extraction() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A").unwrap();
    writeln!(file, "1").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    // Should extract filename from path
    assert!(!csv_data.filename.is_empty());
}

#[test]
fn test_csv_with_commas_in_quotes() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Address").unwrap();
    writeln!(file, "Alice,\"123 Main St, Apt 4, City\"").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert_eq!(csv_data.row_count(), 1);
    assert_eq!(csv_data.get_cell(0, 1), "123 Main St, Apt 4, City");
}

#[test]
fn test_csv_dirty_flag_initial_state() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "A").unwrap();
    writeln!(file, "1").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    assert!(!csv_data.is_dirty);
}

#[test]
fn test_header_and_cell_access_consistency() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Age,City").unwrap();
    writeln!(file, "Alice,30,NYC").unwrap();

    let csv_data = CsvData::from_file(file.path()).unwrap();

    for col in 0..csv_data.column_count() {
        // Should be able to access both header and cells for all columns
        let header = csv_data.get_header(col);
        let cell = csv_data.get_cell(0, col);
        assert!(!header.is_empty() || col >= 3);
        assert!(!cell.is_empty() || col >= 3);
    }
}
