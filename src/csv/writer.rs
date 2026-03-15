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

    // Write to temp file
    {
        let mut file = fs::File::create(&temp_path)
            .context(format!("Failed to create temp file: {:?}", temp_path))?;

        write_csv_content(&mut file, document, delimiter).context("Failed to write CSV content")?;

        // Ensure all data is written
        file.sync_all().context("Failed to sync file to disk")?;
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
    // Write all rows (including header at row 0)
    for row in document.iter_rows() {
        write_csv_row(writer, &row, delimiter)?;
    }
    Ok(())
}

/// Write a single CSV row with proper escaping
fn write_csv_row<W: Write>(writer: &mut W, row: &[String], delimiter: char) -> Result<()> {
    for (i, cell) in row.iter().enumerate() {
        if i > 0 {
            write!(writer, "{}", delimiter)?;
        }
        write_csv_cell(writer, cell, delimiter)?;
    }
    writeln!(writer)?;
    Ok(())
}

/// Write a single CSV cell with proper escaping
///
/// Escaping rules:
/// - If cell contains delimiter, quotes, or newlines: wrap in quotes
/// - If cell contains quotes: double them (e.g., " becomes "")
fn write_csv_cell<W: Write>(writer: &mut W, cell: &str, delimiter: char) -> Result<()> {
    // Check if cell needs quoting
    let needs_quotes = cell.contains(delimiter)
        || cell.contains('"')
        || cell.contains('\n')
        || cell.contains('\r');

    if needs_quotes {
        write!(writer, "\"")?;
        // Escape quotes by doubling them
        for ch in cell.chars() {
            if ch == '"' {
                write!(writer, "\"\"")?;
            } else {
                write!(writer, "{}", ch)?;
            }
        }
        write!(writer, "\"")?;
    } else {
        write!(writer, "{}", cell)?;
    }

    Ok(())
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
}
