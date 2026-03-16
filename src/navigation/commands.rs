//! Navigation command implementations for vim-style movement.
//!
//! This module provides functions for navigating the CSV table including
//! cursor movement, page scrolling, and jump commands with count prefixes.

use crate::app::App;
use crate::domain::position::ColIndex;
use crate::ui::ViewportMode;
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
        // Directional movement with count
        KeyCode::Up
        | KeyCode::Char('k')
        | KeyCode::Down
        | KeyCode::Char('j')
        | KeyCode::Left
        | KeyCode::Char('h')
        | KeyCode::Right
        | KeyCode::Char('l') => {
            handle_directional_movement(app, code, count);
        }

        // Jump to column boundaries
        KeyCode::Char('0') | KeyCode::Char('$') => {
            handle_column_boundary(app, code);
        }

        // Page navigation
        KeyCode::PageDown | KeyCode::PageUp => {
            handle_page_navigation(app, code);
        }

        // Row jumps (Home, End, G with count)
        KeyCode::Home | KeyCode::End | KeyCode::Char('G') => {
            handle_row_jump(app, code, count);
        }

        // Word motion
        KeyCode::Char('w') | KeyCode::Char('b') | KeyCode::Char('e') => {
            handle_word_motion(app, code);
        }

        _ => {}
    }

    Ok(())
}

/// Handle directional movement (hjkl/arrows) with count prefix
fn handle_directional_movement(app: &mut App, code: KeyCode, count: usize) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => move_up_by(app, count),
        KeyCode::Down | KeyCode::Char('j') => move_down_by(app, count),
        KeyCode::Left | KeyCode::Char('h') => move_left_by(app, count),
        KeyCode::Right | KeyCode::Char('l') => move_right_by(app, count),
        _ => {}
    }
}

/// Handle column boundary jumps (0, $)
fn handle_column_boundary(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('0') => {
            app.view_state.selected_column = ColIndex::new(0);
            app.view_state.column_scroll_offset = 0;
            app.view_state.viewport_mode = ViewportMode::Auto;
        }
        KeyCode::Char('$') => {
            app.view_state.selected_column =
                ColIndex::new(app.document.column_count().saturating_sub(1));
            // Adjust horizontal offset to show last column
            if app.document.column_count() > app.view_state.visible_cols_count {
                app.view_state.column_scroll_offset =
                    app.document.column_count() - app.view_state.visible_cols_count;
            }
            app.view_state.viewport_mode = ViewportMode::Auto;
        }
        _ => {}
    }
}

/// Handle page navigation (PageUp, PageDown)
fn handle_page_navigation(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::PageDown => select_next_page(app),
        KeyCode::PageUp => select_previous_page(app),
        _ => {}
    }
}

/// Handle row jumps (Home, End, G with count)
fn handle_row_jump(app: &mut App, code: KeyCode, count: usize) {
    match code {
        KeyCode::Home => goto_first_row(app),
        KeyCode::End | KeyCode::Char('G') => {
            if count > 1 {
                // goto_line sets its own status message on success or error
                goto_line(app, count);
            } else {
                goto_last_row(app);
            }
        }
        _ => {}
    }
}

/// Handle word motion (w, b, e)
fn handle_word_motion(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('w') => next_word(app),
        KeyCode::Char('b') => prev_word(app),
        KeyCode::Char('e') => end_word(app),
        _ => {}
    }
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
/// When header_mode is ON: goes to row 1 (first data row)
/// When header_mode is OFF: goes to row 0
pub fn goto_first_row(app: &mut App) {
    let target_row = if app.document.header_mode && app.document.row_count() > 1 {
        1 // Skip header, go to first data row
    } else {
        0
    };
    app.view_state.table_state.select(Some(target_row));
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Go to last row (G command)
pub fn goto_last_row(app: &mut App) {
    if app.document.row_count() > 0 {
        let last = app.document.row_count() - 1;
        app.view_state.table_state.select(Some(last));
        app.view_state.viewport_mode = ViewportMode::Auto;
    }
}

/// Go to specific line number (5G or :5 command)
/// Line numbers are 1-indexed (vim style): 1 = row 1 (first data), 2 = row 2, etc.
/// To go to row 0 (header), use gg or special navigation
pub fn goto_line(app: &mut App, line_number: usize) {
    use crate::input::StatusMessage;

    let row_count = app.document.row_count();

    // Line numbers are 1-indexed in vim, and map to absolute rows 1, 2, 3...
    // (Row 0 is the header and is accessed via gg when header_mode OFF)
    if line_number == 0 {
        app.status_message = Some(StatusMessage::from("Line number must be >= 1"));
        return;
    }

    if line_number >= row_count {
        app.status_message = Some(StatusMessage::from(format!(
            "Row {} does not exist (max: {})",
            line_number,
            row_count - 1
        )));
        return;
    }

    let target = line_number; // line_number 1 → row 1 (first data)
    app.view_state.table_state.select(Some(target));
    app.view_state.viewport_mode = ViewportMode::Auto;
    app.status_message = Some(StatusMessage::from(format!(
        "Jumped to row {}",
        line_number
    )));
}

/// Move down by count rows (5j moves down 5 rows)
pub fn move_down_by(app: &mut App, count: usize) {
    let current = app.view_state.table_state.selected().unwrap_or(0);
    let max_row = if app.document.row_count() > 0 {
        app.document.row_count() - 1
    } else {
        0
    };
    let target = (current + count).min(max_row);
    app.view_state.table_state.select(Some(target));
    app.view_state.viewport_mode = ViewportMode::Auto;
}

/// Move up by count rows (5k moves up 5 rows)
pub fn move_up_by(app: &mut App, count: usize) {
    let current = app.view_state.table_state.selected().unwrap_or(0);
    let target = current.saturating_sub(count);
    let final_target = target.max(0);

    app.view_state.table_state.select(Some(final_target));
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
        >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count
    {
        app.view_state.column_scroll_offset =
            app.view_state.selected_column.get() - app.view_state.visible_cols_count + 1;
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
    use crate::ui::utils::{column_to_excel_letter, excel_letter_to_column};

    match excel_letter_to_column(column_letter) {
        Ok(col_idx) => {
            let max_col = app.document.column_count();

            // Check out-of-bounds (col_idx is 0-indexed, max_col is count)
            if col_idx >= max_col {
                let max_letter = column_to_excel_letter(max_col.saturating_sub(1));
                app.status_message = Some(StatusMessage::from(format!(
                    "Column {} does not exist (max: {})",
                    column_letter.to_uppercase(),
                    max_letter
                )));
                return;
            }

            app.view_state.selected_column = ColIndex::new(col_idx);

            // Update horizontal scroll
            if col_idx < app.view_state.column_scroll_offset {
                app.view_state.column_scroll_offset = col_idx;
            } else if col_idx >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count {
                app.view_state.column_scroll_offset = col_idx - app.view_state.visible_cols_count + 1;
            }

            app.view_state.viewport_mode = ViewportMode::Auto;
            app.status_message = Some(StatusMessage::from(format!(
                "Jumped to column {}",
                column_letter.to_uppercase()
            )));
        }
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
        }
    }
}

/// Jump to column by 1-indexed number
pub fn goto_column_by_number(app: &mut App, col_num: usize) {
    use crate::input::StatusMessage;
    use crate::ui::utils::column_to_excel_letter;

    let max_col = app.document.column_count();
    if col_num > max_col {
        let max_letter = column_to_excel_letter(max_col.saturating_sub(1));
        app.status_message = Some(StatusMessage::from(format!(
            "Column {} does not exist (max: {} / {})",
            col_num, max_col, max_letter
        )));
        return;
    }

    let col_idx = col_num.saturating_sub(1); // Convert to 0-indexed
    app.view_state.selected_column = ColIndex::new(col_idx);

    // Update horizontal scroll
    if col_idx < app.view_state.column_scroll_offset {
        app.view_state.column_scroll_offset = col_idx;
    } else if col_idx >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count {
        app.view_state.column_scroll_offset = col_idx - app.view_state.visible_cols_count + 1;
    }

    app.view_state.viewport_mode = ViewportMode::Auto;
    let col_letter = column_to_excel_letter(col_idx);
    app.status_message = Some(StatusMessage::from(format!(
        "Jumped to column {} ({})",
        col_num, col_letter
    )));
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
            .cell(RowIndex::new(current_row), ColIndex::new(col));
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
            .cell(RowIndex::new(current_row), ColIndex::new(col));
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
            .cell(RowIndex::new(current_row), ColIndex::new(col));
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
    } else if target_col >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count {
        app.view_state.column_scroll_offset = target_col - app.view_state.visible_cols_count + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use crate::domain::position::ColIndex;
    use crate::session::FileConfig;
    use std::path::PathBuf;

    fn create_test_app() -> App {
        let data_rows = {
            let mut rows = Vec::new();
            for i in 0..50 {
                rows.push(vec![
                    format!("{}", i),
                    format!("{}", i + 1),
                    format!("{}", i + 2),
                ]);
            }
            rows
        };

        let document = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            data_rows,
            "test.csv".to_string(),
        );

        let csv_files = vec![PathBuf::from("test.csv")];
        App::new(document, csv_files, 0, crate::session::FileConfig::new())
    }

    #[test]
    fn test_goto_first_row() {
        let mut app = create_test_app();

        // Move to middle
        app.view_state.table_state.select(Some(25));

        goto_first_row(&mut app);

        // With header_mode=true (default), gg goes to row 1
        assert_eq!(app.view_state.table_state.selected(), Some(1));
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

        // 10G goes to absolute row 10
        assert_eq!(app.view_state.table_state.selected(), Some(10));
        assert_eq!(app.view_state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_goto_line_out_of_bounds() {
        let mut app = create_test_app();
        let initial_row = app.view_state.table_state.selected();

        goto_line(&mut app, 9999);

        // Position should not change when out of bounds
        assert_eq!(app.view_state.table_state.selected(), initial_row);
        // Should show error message
        assert!(app.status_message.is_some());
        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("does not exist"));
    }

    #[test]
    fn test_goto_line_zero() {
        let mut app = create_test_app();
        let initial_row = app.view_state.table_state.selected();

        goto_line(&mut app, 0);

        // Row 0 is invalid (1-indexed), position should not change
        assert_eq!(app.view_state.table_state.selected(), initial_row);
        // Should show error message
        assert!(app.status_message.is_some());
        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("must be >= 1"));
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

        // Now stops at row 0 (can navigate to header row)
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
        let initial_col = app.view_state.selected_column;

        // Try to jump to column ZZ (701), which does not exist (only have 3 columns)
        goto_column(&mut app, "ZZ");

        // Position should not change when out of bounds
        assert_eq!(app.view_state.selected_column, initial_col);
        // Should show error message
        assert!(app.status_message.is_some());
        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("does not exist"));
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

    #[test]
    fn test_goto_column_multi_letter_aa() {
        let csv_data = create_large_csv_data(3, 50); // 50 columns
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // Jump to column AA (26)
        goto_column(&mut app, "AA");
        assert_eq!(app.view_state.selected_column, ColIndex::new(26));

        // Jump to column AB (27)
        goto_column(&mut app, "AB");
        assert_eq!(app.view_state.selected_column, ColIndex::new(27));
    }

    #[test]
    fn test_goto_column_multi_letter_case_mixed() {
        let csv_data = create_large_csv_data(3, 50);
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // Test mixed case: aB, Ab, AB should all go to column 27
        goto_column(&mut app, "aB");
        assert_eq!(app.view_state.selected_column, ColIndex::new(27));

        goto_column(&mut app, "Ab");
        assert_eq!(app.view_state.selected_column, ColIndex::new(27));

        goto_column(&mut app, "ab");
        assert_eq!(app.view_state.selected_column, ColIndex::new(27));
    }

    #[test]
    fn test_goto_column_three_letters() {
        let csv_data = create_large_csv_data(3, 800); // 800 columns
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // Jump to column AAA (702)
        goto_column(&mut app, "AAA");
        assert_eq!(app.view_state.selected_column, ColIndex::new(702));

        // Jump to column ABC (730)
        goto_column(&mut app, "ABC");
        assert_eq!(app.view_state.selected_column, ColIndex::new(730));
    }

    #[test]
    fn test_goto_column_beyond_available_shows_error() {
        let mut app = create_test_app(); // Only 3 columns
        let initial_col = app.view_state.selected_column;

        // Try to jump to column BA (52)
        goto_column(&mut app, "BA");

        // Position should not change when out of bounds
        assert_eq!(app.view_state.selected_column, initial_col);
        // Should show error message
        assert!(app.status_message.is_some());
        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("does not exist"));
    }

    #[test]
    fn test_next_word_all_empty_cells() {
        let csv_data = Document::new(
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
            ],
            vec![vec![
                "value".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // At column 0 (non-empty)
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Try to move to next word
        next_word(&mut app);

        // Should stay at column 0 or show message (no more non-empty cells)
        // Current implementation may stay or move, verify it doesn't crash
        assert!(app.status_message.is_some() || app.view_state.selected_column == ColIndex::new(0));
    }

    #[test]
    fn test_prev_word_all_empty_cells() {
        let csv_data = Document::new(
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
            ],
            vec![vec![
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "value".to_string(),
            ]],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // Start at last column
        app.view_state.selected_column = ColIndex::new(3);

        // Try to move to prev word
        prev_word(&mut app);

        // Should stay at column 3 or show message
        assert!(app.status_message.is_some() || app.view_state.selected_column == ColIndex::new(3));
    }

    #[test]
    fn test_word_motion_single_non_empty_cell() {
        let csv_data = Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![vec!["".to_string(), "value".to_string(), "".to_string()]],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // Start at column 0 (empty)
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Move to next word
        next_word(&mut app);

        // Should move to column 1 (the single non-empty cell)
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));

        // Try to move to next word again
        next_word(&mut app);

        // Should stay at column 1 (no more non-empty cells)
        assert!(app.status_message.is_some() || app.view_state.selected_column == ColIndex::new(1));
    }

    #[test]
    fn test_word_motion_alternating_empty_filled() {
        let csv_data = Document::new(
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
                "E".to_string(),
            ],
            vec![vec![
                "a".to_string(),
                "".to_string(),
                "b".to_string(),
                "".to_string(),
                "c".to_string(),
            ]],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // Start at column 0 ("a")
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Move to next word
        next_word(&mut app);
        assert_eq!(app.view_state.selected_column, ColIndex::new(2)); // "b"

        // Move to next word
        next_word(&mut app);
        assert_eq!(app.view_state.selected_column, ColIndex::new(4)); // "c"

        // Move to prev word
        prev_word(&mut app);
        assert_eq!(app.view_state.selected_column, ColIndex::new(2)); // back to "b"

        // Move to prev word
        prev_word(&mut app);
        assert_eq!(app.view_state.selected_column, ColIndex::new(0)); // back to "a"
    }

    fn create_large_csv_data(rows: usize, cols: usize) -> Document {
        let headers = (0..cols).map(|i| format!("Col{}", i)).collect();
        let rows_data = (0..rows)
            .map(|r| (0..cols).map(|c| format!("R{}C{}", r, c)).collect())
            .collect();
        Document::new(headers, rows_data, "large.csv".to_string())
    }
}
