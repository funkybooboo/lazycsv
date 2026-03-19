//! XLSX/XLS file loading via the calamine crate.
//!
//! Converts spreadsheet data into in-memory rows for the existing Document model.
//! Supports multi-sheet workbooks with user-selectable sheets.

use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Data, DataType as CalamineDataType, Reader};
use std::path::Path;

/// Check if a path is a spreadsheet file (.xlsx or .xls).
pub fn is_spreadsheet(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_ascii_lowercase();
            lower == "xlsx" || lower == "xls"
        })
        .unwrap_or(false)
}

/// Get the list of sheet names from a spreadsheet file.
pub fn get_sheet_names(path: &Path) -> Result<Vec<String>> {
    let workbook = open_workbook_auto(path)
        .context(format!("Failed to open spreadsheet: {}", path.display()))?;
    Ok(workbook.sheet_names())
}

/// Load a specific sheet from a spreadsheet file into rows (header + data).
/// Returns (rows, sheet_name).
/// Loaded sheet data with formulas separated from computed values.
pub struct SheetData {
    /// Rows of computed values (header + data). Cells always contain display values.
    pub rows: Vec<Vec<String>>,
    /// Sheet name.
    pub sheet_name: String,
    /// Cell formulas: (row, col) -> formula text (e.g., "=SUM(B2:B5)").
    /// Only non-empty formulas are included. Coordinates are row/col in `rows`.
    pub formulas: Vec<((usize, usize), String)>,
}

pub fn load_sheet(path: &Path, sheet_name: &str) -> Result<(Vec<Vec<String>>, String)> {
    let data = load_sheet_with_formulas(path, sheet_name)?;
    Ok((data.rows, data.sheet_name))
}

pub fn load_sheet_with_formulas(path: &Path, sheet_name: &str) -> Result<SheetData> {
    let mut workbook = open_workbook_auto(path)
        .context(format!("Failed to open spreadsheet: {}", path.display()))?;

    let range = workbook
        .worksheet_range(sheet_name)
        .context(format!("Sheet '{}' not found", sheet_name))?;

    // Load formulas separately — these go into the FormulaStore, not the cell values.
    let formulas_range = workbook.worksheet_formula(sheet_name).ok();

    let data_start = range.start().unwrap_or((0, 0));
    let formula_start = formulas_range
        .as_ref()
        .and_then(|f| f.start())
        .unwrap_or((0, 0));

    let height = range.height();
    let width = range.width();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(height);
    let mut formulas: Vec<((usize, usize), String)> = Vec::new();

    for r in 0..height {
        let mut string_row: Vec<String> = Vec::with_capacity(width);
        for c in 0..width {
            // Always store the computed value as the cell content
            let cell_str = range.get((r, c)).map(cell_to_string).unwrap_or_default();
            string_row.push(cell_str);

            // Collect formula separately for the FormulaStore
            let abs_row = data_start.0 as usize + r;
            let abs_col = data_start.1 as usize + c;
            let formula = formulas_range.as_ref().and_then(|f| {
                let fr = abs_row.checked_sub(formula_start.0 as usize)?;
                let fc = abs_col.checked_sub(formula_start.1 as usize)?;
                f.get((fr, fc)).filter(|s| !s.is_empty())
            });
            if let Some(f) = formula {
                formulas.push(((r, c), format!("={}", f)));
            }
        }
        rows.push(string_row);
    }

    // Ensure we have at least a header row
    if rows.is_empty() {
        rows.push(vec!["(empty)".to_string()]);
    }

    Ok(SheetData {
        rows,
        sheet_name: sheet_name.to_string(),
        formulas,
    })
}

/// Convert a calamine cell to a String.
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            // Format without trailing zeros for clean display
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::DateTime(_) => {
            // Use chrono conversion for proper date/time formatting
            if let Some(dt) = cell.as_datetime() {
                if dt.time() == chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap() {
                    // Date only (no time component)
                    dt.format("%-m/%-d/%Y").to_string()
                } else {
                    // Date and time
                    dt.format("%-m/%-d/%Y %-I:%M %p").to_string()
                }
            } else {
                // Fallback: raw value
                format!("{}", cell)
            }
        }
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_spreadsheet() {
        assert!(is_spreadsheet(Path::new("file.xlsx")));
        assert!(is_spreadsheet(Path::new("file.XLSX")));
        assert!(is_spreadsheet(Path::new("file.xls")));
        assert!(is_spreadsheet(Path::new("file.XLS")));
        assert!(!is_spreadsheet(Path::new("file.csv")));
        assert!(!is_spreadsheet(Path::new("file.txt")));
        assert!(!is_spreadsheet(Path::new("file")));
    }
}
