pub mod file_manager;
pub mod file_switcher;
pub mod help;
pub mod magnifier;
pub mod sql_editor;
// mod sql_editor_helpers; // DEPRECATED: Old SQL editor code, removed in v0.11.0
pub mod status_bar;
pub mod table;
pub mod utils;
pub mod view_state;

/// Maximum number of columns to display simultaneously
/// This prevents horizontal overflow on standard terminals
pub const MAX_VISIBLE_COLS: usize = 10;

use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Render a centered loading message (used before App exists, e.g. initial file load)
pub fn render_loading(frame: &mut Frame, message: &str) {
    use ratatui::layout::Alignment;
    use ratatui::widgets::Paragraph;

    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Percentage(50),
        ])
        .split(area);
    let paragraph = Paragraph::new(message.to_string()).alignment(Alignment::Center);
    frame.render_widget(paragraph, vertical[1]);
}

/// Main UI rendering function
pub fn render(frame: &mut Frame, app: &mut App) {
    // Split terminal into main area + file switcher + status bar
    // Minimal layout: no heavy borders, just horizontal rules as separators
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Table area (includes title bar + rule)
            Constraint::Length(2), // File switcher (rule + file list)
            Constraint::Length(1), // Status bar (single line, vim-like)
        ])
        .split(frame.area());

    // Render table with row/column numbers
    table::render_table(frame, app, chunks[0]);

    // Render file switcher (always visible)
    file_switcher::render(frame, app, chunks[1]);

    // Render status bar
    status_bar::render(frame, app, chunks[2]);

    // Render help overlay if active
    if app.view_state.help_overlay_visible {
        let search_query = app.view_state.help_search_query.as_deref();
        help::render_help_overlay(frame, app.view_state.help_scroll_offset, search_query);
    }

    // Render SQL editor overlay if active
    if app.mode == crate::app::Mode::SqlEditor {
        if let Some(ref vim_editor) = app.sql_vim_editor {
            sql_editor::render_sql_editor_vim(frame, vim_editor, app.sql_error.as_deref());
        }
    }

    // Render magnifier overlay if active
    if app.magnifier_state.is_some() {
        magnifier::render_magnifier(frame, app, frame.area());
    }

    // Render file manager modal if active
    if app.mode == crate::app::Mode::FileList {
        file_manager::render(frame, app);
    }
}

// Re-export public utilities and types
pub use utils::column_to_excel_letter;
pub use view_state::{ViewState, ViewportMode};
