//! TSV export — tab-separated values.

use anyhow::Result;
use std::io::Write;

/// Write data as TSV (tab-separated values).
/// Tabs and newlines within cells are escaped.
pub fn write_tsv<W: Write>(writer: &mut W, headers: &[String], rows: &[Vec<String>]) -> Result<()> {
    // Header row
    let escaped_headers: Vec<String> = headers.iter().map(|h| escape_tsv(h)).collect();
    writeln!(writer, "{}", escaped_headers.join("\t"))?;

    // Data rows
    for row in rows {
        let escaped: Vec<String> = row.iter().map(|c| escape_tsv(c)).collect();
        writeln!(writer, "{}", escaped.join("\t"))?;
    }

    Ok(())
}

/// Escape a cell value for TSV: replace tabs and newlines with escape sequences.
fn escape_tsv(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_tsv_basic() {
        let headers = vec!["Name".into(), "City".into()];
        let rows = vec![
            vec!["Alice".into(), "New York".into()],
            vec!["Bob".into(), "London".into()],
        ];
        let mut buf = Vec::new();
        write_tsv(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "Name\tCity\nAlice\tNew York\nBob\tLondon\n");
    }

    #[test]
    fn test_write_tsv_escapes_tabs() {
        let headers = vec!["A".into()];
        let rows = vec![vec!["has\ttab".into()]];
        let mut buf = Vec::new();
        write_tsv(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("has\\ttab"));
    }

    #[test]
    fn test_write_tsv_escapes_newlines() {
        let headers = vec!["A".into()];
        let rows = vec![vec!["line1\nline2".into()]];
        let mut buf = Vec::new();
        write_tsv(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("line1\\nline2"));
    }
}
