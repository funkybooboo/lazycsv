//! Right-click context menu handling.

use super::mouse_coords::{resolve_column_header, resolve_row_gutter, resolve_table_cell};
use crate::app::{App, ContextMenu, ContextMenuItem, Mode, VisualMode, VisualSelection};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::InputResult;
use crate::ui::ViewportMode;

pub(crate) fn handle_right_click(app: &mut App, x: u16, y: u16) -> InputResult {
    // Dismiss any existing context menu
    app.context_menu = None;

    if !matches!(
        app.mode,
        Mode::Normal | Mode::VisualBlock | Mode::VisualLine | Mode::VisualColumn
    ) {
        return InputResult::Continue;
    }

    let layout = &app.view_state.mouse_layout;
    let area = layout.table_content_area;

    // Check if right-clicking on the column letters row
    if let Some(col_idx) = resolve_column_header(layout, area, x, y) {
        app.view_state.selected_column = ColIndex::new(col_idx);
        app.context_menu = Some(ContextMenu {
            x,
            y,
            selected: 0,
            items: vec![
                ContextMenuItem::ColumnInsertBefore,
                ContextMenuItem::ColumnInsertAfter,
                ContextMenuItem::Separator,
                ContextMenuItem::ColumnDelete,
            ],
        });
        return InputResult::Continue;
    }

    // Check if right-clicking on the row number gutter
    if let Some(row_idx) = resolve_row_gutter(layout, area, x, y) {
        app.view_state.table_state.select(Some(row_idx));
        app.view_state.viewport_mode = ViewportMode::Auto;
        app.context_menu = Some(ContextMenu {
            x,
            y,
            selected: 0,
            items: vec![
                ContextMenuItem::RowInsertAbove,
                ContextMenuItem::RowInsertBelow,
                ContextMenuItem::Separator,
                ContextMenuItem::RowDelete,
            ],
        });
        return InputResult::Continue;
    }

    let clicked_cell = resolve_table_cell(layout, area, x, y);

    // In normal mode, navigate to the right-clicked cell
    if let Some((target_row, target_col)) = clicked_cell {
        if app.mode == Mode::Normal {
            app.view_state.table_state.select(Some(target_row));
            app.view_state.selected_column = ColIndex::new(target_col);
            app.view_state.viewport_mode = ViewportMode::Auto;
        }
    } else if app.mode == Mode::Normal {
        // Right-clicked outside any cell and no visual selection — nothing to act on
        return InputResult::Continue;
    }

    app.context_menu = Some(ContextMenu {
        x,
        y,
        selected: 0,
        items: vec![
            ContextMenuItem::Cut,
            ContextMenuItem::Copy,
            ContextMenuItem::Paste,
            ContextMenuItem::Separator,
            ContextMenuItem::Clear,
        ],
    });

    InputResult::Continue
}

/// Handle a left-click while the context menu is open.
pub(crate) fn handle_context_menu_click(app: &mut App, x: u16, y: u16) -> InputResult {
    let menu = match app.context_menu.take() {
        Some(m) => m,
        None => return InputResult::Continue,
    };

    // Check if click is inside the menu popup
    let item_count = menu.items.len() as u16;
    let max_label = menu
        .items
        .iter()
        .map(|i| i.label().len())
        .max()
        .unwrap_or(4);
    let popup_width: u16 = (max_label as u16 + 4).max(14);
    let popup_height = item_count + 2;

    let frame_size = crossterm::terminal::size().unwrap_or((80, 24));
    let menu_x = menu.x.min(frame_size.0.saturating_sub(popup_width));
    let menu_y = menu.y.min(frame_size.1.saturating_sub(popup_height));

    // Inner area (inside borders)
    let inner_x = menu_x + 1;
    let inner_y = menu_y + 1;
    let inner_w = popup_width - 2;
    let inner_h = item_count;

    if x >= inner_x && x < inner_x + inner_w && y >= inner_y && y < inner_y + inner_h {
        let clicked_idx = (y - inner_y) as usize;
        if let Some(&item) = menu.items.get(clicked_idx) {
            if item != ContextMenuItem::Separator {
                execute_context_action(app, item);
            }
        }
    }
    // Menu is dismissed whether or not an item was clicked

    InputResult::Continue
}

/// Handle keyboard navigation within the context menu.
/// Called from the key handler when a context menu is open.
pub(crate) fn handle_context_menu_key(
    app: &mut App,
    key: crossterm::event::KeyEvent,
) -> InputResult {
    use crossterm::event::KeyCode;

    let menu = match app.context_menu.as_mut() {
        Some(m) => m,
        None => return InputResult::Continue,
    };

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            // Skip separators when navigating up
            let mut idx = menu.selected;
            loop {
                if idx == 0 {
                    break;
                }
                idx -= 1;
                if menu.items[idx] != ContextMenuItem::Separator {
                    menu.selected = idx;
                    break;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let mut idx = menu.selected;
            loop {
                if idx + 1 >= menu.items.len() {
                    break;
                }
                idx += 1;
                if menu.items[idx] != ContextMenuItem::Separator {
                    menu.selected = idx;
                    break;
                }
            }
        }
        KeyCode::Enter => {
            let item = menu.items[menu.selected];
            app.context_menu = None;
            if item != ContextMenuItem::Separator {
                execute_context_action(app, item);
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.context_menu = None;
        }
        _ => {}
    }

    InputResult::Continue
}

/// Execute a context menu action.
pub(crate) fn execute_context_action(app: &mut App, item: ContextMenuItem) {
    use crate::input::StatusMessage;

    match item {
        ContextMenuItem::Cut => {
            // Cut = yank + clear
            if app.visual_selection.is_some() {
                // Visual mode: yank then delete (existing logic)
                let _ = crate::input::visual_mode::handle_visual_delete(app);
            } else {
                // Single cell: copy value, then clear
                if let Some(row_idx) = app.selected_row() {
                    let col_idx = app.view_state.selected_column;
                    let value = app.document.cell(row_idx, col_idx).to_string();
                    app.clipboard.yank_cell(value.clone());
                    let _ = crate::clipboard::copy_text_to_system_clipboard(&value);
                    let old_value = value;
                    app.document.set_cell(row_idx, col_idx, String::new());
                    app.history.push(crate::history::EditCommand::SetCell {
                        row: row_idx,
                        col: col_idx,
                        old_value,
                        new_value: String::new(),
                    });
                    app.status_message = Some(StatusMessage::from("Cell cut"));
                }
            }
        }
        ContextMenuItem::Copy => {
            if app.visual_selection.is_some() {
                let _ = crate::input::visual_mode::handle_visual_yank(app);
            } else {
                // Single cell copy
                if let Some(row_idx) = app.selected_row() {
                    let col_idx = app.view_state.selected_column;
                    let value = app.document.cell(row_idx, col_idx).to_string();
                    app.clipboard.yank_cell(value.clone());
                    let _ = crate::clipboard::copy_text_to_system_clipboard(&value);
                    app.status_message = Some(StatusMessage::from(format!("Copied: {}", value)));
                }
            }
        }
        ContextMenuItem::Paste => {
            if app.visual_selection.is_some() {
                let _ = crate::input::visual_mode::handle_visual_paste(app);
            } else if let Some(row_idx) = app.selected_row() {
                let col_idx = app.view_state.selected_column;

                if let Some(value) = app.clipboard.cell().map(|s| s.to_string()) {
                    // Single cell paste
                    let old_value = app.document.cell(row_idx, col_idx).to_string();
                    app.document.set_cell(row_idx, col_idx, value.clone());
                    app.history.push(crate::history::EditCommand::SetCell {
                        row: row_idx,
                        col: col_idx,
                        old_value,
                        new_value: value.clone(),
                    });
                    app.status_message = Some(StatusMessage::from(format!("Pasted: {}", value)));
                } else if app.clipboard.region().is_some() {
                    // Paste region at current cell
                    app.visual_selection =
                        Some(VisualSelection::new(row_idx, col_idx, VisualMode::Block));
                    let _ = crate::input::visual_mode::handle_visual_paste(app);
                } else if app.clipboard.rows().is_some() {
                    // Paste rows at current position
                    crate::input::normal_mode::editing::paste_rows_below(app);
                } else {
                    app.status_message = Some(StatusMessage::from("Nothing to paste"));
                }
            }
        }
        ContextMenuItem::Clear => {
            if app.visual_selection.is_some() {
                // Visual delete clears cells (for block mode)
                let _ = crate::input::visual_mode::handle_visual_delete(app);
            } else {
                // Single cell clear
                if let Some(row_idx) = app.selected_row() {
                    let col_idx = app.view_state.selected_column;
                    let old_value = app.document.cell(row_idx, col_idx).to_string();
                    if !old_value.is_empty() {
                        app.document.set_cell(row_idx, col_idx, String::new());
                        app.history.push(crate::history::EditCommand::SetCell {
                            row: row_idx,
                            col: col_idx,
                            old_value,
                            new_value: String::new(),
                        });
                    }
                    app.status_message = Some(StatusMessage::from("Cell cleared"));
                }
            }
        }
        ContextMenuItem::ColumnDelete => {
            crate::input::normal_mode::commands::delete_columns(app);
        }
        ContextMenuItem::ColumnInsertBefore => {
            crate::input::normal_mode::commands::insert_column_before(app);
        }
        ContextMenuItem::ColumnInsertAfter => {
            crate::input::normal_mode::commands::insert_column_after(app);
        }
        ContextMenuItem::RowDelete => {
            crate::input::normal_mode::commands::delete_rows(app);
        }
        ContextMenuItem::RowInsertAbove => {
            // Insert empty row above without entering insert mode
            if let Some(row_idx) = app.selected_row() {
                app.document.insert_row(row_idx);
                app.history
                    .push(crate::history::EditCommand::InsertRow { at: row_idx });
                app.status_message = Some(StatusMessage::from("Inserted row above"));
            }
        }
        ContextMenuItem::RowInsertBelow => {
            // Insert empty row below without entering insert mode
            if let Some(row_idx) = app.selected_row() {
                let new_row = RowIndex::new(row_idx.get() + 1);
                app.document.insert_row(new_row);
                app.history
                    .push(crate::history::EditCommand::InsertRow { at: new_row });
                app.view_state.table_state.select(Some(new_row.get()));
                app.status_message = Some(StatusMessage::from("Inserted row below"));
            }
        }
        ContextMenuItem::Separator => {}
    }
}
