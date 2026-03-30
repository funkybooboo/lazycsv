/// CSV writer module - handles writing Document back to CSV files
///
/// Key features:
/// - Atomic writes (write to temp file, then rename)
/// - Proper CSV escaping (quotes, commas, newlines)
/// - Preserves original file on write failure
use crate::csv::Document;
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Write a Document to a CSV file atomically
///
/// Process:
/// 1. Write to a temporary file in the same directory
/// 2. If successful, atomically rename to target path
/// 3. If failure, temp file is cleaned up and original is preserved
///
/// # Arguments
/// * `document` - The document to write
/// * `path` - Target file path
/// * `delimiter` - CSV delimiter character (usually ',')
pub fn write_csv_atomic(document: &Document, path: &Path, delimiter: char) -> Result<()> {
    // Create temp file in same directory for atomic rename
    let temp_path = path.with_extension("tmp");

    // Write to temp file with buffering for performance
    {
        let file = fs::File::create(&temp_path)
            .context(format!("Failed to create temp file: {:?}", temp_path))?;
        let mut writer = std::io::BufWriter::with_capacity(8 * 1024 * 1024, file);

        write_csv_content(&mut writer, document, delimiter)
            .context("Failed to write CSV content")?;

        // Flush buffer and sync data (not metadata) to disk
        let file = writer
            .into_inner()
            .map_err(|e| anyhow::anyhow!("Failed to flush write buffer: {}", e))?;
        file.sync_data().context("Failed to sync file to disk")?;
    }

    // Atomically rename temp to target (overwrites if exists)
    fs::rename(&temp_path, path).context(format!("Failed to rename temp file to {:?}", path))?;

    Ok(())
}

/// Write CSV content to a writer
pub fn write_csv_content<W: Write>(
    writer: &mut W,
    document: &Document,
    delimiter: char,
) -> Result<()> {
    use crate::csv::row_storage::RowStorage;

    match &document.storage {
        RowStorage::Lazy(s) => {
            // Fast path: write raw mmap bytes for unedited rows, avoiding parse + re-serialize.
            // Header is always written from parsed data (it may differ from raw bytes).
            write_csv_row(writer, s.header(), delimiter)?;

            let count = s.row_offsets().len();
            let edits = s.edits();
            let raw = s.raw_bytes();
            let offsets = s.row_offsets();
            let sort_order = s.sort_order();

            // When unsorted, batch contiguous unedited row ranges into single bulk writes.
            if sort_order.is_none() {
                // Pre-sort edits by index to avoid HashMap lookups in the loop
                let mut sorted_edits: Vec<(usize, &Vec<String>)> = edits
                    .iter()
                    .map(|(&idx, row)| (idx, row))
                    .filter(|&(idx, _)| idx >= 1 && idx < count)
                    .collect();
                sorted_edits.sort_unstable_by_key(|&(idx, _)| idx);

                // Reusable buffer for serializing edited rows (avoids per-row allocation)
                let delim_byte = delimiter as u8;
                let mut row_buf = Vec::with_capacity(256);

                let mut cursor = 1usize;
                for &(edit_idx, edited_row) in &sorted_edits {
                    if edit_idx < cursor {
                        continue;
                    }
                    // Write the contiguous unedited range [cursor, edit_idx) directly from mmap
                    if cursor < edit_idx {
                        let bulk_start = offsets[cursor] as usize;
                        let bulk_end = offsets[edit_idx] as usize;
                        if bulk_start < bulk_end {
                            writer.write_all(&raw[bulk_start..bulk_end])?;
                        }
                    }
                    // Write the edited row using reusable buffer
                    row_buf.clear();
                    for (i, cell) in edited_row.iter().enumerate() {
                        if i > 0 {
                            row_buf.push(delim_byte);
                        }
                        append_csv_cell(&mut row_buf, cell, delimiter);
                    }
                    row_buf.push(b'\n');
                    writer.write_all(&row_buf)?;

                    cursor = edit_idx + 1;
                }
                // Write the remaining unedited tail
                if cursor < count {
                    let bulk_start = offsets[cursor] as usize;
                    let bulk_end = raw.len();
                    if bulk_start < bulk_end {
                        writer.write_all(&raw[bulk_start..bulk_end])?;
                        if raw[raw.len() - 1] != b'\n' {
                            writer.write_all(b"\n")?;
                        }
                    }
                }
            } else {
                // Sorted: physical order differs from logical, write row by row
                for logical_idx in 1..count {
                    if let Some(edited_row) = edits.get(&logical_idx) {
                        write_csv_row(writer, edited_row, delimiter)?;
                    } else {
                        let phys = match sort_order {
                            Some(order) => order[logical_idx - 1],
                            None => logical_idx,
                        };
                        let start = offsets[phys] as usize;
                        let end = if phys + 1 < offsets.len() {
                            offsets[phys + 1] as usize
                        } else {
                            raw.len()
                        };
                        let row_bytes = &raw[start..end];
                        let mut trim_end = row_bytes.len();
                        while trim_end > 0
                            && (row_bytes[trim_end - 1] == b'\n'
                                || row_bytes[trim_end - 1] == b'\r')
                        {
                            trim_end -= 1;
                        }
                        writer.write_all(&row_bytes[..trim_end])?;
                        writeln!(writer)?;
                    }
                }
            }
            Ok(())
        }
        RowStorage::InMemory { .. } => {
            // Standard path: write all rows with escaping
            for row in document.iter_rows() {
                write_csv_row(writer, &row, delimiter)?;
            }
            Ok(())
        }
    }
}

/// Write a single CSV row with proper escaping.
/// Builds the entire row into a reusable buffer to minimize write syscalls.
fn write_csv_row<W: Write>(writer: &mut W, row: &[String], delimiter: char) -> Result<()> {
    let delim_byte = delimiter as u8;
    // Estimate capacity: sum of cell lengths + delimiters + newline
    let cap: usize = row.iter().map(|c| c.len() + 3).sum();
    let mut buf = Vec::with_capacity(cap);

    for (i, cell) in row.iter().enumerate() {
        if i > 0 {
            buf.push(delim_byte);
        }
        append_csv_cell(&mut buf, cell, delimiter);
    }
    buf.push(b'\n');

    writer.write_all(&buf)?;
    Ok(())
}

/// Append a properly escaped CSV cell to a byte buffer.
///
/// Escaping rules:
/// - If cell contains delimiter, quotes, or newlines: wrap in quotes
/// - If cell contains quotes: double them (e.g., " becomes "")
fn append_csv_cell(buf: &mut Vec<u8>, cell: &str, delimiter: char) {
    let needs_quotes = cell.contains(delimiter)
        || cell.contains('"')
        || cell.contains('\n')
        || cell.contains('\r');

    if needs_quotes {
        buf.push(b'"');
        for &byte in cell.as_bytes() {
            if byte == b'"' {
                buf.push(b'"');
            }
            buf.push(byte);
        }
        buf.push(b'"');
    } else {
        buf.extend_from_slice(cell.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use tempfile::TempDir;

    #[test]
    fn test_write_simple_csv() {
        let doc = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
                vec!["4".to_string(), "5".to_string(), "6".to_string()],
            ],
            "test.csv".to_string(),
        );

        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "A,B,C\n1,2,3\n4,5,6\n");
    }

    #[test]
    fn test_escape_quotes() {
        let doc = Document::new(
            vec!["Name".to_string()],
            vec![vec!["He said \"Hello\"".to_string()]],
            "test.csv".to_string(),
        );

        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Name\n\"He said \"\"Hello\"\"\"\n");
    }

    #[test]
    fn test_escape_commas() {
        let doc = Document::new(
            vec!["Name".to_string()],
            vec![vec!["Last, First".to_string()]],
            "test.csv".to_string(),
        );

        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Name\n\"Last, First\"\n");
    }

    #[test]
    fn test_escape_newlines() {
        let doc = Document::new(
            vec!["Description".to_string()],
            vec![vec!["Line 1\nLine 2".to_string()]],
            "test.csv".to_string(),
        );

        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Description\n\"Line 1\nLine 2\"\n");
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.csv");

        let doc = Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
            "test.csv".to_string(),
        );

        // Write the file
        write_csv_atomic(&doc, &file_path, ',').unwrap();

        // Verify it was written correctly
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "A,B\n1,2\n");

        // Verify temp file was cleaned up
        let temp_path = file_path.with_extension("tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.csv");

        // Write original file
        fs::write(&file_path, "old,content\n").unwrap();

        let doc = Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
            "test.csv".to_string(),
        );

        // Overwrite with atomic write
        write_csv_atomic(&doc, &file_path, ',').unwrap();

        // Verify new content
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "A,B\n1,2\n");
    }

    #[test]
    fn test_write_sorted_lazy_file() {
        use crate::csv::row_storage::RowStorage;

        // Create a temp CSV file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("sort_test.csv");
        fs::write(&file_path, "Name,Value\nCharlie,3\nAlice,1\nBob,2\n").unwrap();

        // Force load as lazy storage (bypasses size threshold)
        let storage = RowStorage::lazy_from_file(&file_path, None, false).unwrap();
        let mut doc = Document::from_storage(storage, "sort_test.csv".to_string(), ',');
        assert!(doc.storage.is_lazy(), "Should be lazy storage");

        // Verify pre-sort order
        assert_eq!(doc.iter_rows().nth(1).unwrap()[0], "Charlie");
        assert_eq!(doc.iter_rows().nth(2).unwrap()[0], "Alice");
        assert_eq!(doc.iter_rows().nth(3).unwrap()[0], "Bob");

        // Sort by Name (column 0) ascending
        let no_cancel = std::sync::atomic::AtomicBool::new(false);
        doc.sort_by_columns(&[0], true, &no_cancel);
        assert!(doc.storage.is_lazy(), "Should still be lazy after sort");

        // Verify sorted order via get_row
        assert_eq!(doc.iter_rows().nth(1).unwrap()[0], "Alice");
        assert_eq!(doc.iter_rows().nth(2).unwrap()[0], "Bob");
        assert_eq!(doc.iter_rows().nth(3).unwrap()[0], "Charlie");

        // Write to buffer and verify output is sorted
        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();
        let result = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = result.trim().split('\n').collect();
        assert_eq!(lines[0], "Name,Value");
        assert_eq!(lines[1], "Alice,1");
        assert_eq!(lines[2], "Bob,2");
        assert_eq!(lines[3], "Charlie,3");
    }

    #[test]
    fn test_write_sorted_lazy_file_crlf() {
        use crate::csv::row_storage::RowStorage;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("crlf_test.csv");
        // Write with \r\n line endings (like Windows/weather file)
        fs::write(
            &file_path,
            "Name,Value\r\nCharlie,3\r\nAlice,1\r\nBob,2\r\n",
        )
        .unwrap();

        let storage = RowStorage::lazy_from_file(&file_path, None, false).unwrap();
        let mut doc = Document::from_storage(storage, "crlf_test.csv".to_string(), ',');

        let no_cancel = std::sync::atomic::AtomicBool::new(false);
        doc.sort_by_columns(&[0], true, &no_cancel);

        // Verify sorted in memory
        assert_eq!(doc.iter_rows().nth(1).unwrap()[0], "Alice");
        assert_eq!(doc.iter_rows().nth(2).unwrap()[0], "Bob");
        assert_eq!(doc.iter_rows().nth(3).unwrap()[0], "Charlie");

        // Write to buffer
        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();
        let result = String::from_utf8(output).unwrap();
        // Normalize line endings for comparison
        let result = result.replace("\r\n", "\n");
        let lines: Vec<&str> = result.trim().split('\n').collect();
        assert_eq!(lines.len(), 4, "Should have header + 3 data rows");
        assert_eq!(lines[0], "Name,Value");
        assert_eq!(lines[1], "Alice,1");
        assert_eq!(lines[2], "Bob,2");
        assert_eq!(lines[3], "Charlie,3");
    }

    #[test]
    fn test_write_sorted_lazy_file_atomic_roundtrip() {
        use crate::csv::row_storage::RowStorage;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("roundtrip.csv");
        fs::write(
            &file_path,
            "ID,Name,Score\n3,Charlie,90\n1,Alice,85\n2,Bob,95\n",
        )
        .unwrap();

        // Load as lazy, sort by ID (numeric), save, re-read
        let storage = RowStorage::lazy_from_file(&file_path, None, false).unwrap();
        let mut doc = Document::from_storage(storage, "roundtrip.csv".to_string(), ',');

        let no_cancel = std::sync::atomic::AtomicBool::new(false);
        doc.sort_by_columns(&[0], true, &no_cancel);

        // Write atomically
        let out_path = temp_dir.path().join("output.csv");
        write_csv_atomic(&doc, &out_path, ',').unwrap();

        // Re-read and verify sorted order on disk
        let content = fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = content.trim().split('\n').collect();
        assert_eq!(lines[0], "ID,Name,Score");
        assert_eq!(lines[1], "1,Alice,85");
        assert_eq!(lines[2], "2,Bob,95");
        assert_eq!(lines[3], "3,Charlie,90");
    }

    #[test]
    fn test_write_lazy_file_after_insert_row_atomic_roundtrip() {
        use crate::domain::position::{ColIndex, RowIndex};

        // Create a CSV file, load it, insert a row, save back to same file, verify on disk
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("roundtrip_insert.csv");
        fs::write(&file_path, "Name,Age,City\nAlice,30,NYC\nBob,25,LA\n").unwrap();

        // Load from file (InMemory since small)
        let mut doc = Document::from_file(&file_path, None, false, None).unwrap();

        // Insert row at end and set values
        let new_row_idx = RowIndex::new(doc.row_count());
        doc.insert_row(new_row_idx);
        doc.set_cell(new_row_idx, ColIndex::new(0), "Charlie".to_string());
        doc.set_cell(new_row_idx, ColIndex::new(1), "35".to_string());
        doc.set_cell(new_row_idx, ColIndex::new(2), "Chicago".to_string());

        // Save back to same file (like :wq does)
        let delimiter = doc.delimiter;
        write_csv_atomic(&doc, &file_path, delimiter).unwrap();

        // Read back and verify
        let content = fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = content.trim().split('\n').collect();
        assert_eq!(
            lines.len(),
            4,
            "Should have header + 3 data rows, got: {:?}",
            lines
        );
        assert_eq!(lines[0], "Name,Age,City");
        assert_eq!(lines[1], "Alice,30,NYC");
        assert_eq!(lines[2], "Bob,25,LA");
        assert_eq!(lines[3], "Charlie,35,Chicago");
    }

    #[test]
    fn test_write_lazy_file_after_insert_row() {
        use crate::csv::row_storage::RowStorage;
        use crate::domain::position::{ColIndex, RowIndex};

        // Create a CSV file that will load as Lazy storage
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("insert_test.csv");
        fs::write(&file_path, "Name,Age,City\nAlice,30,NYC\nBob,25,LA\n").unwrap();

        let storage = RowStorage::lazy_from_file(&file_path, None, false).unwrap();
        let mut doc = Document::from_storage(storage, "insert_test.csv".to_string(), ',');
        assert!(doc.storage.is_lazy(), "Should start as lazy storage");

        // Insert a new row at the end (row index 3 = after Bob)
        let new_row_idx = RowIndex::new(doc.row_count());
        doc.insert_row(new_row_idx);

        // After insert_row, storage should be materialized to InMemory
        assert!(!doc.storage.is_lazy(), "Should be InMemory after insert");

        // Set cell values on the new row
        doc.set_cell(new_row_idx, ColIndex::new(0), "Charlie".to_string());
        doc.set_cell(new_row_idx, ColIndex::new(1), "35".to_string());
        doc.set_cell(new_row_idx, ColIndex::new(2), "Chicago".to_string());

        // Write to buffer and verify
        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();
        let result = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = result.trim().split('\n').collect();

        assert_eq!(lines.len(), 4, "Should have header + 3 data rows");
        assert_eq!(lines[0], "Name,Age,City");
        assert_eq!(lines[1], "Alice,30,NYC");
        assert_eq!(lines[2], "Bob,25,LA");
        assert_eq!(lines[3], "Charlie,35,Chicago");
    }

    #[test]
    fn test_write_lazy_bulk_path_with_scattered_edits() {
        use crate::csv::row_storage::RowStorage;
        use crate::domain::position::{ColIndex, RowIndex};

        // Create a CSV with enough rows to test the bulk write path
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bulk_test.csv");
        let mut csv = String::from("Name,Value\n");
        for i in 1..=20 {
            csv.push_str(&format!("Row{},{}\n", i, i * 10));
        }
        fs::write(&file_path, &csv).unwrap();

        let storage = RowStorage::lazy_from_file(&file_path, None, false).unwrap();
        let mut doc = Document::from_storage(storage, "bulk_test.csv".to_string(), ',');
        assert!(doc.storage.is_lazy());

        // Edit a few scattered rows (simulates %s/x/y/g with sparse matches)
        doc.set_cell(RowIndex::new(3), ColIndex::new(0), "EDITED3".to_string());
        doc.set_cell(RowIndex::new(10), ColIndex::new(0), "EDITED10".to_string());
        doc.set_cell(RowIndex::new(15), ColIndex::new(0), "EDITED15".to_string());

        // Write and verify
        let mut output = Vec::new();
        write_csv_content(&mut output, &doc, ',').unwrap();
        let result = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = result.trim().split('\n').collect();

        assert_eq!(lines.len(), 21, "header + 20 data rows");
        assert_eq!(lines[0], "Name,Value");
        assert_eq!(lines[1], "Row1,10"); // unedited
        assert_eq!(lines[2], "Row2,20"); // unedited
        assert_eq!(lines[3], "EDITED3,30"); // edited
        assert_eq!(lines[4], "Row4,40"); // unedited
        assert_eq!(lines[10], "EDITED10,100"); // edited
        assert_eq!(lines[15], "EDITED15,150"); // edited
        assert_eq!(lines[20], "Row20,200"); // unedited tail
    }

    #[test]
    fn test_write_lazy_bulk_path_roundtrip() {
        use crate::csv::row_storage::RowStorage;
        use crate::domain::position::{ColIndex, RowIndex};

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("roundtrip_bulk.csv");
        fs::write(
            &file_path,
            "A,B,C\nfoo,bar,baz\nhello,world,test\nalpha,beta,gamma\n",
        )
        .unwrap();

        let storage = RowStorage::lazy_from_file(&file_path, None, false).unwrap();
        let mut doc = Document::from_storage(storage, "roundtrip_bulk.csv".to_string(), ',');

        // Edit first and last data rows
        doc.set_cell(RowIndex::new(1), ColIndex::new(1), "CHANGED".to_string());
        doc.set_cell(RowIndex::new(3), ColIndex::new(2), "MODIFIED".to_string());

        // Write atomically and re-read
        let out_path = temp_dir.path().join("output.csv");
        write_csv_atomic(&doc, &out_path, ',').unwrap();

        let content = fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = content.trim().split('\n').collect();
        assert_eq!(lines[0], "A,B,C");
        assert_eq!(lines[1], "foo,CHANGED,baz");
        assert_eq!(lines[2], "hello,world,test");
        assert_eq!(lines[3], "alpha,beta,MODIFIED");
    }
}
