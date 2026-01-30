mod help;
mod status;
mod table;
pub mod utils;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Main UI rendering function
pub fn render(frame: &mut Frame, app: &mut App) {
    // Split terminal into main area + sheet switcher + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Table area
            Constraint::Length(3), // Sheet switcher
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    // Render table with row/column numbers
    table::render_table(frame, app, chunks[0]);

    // Render sheet switcher (always visible)
    status::render_sheet_switcher(frame, app, chunks[1]);

    // Render status bar
    status::render_status_bar(frame, app, chunks[2]);

    // Render help overlay if active
    if app.ui.show_cheatsheet {
        help::render_cheatsheet(frame);
    }
}

// Re-export public utilities
pub use utils::column_index_to_letter;
