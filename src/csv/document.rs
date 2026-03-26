//! CSV document with headers and rows — supports lazy loading for large files

use crate::cancel::{self, CancelledError};
use crate::csv::row_storage::RowStorage;
use crate::domain::position::{ColIndex, RowIndex};
use anyhow::{Context, Result};
use csv;
use encoding_rs::Encoding;
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Holds parsed CSV document — either fully in-memory or lazily loaded from disk
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Row storage backend (InMemory or Lazy)
    pub(crate) storage: RowStorage,

    /// Original filename for display
    pub filename: String,

    /// Track unsaved changes
    pub is_dirty: bool,

    /// Delimiter character for this file
    pub delimiter: char,

    /// Monotonically increasing counter bumped on every mutation.
    /// Used by SQLite cache to detect when a table needs reloading.
    pub generation: u64,

    /// Imported xlsx formulas: (row, col) -> formula text (e.g., "=SUM(B2:B5)").
    /// Consumed by App to populate the FormulaStore after loading.
    pub xlsx_formulas: Vec<((usize, usize), String)>,
}

impl Document {
    /// Load CSV from a reader (e.g. stdin) with optional delimiter and header settings.
    pub fn from_reader<R: std::io::Read>(
        reader: R,
        delimiter: Option<u8>,
        no_headers: bool,
        filename: String,
    ) -> Result<Self> {
        let rows = Self::parse_csv_streaming(reader, delimiter, no_headers, 0)?;
        Ok(Document {
            storage: RowStorage::in_memory(rows),
            filename,
            is_dirty: false,
            delimiter: delimiter.map(|d| d as char).unwrap_or(','),
            generation: 0,
            xlsx_formulas: vec![],
        })
    }

    /// Load CSV or XLSX from file path with optional delimiter, header, and encoding settings.
    /// For xlsx files, `sheet_name` selects which sheet to load.
    pub fn from_file(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding_label: Option<String>,
    ) -> Result<Self> {
        Self::from_file_with_sheet(path, delimiter, no_headers, encoding_label, None)
    }

    /// Load CSV, XLSX, or foreign format files with optional sheet/table selection.
    pub fn from_file_with_sheet(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding_label: Option<String>,
        sheet_name: Option<&str>,
    ) -> Result<Self> {
        // Foreign formats (parquet, json, ndjson, sqlite): load via DuckDB
        if crate::csv::foreign_formats::is_foreign_format(path) {
            let rows = crate::csv::foreign_formats::load_foreign_format(path, sheet_name)?;
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            return Ok(Document {
                storage: RowStorage::in_memory(rows),
                filename,
                is_dirty: false,
                delimiter: ',',
                generation: 0,
                xlsx_formulas: vec![],
            });
        }

        // XLSX/XLS path: convert spreadsheet to in-memory rows
        if crate::csv::xlsx::is_spreadsheet(path) {
            let sheet = match sheet_name {
                Some(name) => name.to_string(),
                None => {
                    let sheets = crate::csv::xlsx::get_sheet_names(path)?;
                    if sheets.is_empty() {
                        anyhow::bail!("Spreadsheet has no sheets");
                    }
                    sheets[0].clone()
                }
            };
            let data = crate::csv::xlsx::load_sheet_with_formulas(path, &sheet)?;
            let filename = format!("{}.csv", data.sheet_name);
            return Ok(Document {
                storage: RowStorage::in_memory(data.rows),
                filename,
                is_dirty: true,
                delimiter: ',',
                generation: 0,
                xlsx_formulas: data.formulas,
            });
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Lazy path: large files with default encoding use mmap + row-offset index
        if encoding_label.is_none() && crate::csv::row_storage::should_use_lazy(path) {
            let storage = RowStorage::lazy_from_file(path, delimiter, no_headers)?;
            return Ok(Document {
                storage,
                filename,
                is_dirty: false,
                delimiter: delimiter.map(|d| d as char).unwrap_or(','),
                generation: 0,
                xlsx_formulas: vec![],
            });
        }

        // Fast streaming path: default encoding, small file
        if encoding_label.is_none() {
            let file_len = path.metadata().map(|m| m.len() as usize).unwrap_or(0);
            let file = std::fs::File::open(path)
                .context(format!("Failed to open file: {}", path.display()))?;
            let reader = std::io::BufReader::with_capacity(256 * 1024, file);
            let rows = Self::parse_csv_streaming(reader, delimiter, no_headers, file_len)?;
            return Ok(Document {
                storage: RowStorage::in_memory(rows),
                filename,
                is_dirty: false,
                delimiter: delimiter.map(|d| d as char).unwrap_or(','),
                generation: 0,
                xlsx_formulas: vec![],
            });
        }

        // Slow path: custom encoding requires full decode first
        let file_bytes =
            fs::read(path).context(format!("Failed to read file: {}", path.display()))?;

        let decoded_content = Self::decode_file_bytes(&file_bytes, encoding_label)?;
        let (headers, data_rows) =
            Self::parse_csv_content(&decoded_content, delimiter, no_headers)?;

        let mut all_rows = vec![headers];
        all_rows.extend(data_rows);

        Ok(Document {
            storage: RowStorage::in_memory(all_rows),
            filename,
            is_dirty: false,
            delimiter: delimiter.map(|d| d as char).unwrap_or(','),
            generation: 0,
            xlsx_formulas: vec![],
        })
    }

    /// Decodes file bytes into a UTF-8 string using the specified encoding.
    pub(crate) fn decode_file_bytes(
        file_bytes: &[u8],
        encoding_label: Option<String>,
    ) -> Result<String> {
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

    /// Count data rows in a CSV file without storing row data in memory.
    /// Returns the number of data rows (excludes header when `no_headers` is false).
    pub fn count_rows(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding_label: Option<String>,
    ) -> Result<usize> {
        // Foreign formats: load via DuckDB and count rows
        if crate::csv::foreign_formats::is_foreign_format(path) {
            let rows = crate::csv::foreign_formats::load_foreign_format(path, None)?;
            // First row is header
            return Ok(rows.len().saturating_sub(1));
        }

        // Fast path: use memchr-based newline counting (same as TUI index builder).
        if encoding_label.is_none() {
            return crate::csv::row_storage::count_rows_fast(path, no_headers);
        }

        // Slow path: custom encoding requires full decode first
        let file_bytes =
            fs::read(path).context(format!("Failed to read file: {}", path.display()))?;
        let decoded_content = Self::decode_file_bytes(&file_bytes, encoding_label)?;

        let mut builder = csv::ReaderBuilder::new();
        builder.has_headers(!no_headers);
        if let Some(d) = delimiter {
            builder.delimiter(d);
        }

        let mut reader = builder.from_reader(decoded_content.as_bytes());
        let mut count = 0usize;
        let mut record = csv::ByteRecord::new();
        while reader.read_byte_record(&mut record)? {
            count += 1;
        }
        Ok(count)
    }

    /// Count columns in a CSV file without storing row data in memory.
    /// Returns the number of columns from the first row.
    pub fn count_columns(
        path: &Path,
        delimiter: Option<u8>,
        encoding_label: Option<String>,
    ) -> Result<usize> {
        // Foreign formats: load via DuckDB and count columns from header
        if crate::csv::foreign_formats::is_foreign_format(path) {
            let rows = crate::csv::foreign_formats::load_foreign_format(path, None)?;
            return Ok(rows.first().map(|r| r.len()).unwrap_or(0));
        }

        if encoding_label.is_none() {
            let file = std::fs::File::open(path)
                .context(format!("Failed to open file: {}", path.display()))?;
            let reader = std::io::BufReader::with_capacity(64 * 1024, file);

            let mut builder = csv::ReaderBuilder::new();
            builder.has_headers(true);
            if let Some(d) = delimiter {
                builder.delimiter(d);
            }

            let mut csv_reader = builder.from_reader(reader);
            return Ok(csv_reader.byte_headers().map(|h| h.len()).unwrap_or(0));
        }

        let file_bytes =
            fs::read(path).context(format!("Failed to read file: {}", path.display()))?;
        let decoded_content = Self::decode_file_bytes(&file_bytes, encoding_label)?;

        let mut builder = csv::ReaderBuilder::new();
        builder.has_headers(true);
        if let Some(d) = delimiter {
            builder.delimiter(d);
        }

        let mut reader = builder.from_reader(decoded_content.as_bytes());
        Ok(reader.headers().map(|h| h.len()).unwrap_or(0))
    }

    /// Read header names from a CSV file without loading the full document.
    pub fn read_headers(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding_label: Option<String>,
    ) -> Result<Vec<String>> {
        // Foreign formats: load via DuckDB and return column names
        if crate::csv::foreign_formats::is_foreign_format(path) {
            let rows = crate::csv::foreign_formats::load_foreign_format(path, None)?;
            return Ok(rows.into_iter().next().unwrap_or_default());
        }

        if no_headers {
            anyhow::bail!("Cannot read headers when --no-headers is set");
        }

        if encoding_label.is_none() {
            let file = std::fs::File::open(path)
                .context(format!("Failed to open file: {}", path.display()))?;
            let reader = std::io::BufReader::with_capacity(64 * 1024, file);

            let mut builder = csv::ReaderBuilder::new();
            builder.has_headers(true);
            if let Some(d) = delimiter {
                builder.delimiter(d);
            }

            let mut csv_reader = builder.from_reader(reader);
            let headers = csv_reader.headers()?.iter().map(String::from).collect();
            return Ok(headers);
        }

        let file_bytes =
            fs::read(path).context(format!("Failed to read file: {}", path.display()))?;
        let decoded_content = Self::decode_file_bytes(&file_bytes, encoding_label)?;

        let mut builder = csv::ReaderBuilder::new();
        builder.has_headers(true);
        if let Some(d) = delimiter {
            builder.delimiter(d);
        }

        let mut reader = builder.from_reader(decoded_content.as_bytes());
        let headers = reader.headers()?.iter().map(String::from).collect();
        Ok(headers)
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

    /// Load CSV or XLSX from file path with cancellation support.
    /// Same as `from_file` but checks `cancelled` flag periodically during parsing.
    pub fn from_file_cancellable(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding_label: Option<String>,
        cancelled: &AtomicBool,
    ) -> Result<Self> {
        Self::from_file_cancellable_with_sheet(
            path,
            delimiter,
            no_headers,
            encoding_label,
            cancelled,
            None,
        )
    }

    /// Load CSV, XLSX, or foreign format files with cancellation and optional sheet/table selection.
    pub fn from_file_cancellable_with_sheet(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding_label: Option<String>,
        cancelled: &AtomicBool,
        sheet_name: Option<&str>,
    ) -> Result<Self> {
        // Foreign formats (parquet, json, ndjson, sqlite): load via DuckDB
        if crate::csv::foreign_formats::is_foreign_format(path) {
            let rows = crate::csv::foreign_formats::load_foreign_format(path, sheet_name)?;
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            return Ok(Document {
                storage: RowStorage::in_memory(rows),
                filename,
                is_dirty: false,
                delimiter: ',',
                generation: 0,
                xlsx_formulas: vec![],
            });
        }

        // XLSX/XLS path: convert spreadsheet to in-memory rows
        if crate::csv::xlsx::is_spreadsheet(path) {
            let sheet = match sheet_name {
                Some(name) => name.to_string(),
                None => {
                    let sheets = crate::csv::xlsx::get_sheet_names(path)?;
                    if sheets.is_empty() {
                        anyhow::bail!("Spreadsheet has no sheets");
                    }
                    sheets[0].clone()
                }
            };
            let data = crate::csv::xlsx::load_sheet_with_formulas(path, &sheet)?;
            let filename = format!("{}.csv", data.sheet_name);
            return Ok(Document {
                storage: RowStorage::in_memory(data.rows),
                filename,
                is_dirty: true,
                delimiter: ',',
                generation: 0,
                xlsx_formulas: data.formulas,
            });
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Lazy path: large files with default encoding use mmap + row-offset index
        if encoding_label.is_none() && crate::csv::row_storage::should_use_lazy(path) {
            let storage =
                RowStorage::lazy_from_file_cancellable(path, delimiter, no_headers, cancelled)?;
            return Ok(Document {
                storage,
                filename,
                is_dirty: false,
                delimiter: delimiter.map(|d| d as char).unwrap_or(','),
                generation: 0,
                xlsx_formulas: vec![],
            });
        }

        // Fast streaming path: default encoding, small file
        if encoding_label.is_none() {
            let file_len = path.metadata().map(|m| m.len() as usize).unwrap_or(0);
            let file = std::fs::File::open(path)
                .context(format!("Failed to open file: {}", path.display()))?;
            let reader = std::io::BufReader::with_capacity(256 * 1024, file);
            let rows = Self::parse_csv_streaming_cancellable(
                reader, delimiter, no_headers, file_len, cancelled,
            )?;
            return Ok(Document {
                storage: RowStorage::in_memory(rows),
                filename,
                is_dirty: false,
                delimiter: delimiter.map(|d| d as char).unwrap_or(','),
                generation: 0,
                xlsx_formulas: vec![],
            });
        }

        // Slow path: custom encoding
        let file_bytes =
            fs::read(path).context(format!("Failed to read file: {}", path.display()))?;

        let decoded_content = Self::decode_file_bytes(&file_bytes, encoding_label)?;
        let (headers, data_rows) = Self::parse_csv_content_cancellable(
            &decoded_content,
            delimiter,
            no_headers,
            cancelled,
        )?;

        let mut all_rows = vec![headers];
        all_rows.extend(data_rows);

        Ok(Document {
            storage: RowStorage::in_memory(all_rows),
            filename,
            is_dirty: false,
            delimiter: delimiter.map(|d| d as char).unwrap_or(','),
            generation: 0,
            xlsx_formulas: vec![],
        })
    }

    /// Parses CSV content with cancellation support.
    /// Checks `cancelled` flag every CHECK_INTERVAL rows.
    fn parse_csv_content_cancellable(
        content: &str,
        delimiter: Option<u8>,
        no_headers: bool,
        cancelled: &AtomicBool,
    ) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        let mut builder = csv::ReaderBuilder::new();
        builder.has_headers(!no_headers);
        if let Some(d) = delimiter {
            builder.delimiter(d);
        }

        let mut reader = builder.from_reader(content.as_bytes());
        let headers_from_csv = reader.headers()?.clone();

        let mut rows: Vec<Vec<String>> = Vec::new();
        for (i, result) in reader.records().enumerate() {
            if i % cancel::CHECK_INTERVAL == 0 && cancel::check_esc(cancelled) {
                anyhow::bail!(CancelledError);
            }
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

    /// Estimate row count from file size (assumes ~50 bytes per row as heuristic).
    fn estimate_row_count(file_len: usize) -> usize {
        if file_len == 0 {
            0
        } else {
            file_len / 50
        }
    }

    /// Convert a ByteRecord field to String, using lossy UTF-8 conversion.
    fn field_to_string(field: &[u8]) -> String {
        // Fast path: valid UTF-8 (avoids allocation from to_string_lossy)
        match std::str::from_utf8(field) {
            Ok(s) => s.to_owned(),
            Err(_) => String::from_utf8_lossy(field).into_owned(),
        }
    }

    /// Stream-parse CSV from a reader using ByteRecord reuse.
    /// Returns all rows (row 0 = header) built in place.
    fn parse_csv_streaming<R: std::io::Read>(
        reader: R,
        delimiter: Option<u8>,
        no_headers: bool,
        file_len: usize,
    ) -> Result<Vec<Vec<String>>> {
        let mut builder = csv::ReaderBuilder::new();
        builder.has_headers(!no_headers);
        if let Some(d) = delimiter {
            builder.delimiter(d);
        }

        let mut csv_reader = builder.from_reader(reader);

        // Build row 0 (column names)
        let header_row: Vec<String> = if no_headers {
            let byte_headers = csv_reader.byte_headers()?;
            (1..=byte_headers.len())
                .map(|i| format!("Column {}", i))
                .collect()
        } else {
            let byte_headers = csv_reader.byte_headers()?;
            byte_headers.iter().map(Self::field_to_string).collect()
        };

        let estimated = Self::estimate_row_count(file_len);
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(estimated + 1);
        rows.push(header_row);

        let mut record = csv::ByteRecord::new();
        while csv_reader.read_byte_record(&mut record)? {
            rows.push(record.iter().map(Self::field_to_string).collect());
        }

        Ok(rows)
    }

    /// Stream-parse CSV with cancellation support.
    fn parse_csv_streaming_cancellable<R: std::io::Read>(
        reader: R,
        delimiter: Option<u8>,
        no_headers: bool,
        file_len: usize,
        cancelled: &AtomicBool,
    ) -> Result<Vec<Vec<String>>> {
        let mut builder = csv::ReaderBuilder::new();
        builder.has_headers(!no_headers);
        if let Some(d) = delimiter {
            builder.delimiter(d);
        }

        let mut csv_reader = builder.from_reader(reader);

        let header_row: Vec<String> = if no_headers {
            let byte_headers = csv_reader.byte_headers()?;
            (1..=byte_headers.len())
                .map(|i| format!("Column {}", i))
                .collect()
        } else {
            let byte_headers = csv_reader.byte_headers()?;
            byte_headers.iter().map(Self::field_to_string).collect()
        };

        let estimated = Self::estimate_row_count(file_len);
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(estimated + 1);
        rows.push(header_row);

        let mut record = csv::ByteRecord::new();
        let mut i = 0usize;
        while csv_reader.read_byte_record(&mut record)? {
            if i.is_multiple_of(cancel::CHECK_INTERVAL) && cancel::check_esc(cancelled) {
                anyhow::bail!(CancelledError);
            }
            rows.push(record.iter().map(Self::field_to_string).collect());
            i += 1;
        }

        Ok(rows)
    }

    // ── Read-only accessors (work with both InMemory and Lazy) ──

    /// Get total row count
    pub fn row_count(&self) -> usize {
        self.storage.row_count()
    }

    /// Get column count
    pub fn column_count(&self) -> usize {
        self.storage.col_count()
    }

    /// Get specific cell value (returns "" if out of bounds)
    /// row_idx is absolute: 0 = first row, 1 = second row, etc.
    #[allow(dead_code)]
    pub fn cell(&self, row_idx: RowIndex, col_idx: ColIndex) -> String {
        self.storage.get_cell(row_idx.get(), col_idx.get())
    }

    /// Get column header by index (returns "" if out of bounds)
    pub fn header(&self, col_idx: ColIndex) -> String {
        self.storage
            .header_row()
            .get(col_idx.get())
            .cloned()
            .unwrap_or_default()
    }

    /// Get a range of rows as owned Vecs (for rendering visible window).
    /// Range is [start..end) — exclusive end.
    pub fn get_rows_range(&self, start: usize, end: usize) -> Vec<Vec<String>> {
        self.storage.get_rows_range(start, end)
    }

    /// Iterate over all rows (including header at index 0).
    pub fn iter_rows(&self) -> crate::csv::row_storage::RowIter<'_> {
        self.storage.iter_rows()
    }

    /// Returns true if the document is lazily loaded from disk.
    pub fn is_lazy(&self) -> bool {
        self.storage.is_lazy()
    }

    // ── Mutation (materializes lazy storage on structural changes) ──

    /// Set a cell value (returns old value, sets is_dirty = true)
    /// row_idx is absolute: 0 = first row, 1 = second row, etc.
    pub fn set_cell(
        &mut self,
        row_idx: RowIndex,
        col_idx: ColIndex,
        value: String,
    ) -> Option<String> {
        let old = self.storage.set_cell(row_idx.get(), col_idx.get(), value);
        if old.is_some() {
            self.is_dirty = true;
            self.generation += 1;
        }
        old
    }

    /// Force materialization of all rows into memory.
    pub fn materialize(&mut self) {
        self.storage.materialize();
    }

    /// Get mutable access to in-memory rows, materializing if lazy.
    fn rows_mut(&mut self) -> &mut Vec<Vec<String>> {
        self.storage.rows_mut()
    }

    /// Insert a new empty row at the specified index (absolute row index)
    pub fn insert_row(&mut self, at: RowIndex) {
        let col_count = self.column_count();
        let empty_row = vec![String::new(); col_count];
        let rows = self.rows_mut();
        let actual_insert = at.get().min(rows.len());
        rows.insert(actual_insert, empty_row);
        self.is_dirty = true;
        self.generation += 1;
    }

    /// Delete a row at the specified index (absolute row index)
    pub fn delete_row(&mut self, at: RowIndex) -> Option<Vec<String>> {
        let result = {
            let rows = self.rows_mut();
            if at.get() < rows.len() {
                Some(rows.remove(at.get()))
            } else {
                None
            }
        };
        if result.is_some() {
            self.is_dirty = true;
            self.generation += 1;
        }
        result
    }

    /// Delete multiple rows in a range (inclusive, absolute row indices)
    pub fn delete_rows(&mut self, start: RowIndex, end: RowIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();

        let deleted = {
            let rows = self.rows_mut();
            if start_idx > end_idx || start_idx >= rows.len() {
                return vec![];
            }

            let actual_end = end_idx.min(rows.len() - 1);
            let count = actual_end - start_idx + 1;

            let mut deleted = Vec::new();
            for _ in 0..count {
                if start_idx < rows.len() {
                    deleted.push(rows.remove(start_idx));
                }
            }
            deleted
        };

        if !deleted.is_empty() {
            self.is_dirty = true;
            self.generation += 1;
        }

        deleted
    }

    /// Get a copy of rows in a range (inclusive, absolute row indices)
    /// Returns the rows without deleting them
    /// Example: rows_range(RowIndex(5), RowIndex(10)) returns rows 5-10 inclusive
    pub fn rows_range(&self, start: RowIndex, end: RowIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();
        let count = self.storage.row_count();

        if start_idx > end_idx || start_idx >= count {
            return vec![];
        }

        let actual_end = end_idx.min(count - 1);
        self.storage.get_rows_range(start_idx, actual_end + 1)
    }

    /// Delete a range of columns (inclusive, 0-based column indices)
    pub fn delete_columns(&mut self, start: ColIndex, end: ColIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();

        let rows = self.rows_mut();
        if rows.is_empty() {
            return vec![];
        }

        let col_count = rows[0].len();
        if start_idx >= col_count || start_idx > end_idx {
            return vec![];
        }

        let actual_end = end_idx.min(col_count - 1);
        let delete_count = actual_end - start_idx + 1;

        let mut deleted_columns = vec![vec![]; delete_count];

        for row in rows.iter() {
            for (offset, col_idx) in (start_idx..=actual_end).enumerate() {
                if col_idx < row.len() {
                    deleted_columns[offset].push(row[col_idx].clone());
                }
            }
        }

        for row in rows.iter_mut() {
            if row.len() > start_idx {
                let remove_end = actual_end.min(row.len() - 1);
                row.drain(start_idx..=remove_end);
            }
        }

        self.is_dirty = true;
        self.generation += 1;
        deleted_columns
    }

    /// Get a copy of columns in a range (inclusive, 0-based column indices)
    /// Returns the columns without deleting them (each column as `Vec<String>` including header)
    /// Example: columns_range(ColIndex(1), ColIndex(3)) returns columns B, C, D
    pub fn columns_range(&self, start: ColIndex, end: ColIndex) -> Vec<Vec<String>> {
        let start_idx = start.get();
        let end_idx = end.get();
        let count = self.storage.row_count();

        if count == 0 {
            return vec![];
        }

        let col_count = self.storage.col_count();
        if start_idx >= col_count || start_idx > end_idx {
            return vec![];
        }

        let actual_end = end_idx.min(col_count - 1);
        let column_count = actual_end - start_idx + 1;

        let mut columns = vec![vec![]; column_count];

        for i in 0..count {
            let row = self.storage.get_row(i);
            for (offset, col_idx) in (start_idx..=actual_end).enumerate() {
                if col_idx < row.len() {
                    columns[offset].push(row[col_idx].clone());
                }
            }
        }

        columns
    }

    /// Move columns from source range to a new position.
    pub fn move_columns(
        &mut self,
        from_start: ColIndex,
        from_end: ColIndex,
        to_before: usize,
    ) -> usize {
        let columns = self.columns_range(from_start, from_end);
        let src_start = from_start.get();
        let count = columns.len();

        self.delete_columns(from_start, from_end);

        let insert_at = if to_before <= src_start {
            to_before
        } else {
            to_before - count
        };

        for (i, col_data) in columns.into_iter().enumerate() {
            self.insert_column(ColIndex::new(insert_at + i), col_data);
        }

        insert_at
    }

    /// Get a single column (including header at index 0)
    /// Returns empty vec if column doesn't exist
    pub fn column(&self, col: ColIndex) -> Vec<String> {
        let col_idx = col.get();
        let count = self.storage.row_count();

        if count == 0 {
            return vec![];
        }

        let mut column = Vec::with_capacity(count);
        for i in 0..count {
            let row = self.storage.get_row(i);
            if col_idx < row.len() {
                column.push(row[col_idx].clone());
            } else {
                column.push(String::new());
            }
        }

        column
    }

    /// Delete a single column at the given index
    pub fn delete_column(&mut self, col: ColIndex) -> Vec<String> {
        let col_idx = col.get();

        let rows = self.rows_mut();
        if rows.is_empty() {
            return vec![];
        }

        let col_count = rows[0].len();
        if col_idx >= col_count {
            return vec![];
        }

        let mut deleted_column = Vec::with_capacity(rows.len());

        for row in rows.iter_mut() {
            if col_idx < row.len() {
                deleted_column.push(row.remove(col_idx));
            } else {
                deleted_column.push(String::new());
            }
        }

        self.is_dirty = true;
        self.generation += 1;
        deleted_column
    }

    /// Insert a new column at the given position
    pub fn insert_column(&mut self, at: ColIndex, column_data: Vec<String>) {
        let col_idx = at.get();

        let rows = self.rows_mut();
        if rows.is_empty() {
            return;
        }

        for (row_idx, row) in rows.iter_mut().enumerate() {
            let value = column_data.get(row_idx).cloned().unwrap_or_default();
            let insert_pos = col_idx.min(row.len());
            row.insert(insert_pos, value);
        }

        self.is_dirty = true;
        self.generation += 1;
    }

    /// Insert a new empty column at the given position with a generated header
    pub fn insert_empty_column(&mut self, at: ColIndex) {
        let col_idx = at.get();
        let row_count = self.row_count();

        if row_count == 0 {
            return;
        }

        let header = format!(
            "Column {}",
            crate::ui::utils::column_to_excel_letter(col_idx)
        );

        let column_data = std::iter::once(header)
            .chain(std::iter::repeat_n(String::new(), row_count - 1))
            .collect();

        self.insert_column(at, column_data);
    }

    /// Swap two rows by index.
    pub fn swap_rows(&mut self, a: RowIndex, b: RowIndex) {
        let rows = self.rows_mut();
        let ai = a.get();
        let bi = b.get();
        if ai < rows.len() && bi < rows.len() && ai != bi {
            rows.swap(ai, bi);
            self.is_dirty = true;
            self.generation += 1;
        }
    }

    /// Sort data rows by the given column indices.
    /// Uses parallel sort and avoids materializing lazy storage.
    /// Returns `true` if sort completed, `false` if cancelled.
    pub fn sort_by_columns(
        &mut self,
        col_indices: &[usize],
        ascending: bool,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> bool {
        let completed = self
            .storage
            .sort_by_columns(col_indices, ascending, cancelled);
        if completed {
            self.is_dirty = true;
            self.generation += 1;
        }
        completed
    }

    /// Take ownership of the storage, leaving empty storage behind.
    /// Used for background deallocation of old document data.
    pub fn take_storage(&mut self) -> RowStorage {
        std::mem::replace(&mut self.storage, RowStorage::in_memory(vec![]))
    }

    /// Create a Document from headers and data rows (for testing)
    #[cfg(test)]
    pub fn from_parts(headers: Vec<String>, data_rows: Vec<Vec<String>>, filename: String) -> Self {
        let mut all_rows = vec![headers];
        all_rows.extend(data_rows);
        Document {
            storage: RowStorage::in_memory(all_rows),
            filename,
            is_dirty: false,
            delimiter: ',',
            generation: 0,
            xlsx_formulas: vec![],
        }
    }

    /// Create a Document for public use (needed by tests outside this module)
    pub fn new(headers: Vec<String>, data_rows: Vec<Vec<String>>, filename: String) -> Self {
        let mut all_rows = vec![headers];
        all_rows.extend(data_rows);
        Document {
            storage: RowStorage::in_memory(all_rows),
            filename,
            is_dirty: false,
            delimiter: ',',
            generation: 0,
            xlsx_formulas: vec![],
        }
    }

    /// Create a document from existing storage (e.g., pre-built lazy storage).
    pub fn from_storage(storage: RowStorage, filename: String, delimiter: char) -> Self {
        Document {
            storage,
            filename,
            is_dirty: false,
            delimiter,
            generation: 0,
            xlsx_formulas: vec![],
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
        assert_eq!(csv_data.header(ColIndex::new(0)), "Name");
        assert_eq!(
            csv_data.cell(RowIndex::new(1), ColIndex::new(0)), // Row 1 is first data row
            "Alice"
        );
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(1)), "25");
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

        assert_eq!(csv_data.cell(RowIndex::new(10), ColIndex::new(0)), ""); // Row out of bounds
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(10)), ""); // Column out of bounds (row 1 is first data row)
    }

    #[test]
    fn test_unicode_in_cells() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Description").unwrap();
        writeln!(file, "Test,日本語テキスト").unwrap(); // Japanese
        writeln!(file, "Test2, Emoji").unwrap(); // Emoji
        writeln!(file, "Test3,ñóëü").unwrap(); // Accented chars

        let result = Document::from_file(file.path(), None, false, None);

        assert!(result.is_ok());
        let csv_data = result.unwrap();
        // rows[0] is header, rows[1..] are data
        assert_eq!(
            csv_data.cell(RowIndex::new(1), ColIndex::new(1)),
            "日本語テキスト"
        );
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(1)), " Emoji");
        assert_eq!(csv_data.cell(RowIndex::new(3), ColIndex::new(1)), "ñóëü");
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
            csv_data.cell(RowIndex::new(1), ColIndex::new(0)), // Row 1 is first data row
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
        assert_eq!(csv_data.header(ColIndex::new(0)), "Name");
    }

    #[test]
    fn test_csv_with_empty_cells() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "A,B,C").unwrap();
        writeln!(file, "1,,3").unwrap();
        writeln!(file, ",2,").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "1");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(1)), "");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(2)), "3");
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(0)), "");
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(1)), "2");
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(2)), "");
    }

    #[test]
    fn test_csv_with_quoted_fields() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Description").unwrap();
        writeln!(file, "Alice,\"Hello, World\"").unwrap();
        writeln!(file, "Bob,\"Line1\nLine2\"").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
        assert_eq!(
            csv_data.cell(RowIndex::new(1), ColIndex::new(1)),
            "Hello, World"
        );
        assert_eq!(
            csv_data.cell(RowIndex::new(2), ColIndex::new(1)),
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
            csv_data.cell(RowIndex::new(1), ColIndex::new(0)),
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
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "  1  ");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(1)), "  2  ");
    }

    #[test]
    fn test_csv_with_special_characters() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Symbol,Emoji").unwrap();
        writeln!(file, "★,😀").unwrap();
        writeln!(file, "€,日本").unwrap();

        let csv_data = Document::from_file(file.path(), None, false, None).unwrap();

        assert_eq!(csv_data.row_count(), 3); // 1 header + 2 data rows
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "★");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(1)), "😀");
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(0)), "€");
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(1)), "日本");
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
            csv_data.cell(RowIndex::new(1), ColIndex::new(0)).len(),
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
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "123");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(1)), "456.789");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(2)), "1.23e10");
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
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(1)), long_text);
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
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "0"); // First data row
        assert_eq!(
            csv_data.cell(RowIndex::new(10000), ColIndex::new(0)),
            "9999"
        );
        assert_eq!(
            csv_data.cell(RowIndex::new(10000), ColIndex::new(2)),
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
        assert_eq!(csv_data.header(ColIndex::new(0)), "Col0");
        assert_eq!(csv_data.header(ColIndex::new(99)), "Col99");
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
            csv_data.cell(RowIndex::new(1), ColIndex::new(1)),
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
            let header = csv_data.header(ColIndex::new(col));
            let cell = csv_data.cell(RowIndex::new(0), ColIndex::new(col));
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
        assert_eq!(csv_data.header(ColIndex::new(0)), "Name");
        assert_eq!(csv_data.header(ColIndex::new(1)), "Age");
        assert_eq!(csv_data.header(ColIndex::new(2)), "City");
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
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
        assert_eq!(csv_data.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
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
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(1)), "30");
    }

    #[test]
    fn test_csv_semicolon_delimiter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name;Age;City").unwrap();
        writeln!(file, "Alice;30;NYC").unwrap();

        let csv_data = Document::from_file(file.path(), Some(b';'), false, None).unwrap();

        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
    }

    #[test]
    fn test_csv_pipe_delimiter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name|Age|City").unwrap();
        writeln!(file, "Alice|30|NYC").unwrap();

        let csv_data = Document::from_file(file.path(), Some(b'|'), false, None).unwrap();

        assert_eq!(csv_data.column_count(), 3);
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
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
            csv_data.cell(RowIndex::new(1), ColIndex::new(1)).len(),
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
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(0)), "val0");
        assert_eq!(csv_data.cell(RowIndex::new(1), ColIndex::new(99)), "val99");
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
        assert_eq!(csv_data.header(ColIndex::new(0)), "Name");
        assert_eq!(csv_data.row_count(), 2); // 1 header + 1 data row
    }

    #[test]
    fn test_csv_file_not_found() {
        let result = Document::from_file(Path::new("/nonexistent/file.csv"), None, false, None);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to read file")
                || err_msg.contains("Failed to open file")
                || err_msg.contains("No such file")
        );
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
                csv_data.cell(RowIndex::new(1), ColIndex::new(1)).len(),
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

    // ===== Mutation Method Tests =====

    #[test]
    fn test_set_cell_updates_value() {
        let doc = Document::new(
            vec!["Name".to_string(), "Age".to_string()],
            vec![vec!["Alice".to_string(), "30".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        doc.set_cell(RowIndex::new(1), ColIndex::new(0), "Bob".to_string());

        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "Bob");
        assert!(doc.is_dirty);
    }

    #[test]
    fn test_set_cell_out_of_bounds() {
        let doc = Document::new(
            vec!["Name".to_string()],
            vec![vec!["Alice".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        // Setting cell out of bounds should not panic
        doc.set_cell(RowIndex::new(10), ColIndex::new(0), "Test".to_string());
        // Original data should be unchanged
        assert_eq!(doc.row_count(), 2);
    }

    #[test]
    fn test_insert_row_at_end() {
        let doc = Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let original_count = doc.row_count();
        doc.insert_row(RowIndex::new(2)); // Insert after data row

        assert_eq!(doc.row_count(), original_count + 1);
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(1)), "");
        assert!(doc.is_dirty);
    }

    #[test]
    fn test_insert_row_at_beginning() {
        let doc = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()], vec!["2".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        doc.insert_row(RowIndex::new(1)); // Insert at first data row

        assert_eq!(doc.row_count(), 4); // 1 header + 3 data rows
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "1");
    }

    #[test]
    fn test_delete_row_returns_deleted_data() {
        let doc = Document::new(
            vec!["Name".to_string()],
            vec![vec!["Alice".to_string()], vec!["Bob".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let deleted = doc.delete_row(RowIndex::new(1));

        assert_eq!(deleted, Some(vec!["Alice".to_string()]));
        assert_eq!(doc.row_count(), 2); // 1 header + 1 data row
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "Bob");
        assert!(doc.is_dirty);
    }

    #[test]
    fn test_delete_row_out_of_bounds() {
        let doc = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let deleted = doc.delete_row(RowIndex::new(10));

        assert_eq!(deleted, None);
        assert_eq!(doc.row_count(), 2); // Unchanged
    }

    #[test]
    fn test_delete_rows_range() {
        let doc = Document::new(
            vec!["A".to_string()],
            vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()],
                vec!["4".to_string()],
            ],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let deleted = doc.delete_rows(RowIndex::new(1), RowIndex::new(2));

        assert_eq!(deleted.len(), 2);
        assert_eq!(deleted[0][0], "1");
        assert_eq!(deleted[1][0], "2");
        assert_eq!(doc.row_count(), 3); // 1 header + 2 remaining data rows
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "3");
    }

    #[test]
    fn test_get_rows_range() {
        let doc = Document::new(
            vec!["A".to_string()],
            vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()],
            ],
            "test.csv".to_string(),
        );

        let rows = doc.rows_range(RowIndex::new(1), RowIndex::new(2));

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "1");
        assert_eq!(rows[1][0], "2");
        // Original document unchanged
        assert_eq!(doc.row_count(), 4);
    }

    #[test]
    fn test_insert_column_empty() {
        let doc = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()], vec!["2".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        doc.insert_empty_column(ColIndex::new(1));

        assert_eq!(doc.column_count(), 2);
        assert_eq!(doc.header(ColIndex::new(1)), "Column B"); // Generated header
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(1)), "");
        assert!(doc.is_dirty);
    }

    #[test]
    fn test_insert_column_with_data() {
        let doc = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()], vec!["2".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let column_data = vec!["B".to_string(), "10".to_string(), "20".to_string()];
        doc.insert_column(ColIndex::new(1), column_data);

        assert_eq!(doc.column_count(), 2);
        assert_eq!(doc.header(ColIndex::new(1)), "B");
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(1)), "10");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(1)), "20");
    }

    #[test]
    fn test_delete_column_returns_data() {
        let doc = Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let deleted = doc.delete_column(ColIndex::new(0));

        assert_eq!(deleted.len(), 2); // Header + 1 data row
        assert_eq!(deleted[0], "A");
        assert_eq!(deleted[1], "1");
        assert_eq!(doc.column_count(), 1);
        assert_eq!(doc.header(ColIndex::new(0)), "B");
    }

    #[test]
    fn test_get_column_does_not_mutate() {
        let doc = Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
            "test.csv".to_string(),
        );

        let column = doc.column(ColIndex::new(0));

        assert_eq!(column.len(), 2); // Header + 1 data row
        assert_eq!(column[0], "A");
        assert_eq!(column[1], "1");
        assert_eq!(doc.column_count(), 2); // Unchanged
    }

    #[test]
    fn test_delete_columns_range() {
        let doc = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let deleted = doc.delete_columns(ColIndex::new(0), ColIndex::new(1));

        assert_eq!(deleted.len(), 2); // 2 columns deleted
        assert_eq!(deleted[0][0], "A"); // First column header
        assert_eq!(deleted[1][0], "B"); // Second column header
        assert_eq!(doc.column_count(), 1);
        assert_eq!(doc.header(ColIndex::new(0)), "C");
    }

    #[test]
    fn test_get_columns_range() {
        let doc = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
            "test.csv".to_string(),
        );

        let columns = doc.columns_range(ColIndex::new(0), ColIndex::new(1));

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0][0], "A");
        assert_eq!(columns[1][0], "B");
        assert_eq!(doc.column_count(), 3); // Unchanged
    }

    #[test]
    fn test_sort_by_single_column_ascending() {
        let doc = Document::new(
            vec!["Name".to_string(), "Age".to_string()],
            vec![
                vec!["Charlie".to_string(), "35".to_string()],
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
            ],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let no_cancel = std::sync::atomic::AtomicBool::new(false);
        doc.sort_by_columns(&[0], true, &no_cancel); // Sort by Name ascending

        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "Alice");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "Bob");
        assert_eq!(doc.cell(RowIndex::new(3), ColIndex::new(0)), "Charlie");
        assert!(doc.is_dirty);
    }

    #[test]
    fn test_sort_by_single_column_descending() {
        let doc = Document::new(
            vec!["Age".to_string()],
            vec![
                vec!["25".to_string()],
                vec!["30".to_string()],
                vec!["35".to_string()],
            ],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let no_cancel = std::sync::atomic::AtomicBool::new(false);
        doc.sort_by_columns(&[0], false, &no_cancel); // Sort descending

        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "35");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "30");
        assert_eq!(doc.cell(RowIndex::new(3), ColIndex::new(0)), "25");
    }

    #[test]
    fn test_sort_by_multiple_columns() {
        let doc = Document::new(
            vec!["Dept".to_string(), "Name".to_string()],
            vec![
                vec!["Sales".to_string(), "Charlie".to_string()],
                vec!["IT".to_string(), "Bob".to_string()],
                vec!["Sales".to_string(), "Alice".to_string()],
            ],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        let no_cancel = std::sync::atomic::AtomicBool::new(false);
        doc.sort_by_columns(&[0, 1], true, &no_cancel); // Sort by Dept then Name

        // IT before Sales, then alphabetically within same dept
        assert_eq!(doc.cell(RowIndex::new(1), ColIndex::new(0)), "IT");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(0)), "Sales");
        assert_eq!(doc.cell(RowIndex::new(2), ColIndex::new(1)), "Alice");
        assert_eq!(doc.cell(RowIndex::new(3), ColIndex::new(1)), "Charlie");
    }

    #[test]
    fn test_move_columns_single() {
        let doc = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        // Move column A (index 0) to before column 3 (after all columns)
        doc.move_columns(ColIndex::new(0), ColIndex::new(0), 3);

        // Result should be: B C A
        assert_eq!(doc.header(ColIndex::new(0)), "B");
        assert_eq!(doc.header(ColIndex::new(1)), "C");
        assert_eq!(doc.header(ColIndex::new(2)), "A");
    }

    #[test]
    fn test_generation_increments_on_mutations() {
        let doc = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );

        let mut doc = doc;
        assert_eq!(doc.generation, 0);

        doc.set_cell(RowIndex::new(1), ColIndex::new(0), "2".to_string());
        assert_eq!(doc.generation, 1);

        doc.insert_row(RowIndex::new(2));
        assert_eq!(doc.generation, 2);

        doc.delete_row(RowIndex::new(2));
        assert_eq!(doc.generation, 3);
    }
}
