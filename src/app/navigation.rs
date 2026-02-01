use super::{messages, App, ViewportMode};
use crate::domain::position::ColIndex;
use crate::ui::MAX_VISIBLE_COLS;
use anyhow::Result;
use crossterm::event::KeyCode;

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

        _ => {}
    }

    Ok(())
}

fn select_next_page(app: &mut App) {
    const PAGE_SIZE: usize = 20;
    let i = match app.view_state.table_state.selected() {
        Some(i) => (i + PAGE_SIZE).min(app.document.row_count().saturating_sub(1)),
        None => 0,
    };
    app.view_state.table_state.select(Some(i));
}

fn select_previous_page(app: &mut App) {
    const PAGE_SIZE: usize = 20;
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
