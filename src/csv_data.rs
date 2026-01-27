use anyhow::{Context, Result};
use std::path::Path;

/// Holds parsed CSV data in memory
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
    /// Load CSV from file path
    pub fn from_file(path: &Path) -> Result<Self> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .context(format!("Failed to open CSV file: {}", path.display()))?;

        // Extract headers
        let headers = reader
            .headers()
            .context("Failed to read CSV headers - file may be empty or malformed")?
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Read all rows (memory-bounded for MVP)
        let mut rows = Vec::new();
        for (line_num, result) in reader.records().enumerate() {
            let record =
                result.context(format!("Failed to parse CSV row at line {}", line_num + 2))?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            rows.push(row);
        }

        Ok(CsvData {
            headers,
            rows,
            filename,
            is_dirty: false,
        })
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

    // Phase 2: Cell editing methods (to be implemented)
    // pub fn set_cell(&mut self, row: usize, col: usize, value: String)
    // pub fn save_to_file(&self, path: &Path) -> Result<()>

    // Phase 3: Row/column operations (to be implemented)
    // pub fn add_row(&mut self, at: usize)
    // pub fn delete_row(&mut self, at: usize)
    // pub fn add_column(&mut self, at: usize, header: String)
    // pub fn delete_column(&mut self, at: usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age,City").unwrap();
        writeln!(file, "Alice,30,NYC").unwrap();
        writeln!(file, "Bob,25,LA").unwrap();

        let csv_data = CsvData::from_file(file.path()).unwrap();

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

        let csv_data = CsvData::from_file(file.path()).unwrap();

        assert_eq!(csv_data.column_count(), 2);
        assert_eq!(csv_data.row_count(), 0);
    }

    #[test]
    fn test_get_cell_out_of_bounds() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age").unwrap();
        writeln!(file, "Alice,30").unwrap();

        let csv_data = CsvData::from_file(file.path()).unwrap();

        assert_eq!(csv_data.get_cell(10, 0), ""); // Row out of bounds
        assert_eq!(csv_data.get_cell(0, 10), ""); // Column out of bounds
    }
}
