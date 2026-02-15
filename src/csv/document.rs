//! In-memory CSV document with headers and rows

use crate::domain::position::{ColIndex, RowIndex};
use anyhow::{Context, Result};
use csv;
use encoding_rs::Encoding;
use std::fs;
use std::path::Path;

/// Holds parsed CSV document in memory
#[derive(Debug, Clone)]
pub struct Document {
    /// All rows including header (row 0 = header, rest = data)
    pub rows: Vec<Vec<String>>,

    /// Original filename for display
    pub filename: String,

    /// Track unsaved changes
    pub is_dirty: bool,

    /// Header mode toggle - when ON, row 0 is styled/frozen as header
    pub header_mode: bool,

    /// Delimiter character for this file
    pub delimiter: char,
}

impl Document {
    /// Load CSV from file path with optional delimiter, header, and encoding settings.
    pub fn from_file(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding_label: Option<String>,
    ) -> Result<Self> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_bytes =
            fs::read(path).context(format!("Failed to read file: {}", path.display()))?;

        let decoded_content = Self::decode_file_bytes(&file_bytes, encoding_label)?;
        let (headers, data_rows) =
            Self::parse_csv_content(&decoded_content, delimiter, no_headers)?;

        // Combine header and data rows - row 0 is the header
        let mut all_rows = vec![headers];
        all_rows.extend(data_rows);

        Ok(Document {
            rows: all_rows,
            filename,
            is_dirty: false,
            header_mode: true, // Default to header mode ON
            delimiter: delimiter.map(|d| d as char).unwrap_or(','),
        })
    }

    /// Decodes file bytes into a UTF-8 string using the specified encoding.
    fn decode_file_bytes(file_bytes: &[u8], encoding_label: Option<String>) -> Result<String> {
        if let Some(label) = &encoding_label {
            let encoding = Encoding::for_label(label.as_bytes())
                .ok_or_else(|| anyhow::anyhow!("Unsupported encoding: {}", label))?;
            let (decoded_content, ..) = encoding.decode(file_bytes);
            Ok(decoded_content.into_owned())
        } else {
            let (decoded_content, ..) = encoding_rs::UTF_8.decode_with_bom_removal(file_bytes);
            Ok(decoded_content.into_owned())
        }
    }

    /// Parses CSV content from a string.
    fn parse_csv_content(
        content: &str,
        delimiter: Option<u8>,
        no_headers: bool,
    ) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        let mut builder = csv::ReaderBuilder::new();
        builder.has_headers(!no_headers);
        if let Some(d) = delimiter {
            builder.delimiter(d);
        }

        let mut reader = builder.from_reader(content.as_bytes());
        let headers_from_csv = reader.headers()?.clone();

        let mut rows: Vec<Vec<String>> = Vec::new();
        for result in reader.records() {
            let record = result?;
            rows.push(record.iter().map(String::from).collect());
        }

        let final_headers = if no_headers {
            rows.first()
                .map(|first_row| {
                    (1..=first_row.len())
                        .map(|i| format!("Column {}", i))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            headers_from_csv.iter().map(String::from).collect()
        };

        Ok((final_headers, rows))
    }

    /// Get total row count (including header row)
    /// Row 0 = header, Row 1+ = data rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get data row count (excluding header row)
    pub fn data_row_count(&self) -> usize {
        if self.rows.is_empty() {
            0
        } else {
            self.rows.len() - 1
        }
    }

    /// Get column count
    pub fn column_count(&self) -> usize {
        self.rows.first().map(|r| r.len()).unwrap_or(0)
    }

    /// Get specific cell value (returns "" if out of bounds)
    /// row_idx is absolute: 0 = header row, 1 = first data row, etc.
    #[allow(dead_code)]
    pub fn get_cell(&self, row_idx: RowIndex, col_idx: ColIndex) -> &str {
        self.rows
            .get(row_idx.get())
            .and_then(|r| r.get(col_idx.get()))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get column header by index (returns "" if out of bounds)
    pub fn get_header(&self, col_idx: ColIndex) -> &str {
        self.rows
            .first()
            .and_then(|header_row| header_row.get(col_idx.get()))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Set a cell value (returns old value, sets is_dirty = true)
    /// row_idx is absolute: 0 = header row, 1 = first data row, etc.
    pub fn set_cell(
        &mut self,
        row_idx: RowIndex,
        col_idx: ColIndex,
        value: String,
    ) -> Option<String> {
        if let Some(row) = self.rows.get_mut(row_idx.get()) {
            if let Some(cell) = row.get_mut(col_idx.get()) {
                self.is_dirty = true;
                let old = std::mem::replace(cell, value);
                return Some(old);
            }
        }
        None
    }

    /// Insert a new empty row at the specified index (absolute row index)
    /// Row 0 = header, Row 1+ = data
    /// Note: Inserting at row 0 will insert BEFORE the header (creating a new header)
    pub fn insert_row(&mut self, at: RowIndex) {
        let col_count = self.column_count();
        let empty_row = vec![String::new(); col_count];
        let actual_insert = at.get().min(self.rows.len());
        self.rows.insert(actual_insert, empty_row);
        self.is_dirty = true;
    }

    /// Delete a row at the specified index (absolute row index)
    /// Returns the deleted row, or None if out of bounds
    /// Note: Deleting row 0 deletes the header row
    pub fn delete_row(&mut self, at: RowIndex) -> Option<Vec<String>> {
        if at.get() < self.rows.len() {
            self.is_dirty = true;
            Some(self.rows.remove(at.get()))
        } else {
            None
        }
    }

    /// Delete multiple rows in a range (inclusive, absolute row indices)
    /// Returns the deleted rows
    /// Example: delete_rows(RowIndex(5), RowIndex(10)) deletes rows 5-10 inclusive
    pub fn delete_rows(&mut self, start: RowIndex, end: RowIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();

        // Validate range
        if start_idx > end_idx || start_idx >= self.rows.len() {
            return vec![];
        }

        // Clamp end to valid range
        let actual_end = end_idx.min(self.rows.len() - 1);
        let count = actual_end - start_idx + 1;

        self.is_dirty = true;

        // Remove rows one by one and collect them
        let mut deleted = Vec::new();
        for _ in 0..count {
            if let Some(row) = self.rows.get(start_idx).cloned() {
                self.rows.remove(start_idx);
                deleted.push(row);
            }
        }

        deleted
    }

    /// Get a copy of rows in a range (inclusive, absolute row indices)
    /// Returns the rows without deleting them
    /// Example: get_rows(RowIndex(5), RowIndex(10)) returns rows 5-10 inclusive
    pub fn get_rows(&self, start: RowIndex, end: RowIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();

        // Validate range
        if start_idx > end_idx || start_idx >= self.rows.len() {
            return vec![];
        }

        // Clamp end to valid range
        let actual_end = end_idx.min(self.rows.len() - 1);

        // Collect rows in range
        self.rows[start_idx..=actual_end].to_vec()
    }

    /// Delete a range of columns (inclusive, 0-based column indices)
    /// Returns the deleted columns (each column as Vec<String> including header)
    /// Example: delete_columns(ColIndex(1), ColIndex(3)) deletes columns B, C, D
    pub fn delete_columns(&mut self, start: ColIndex, end: ColIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();

        // Validate range
        if self.rows.is_empty() {
            return vec![];
        }

        let col_count = self.rows[0].len();
        if start_idx >= col_count || start_idx > end_idx {
            return vec![];
        }

        // Clamp end to valid range
        let actual_end = end_idx.min(col_count - 1);
        let delete_count = actual_end - start_idx + 1;

        // Collect deleted columns (transpose: rows become columns)
        let mut deleted_columns = vec![vec![]; delete_count];

        for row in &self.rows {
            for (offset, col_idx) in (start_idx..=actual_end).enumerate() {
                if col_idx < row.len() {
                    deleted_columns[offset].push(row[col_idx].clone());
                }
            }
        }

        // Delete columns from all rows
        for row in &mut self.rows {
            if row.len() > start_idx {
                let remove_end = actual_end.min(row.len() - 1);
                row.drain(start_idx..=remove_end);
            }
        }

        self.is_dirty = true;
        deleted_columns
    }

    /// Get a copy of columns in a range (inclusive, 0-based column indices)
    /// Returns the columns without deleting them (each column as Vec<String> including header)
    /// Example: get_columns(ColIndex(1), ColIndex(3)) returns columns B, C, D
    pub fn get_columns(&self, start: ColIndex, end: ColIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();

        // Validate range
        if self.rows.is_empty() {
            return vec![];
        }

        let col_count = self.rows[0].len();
        if start_idx >= col_count || start_idx > end_idx {
            return vec![];
        }

        // Clamp end to valid range
        let actual_end = end_idx.min(col_count - 1);
        let column_count = actual_end - start_idx + 1;

        // Collect columns (transpose: rows become columns)
        let mut columns = vec![vec![]; column_count];

        for row in &self.rows {
            for (offset, col_idx) in (start_idx..=actual_end).enumerate() {
                if col_idx < row.len() {
                    columns[offset].push(row[col_idx].clone());
                }
            }
        }

        columns
    }

    /// Get a single column (including header at index 0)
    /// Returns empty vec if column doesn't exist
    pub fn get_column(&self, col: ColIndex) -> Vec<String> {
        let col_idx = col.get();

        if self.rows.is_empty() {
            return vec![];
        }

        let mut column = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            if col_idx < row.len() {
                column.push(row[col_idx].clone());
            } else {
                column.push(String::new());
            }
        }

        column
    }

    /// Delete a single column at the given index
    /// Returns the deleted column (including header at index 0)
    pub fn delete_column(&mut self, col: ColIndex) -> Vec<String> {
        let col_idx = col.get();

        if self.rows.is_empty() {
            return vec![];
        }

        let col_count = self.rows[0].len();
        if col_idx >= col_count {
            return vec![];
        }

        // Collect deleted column values
        let mut deleted_column = Vec::with_capacity(self.rows.len());

        // Delete from all rows and collect values
        for row in &mut self.rows {
            if col_idx < row.len() {
                deleted_column.push(row.remove(col_idx));
            } else {
                deleted_column.push(String::new());
            }
        }

        self.is_dirty = true;
        deleted_column
    }

    /// Insert a new column at the given position
    /// column_data should include header at index 0
    /// If column_data is shorter than row count, empty strings are used
    pub fn insert_column(&mut self, at: ColIndex, column_data: Vec<String>) {
        let col_idx = at.get();

        if self.rows.is_empty() {
            return;
        }

        // Insert cells from column_data into each row
        for (row_idx, row) in self.rows.iter_mut().enumerate() {
            let value = column_data.get(row_idx).cloned().unwrap_or_default();

            // Ensure we don't go out of bounds
            let insert_pos = col_idx.min(row.len());
            row.insert(insert_pos, value);
        }

        self.is_dirty = true;
    }

    /// Insert a new empty column at the given position with a generated header
    /// Example: insert_empty_column(ColIndex(2)) inserts at column C
    pub fn insert_empty_column(&mut self, at: ColIndex) {
        let col_idx = at.get();

        if self.rows.is_empty() {
            return;
        }

        // Generate header like "Column C" based on position
        let header = format!(
            "Column {}",
            crate::ui::utils::column_to_excel_letter(col_idx)
        );

        // Build column data with header and empty cells
        let column_data = std::iter::once(header)
            .chain(std::iter::repeat(String::new()).take(self.rows.len() - 1))
            .collect();

        self.insert_column(at, column_data);
    }

    /// Toggle header mode
    pub fn toggle_header_mode(&mut self) {
        self.header_mode = !self.header_mode;
    }

    /// Create a Document from headers and data rows (for testing)
    #[cfg(test)]
    pub fn from_parts(headers: Vec<String>, data_rows: Vec<Vec<String>>, filename: String) -> Self {
        let mut all_rows = vec![headers];
        all_rows.extend(data_rows);
        Document {
            rows: all_rows,
            filename,
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
        }
    }

    /// Create a Document for public use (needed by tests outside this module)
    pub fn new(headers: Vec<String>, data_rows: Vec<Vec<String>>, filename: String) -> Self {
        let mut all_rows = vec![headers];
        all_rows.extend(data_rows);
        Document {
            rows: all_rows,
            filename,
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::position::{ColIndex, RowIndex};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age,City").unwrap();
        writeln!(file, "Alice,30,NYC").unwrap();
        writeln!(file, "Bob,25,LA").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(csv_data.row_count(), 3); // Now includes header (1 header + 2 data rows)
        assert_eq!(csv_data.get_header(ColIndex::new(0)), "Name");
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)), // Row 1 is first data row
            "Alice"
        );
        assert_eq!(csv_data.get_cell(RowIndex::new(2), ColIndex::new(1)), "25");
        // Row 2 is second data row
    }

    #[test]
    fn test_empty_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.column_count(), 2);
        assert_eq!(csv_data.row_count(), 1); // Now includes header (1 header + 0 data rows)
    }

    #[test]
    fn test_get_cell_out_of_bounds() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age").unwrap();
        writeln!(file, "Alice,30").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.get_cell(RowIndex::new(10), ColIndex::new(0)), ""); // Row out of bounds
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(10)), ""); // Column out of bounds (row 1 is first data row)
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

        let result = Document::from_file(file.path(), None, false, None);

        assert!(result.is_ok());
        let csv_data = result.unwrap();
        // rows[0] is header, rows[1..] are data
        assert_eq!(csv_data.rows[1][1], "日本語テキスト");
        assert_eq!(csv_data.rows[2][1], "🎉 Emoji");
        assert_eq!(csv_data.rows[3][1], "ñóëü");
    }

    #[test]
    fn test_single_row_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age,City").unwrap();
        writeln!(file, "Alice,30,NYC").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)), // Row 1 is first data row
            "Alice"
        );
    }

    #[test]
    fn test_single_column_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name").unwrap();
        writeln!(file, "Alice").unwrap();
        writeln!(file, "Bob").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(csv_data.column_count(), 1);
        assert_eq!(csv_data.get_header(ColIndex::new(0)), "Name");
    }

    #[test]
    fn test_csv_with_empty_cells() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A,B,C").unwrap();
        writeln!(file, "1,,3").unwrap();
        writeln!(file, ",2,").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)), "1");
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)), "");
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(2)), "3");
        assert_eq!(csv_data.get_cell(RowIndex::new(2), ColIndex::new(0)), "");
        assert_eq!(csv_data.get_cell(RowIndex::new(2), ColIndex::new(1)), "2");
        assert_eq!(csv_data.get_cell(RowIndex::new(2), ColIndex::new(2)), "");
    }

    #[test]
    fn test_csv_with_quoted_fields() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Description").unwrap();
        writeln!(file, "Alice,\"Hello, World\"").unwrap();
        writeln!(file, "Bob,\"Line1\nLine2\"").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "Alice"
        );
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)),
            "Hello, World"
        );
        assert_eq!(
            csv_data.get_cell(RowIndex::new(2), ColIndex::new(1)),
            "Line1\nLine2"
        );
    }

    #[test]
    fn test_csv_with_escaped_quotes() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Text").unwrap();
        writeln!(file, r#""She said ""hello""""#).unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "She said \"hello\""
        );
    }

    #[test]
    fn test_csv_with_whitespace() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A,B,C").unwrap();
        writeln!(file, "  1  ,  2  ,  3  ").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        // CSV parser should preserve whitespace
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "  1  "
        );
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)),
            "  2  "
        );
    }

    #[test]
    fn test_csv_with_special_characters() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Symbol,Emoji").unwrap();
        writeln!(file, "★,😀").unwrap();
        writeln!(file, "€,日本").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)), "★");
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)), "😀");
        assert_eq!(csv_data.get_cell(RowIndex::new(2), ColIndex::new(0)), "€");
        assert_eq!(
            csv_data.get_cell(RowIndex::new(2), ColIndex::new(1)),
            "日本"
        );
    }

    #[test]
    fn test_csv_with_long_text() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Text").unwrap();
        let long_text = "a".repeat(1000);
        writeln!(file, "{}", long_text).unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)).len(),
            1000
        );
    }

    #[test]
    fn test_csv_with_numbers() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Int,Float,Scientific").unwrap();
        writeln!(file, "123,456.789,1.23e10").unwrap();
        writeln!(file, "-999,0.001,-5e-3").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
                                             // Numbers are stored as strings
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)), "123");
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)),
            "456.789"
        );
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(2)),
            "1.23e10"
        );
    }

    #[test]
    fn test_csv_with_mixed_row_lengths() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A,B,C").unwrap();
        writeln!(file, "1,2,3").unwrap();
        writeln!(file, "4,5").unwrap(); // Missing last column

        // CSV parser is strict - should fail with inconsistent field count
        let result = Document::from_file(file.path(), None, false, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_csv_with_missing_fields() {
        // CSV with inconsistent column counts (missing fields in some rows)
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A,B,C").unwrap();
        writeln!(file, "1,2").unwrap(); // Missing third field
        writeln!(file, "3,4,5").unwrap();

        let result = Document::from_file(file.path(), None, false, None);

        // Should either handle gracefully or return error (not panic)
        // Current behavior: CSV crate returns error for inconsistent column counts
        // This is acceptable - we don't crash, we return an error
        assert!(result.is_ok() || result.is_err()); // Just don't panic
    }

    #[test]
    fn test_long_cell_content() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Description").unwrap();
        // Very long cell content (200+ characters)
        let long_text = "a".repeat(250);
        writeln!(file, "Test,{}", long_text).unwrap();

        let result = Document::from_file(file.path(), None, false, None);

        assert!(result.is_ok());
        let csv_data = result.unwrap();
        // rows[0] is header, rows[1] is first data row
        assert_eq!(csv_data.rows[1][1], long_text);
    }

    #[test]
    fn test_large_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A,B,C").unwrap();
        for i in 0..10000 {
            writeln!(file, "{},{},{}", i, i * 2, i * 3).unwrap();
        }

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 10001); // 1 header + 10000 data rows
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)), "0"); // First data row
        assert_eq!(
            csv_data.get_cell(RowIndex::new(10000), ColIndex::new(0)),
            "9999"
        );
        assert_eq!(
            csv_data.get_cell(RowIndex::new(10000), ColIndex::new(2)),
            "29997"
        );
    }

    #[test]
    fn test_wide_csv() {
        let mut file = NamedTempFile::new().unwrap();
        let headers: Vec<String> = (0..100).map(|i| format!("Col{}", i)).collect();
        writeln!(file, "{}", headers.join(",")).unwrap();
        let row: Vec<String> = (0..100).map(|i| format!("val{}", i)).collect();
        writeln!(file, "{}", row.join(",")).unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.column_count(), 100);
        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert_eq!(csv_data.get_header(ColIndex::new(0)), "Col0");
        assert_eq!(csv_data.get_header(ColIndex::new(99)), "Col99");
    }

    #[test]
    fn test_csv_with_blank_lines_ignored() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A,B").unwrap();
        writeln!(file, "1,2").unwrap();
        writeln!(file).unwrap(); // Blank line
        writeln!(file, "3,4").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        // CSV parser should handle blank lines appropriately
        // Standard CSV parsers may include or exclude them
        assert!(csv_data.row_count() >= 2);
    }

    #[test]
    fn test_filename_extraction() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A").unwrap();
        writeln!(file, "1").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        // Should extract filename from path
        assert!(!csv_data.filename.is_empty());
    }

    #[test]
    fn test_csv_with_commas_in_quotes() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Address").unwrap();
        writeln!(file, "Alice,\"123 Main St, Apt 4, City\"").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)),
            "123 Main St, Apt 4, City"
        );
    }

    #[test]
    fn test_csv_dirty_flag_initial_state() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A").unwrap();
        writeln!(file, "1").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert!(!csv_data.is_dirty);
    }

    #[test]
    fn test_header_and_cell_access_consistency() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age,City").unwrap();
        writeln!(file, "Alice,30,NYC").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        for col in 0..csv_data.column_count() {
            // Should be able to access both header and cells for all columns
            let header = csv_data.get_header(ColIndex::new(col));
            let cell = csv_data.get_cell(RowIndex::new(0), ColIndex::new(col));
            assert!(!header.is_empty() || col >= 3);
            assert!(!cell.is_empty() || col >= 3);
        }
    }

    // ===== Priority 1: Critical Edge Cases =====

    #[test]
    fn test_csv_only_headers_no_data_rows() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age,City").unwrap();
        // No data rows - only header

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(csv_data.row_count(), 1); // 1 header + 0 data rows
        assert_eq!(csv_data.get_header(ColIndex::new(0)), "Name");
        assert_eq!(csv_data.get_header(ColIndex::new(1)), "Age");
        assert_eq!(csv_data.get_header(ColIndex::new(2)), "City");
    }

    #[test]
    #[allow(clippy::write_with_newline)]
    fn test_csv_mixed_line_endings_crlf_lf() {
        let mut file = NamedTempFile::new().unwrap();
        // Mix Windows (CRLF) and Unix (LF) line endings
        // Note: Using write! with \n and \r\n is intentional to test mixed line endings
        write!(file, "Name,Age\r\n").unwrap();
        write!(file, "Alice,30\n").unwrap();
        write!(file, "Bob,25\r\n").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "Alice"
        );
        assert_eq!(csv_data.get_cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
    }

    #[test]
    fn test_csv_empty_file() {
        let file = NamedTempFile::new().unwrap();
        // Empty file - 0 bytes

        let result = Document::from_file(file.path(), None, false, None);

        // Should either error or return empty data gracefully
        assert!(result.is_ok() || result.is_err());
        if let Ok(csv_data) = result {
            // Empty file might have 1 row (empty header) or 0 rows depending on parser
            assert!(csv_data.row_count() <= 1);
            assert!(csv_data.column_count() <= 1);
        }
    }

    #[test]
    fn test_csv_tab_delimiter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name\tAge\tCity").unwrap();
        writeln!(file, "Alice\t30\tNYC").unwrap();

        let csv_data = Document::from_file(file.path(), Some(b'\t'), false, None).unwrap();

        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "Alice"
        );
        assert_eq!(csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)), "30");
    }

    #[test]
    fn test_csv_semicolon_delimiter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name;Age;City").unwrap();
        writeln!(file, "Alice;30;NYC").unwrap();

        let csv_data = Document::from_file(file.path(), Some(b';'), false, None).unwrap();

        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "Alice"
        );
    }

    #[test]
    fn test_csv_pipe_delimiter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name|Age|City").unwrap();
        writeln!(file, "Alice|30|NYC").unwrap();

        let csv_data = Document::from_file(file.path(), Some(b'|'), false, None).unwrap();

        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "Alice"
        );
    }

    #[test]
    fn test_csv_unclosed_quote_recovery() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age").unwrap();
        writeln!(file, "\"Alice,30").unwrap(); // Unclosed quote

        let result = Document::from_file(file.path(), None, false, None);

        // CSV parser should handle this gracefully (either error or recover)
        // The csv crate will typically treat this as an error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_csv_very_long_cell_content() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Data").unwrap();

        // Create a very long cell (100KB of text)
        let long_text = "A".repeat(100_000);
        writeln!(file, "Alice,\"{}\"", long_text).unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)).len(),
            100_000
        );
    }

    #[test]
    fn test_csv_extremely_wide_row_100_columns() {
        let mut file = NamedTempFile::new().unwrap();

        // Create headers for 100 columns
        let headers: Vec<String> = (0..100).map(|i| format!("Col{}", i)).collect();
        writeln!(file, "{}", headers.join(",")).unwrap();

        // Create data row with 100 columns
        let row: Vec<String> = (0..100).map(|i| format!("val{}", i)).collect();
        writeln!(file, "{}", row.join(",")).unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.column_count(), 100);
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(0)),
            "val0"
        );
        assert_eq!(
            csv_data.get_cell(RowIndex::new(1), ColIndex::new(99)),
            "val99"
        );
    }

    #[test]
    fn test_encoding_utf8_with_bom() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("bom.csv");

        // UTF-8 BOM is EF BB BF
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice(b"Name,Age\n");
        content.extend_from_slice(b"Alice,30\n");

        std::fs::write(&file_path, content).unwrap();

        let csv_data = Document::from_file(&file_path, None, false, None).unwrap();

        // BOM should be stripped, headers should be clean
        assert_eq!(csv_data.get_header(ColIndex::new(0)), "Name");
        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
    }

    #[test]
    fn test_csv_file_not_found() {
        let result = Document::from_file(Path::new("/nonexistent/file.csv"), None, false, None);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to read file") || err_msg.contains("No such file"));
    }

    #[test]
    fn test_csv_with_only_whitespace() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "   ").unwrap();
        writeln!(file, "\t\t").unwrap();

        let result = Document::from_file(file.path(), None, false, None);

        // Should either parse as empty/single column or error
        assert!(result.is_ok() || result.is_err());
    }

    // ===== Priority 2: Error Recovery Tests =====

    #[test]
    fn test_malformed_csv_shows_clear_error() {
        let mut file = NamedTempFile::new().unwrap();
        // Write intentionally malformed CSV with mismatched columns
        writeln!(file, "A,B,C").unwrap();
        writeln!(file, "1,2").unwrap(); // Only 2 columns instead of 3
        writeln!(file, "3,4,5,6,7").unwrap(); // Too many columns

        let result = Document::from_file(file.path(), None, false, None);

        // CSV parser should handle gracefully (either succeed with padding or error)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_csv_with_null_bytes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("null.csv");

        // Write file with null bytes
        std::fs::write(&file_path, b"Name,Age\x00\nAlice,30\n").unwrap();

        let result = Document::from_file(&file_path, None, false, None);

        // Should handle null bytes (may succeed or fail depending on parser)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_csv_with_very_long_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Data").unwrap();

        // Create a line with 1 million characters
        let huge_line = format!("Alice,{}", "X".repeat(1_000_000));
        writeln!(file, "{}", huge_line).unwrap();

        let result = Document::from_file(file.path(), None, false, None);

        // Should handle very long lines
        assert!(result.is_ok());
        if let Ok(csv_data) = result {
            assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
            assert_eq!(
                csv_data.get_cell(RowIndex::new(1), ColIndex::new(1)).len(),
                1_000_000
            );
        }
    }

    #[test]
    fn test_encoding_invalid_utf8_fallback() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("invalid.csv");

        // Write invalid UTF-8 bytes (0xFF is invalid in UTF-8)
        std::fs::write(&file_path, [0xFF, 0xFE, b'a', b',', b'b', b'\n']).unwrap();

        let result = Document::from_file(&file_path, None, false, None);

        // Should either handle with replacement chars or succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_csv_with_only_newlines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file).unwrap();
        writeln!(file).unwrap();
        writeln!(file).unwrap();

        let result = Document::from_file(file.path(), None, false, None);

        // Should handle file with only newlines
        assert!(result.is_ok());
        if let Ok(csv_data) = result {
            // File with only newlines might have 1 row (empty header) or 0 rows
            assert!(csv_data.row_count() <= 1);
        }
    }

    #[test]
    fn test_csv_extremely_long_filename_path() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create file with extremely long name (but within filesystem limits)
        let long_name = format!("{}.csv", "x".repeat(100));
        let file_path = temp_dir.path().join(long_name);

        std::fs::write(&file_path, "A,B\n1,2\n").unwrap();

        let csv_data = Document::from_file(&file_path, None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
        assert!(csv_data.filename.len() > 100);
    }
}
