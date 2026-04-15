//! Markdown table export — GitHub Flavored Markdown.

use anyhow::Result;
use std::io::Write;

/// Write data as a GFM (GitHub Flavored Markdown) table.
pub fn write_markdown<W: Write>(
    writer: &mut W,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<()> {
    if headers.is_empty() {
        return Ok(());
    }

    // Header row
    let escaped_headers: Vec<String> = headers.iter().map(|h| escape_md(h)).collect();
    writeln!(writer, "| {} |", escaped_headers.join(" | "))?;

    // Separator row
    let separators: Vec<&str> = headers.iter().map(|_| "---").collect();
    writeln!(writer, "| {} |", separators.join(" | "))?;

    // Data rows
    for row in rows {
        let cells: Vec<String> = (0..headers.len())
            .map(|i| escape_md(row.get(i).map(|s| s.as_str()).unwrap_or("")))
            .collect();
        writeln!(writer, "| {} |", cells.join(" | "))?;
    }

    Ok(())
}

/// Escape a cell value for Markdown: escape pipe characters and newlines.
fn escape_md(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "<br>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_markdown_basic() {
        let headers = vec!["Name".into(), "Age".into()];
        let rows = vec![
            vec!["Alice".into(), "30".into()],
            vec!["Bob".into(), "25".into()],
        ];
        let mut buf = Vec::new();
        write_markdown(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("| Name | Age |"));
        assert!(output.contains("| --- | --- |"));
        assert!(output.contains("| Alice | 30 |"));
        assert!(output.contains("| Bob | 25 |"));
    }

    #[test]
    fn test_write_markdown_escapes_pipes() {
        let headers = vec!["A".into()];
        let rows = vec![vec!["a|b".into()]];
        let mut buf = Vec::new();
        write_markdown(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("a\\|b"));
    }

    #[test]
    fn test_write_markdown_empty_headers() {
        let headers: Vec<String> = vec![];
        let rows: Vec<Vec<String>> = vec![];
        let mut buf = Vec::new();
        write_markdown(&mut buf, &headers, &rows).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_write_markdown_newlines_become_br() {
        let headers = vec!["Note".into()];
        let rows = vec![vec!["line1\nline2".into()]];
        let mut buf = Vec::new();
        write_markdown(&mut buf, &headers, &rows).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("line1<br>line2"));
    }
}
