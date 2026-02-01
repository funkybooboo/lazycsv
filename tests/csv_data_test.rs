use lazycsv::CsvData;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_valid_csv() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Age,City").unwrap();
    writeln!(file, "Alice,30,NYC").unwrap();
    writeln!(file, "Bob,25,LA").unwrap();

    let csv_data = CsvData::from_file(file.path(), None, false, None).unwrap();

    assert_eq!(csv_data.column_count(), 3);
    assert_eq!(csv_data.row_count(), 2);
    assert_eq!(csv_data.get_header(0), "Name");
    assert_eq!(csv_data.get_cell(0, 0), "Alice");
    assert_eq!(csv_data.get_cell(1, 1), "25");
}

#[test]
fn test_empty_csv() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Age").unwrap();

    let csv_data = CsvData::from_file(file.path(), None, false, None).unwrap();

    assert_eq!(csv_data.column_count(), 2);
    assert_eq!(csv_data.row_count(), 0);
}

#[test]
fn test_get_cell_out_of_bounds() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Age").unwrap();
    writeln!(file, "Alice,30").unwrap();

    let csv_data = CsvData::from_file(file.path(), None, false, None).unwrap();

    assert_eq!(csv_data.get_cell(10, 0), ""); // Row out of bounds
    assert_eq!(csv_data.get_cell(0, 10), ""); // Column out of bounds
}

#[test]
fn test_unicode_in_cells() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Name,Description").unwrap();
    writeln!(file, "Test,日本語テキスト").unwrap(); // Japanese
    writeln!(file, "Test2,🎉 Emoji").unwrap(); // Emoji
    writeln!(file, "Test3,ñóëü").unwrap(); // Accented chars

    let result = CsvData::from_file(file.path(), None, false, None);

    assert!(result.is_ok());
    let csv_data = result.unwrap();
    assert_eq!(csv_data.rows[0][1], "日本語テキスト");
    assert_eq!(csv_data.rows[1][1], "🎉 Emoji");
    assert_eq!(csv_data.rows[2][1], "ñóëü");
}
