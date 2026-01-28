use super::{App, ViewportMode};
use anyhow::Result;
use crossterm::event::KeyCode;

/// Handle navigation keys with optional count prefix
pub fn handle_navigation(app: &mut App, code: KeyCode) -> Result<()> {
    // Consume count prefix (e.g., "5" from command_count for 5j)
    let count = app
        .command_count
        .take()
        .and_then(|s| s.parse().ok())
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

        // Word forward (with count: 3w moves right 3 columns)
        KeyCode::Char('w') => {
            move_right_by(app, count);
        }

        // Word backward (with count: 3b moves left 3 columns)
        KeyCode::Char('b') => {
            move_left_by(app, count);
        }

        // First column
        KeyCode::Char('0') => {
            app.selected_col = 0;
            app.horizontal_offset = 0;
            app.viewport_mode = ViewportMode::Auto;
        }

        // Last column
        KeyCode::Char('$') => {
            app.selected_col = app.csv_data.column_count().saturating_sub(1);
            // Adjust horizontal offset to show last column
            let max_visible_cols = 10;
            if app.csv_data.column_count() > max_visible_cols {
                app.horizontal_offset = app.csv_data.column_count() - max_visible_cols;
            }
            app.viewport_mode = ViewportMode::Auto;
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
                app.status_message = Some(format!("Jumped to line {}", count));
            } else {
                goto_last_row(app);
            }
        }

        _ => {}
    }

    Ok(())
}

fn select_next_col(app: &mut App) {
    if app.selected_col < app.csv_data.column_count().saturating_sub(1) {
        app.selected_col += 1;

        // Auto-scroll horizontally if needed
        let max_visible_cols = 10;
        if app.selected_col >= app.horizontal_offset + max_visible_cols {
            app.horizontal_offset = app.selected_col - max_visible_cols + 1;
        }
    }
}

fn select_previous_col(app: &mut App) {
    if app.selected_col > 0 {
        app.selected_col -= 1;

        // Auto-scroll horizontally if needed
        if app.selected_col < app.horizontal_offset {
            app.horizontal_offset = app.selected_col;
        }
    }
}

fn select_next_page(app: &mut App) {
    const PAGE_SIZE: usize = 20;
    let i = match app.table_state.selected() {
        Some(i) => (i + PAGE_SIZE).min(app.csv_data.row_count().saturating_sub(1)),
        None => 0,
    };
    app.table_state.select(Some(i));
}

fn select_previous_page(app: &mut App) {
    const PAGE_SIZE: usize = 20;
    let i = match app.table_state.selected() {
        Some(i) => i.saturating_sub(PAGE_SIZE),
        None => 0,
    };
    app.table_state.select(Some(i));
}

/// Go to first row (gg command)
pub fn goto_first_row(app: &mut App) {
    app.table_state.select(Some(0));
    app.viewport_mode = ViewportMode::Auto;
}

/// Go to last row (G command)
pub fn goto_last_row(app: &mut App) {
    let last = app.csv_data.row_count().saturating_sub(1);
    app.table_state.select(Some(last));
    app.viewport_mode = ViewportMode::Auto;
}

/// Go to specific line number (5G or :5 command)
pub fn goto_line(app: &mut App, line_number: usize) {
    let row_count = app.csv_data.row_count();

    // Line numbers are 1-indexed in vim, but we use 0-indexed internally
    let target = if line_number == 0 {
        0
    } else {
        (line_number - 1).min(row_count.saturating_sub(1))
    };

    app.table_state.select(Some(target));
    app.viewport_mode = ViewportMode::Auto;
}

/// Move down by count rows (5j moves down 5 rows)
pub fn move_down_by(app: &mut App, count: usize) {
    let current = app.table_state.selected().unwrap_or(0);
    let target = (current + count).min(app.csv_data.row_count().saturating_sub(1));
    app.table_state.select(Some(target));
    app.viewport_mode = ViewportMode::Auto;
}

/// Move up by count rows (5k moves up 5 rows)
pub fn move_up_by(app: &mut App, count: usize) {
    let current = app.table_state.selected().unwrap_or(0);
    let target = current.saturating_sub(count);
    app.table_state.select(Some(target));
    app.viewport_mode = ViewportMode::Auto;
}

/// Move right by count columns (3l moves right 3 columns)
pub fn move_right_by(app: &mut App, count: usize) {
    for _ in 0..count {
        select_next_col(app);
    }
    app.viewport_mode = ViewportMode::Auto;
}

/// Move left by count columns (3h moves left 3 columns)
pub fn move_left_by(app: &mut App, count: usize) {
    for _ in 0..count {
        select_previous_col(app);
    }
    app.viewport_mode = ViewportMode::Auto;
}
