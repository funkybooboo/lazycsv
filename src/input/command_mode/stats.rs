//! Column statistics commands (:stats, :sum, :avg, :count, :distinct)

use crate::app::App;
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use crate::ui::utils::excel_letter_to_column;
use anyhow::Result;
use std::collections::HashSet;

/// Resolve a column specifier to a 0-based column index.
/// Accepts: 1-based number, header name (case-insensitive), or Excel letter (A, B, AA).
fn resolve_column(app: &App, spec: &str) -> std::result::Result<usize, String> {
    // Try as 1-based number
    if let Ok(num) = spec.parse::<usize>() {
        if num == 0 || num > app.document.column_count() {
            return Err(format!(
                "Column {} out of range (1-{})",
                num,
                app.document.column_count()
            ));
        }
        return Ok(num - 1);
    }

    // Try header name (case-insensitive)
    let header_row = app.document.storage.header_row();
    if let Some(idx) = header_row
        .iter()
        .position(|name| name.eq_ignore_ascii_case(spec))
    {
        return Ok(idx);
    }

    // Try Excel-style column letter
    if spec.chars().all(|c| c.is_ascii_alphabetic()) {
        match excel_letter_to_column(spec) {
            Ok(idx) if idx < app.document.column_count() => return Ok(idx),
            _ => {}
        }
    }

    Err(format!("Column \"{}\" not found", spec))
}

/// Get the header name for display purposes.
fn column_display_name(app: &App, col_idx: usize) -> String {
    let header_row = app.document.storage.header_row();
    header_row
        .get(col_idx)
        .cloned()
        .unwrap_or_else(|| format!("Column {}", col_idx + 1))
}

/// Iterate over data cell values for a column.
fn data_values(app: &App, col_idx: usize) -> Vec<String> {
    let row_count = app.document.row_count();
    // Collect all rows - no special treatment for row 0
    (0..row_count)
        .map(|r| app.document.storage.get_cell(r, col_idx))
        .collect()
}

/// Parse a cell value as f64, returning None for empty/non-numeric values.
pub(crate) fn parse_numeric(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Strip commas for locale-formatted numbers (e.g., "1,234.56")
    let cleaned = trimmed.replace(',', "");
    cleaned.parse::<f64>().ok()
}

/// Format a float nicely (no trailing zeros for integers).
pub(crate) fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        // Up to 4 decimal places, trim trailing zeros
        let s = format!("{:.4}", n);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

/// Execute :sum <col>
pub fn execute_sum(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    let Some(arg) = arg else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :sum <column> (e.g., :sum A, :sum Price, :sum 1)",
        ));
        return Ok(InputResult::Continue);
    };

    let col_idx = match resolve_column(app, arg.trim()) {
        Ok(idx) => idx,
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
            return Ok(InputResult::Continue);
        }
    };

    let values = data_values(app, col_idx);
    let nums: Vec<f64> = values.iter().filter_map(|v| parse_numeric(v)).collect();

    if nums.is_empty() {
        app.status_message = Some(StatusMessage::from(format!(
            "{}: no numeric values",
            column_display_name(app, col_idx)
        )));
    } else {
        let sum: f64 = nums.iter().sum();
        app.status_message = Some(StatusMessage::from(format!(
            "{}: sum = {}",
            column_display_name(app, col_idx),
            format_number(sum)
        )));
    }
    Ok(InputResult::Continue)
}

/// Execute :avg <col>
pub fn execute_avg(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    let Some(arg) = arg else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :avg <column> (e.g., :avg A, :avg Price, :avg 1)",
        ));
        return Ok(InputResult::Continue);
    };

    let col_idx = match resolve_column(app, arg.trim()) {
        Ok(idx) => idx,
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
            return Ok(InputResult::Continue);
        }
    };

    let values = data_values(app, col_idx);
    let nums: Vec<f64> = values.iter().filter_map(|v| parse_numeric(v)).collect();

    if nums.is_empty() {
        app.status_message = Some(StatusMessage::from(format!(
            "{}: no numeric values",
            column_display_name(app, col_idx)
        )));
    } else {
        let sum: f64 = nums.iter().sum();
        let avg = sum / nums.len() as f64;
        app.status_message = Some(StatusMessage::from(format!(
            "{}: avg = {} ({} values)",
            column_display_name(app, col_idx),
            format_number(avg),
            nums.len()
        )));
    }
    Ok(InputResult::Continue)
}

/// Execute :count <col>
pub fn execute_count(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    let Some(arg) = arg else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :count <column> (e.g., :count A, :count Name, :count 1)",
        ));
        return Ok(InputResult::Continue);
    };

    let col_idx = match resolve_column(app, arg.trim()) {
        Ok(idx) => idx,
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
            return Ok(InputResult::Continue);
        }
    };

    let values = data_values(app, col_idx);
    let non_empty = values.iter().filter(|v| !v.trim().is_empty()).count();
    let total = values.len();

    app.status_message = Some(StatusMessage::from(format!(
        "{}: {} non-empty / {} total",
        column_display_name(app, col_idx),
        non_empty,
        total
    )));
    Ok(InputResult::Continue)
}

/// Execute :distinct <col>
pub fn execute_distinct(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    let Some(arg) = arg else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :distinct <column> (e.g., :distinct A, :distinct Status, :distinct 1)",
        ));
        return Ok(InputResult::Continue);
    };

    let col_idx = match resolve_column(app, arg.trim()) {
        Ok(idx) => idx,
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
            return Ok(InputResult::Continue);
        }
    };

    let values = data_values(app, col_idx);
    let distinct: HashSet<&str> = values.iter().map(|v| v.trim()).collect();
    // Don't count empty string as a distinct value
    let distinct_count = distinct.iter().filter(|v| !v.is_empty()).count();

    app.status_message = Some(StatusMessage::from(format!(
        "{}: {} distinct values",
        column_display_name(app, col_idx),
        distinct_count
    )));
    Ok(InputResult::Continue)
}

/// Execute :stats <col> — show all statistics for a column
pub fn execute_stats(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    let Some(arg) = arg else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :stats <column> (e.g., :stats A, :stats Price, :stats 1)",
        ));
        return Ok(InputResult::Continue);
    };

    let col_idx = match resolve_column(app, arg.trim()) {
        Ok(idx) => idx,
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
            return Ok(InputResult::Continue);
        }
    };

    let name = column_display_name(app, col_idx);
    let values = data_values(app, col_idx);
    let total = values.len();
    let non_empty = values.iter().filter(|v| !v.trim().is_empty()).count();
    let distinct: HashSet<&str> = values
        .iter()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .collect();
    let nums: Vec<f64> = values.iter().filter_map(|v| parse_numeric(v)).collect();

    let msg = if nums.is_empty() {
        format!(
            "{}: count={}/{} distinct={}",
            name,
            non_empty,
            total,
            distinct.len()
        )
    } else {
        let sum: f64 = nums.iter().sum();
        let avg = sum / nums.len() as f64;
        let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        format!(
            "{}: sum={} avg={} min={} max={} count={}/{} distinct={}",
            name,
            format_number(sum),
            format_number(avg),
            format_number(min),
            format_number(max),
            non_empty,
            total,
            distinct.len()
        )
    };

    app.status_message = Some(StatusMessage::from(msg));
    Ok(InputResult::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_integer() {
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(-100.0), "-100");
        assert_eq!(format_number(0.0), "0");
    }

    #[test]
    fn test_format_number_decimal() {
        assert_eq!(format_number(3.15), "3.15");
        assert_eq!(format_number(1.5), "1.5");
        assert_eq!(format_number(0.1234), "0.1234");
    }

    #[test]
    fn test_parse_numeric_valid() {
        assert_eq!(parse_numeric("42"), Some(42.0));
        assert_eq!(parse_numeric("3.15"), Some(3.15));
        assert_eq!(parse_numeric("-10"), Some(-10.0));
        assert_eq!(parse_numeric("  42  "), Some(42.0));
        assert_eq!(parse_numeric("1,234.56"), Some(1234.56));
    }

    #[test]
    fn test_parse_numeric_invalid() {
        assert_eq!(parse_numeric(""), None);
        assert_eq!(parse_numeric("  "), None);
        assert_eq!(parse_numeric("hello"), None);
        assert_eq!(parse_numeric("N/A"), None);
    }
}
