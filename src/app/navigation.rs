use super::App;
use crossterm::event::KeyCode;

/// Handle navigation keys
pub fn handle_navigation(app: &mut App, code: KeyCode) {
    match code {
        // Move up
        KeyCode::Up | KeyCode::Char('k') => {
            select_previous_row(app);
        }

        // Move down
        KeyCode::Down | KeyCode::Char('j') => {
            select_next_row(app);
        }

        // Move left (previous column)
        KeyCode::Left | KeyCode::Char('h') => {
            select_previous_col(app);
        }

        // Move right (next column)
        KeyCode::Right | KeyCode::Char('l') => {
            select_next_col(app);
        }

        // Word forward (next column, same as 'l')
        KeyCode::Char('w') => {
            select_next_col(app);
        }

        // Word backward (previous column, same as 'h')
        KeyCode::Char('b') => {
            select_previous_col(app);
        }

        // First column
        KeyCode::Char('0') => {
            app.selected_col = 0;
            app.horizontal_offset = 0;
        }

        // Last column
        KeyCode::Char('$') => {
            app.selected_col = app.csv_data.column_count().saturating_sub(1);
            // Adjust horizontal offset to show last column
            let max_visible_cols = 10;
            if app.csv_data.column_count() > max_visible_cols {
                app.horizontal_offset = app.csv_data.column_count() - max_visible_cols;
            }
        }

        // Page down
        KeyCode::PageDown | KeyCode::Char('d') if code == KeyCode::Char('d') => {
            // Note: Ctrl+d is tricky to detect, using PageDown for now
            select_next_page(app);
        }

        // Page up
        KeyCode::PageUp | KeyCode::Char('u') if code == KeyCode::Char('u') => {
            // Note: Ctrl+u is tricky to detect, using PageUp for now
            select_previous_page(app);
        }

        // Home (first row) - gg in vim
        KeyCode::Home | KeyCode::Char('g') => {
            app.table_state.select(Some(0));
        }

        // End (last row) - G in vim
        KeyCode::End | KeyCode::Char('G') => {
            if app.csv_data.row_count() > 0 {
                app.table_state.select(Some(app.csv_data.row_count() - 1));
            }
        }

        _ => {}
    }
}

fn select_next_row(app: &mut App) {
    let i = match app.table_state.selected() {
        Some(i) => {
            if i < app.csv_data.row_count().saturating_sub(1) {
                i + 1
            } else {
                i
            }
        }
        None => 0,
    };
    app.table_state.select(Some(i));
}

fn select_previous_row(app: &mut App) {
    let i = match app.table_state.selected() {
        Some(i) => i.saturating_sub(1),
        None => 0,
    };
    app.table_state.select(Some(i));
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
