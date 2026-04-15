//! XLSX export via rust_xlsxwriter.

use anyhow::{Context, Result};
use std::path::Path;

/// Write data as an XLSX spreadsheet.
/// Headers are bold in row 0. Numeric cells stored as numbers.
pub fn write_xlsx(path: &Path, headers: &[String], rows: &[Vec<String>]) -> Result<()> {
    use rust_xlsxwriter::{Format, Workbook};

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Sheet1")?;

    let bold = Format::new().set_bold();

    // Write headers
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, header, &bold)?;
    }

    // Write data rows
    for (row_idx, row) in rows.iter().enumerate() {
        let xlsx_row = (row_idx + 1) as u32; // +1 for header
        for (col_idx, cell) in row.iter().enumerate() {
            let col = col_idx as u16;
            if cell.is_empty() {
                // Leave empty cells blank
                continue;
            }
            // Try to write as number for proper Excel handling
            if let Ok(n) = cell.parse::<f64>() {
                worksheet.write_number(xlsx_row, col, n)?;
            } else if cell.eq_ignore_ascii_case("true") || cell.eq_ignore_ascii_case("false") {
                worksheet.write_boolean(xlsx_row, col, cell.eq_ignore_ascii_case("true"))?;
            } else {
                worksheet.write_string(xlsx_row, col, cell)?;
            }
        }
    }

    workbook
        .save(path)
        .context(format!("Failed to write XLSX: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_xlsx_creates_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("xlsx");
        let headers = vec!["Name".into(), "Value".into()];
        let rows = vec![
            vec!["Alice".into(), "42".into()],
            vec!["Bob".into(), "hello".into()],
        ];
        write_xlsx(&path, &headers, &rows).unwrap();
        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_write_xlsx_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("xlsx");
        let headers = vec!["A".into(), "B".into()];
        let rows = vec![vec!["1".into(), "hello".into()]];
        write_xlsx(&path, &headers, &rows).unwrap();

        // Read back with calamine
        use calamine::{open_workbook_auto, Data, Reader};
        let mut wb = open_workbook_auto(&path).unwrap();
        let range = wb.worksheet_range("Sheet1").unwrap();
        let cells: Vec<Vec<String>> = range
            .rows()
            .map(|row| {
                row.iter()
                    .map(|c| match c {
                        Data::String(s) => s.clone(),
                        Data::Float(f) => {
                            if f.fract() == 0.0 {
                                format!("{}", *f as i64)
                            } else {
                                format!("{}", f)
                            }
                        }
                        Data::Int(i) => format!("{}", i),
                        _ => String::new(),
                    })
                    .collect()
            })
            .collect();

        assert_eq!(cells[0], vec!["A", "B"]); // headers
        assert_eq!(cells[1], vec!["1", "hello"]); // data
        let _ = std::fs::remove_file(&path);
    }
}
