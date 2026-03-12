//! Range command parser for vim-style range operations
//!
//! Handles commands like:
//! - `%d`, `%y` - All data rows
//! - `.d`, `.y` - Current row
//! - `$d`, `$y` - Last row
//! - `:5,10d`, `:5,10y` - Numeric row ranges
//! - `:B,Dd`, `:A,Ey` - Column ranges
//! - `:D m A` - Move columns

use crate::app::App;
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use crate::ui::utils::{column_to_excel_letter, excel_letter_to_column};
use crate::{ColIndex, RowIndex};
use anyhow::Result;

/// Parse and execute range commands (e.g., %d, 5,10d, B,Dd)
///
/// Returns Some(Result) if command was a range command, None if not
pub fn parse_and_execute(app: &mut App, cmd: &str) -> Option<Result<InputResult>> {
    // Check if command contains a range pattern or special markers
    // Patterns: 5,10d, %d, .d, $d, 5,10y, %y, etc.

    // Match special range markers
    if let Some(operation) = cmd.strip_prefix('%') {
        return Some(execute_percent_range(app, operation));
    }

    // Match .d or .y (current row)
    if let Some(operation) = cmd.strip_prefix('.') {
        return Some(execute_current_row(app, operation));
    }

    // Match $d or $y (last row)
    if let Some(operation) = cmd.strip_prefix('$') {
        return Some(execute_last_row(app, operation));
    }

    // Match numeric or column ranges: 5,10d or B,Dd
    // BUT: Don't match if command starts with a known command word (e.g., "new Name,Age")
    if cmd.contains(',') {
        // Check if it starts with a number (numeric range like 5,10d)
        if cmd.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Some(execute_comma_range(app, cmd));
        }

        // Check if it's a column range pattern (single letters before comma, like B,Dd or AA,BBd)
        if let Some(comma_pos) = cmd.find(',') {
            let start_str = &cmd[0..comma_pos];
            // Column range: starts with letters only (no spaces, no other chars)
            if !start_str.is_empty() && start_str.chars().all(|c| c.is_ascii_alphabetic()) {
                return Some(execute_comma_range(app, cmd));
            }
        }
    }

    // Not a range command
    None
}

/// Execute % range commands (all data rows)
fn execute_percent_range(app: &mut App, operation: &str) -> Result<InputResult> {
    match operation {
        "d" => {
            // Delete all data rows (excluding header)
            let row_count = app.document.data_row_count();
            if row_count == 0 {
                app.status_message = Some(StatusMessage::from("No data rows to delete"));
                return Ok(InputResult::Continue);
            }

            // Delete rows 1 to row_count (all data rows, preserving header at row 0)
            let deleted = app
                .document
                .delete_rows(RowIndex::new(1), RowIndex::new(row_count));
            app.status_message = Some(StatusMessage::from(format!(
                "Deleted {} row(s)",
                deleted.len()
            )));

            // Move cursor to row 1 (or row 0 if no data rows left)
            if app.document.data_row_count() > 0 {
                app.view_state.table_state.select(Some(1));
            } else {
                app.view_state.table_state.select(Some(0));
            }

            Ok(InputResult::Continue)
        }
        "y" => {
            // Yank all data rows
            let row_count = app.document.data_row_count();
            if row_count == 0 {
                app.status_message = Some(StatusMessage::from("No data rows to yank"));
                return Ok(InputResult::Continue);
            }

            let yanked = app
                .document
                .rows_range(RowIndex::new(1), RowIndex::new(row_count));
            let count = yanked.len();
            app.clipboard.yank_rows(yanked);
            app.status_message = Some(StatusMessage::from(format!(
                "Yanked {} row(s) to clipboard",
                count
            )));

            Ok(InputResult::Continue)
        }
        _ => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown range operation: :%{}",
                operation
            )));
            Ok(InputResult::Continue)
        }
    }
}

/// Execute . range commands (current row)
fn execute_current_row(app: &mut App, operation: &str) -> Result<InputResult> {
    match operation {
        "d" => {
            // Delete current row
            if let Some(row_idx) = app.selected_row() {
                if let Some(_deleted) = app.document.delete_row(row_idx) {
                    app.status_message = Some(StatusMessage::from("Deleted 1 row"));

                    // Adjust cursor position
                    let new_row_count = app.document.data_row_count();
                    let current_pos = row_idx.get();

                    if new_row_count == 0 {
                        // No data rows left, move to header
                        app.view_state.table_state.select(Some(0));
                    } else if current_pos > new_row_count {
                        // Cursor past end, move to last row
                        app.view_state.table_state.select(Some(new_row_count));
                    }
                    // Otherwise keep cursor at same position
                } else {
                    app.status_message = Some(StatusMessage::from("Failed to delete row"));
                }
            } else {
                app.status_message = Some(StatusMessage::from("No row selected"));
            }
            Ok(InputResult::Continue)
        }
        "y" => {
            // Yank current row
            if let Some(row_idx) = app.selected_row() {
                let yanked = app.document.rows_range(row_idx, row_idx);
                if !yanked.is_empty() {
                    app.clipboard.yank_rows(yanked);
                    app.status_message = Some(StatusMessage::from("Yanked 1 row to clipboard"));
                } else {
                    app.status_message = Some(StatusMessage::from("Failed to yank row"));
                }
            } else {
                app.status_message = Some(StatusMessage::from("No row selected"));
            }
            Ok(InputResult::Continue)
        }
        _ => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown range operation: :.{}",
                operation
            )));
            Ok(InputResult::Continue)
        }
    }
}

/// Execute $ range commands (last row)
fn execute_last_row(app: &mut App, operation: &str) -> Result<InputResult> {
    match operation {
        "d" => {
            // Delete last row
            let row_count = app.document.data_row_count();
            if row_count == 0 {
                app.status_message = Some(StatusMessage::from("No data rows to delete"));
                return Ok(InputResult::Continue);
            }

            if let Some(_deleted) = app.document.delete_row(RowIndex::new(row_count)) {
                app.status_message = Some(StatusMessage::from("Deleted 1 row"));

                // Move cursor to new last row if cursor was on deleted row
                if let Some(current_row) = app.selected_row() {
                    if current_row.get() > app.document.data_row_count() {
                        app.view_state
                            .table_state
                            .select(Some(app.document.data_row_count()));
                    }
                }
            } else {
                app.status_message = Some(StatusMessage::from("Failed to delete row"));
            }
            Ok(InputResult::Continue)
        }
        "y" => {
            // Yank last row
            let row_count = app.document.data_row_count();
            if row_count == 0 {
                app.status_message = Some(StatusMessage::from("No data rows to yank"));
                return Ok(InputResult::Continue);
            }

            let yanked = app
                .document
                .rows_range(RowIndex::new(row_count), RowIndex::new(row_count));
            if !yanked.is_empty() {
                app.clipboard.yank_rows(yanked);
                app.status_message = Some(StatusMessage::from("Yanked 1 row to clipboard"));
            } else {
                app.status_message = Some(StatusMessage::from("Failed to yank row"));
            }
            Ok(InputResult::Continue)
        }
        _ => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown range operation: :${}",
                operation
            )));
            Ok(InputResult::Continue)
        }
    }
}

/// Execute comma-separated range commands (5,10d or B,Dd or D m A)
fn execute_comma_range(app: &mut App, cmd: &str) -> Result<InputResult> {
    let Some(comma_pos) = cmd.find(',') else {
        return Ok(InputResult::Continue);
    };

    let start_str = &cmd[0..comma_pos];
    let rest = &cmd[comma_pos + 1..];

    // Try numeric range first
    if let Ok(start_num) = start_str.parse::<usize>() {
        return execute_numeric_range(app, start_num, rest);
    }

    // Try column range (B,Dd or D m A)
    if start_str.chars().all(|c| c.is_ascii_alphabetic()) {
        return execute_column_range(app, start_str, rest);
    }

    Ok(InputResult::Continue)
}

/// Execute numeric row ranges (5,10d or 5,10y)
fn execute_numeric_range(app: &mut App, start_num: usize, rest: &str) -> Result<InputResult> {
    // Find where the operation starts (last letter)
    let Some(last_char) = rest.chars().last() else {
        return Ok(InputResult::Continue);
    };

    let operation = last_char;
    let end_str = &rest[0..rest.len() - 1];

    // Parse end number
    let Ok(end_num) = end_str.parse::<usize>() else {
        return Ok(InputResult::Continue);
    };

    match operation {
        'd' => delete_row_range(app, start_num, end_num),
        'y' => yank_row_range(app, start_num, end_num),
        _ => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown range operation: {}",
                operation
            )));
            Ok(InputResult::Continue)
        }
    }
}

/// Delete a range of rows
fn delete_row_range(app: &mut App, start_num: usize, end_num: usize) -> Result<InputResult> {
    if start_num == 0 || end_num == 0 {
        app.status_message = Some(StatusMessage::from(
            "Row numbers must be >= 1 (row 0 is header)",
        ));
        return Ok(InputResult::Continue);
    }

    if start_num > end_num {
        app.status_message = Some(StatusMessage::from("Invalid range: start must be <= end"));
        return Ok(InputResult::Continue);
    }

    let deleted = app
        .document
        .delete_rows(RowIndex::new(start_num), RowIndex::new(end_num));

    if deleted.is_empty() {
        app.status_message = Some(StatusMessage::from("No rows deleted (range out of bounds)"));
    } else {
        app.status_message = Some(StatusMessage::from(format!(
            "Deleted {} row(s)",
            deleted.len()
        )));

        // Adjust cursor position
        if let Some(current_row) = app.selected_row() {
            let new_row_count = app.document.data_row_count();
            if new_row_count == 0 {
                app.view_state.table_state.select(Some(0));
            } else if current_row.get() > new_row_count {
                app.view_state.table_state.select(Some(new_row_count));
            }
        }
    }

    Ok(InputResult::Continue)
}

/// Yank a range of rows
fn yank_row_range(app: &mut App, start_num: usize, end_num: usize) -> Result<InputResult> {
    if start_num == 0 || end_num == 0 {
        app.status_message = Some(StatusMessage::from(
            "Row numbers must be >= 1 (row 0 is header)",
        ));
        return Ok(InputResult::Continue);
    }

    if start_num > end_num {
        app.status_message = Some(StatusMessage::from("Invalid range: start must be <= end"));
        return Ok(InputResult::Continue);
    }

    let yanked = app
        .document
        .rows_range(RowIndex::new(start_num), RowIndex::new(end_num));

    if yanked.is_empty() {
        app.status_message = Some(StatusMessage::from("No rows yanked (range out of bounds)"));
    } else {
        let count = yanked.len();
        app.clipboard.yank_rows(yanked);
        app.status_message = Some(StatusMessage::from(format!(
            "Yanked {} row(s) to clipboard",
            count
        )));
    }

    Ok(InputResult::Continue)
}

/// Execute column ranges (B,Dd or A,Ey or D m A)
fn execute_column_range(app: &mut App, start_str: &str, rest: &str) -> Result<InputResult> {
    // Check for move command: "D m A" or "D m 0"
    let words: Vec<&str> = rest.split_whitespace().collect();
    if words.len() == 3 && words[1] == "m" {
        return execute_column_move(app, start_str, words[0], words[2]);
    }

    // Must be delete or yank operation
    let Some(last_char) = rest.chars().last() else {
        app.status_message = Some(StatusMessage::from(
            "Incomplete column range command (expected format: :A,Bd or :A,By)",
        ));
        return Ok(InputResult::Continue);
    };

    let operation = last_char;
    let end_str = &rest[0..rest.len() - 1];

    // Check if both start and end are letters (column names) and end_str is not empty
    if end_str.is_empty() || !end_str.chars().all(|c| c.is_ascii_alphabetic()) {
        app.status_message = Some(StatusMessage::from(
            "Incomplete column range command (expected format: :A,Bd or :A,By)",
        ));
        return Ok(InputResult::Continue);
    }

    // Convert column letters to indices
    let start_col = match excel_letter_to_column(&start_str.to_uppercase()) {
        Ok(c) => c,
        Err(e) => {
            app.status_message = Some(StatusMessage::from(e));
            return Ok(InputResult::Continue);
        }
    };
    let end_col = match excel_letter_to_column(&end_str.to_uppercase()) {
        Ok(c) => c,
        Err(e) => {
            app.status_message = Some(StatusMessage::from(e));
            return Ok(InputResult::Continue);
        }
    };

    match operation {
        'd' => delete_column_range(app, start_col, end_col, start_str),
        'y' => yank_column_range(app, start_col, end_col),
        _ => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown range operation: {}",
                operation
            )));
            Ok(InputResult::Continue)
        }
    }
}

/// Move columns (D m A)
fn execute_column_move(
    app: &mut App,
    start_str: &str,
    end_str: &str,
    dest_str: &str,
) -> Result<InputResult> {
    if !end_str.chars().all(|c| c.is_ascii_alphabetic()) {
        app.status_message = Some(StatusMessage::from("Invalid end column in move command"));
        return Ok(InputResult::Continue);
    }

    let start_col = match excel_letter_to_column(&start_str.to_uppercase()) {
        Ok(c) => c,
        Err(e) => {
            app.status_message = Some(StatusMessage::from(e));
            return Ok(InputResult::Continue);
        }
    };
    let end_col = match excel_letter_to_column(&end_str.to_uppercase()) {
        Ok(c) => c,
        Err(e) => {
            app.status_message = Some(StatusMessage::from(e));
            return Ok(InputResult::Continue);
        }
    };

    if start_col > end_col {
        app.status_message = Some(StatusMessage::from(
            "Invalid range: start column must be <= end column",
        ));
        return Ok(InputResult::Continue);
    }

    let max_col = app.document.column_count();
    if start_col >= max_col {
        app.status_message = Some(StatusMessage::from(format!(
            "Column {} does not exist (max: {})",
            start_str.to_uppercase(),
            column_to_excel_letter(max_col.saturating_sub(1))
        )));
        return Ok(InputResult::Continue);
    }

    // Parse destination
    let to_before = if dest_str == "0" {
        0usize
    } else if dest_str.chars().all(|c| c.is_ascii_alphabetic()) {
        match excel_letter_to_column(&dest_str.to_uppercase()) {
            Ok(dest_col) => dest_col + 1, // "after" that column
            Err(e) => {
                app.status_message = Some(StatusMessage::from(e));
                return Ok(InputResult::Continue);
            }
        }
    } else {
        app.status_message = Some(StatusMessage::from(
            "Invalid destination: use a column letter or 0",
        ));
        return Ok(InputResult::Continue);
    };

    // Check if destination is inside source range (no-op)
    if to_before >= start_col && to_before <= end_col + 1 {
        app.status_message = Some(StatusMessage::from(
            "Columns already in position (destination inside source range)",
        ));
        return Ok(InputResult::Continue);
    }

    let count = end_col - start_col + 1;
    let result =
        app.document
            .move_columns(ColIndex::new(start_col), ColIndex::new(end_col), to_before);

    app.view_state.selected_column = ColIndex::new(result);
    app.status_message = Some(StatusMessage::from(format!("Moved {} column(s)", count)));

    Ok(InputResult::Continue)
}

/// Delete a range of columns
fn delete_column_range(
    app: &mut App,
    start_col: usize,
    end_col: usize,
    start_str: &str,
) -> Result<InputResult> {
    if start_col > end_col {
        app.status_message = Some(StatusMessage::from(
            "Invalid range: start column must be <= end column",
        ));
        return Ok(InputResult::Continue);
    }

    let max_col = app.document.column_count();
    if start_col >= max_col {
        app.status_message = Some(StatusMessage::from(format!(
            "Column {} does not exist (max: {})",
            start_str.to_uppercase(),
            column_to_excel_letter(max_col.saturating_sub(1))
        )));
        return Ok(InputResult::Continue);
    }

    let deleted = app
        .document
        .delete_columns(ColIndex::new(start_col), ColIndex::new(end_col));

    if deleted.is_empty() {
        app.status_message = Some(StatusMessage::from(
            "No columns deleted (range out of bounds)",
        ));
    } else {
        app.status_message = Some(StatusMessage::from(format!(
            "Deleted {} column(s)",
            deleted.len()
        )));

        // Adjust cursor position
        let current_col = app.view_state.selected_column.get();
        let new_col_count = app.document.column_count();

        if new_col_count == 0 {
            // No columns left, shouldn't happen but handle it
            app.view_state.selected_column = ColIndex::new(0);
        } else if current_col >= end_col {
            // Cursor at or after deleted range
            let new_pos = current_col.saturating_sub(deleted.len());
            app.view_state.selected_column = ColIndex::new(new_pos.min(new_col_count - 1));
        } else if current_col >= start_col {
            // Cursor in deleted range, move to start
            app.view_state.selected_column = ColIndex::new(start_col.min(new_col_count - 1));
        }
    }

    Ok(InputResult::Continue)
}

/// Yank a range of columns
fn yank_column_range(app: &mut App, start_col: usize, end_col: usize) -> Result<InputResult> {
    if start_col > end_col {
        app.status_message = Some(StatusMessage::from(
            "Invalid range: start column must be <= end column",
        ));
        return Ok(InputResult::Continue);
    }

    let yanked = app
        .document
        .columns_range(ColIndex::new(start_col), ColIndex::new(end_col));

    if yanked.is_empty() {
        app.status_message = Some(StatusMessage::from(
            "No columns yanked (range out of bounds)",
        ));
    } else {
        let count = yanked.len();
        app.clipboard.yank_columns(yanked);
        app.status_message = Some(StatusMessage::from(format!(
            "Yanked {} column(s) to clipboard",
            count
        )));
    }

    Ok(InputResult::Continue)
}
