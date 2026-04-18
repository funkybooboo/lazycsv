//! ODS export via spreadsheet-ods.

use anyhow::{Context, Result};
use spreadsheet_ods::{Sheet, WorkBook};
use std::path::Path;

pub fn write_ods(path: &Path, headers: &[String], rows: &[Vec<String>]) -> Result<()> {
    let mut wb = WorkBook::new_empty();
    let mut sheet = Sheet::new("Sheet1");

    for (col, header) in headers.iter().enumerate() {
        sheet.set_value(0, col as u32, header.as_str());
    }

    for (row_idx, row) in rows.iter().enumerate() {
        let ods_row = (row_idx + 1) as u32;
        for (col_idx, cell) in row.iter().enumerate() {
            let col = col_idx as u32;
            if cell.is_empty() {
                continue;
            }
            if let Some(n) = parse_as_number(cell) {
                sheet.set_value(ods_row, col, n);
            } else if cell.eq_ignore_ascii_case("true") || cell.eq_ignore_ascii_case("false") {
                sheet.set_value(ods_row, col, cell.eq_ignore_ascii_case("true"));
            } else {
                sheet.set_value(ods_row, col, cell.as_str());
            }
        }
    }

    wb.push_sheet(sheet);
    spreadsheet_ods::write_ods(&mut wb, path)
        .context(format!("Failed to write ODS: {}", path.display()))?;

    Ok(())
}

fn parse_as_number(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    let cleaned = trimmed
        .trim_start_matches('$')
        .trim_start_matches('€')
        .trim_start_matches('£')
        .trim_start_matches('¥');
    let cleaned = cleaned.replace(',', "");
    cleaned.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_ods_creates_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("ods");
        let headers = vec!["Name".into(), "Value".into()];
        let rows = vec![
            vec!["Alice".into(), "42".into()],
            vec!["Bob".into(), "hello".into()],
        ];
        write_ods(&path, &headers, &rows).unwrap();
        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_write_ods_roundtrip_via_calamine() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("ods");
        let headers = vec!["A".into(), "B".into()];
        let rows = vec![vec!["1".into(), "hello".into()]];
        write_ods(&path, &headers, &rows).unwrap();

        use calamine::{open_workbook_auto, Data, Reader};
        let mut wb = open_workbook_auto(&path).unwrap();
        let sheets = wb.sheet_names();
        let range = wb.worksheet_range(&sheets[0]).unwrap();
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

        assert_eq!(cells[0], vec!["A", "B"]);
        assert_eq!(cells[1], vec!["1", "hello"]);
        let _ = std::fs::remove_file(&path);
    }
}
