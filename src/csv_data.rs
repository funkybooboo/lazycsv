use anyhow::{Context, Result};
use csv;
use encoding_rs::Encoding;
use std::fs;
use std::path::Path;

/// Holds parsed CSV data in memory
#[derive(Debug)]
pub struct CsvData {
    /// Column headers (first row)
    pub headers: Vec<String>,

    /// All data rows (excluding header)
    pub rows: Vec<Vec<String>>,

    /// Original filename for display
    pub filename: String,

    /// Track unsaved changes (Phase 2)
    pub is_dirty: bool,
}

impl CsvData {
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
        let (headers, rows) = Self::parse_csv_content(&decoded_content, delimiter, no_headers)?;

        Ok(CsvData {
            headers,
            rows,
            filename,
            is_dirty: false,
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

    /// Get total row count (excluding headers)
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get column count
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    /// Get specific cell value (returns "" if out of bounds)
    #[allow(dead_code)]
    pub fn get_cell(&self, row: usize, col: usize) -> &str {
        self.rows
            .get(row)
            .and_then(|r| r.get(col))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get column header by index (returns "" if out of bounds)
    pub fn get_header(&self, col: usize) -> &str {
        self.headers.get(col).map(|s| s.as_str()).unwrap_or("")
    }

    // v0.4.0-v0.6.0: Cell editing methods (to be implemented)
    // pub fn set_cell(&mut self, row: usize, col: usize, value: String)
    // pub fn save_to_file(&self, path: &Path) -> Result<()>

    // v0.7.0-v0.8.0: Row/column operations (to be implemented)
    // pub fn add_row(&mut self, at: usize)
    // pub fn delete_row(&mut self, at: usize)
    // pub fn add_column(&mut self, at: usize, header: String)
    // pub fn delete_column(&mut self, at: usize)
}
