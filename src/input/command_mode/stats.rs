//! Column statistics commands (:stats, :sum, :avg, :count, :distinct)
//! and shared statistics computation for visual mode and footer row.

use crate::app::{App, VisualMode, VisualSelection};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use crate::ui::utils::excel_letter_to_column;
use anyhow::Result;
use std::collections::HashSet;

/// Aggregated statistics for a set of cell values.
pub(crate) struct SelectionStats {
    /// Total number of cells
    pub count: usize,
    /// Number of cells that parsed as numeric
    pub numeric_count: usize,
    /// Sum of numeric values
    pub sum: f64,
    /// Average of numeric values
    pub avg: f64,
    /// Minimum numeric value
    pub min: f64,
    /// Maximum numeric value
    pub max: f64,
}

/// Compute aggregate statistics for a flat slice of cell string values (single pass).
pub(crate) fn compute_selection_stats(values: &[String]) -> SelectionStats {
    let count = values.len();
    let mut numeric_count = 0usize;
    let mut sum = 0.0f64;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;

    for v in values {
        if let Some(n) = parse_numeric(v) {
            numeric_count += 1;
            sum += n;
            if n < min {
                min = n;
            }
            if n > max {
                max = n;
            }
        }
    }

    let avg = if numeric_count > 0 {
        sum / numeric_count as f64
    } else {
        0.0
    };

    SelectionStats {
        count,
        numeric_count,
        sum,
        avg,
        min,
        max,
    }
}

/// Collect cell values from a visual selection, organized by column.
///
/// Returns a Vec of (col_index, Vec<cell_values>) pairs for each column
/// in the selection. Column mode skips row 0 (header row).
pub(crate) fn collect_visual_selection_values(
    app: &App,
    sel: &VisualSelection,
) -> Vec<(usize, Vec<String>)> {
    let (start_row, end_row, start_col, end_col) = sel.bounds();

    let (r_start, r_end, c_start, c_end) = match sel.mode {
        VisualMode::Block => (
            start_row.get(),
            end_row.get(),
            start_col.get(),
            end_col.get(),
        ),
        VisualMode::Line => (
            start_row.get(),
            end_row.get(),
            0,
            app.document.column_count().saturating_sub(1),
        ),
        VisualMode::Column => (
            1, // skip header row
            app.document.row_count().saturating_sub(1),
            start_col.get(),
            end_col.get(),
        ),
    };

    let mut columns = Vec::new();
    for col_idx in c_start..=c_end {
        let mut col_values = Vec::new();
        for row_idx in r_start..=r_end {
            let cell = app
                .document
                .cell(RowIndex::new(row_idx), ColIndex::new(col_idx));
            col_values.push(cell);
        }
        columns.push((col_idx, col_values));
    }
    columns
}

/// Collect all cell values from a visual selection as a flat list.
pub(crate) fn collect_visual_selection_flat(app: &App, sel: &VisualSelection) -> Vec<String> {
    let columns = collect_visual_selection_values(app, sel);
    columns.into_iter().flat_map(|(_, vals)| vals).collect()
}

/// Compute the total number of cells in a visual selection (O(1) from bounds).
pub(crate) fn visual_selection_cell_count(app: &App, sel: &VisualSelection) -> usize {
    let (start_row, end_row, start_col, end_col) = sel.bounds();

    let (rows, cols) = match sel.mode {
        VisualMode::Block => (
            end_row.get() - start_row.get() + 1,
            end_col.get() - start_col.get() + 1,
        ),
        VisualMode::Line => (
            end_row.get() - start_row.get() + 1,
            app.document.column_count(),
        ),
        VisualMode::Column => (
            app.document.row_count().saturating_sub(1), // skip header
            end_col.get() - start_col.get() + 1,
        ),
    };
    rows * cols
}

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

/// Execute :stats <col> — show all statistics for a column.
/// When called with no argument in visual mode, opens the stats overlay popup.
pub fn execute_stats(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    let Some(arg) = arg else {
        // In visual mode with no arg, open the stats overlay popup
        if matches!(
            app.mode,
            crate::app::Mode::VisualBlock
                | crate::app::Mode::VisualLine
                | crate::app::Mode::VisualColumn
        ) {
            open_stats_overlay(app);
            return Ok(InputResult::Continue);
        }
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

/// Open the statistics overlay popup for the current visual selection.
pub(crate) fn open_stats_overlay(app: &mut App) {
    use crate::ui::view_state::{ColumnStats, StatsOverlayData};

    let sel = match app.visual_selection {
        Some(sel) => sel,
        None => return,
    };

    let columns_data = collect_visual_selection_values(app, &sel);
    let header_row = app.document.storage.header_row();

    let mut column_stats = Vec::new();
    for (col_idx, values) in &columns_data {
        let name = header_row
            .get(*col_idx)
            .cloned()
            .unwrap_or_else(|| format!("Column {}", col_idx + 1));

        let stats = compute_selection_stats(values);
        let non_empty = values.iter().filter(|v| !v.trim().is_empty()).count();
        let distinct: HashSet<&str> = values
            .iter()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .collect();

        let (sum, avg, min, max) = if stats.numeric_count > 0 {
            (
                Some(stats.sum),
                Some(stats.avg),
                Some(stats.min),
                Some(stats.max),
            )
        } else {
            (None, None, None, None)
        };

        column_stats.push(ColumnStats {
            name,
            total_count: stats.count,
            non_empty_count: non_empty,
            numeric_count: stats.numeric_count,
            sum,
            avg,
            min,
            max,
            distinct_count: distinct.len(),
        });
    }

    let (start_row, end_row, start_col, end_col) = sel.bounds();
    let title = format!(
        "Selection Statistics ({}-{}, {}-{})",
        start_row.get() + 1,
        end_row.get() + 1,
        crate::ui::utils::column_to_excel_letter(start_col.get()),
        crate::ui::utils::column_to_excel_letter(end_col.get()),
    );

    if column_stats.is_empty() {
        app.status_message = Some(StatusMessage::from("No columns in selection"));
        return;
    }

    app.view_state.stats_overlay_data = Some(StatsOverlayData {
        title,
        columns: column_stats,
        scroll_offset: 0,
    });
    app.view_state.stats_overlay_visible = true;
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

    #[test]
    fn test_compute_selection_stats_numeric() {
        let values: Vec<String> = vec!["10", "20", "30", "40", "50"]
            .into_iter()
            .map(String::from)
            .collect();
        let stats = compute_selection_stats(&values);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.numeric_count, 5);
        assert_eq!(stats.sum, 150.0);
        assert_eq!(stats.avg, 30.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 50.0);
    }

    #[test]
    fn test_compute_selection_stats_mixed() {
        let values: Vec<String> = vec!["10", "hello", "30", "", "50"]
            .into_iter()
            .map(String::from)
            .collect();
        let stats = compute_selection_stats(&values);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.numeric_count, 3);
        assert_eq!(stats.sum, 90.0);
        assert_eq!(stats.avg, 30.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 50.0);
    }

    #[test]
    fn test_compute_selection_stats_no_numeric() {
        let values: Vec<String> = vec!["hello", "world", ""]
            .into_iter()
            .map(String::from)
            .collect();
        let stats = compute_selection_stats(&values);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.numeric_count, 0);
        assert_eq!(stats.sum, 0.0);
        assert_eq!(stats.avg, 0.0);
    }

    #[test]
    fn test_compute_selection_stats_empty() {
        let values: Vec<String> = Vec::new();
        let stats = compute_selection_stats(&values);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.numeric_count, 0);
    }

    #[test]
    fn test_compute_selection_stats_single_value() {
        let values: Vec<String> = vec!["42.5".to_string()];
        let stats = compute_selection_stats(&values);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.numeric_count, 1);
        assert_eq!(stats.sum, 42.5);
        assert_eq!(stats.avg, 42.5);
        assert_eq!(stats.min, 42.5);
        assert_eq!(stats.max, 42.5);
    }

    #[test]
    fn test_compute_selection_stats_with_commas() {
        let values: Vec<String> = vec!["1,000", "2,500.50"]
            .into_iter()
            .map(String::from)
            .collect();
        let stats = compute_selection_stats(&values);
        assert_eq!(stats.numeric_count, 2);
        assert_eq!(stats.sum, 3500.5);
    }
}
