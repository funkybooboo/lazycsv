//! Navigation operations for Normal mode

use crate::app::App;
use crate::navigation;

/// Move down one row (Enter key)
pub fn move_down(app: &mut App) {
    navigation::commands::move_down_by(app, 1);
}

/// Page down (Ctrl+d)
pub fn page_down(app: &mut App) {
    let count = app
        .input_state
        .command_count
        .take()
        .map(|n| n.get())
        .unwrap_or(1);
    for _ in 0..count {
        let current = app.view_state.table_state.selected().unwrap_or(0);
        let target =
            (current + navigation::PAGE_SIZE).min(app.document.row_count().saturating_sub(1));
        app.view_state.table_state.select(Some(target));
    }
}

/// Page up (Ctrl+u)
pub fn page_up(app: &mut App) {
    let count = app
        .input_state
        .command_count
        .take()
        .map(|n| n.get())
        .unwrap_or(1);
    for _ in 0..count {
        let current = app.view_state.table_state.selected().unwrap_or(0);
        let target = current.saturating_sub(navigation::PAGE_SIZE);
        app.view_state.table_state.select(Some(target));
    }
}
