//! Navigation command implementations for vim-style movement.
//!
//! This module provides functions for navigating the CSV table including
//! cursor movement, page scrolling, and jump commands with count prefixes.

use crate::app::{messages, App};
use crate::domain::position::ColIndex;
use crate::ui::{ViewportMode, MAX_VISIBLE_COLS};
use anyhow::Result;
use crossterm::event::KeyCode;

/// Rows per page for PageUp/PageDown navigation
pub const PAGE_SIZE: usize = 20;

/// Handle navigation keys with optional count prefix
pub fn handle_navigation(app: &mut App, code: KeyCode) -> Result<()> {
    // Consume count prefix (e.g., 5 from command_count for 5j)
    let count = app
        .input_state
        .command_count
        .take()
        .map(|n| n.get())
        .unwrap_or(1);
    match code {
        // Move up (with count: 5k moves up 5 rows)
        KeyCode::Up | KeyCode::Char('k') => {
            move_up_by(app, count);
        }

        // Move down (with count: 5j moves down 5 rows)
        KeyCode::Down | KeyCode::Char('j') => {
            move_down_by(app, count);
        }

        // Move left (with count: 3h moves left 3 columns)
        KeyCode::Left | KeyCode::Char('h') => {
            move_left_by(app, count);
        }

        // Move right (with count: 3l moves right 3 columns)
        KeyCode::Right | KeyCode::Char('l') => {
            move_right_by(app, count);
        }

        // First column
        KeyCode::Char('0') => {
            app.view_state.selected_column = ColIndex::new(0);
            app.view_state.column_scroll_offset = 0;
            app.view_state.viewport_mode = ViewportMode::Auto;
        }

        // Last column
        KeyCode::Char('$') => {
            app.view_state.selected_column =
                ColIndex::new(app.document.column_count().saturating_sub(1));
            // Adjust horizontal offset to show last column
            if app.document.column_count() > MAX_VISIBLE_COLS {
                app.view_state.column_scroll_offset =
                    app.document.column_count() - MAX_VISIBLE_COLS;
            }
            app.view_state.viewport_mode = ViewportMode::Auto;
        }

        // Page down
        KeyCode::PageDown | KeyCode::Char('d') if code == KeyCode::Char('d') => {
            select_next_page(app);
        }

        // Page up
        KeyCode::PageUp | KeyCode::Char('u') if code == KeyCode::Char('u') => {
            select_previous_page(app);
        }

        // Home (first row) - will be handled by gg multi-key
        KeyCode::Home => {
            goto_first_row(app);
        }

        // End/G - Go to last row, or specific line with count (5G goes to line 5)
        KeyCode::End | KeyCode::Char('G') => {
            if count > 1 {
                goto_line(app, count);
                app.status_message = Some(messages::jumped_to_line(count).into());
            } else {
                goto_last_row(app);
            }
        }

        // Word motion: next non-empty cell
        KeyCode::Char('w') => {
            next_word(app);
        }

        // Word motion: previous non-empty cell
        KeyCode::Char('b') => {
            prev_word(app);
        }

        // Word motion: last non-empty cell
        KeyCode::Char('e') => {
            end_word(app);
        }

        _ => {}
    }

    Ok(())
}

fn select_next_page(app: &mut App) {
    let i = match app.view_state.table_state.selected() {
        Some(i) => (i + PAGE_SIZE).min(app.document.row_count().saturating_sub(1)),
        None => 0,
    };
    app.view_state.table_state.select(Some(i));
}

fn select_previous_page(app: &mut App) {
    let i = match app.view_state.table_state.selected() {
        Some(i) => i.saturating_sub(PAGE_SIZE),
        None => 0,
    };
    app.view_state.table_state.select(Some(i));
}

/// Go to first row (gg command)
pub fn goto_first_row(app: &mut App) {
    app.view_state.table_state.select(Some(0));
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Go to last row (G command)
pub fn goto_last_row(app: &mut App) {
    let last = app.document.row_count().saturating_sub(1);
    app.view_state.table_state.select(Some(last));
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Go to specific line number (5G or :5 command)
pub fn goto_line(app: &mut App, line_number: usize) {
    let row_count = app.document.row_count();

    // Line numbers are 1-indexed in vim, but we use 0-indexed internally
    let target = if line_number == 0 {
        0
    } else {
        (line_number - 1).min(row_count.saturating_sub(1))
    };

    app.view_state.table_state.select(Some(target));
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Move down by count rows (5j moves down 5 rows)
pub fn move_down_by(app: &mut App, count: usize) {
    let current = app.view_state.table_state.selected().unwrap_or(0);
    let target = (current + count).min(app.document.row_count().saturating_sub(1));
    app.view_state.table_state.select(Some(target));
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Move up by count rows (5k moves up 5 rows)
pub fn move_up_by(app: &mut App, count: usize) {
    let current = app.view_state.table_state.selected().unwrap_or(0);
    let target = current.saturating_sub(count);
    app.view_state.table_state.select(Some(target));
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Move right by count columns (3l moves right 3 columns)
pub fn move_right_by(app: &mut App, count: usize) {
    let new_col = app
        .view_state
        .selected_column
        .saturating_add(count)
        .get()
        .min(app.document.column_count().saturating_sub(1));
    app.view_state.selected_column = ColIndex::new(new_col);
    if app.view_state.selected_column.get()
        >= app.view_state.column_scroll_offset + MAX_VISIBLE_COLS
    {
        app.view_state.column_scroll_offset =
            app.view_state.selected_column.get() - MAX_VISIBLE_COLS + 1;
    }
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Move left by count columns (3h moves left 3 columns)
pub fn move_left_by(app: &mut App, count: usize) {
    let new_col = app.view_state.selected_column.saturating_sub(count);
    app.view_state.selected_column = new_col;
    if app.view_state.selected_column.get() < app.view_state.column_scroll_offset {
        app.view_state.column_scroll_offset = new_col.get();
    }
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Jump to column by Excel-style letter (A, B, ..., AA, AB, ...)
pub fn goto_column(app: &mut App, column_letter: &str) {
    use crate::input::StatusMessage;
    use crate::ui::utils::excel_letter_to_column;

    match excel_letter_to_column(column_letter) {
        Ok(col_idx) => {
            let max_col = app.document.column_count().saturating_sub(1);
            let target_col = col_idx.min(max_col);

            app.view_state.selected_column = ColIndex::new(target_col);

            // Update horizontal scroll
            if target_col < app.view_state.column_scroll_offset {
                app.view_state.column_scroll_offset = target_col;
            } else if target_col >= app.view_state.column_scroll_offset + MAX_VISIBLE_COLS {
                app.view_state.column_scroll_offset = target_col - MAX_VISIBLE_COLS + 1;
            }

            app.view_state.viewport_mode = ViewportMode::Auto;
            app.status_message = Some(StatusMessage::from(format!(
                "Jumped to column {}",
                column_letter
            )));
        }
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
        }
    }
}

/// Move to next non-empty cell in current row (w)
pub fn next_word(app: &mut App) {
    use crate::domain::position::RowIndex;
    use crate::input::StatusMessage;

    let current_row = app.view_state.table_state.selected().unwrap_or(0);
    let current_col = app.view_state.selected_column.get();
    let max_col = app.document.column_count().saturating_sub(1);

    for col in (current_col + 1)..=max_col {
        let cell = app
            .document
            .get_cell(RowIndex::new(current_row), ColIndex::new(col));
        if !cell.is_empty() {
            app.view_state.selected_column = ColIndex::new(col);
            update_horizontal_scroll(app, col);
            app.view_state.viewport_mode = ViewportMode::Auto;
            return;
        }
    }
    app.status_message = Some(StatusMessage::from("No more non-empty cells"));
}

/// Move to previous non-empty cell in current row (b)
pub fn prev_word(app: &mut App) {
    use crate::domain::position::RowIndex;
    use crate::input::StatusMessage;

    let current_row = app.view_state.table_state.selected().unwrap_or(0);
    let current_col = app.view_state.selected_column.get();

    if current_col == 0 {
        app.status_message = Some(StatusMessage::from("Already at first column"));
        return;
    }

    for col in (0..current_col).rev() {
        let cell = app
            .document
            .get_cell(RowIndex::new(current_row), ColIndex::new(col));
        if !cell.is_empty() {
            app.view_state.selected_column = ColIndex::new(col);
            update_horizontal_scroll(app, col);
            app.view_state.viewport_mode = ViewportMode::Auto;
            return;
        }
    }
    app.status_message = Some(StatusMessage::from("No previous non-empty cells"));
}

/// Move to last non-empty cell in current row (e)
pub fn end_word(app: &mut App) {
    use crate::domain::position::RowIndex;
    use crate::input::StatusMessage;

    let current_row = app.view_state.table_state.selected().unwrap_or(0);
    let max_col = app.document.column_count().saturating_sub(1);

    for col in (0..=max_col).rev() {
        let cell = app
            .document
            .get_cell(RowIndex::new(current_row), ColIndex::new(col));
        if !cell.is_empty() {
            app.view_state.selected_column = ColIndex::new(col);
            update_horizontal_scroll(app, col);
            app.view_state.viewport_mode = ViewportMode::Auto;
            return;
        }
    }
    // All cells are empty, go to last column
    app.view_state.selected_column = ColIndex::new(max_col);
    update_horizontal_scroll(app, max_col);
    app.status_message = Some(StatusMessage::from("All cells empty"));
}

/// Helper to update horizontal scroll position
fn update_horizontal_scroll(app: &mut App, target_col: usize) {
    if target_col < app.view_state.column_scroll_offset {
        app.view_state.column_scroll_offset = target_col;
    } else if target_col >= app.view_state.column_scroll_offset + MAX_VISIBLE_COLS {
        app.view_state.column_scroll_offset = target_col - MAX_VISIBLE_COLS + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use crate::domain::position::ColIndex;
    use std::path::PathBuf;

    fn create_test_app() -> App {
        let document = Document {
            headers: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            rows: {
                let mut rows = Vec::new();
                for i in 0..50 {
                    rows.push(vec![
                        format!("{}", i),
                        format!("{}", i + 1),
                        format!("{}", i + 2),
                    ]);
                }
                rows
            },
            filename: "test.csv".to_string(),
            is_dirty: false,
        };

        let csv_files = vec![PathBuf::from("test.csv")];
        App::new(document, csv_files, 0, crate::session::FileConfig::new())
    }

    #[test]
    fn test_goto_first_row() {
        let mut app = create_test_app();

        // Move to middle
        app.view_state.table_state.select(Some(25));

        goto_first_row(&mut app);

        assert_eq!(app.view_state.table_state.selected(), Some(0));
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_goto_last_row() {
        let mut app = create_test_app();

        goto_last_row(&mut app);

        let last_row = app.document.row_count().saturating_sub(1);
        assert_eq!(app.view_state.table_state.selected(), Some(last_row));
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_goto_line_valid() {
        let mut app = create_test_app();

        goto_line(&mut app, 10);

        assert_eq!(app.view_state.table_state.selected(), Some(9)); // 0-indexed
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_goto_line_out_of_bounds() {
        let mut app = create_test_app();
        let max_row = app.document.row_count().saturating_sub(1);

        goto_line(&mut app, 9999);

        assert_eq!(app.view_state.table_state.selected(), Some(max_row));
    }

    #[test]
    fn test_goto_line_zero() {
        let mut app = create_test_app();

        goto_line(&mut app, 0);

        assert_eq!(app.view_state.table_state.selected(), Some(0));
    }

    #[test]
    fn test_move_down_by_with_count() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(5));

        move_down_by(&mut app, 10);

        assert_eq!(app.view_state.table_state.selected(), Some(15));
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_move_down_saturating_at_last_row() {
        let mut app = create_test_app();
        let last_row = app.document.row_count().saturating_sub(1);
        app.view_state.table_state.select(Some(last_row - 5));

        move_down_by(&mut app, 100);

        assert_eq!(app.view_state.table_state.selected(), Some(last_row));
    }

    #[test]
    fn test_move_up_by_with_count() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(20));

        move_up_by(&mut app, 10);

        assert_eq!(app.view_state.table_state.selected(), Some(10));
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_move_up_saturating_at_zero() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(5));

        move_up_by(&mut app, 100);

        assert_eq!(app.view_state.table_state.selected(), Some(0));
    }

    #[test]
    fn test_move_right_by_with_count() {
        let mut app = create_test_app();
        app.view_state.selected_column = ColIndex::new(0);

        move_right_by(&mut app, 2);

        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_move_right_saturating_at_last_column() {
        let mut app = create_test_app();
        let last_col = app.document.column_count().saturating_sub(1);
        app.view_state.selected_column = ColIndex::new(0);

        move_right_by(&mut app, 999);

        assert_eq!(app.view_state.selected_column, ColIndex::new(last_col));
    }

    #[test]
    fn test_move_left_by_with_count() {
        let mut app = create_test_app();
        app.view_state.selected_column = ColIndex::new(2);

        move_left_by(&mut app, 1);

        assert_eq!(app.view_state.selected_column, ColIndex::new(1));
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_move_left_saturating_at_zero() {
        let mut app = create_test_app();
        app.view_state.selected_column = ColIndex::new(1);

        move_left_by(&mut app, 100);

        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_select_next_page() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(0));

        select_next_page(&mut app);

        assert_eq!(app.view_state.table_state.selected(), Some(PAGE_SIZE));
    }

    #[test]
    fn test_select_previous_page() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(PAGE_SIZE));

        select_previous_page(&mut app);

        assert_eq!(app.view_state.table_state.selected(), Some(0));
    }

    #[test]
    fn test_page_down_at_end() {
        let mut app = create_test_app();
        let last_row = app.document.row_count().saturating_sub(1);
        app.view_state.table_state.select(Some(last_row - 5));

        select_next_page(&mut app);

        assert_eq!(app.view_state.table_state.selected(), Some(last_row));
    }

    #[test]
    fn test_page_up_at_beginning() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(5));

        select_previous_page(&mut app);

        assert_eq!(app.view_state.table_state.selected(), Some(0));
    }

    #[test]
    fn test_goto_column_valid() {
        let mut app = create_test_app();

        goto_column(&mut app, "A");
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        goto_column(&mut app, "B");
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));

        goto_column(&mut app, "C");
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
    }

    #[test]
    fn test_goto_column_case_insensitive() {
        let mut app = create_test_app();

        goto_column(&mut app, "a");
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        goto_column(&mut app, "b");
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));
    }

    #[test]
    fn test_goto_column_out_of_bounds() {
        let mut app = create_test_app();

        // Try to jump to column ZZ (701), which does not exist (only have 3 columns)
        goto_column(&mut app, "ZZ");

        // Should clamp to last column (2)
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
    }

    #[test]
    fn test_goto_column_invalid() {
        let mut app = create_test_app();
        let initial_col = app.view_state.selected_column;

        // Invalid column letter
        goto_column(&mut app, "123");

        // Should stay at same position
        assert_eq!(app.view_state.selected_column, initial_col);
        // Should have error message
        assert!(app.status_message.is_some());
    }
}
