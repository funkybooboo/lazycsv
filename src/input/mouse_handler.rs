//! Mouse event handling for the TUI.
//!
//! Maps terminal mouse coordinates to logical cells, file list items,
//! and scroll actions using layout information captured during rendering.

use crate::app::{App, ContextMenu, ContextMenuItem, Mode, VisualMode, VisualSelection};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::handler::{enter_insert_mode, CursorPosition, InitialContent};
use crate::input::InputResult;
use crate::ui::ViewportMode;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::io::Write;
use std::time::Instant;

/// Maximum interval between two clicks to count as a double-click.
const DOUBLE_CLICK_MS: u128 = 400;

/// Pixel threshold for detecting a column border click (distance from right edge).
const RESIZE_GRAB_ZONE: u16 = 2;

/// Set the mouse pointer shape via OSC 22 escape sequence.
/// Supported by xterm, foot, WezTerm, and some other terminals.
/// Falls back gracefully (ignored by unsupported terminals).
fn set_pointer_shape(name: &str) {
    let _ = write!(std::io::stdout(), "\x1b]22;{}\x07", name);
    let _ = std::io::stdout().flush();
}

/// Reset mouse pointer to default shape.
fn reset_pointer() {
    set_pointer_shape("default");
}

/// Handle a mouse event and return the appropriate input result.
pub fn handle_mouse(app: &mut App, event: MouseEvent) -> InputResult {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let (x, y) = (event.column, event.row);

            // If context menu is open, handle click on it or dismiss
            if app.context_menu.is_some() {
                return handle_context_menu_click(app, x, y);
            }

            // Check for column resize grab on header row border
            if try_start_column_resize(app, x, y) {
                return InputResult::Continue;
            }

            // Check for double-click (same position within time threshold)
            let is_double =
                if let Some((prev_time, prev_x, prev_y)) = app.view_state.mouse_layout.last_click {
                    prev_x == x && prev_y == y && prev_time.elapsed().as_millis() < DOUBLE_CLICK_MS
                } else {
                    false
                };

            if is_double {
                app.view_state.mouse_layout.last_click = None;
                handle_double_click(app, x, y)
            } else {
                app.view_state.mouse_layout.last_click = Some((Instant::now(), x, y));
                handle_left_click(app, x, y)
            }
        }
        MouseEventKind::Down(MouseButton::Right) => {
            handle_right_click(app, event.column, event.row)
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            // If a column resize is active, handle it
            if app.view_state.mouse_layout.col_resize.is_some() {
                return handle_column_resize_drag(app, event.column);
            }
            // Column or row reorder drags are finalized on mouse-up;
            // during drag, just show the drop target by highlighting
            if app.view_state.mouse_layout.col_reorder.is_some()
                || app.view_state.mouse_layout.row_reorder.is_some()
            {
                return handle_reorder_drag(app, event.column, event.row);
            }
            handle_drag(app, event.column, event.row)
        }
        MouseEventKind::Up(MouseButton::Left) => {
            let had_drag = app.view_state.mouse_layout.col_reorder.is_some()
                || app.view_state.mouse_layout.row_reorder.is_some()
                || app.view_state.mouse_layout.col_resize.is_some();

            // Finalize column reorder
            if let Some((src_col, _)) = app.view_state.mouse_layout.col_reorder.take() {
                reset_pointer();
                return finalize_column_reorder(app, src_col, event.column, event.row);
            }
            // Finalize row reorder
            if let Some((src_row, _)) = app.view_state.mouse_layout.row_reorder.take() {
                reset_pointer();
                return finalize_row_reorder(app, src_row, event.column, event.row);
            }
            app.view_state.mouse_layout.col_resize = None;
            if had_drag {
                reset_pointer();
            }
            InputResult::Continue
        }
        MouseEventKind::Moved => handle_mouse_move(app, event.column, event.row),
        MouseEventKind::ScrollUp => handle_scroll(app, true),
        MouseEventKind::ScrollDown => handle_scroll(app, false),
        _ => InputResult::Continue,
    }
}

/// Handle a left mouse click at terminal coordinates (x, y).
fn handle_left_click(app: &mut App, x: u16, y: u16) -> InputResult {
    // Check for column header click (start column reorder drag)
    if matches!(
        app.mode,
        Mode::Normal | Mode::VisualBlock | Mode::VisualLine | Mode::VisualColumn
    ) {
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        if let Some(col_idx) = resolve_column_header(layout, area, x, y) {
            app.view_state.mouse_layout.col_reorder = Some((col_idx, col_idx));
            app.view_state.selected_column = ColIndex::new(col_idx);
            set_pointer_shape("move");
            return InputResult::Continue;
        }
        // Check for row gutter click (start row reorder drag)
        if let Some(row_idx) = resolve_row_gutter(layout, area, x, y) {
            app.view_state.mouse_layout.row_reorder = Some((row_idx, row_idx));
            app.view_state.table_state.select(Some(row_idx));
            app.view_state.viewport_mode = ViewportMode::Auto;
            set_pointer_shape("move");
            return InputResult::Continue;
        }
    }

    match app.mode {
        Mode::FileList => handle_file_list_click(app, x, y),
        Mode::Normal | Mode::VisualBlock | Mode::VisualLine | Mode::VisualColumn => {
            handle_table_click(app, x, y)
        }
        Mode::Insert => handle_insert_click(app, x, y),
        _ => InputResult::Continue,
    }
}

/// Handle a double-click: navigate to the cell, then enter insert mode.
fn handle_double_click(app: &mut App, x: u16, y: u16) -> InputResult {
    if app.mode != Mode::Normal {
        return InputResult::Continue;
    }

    // First navigate to the clicked cell (reuse single-click logic)
    handle_table_click(app, x, y);

    // Enter insert mode with cursor at end of cell content
    if app.selected_row().is_some() {
        enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
    }

    InputResult::Continue
}

/// Handle a click while in insert mode: move the edit cursor within the cell,
/// or commit and exit insert mode if clicking outside the edited cell.
fn handle_insert_click(app: &mut App, x: u16, y: u16) -> InputResult {
    // Clone layout data to avoid borrow conflicts with &mut app
    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    // Determine which cell was clicked (if any)
    let clicked_cell = resolve_table_cell(&layout, area, x, y);

    let current_row = app.view_state.table_state.selected().unwrap_or(usize::MAX);
    let current_col = app.view_state.selected_column.get();

    match clicked_cell {
        Some((row, col)) if row == current_row && col == current_col => {
            // Clicked inside the currently edited cell — move cursor
            move_insert_cursor(app, &layout, area, x);
        }
        _ => {
            // Clicked outside the edited cell — cancel edit (like Escape), then navigate
            app.formula_completion = None;
            app.edit_buffer = None;
            app.mode = Mode::Normal;
            if let Some((row, col)) = clicked_cell {
                app.view_state.table_state.select(Some(row));
                app.view_state.selected_column = ColIndex::new(col);
                app.view_state.viewport_mode = ViewportMode::Auto;
                if col >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count {
                    app.view_state.column_scroll_offset =
                        col - app.view_state.visible_cols_count + 1;
                } else if col < app.view_state.column_scroll_offset {
                    app.view_state.column_scroll_offset = col;
                }
            }
        }
    }

    InputResult::Continue
}

/// Handle mouse move to detect column resize hover zones and update pointer shape.
fn handle_mouse_move(app: &mut App, x: u16, y: u16) -> InputResult {
    let old_hover = app.view_state.mouse_layout.resize_hover_col;
    app.view_state.mouse_layout.resize_hover_col = detect_resize_border(app, x, y);

    // Update pointer shape based on hover zone
    let on_resize = app.view_state.mouse_layout.resize_hover_col.is_some();
    let was_on_resize = old_hover.is_some();

    if on_resize && !was_on_resize {
        // col-resize: horizontal double-arrow (↔)
        set_pointer_shape("col-resize");
    } else if !on_resize && was_on_resize {
        reset_pointer();
    }

    InputResult::Continue
}

/// Check if mouse position is near a column border in the header row.
/// Returns the display column index (into raw_widths) if hovering on a border.
fn detect_resize_border(app: &App, x: u16, y: u16) -> Option<usize> {
    let layout = &app.view_state.mouse_layout;
    let area = layout.table_content_area;

    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    let rel_x = x - area.x;
    let rel_y = (y - area.y) as usize;

    if rel_y != 0 {
        return None;
    }

    let mut edge_x: u16 = 0;
    for (i, &width) in layout.raw_widths.iter().enumerate() {
        edge_x += width;
        if i > 0 && rel_x + RESIZE_GRAB_ZONE >= edge_x && rel_x <= edge_x + RESIZE_GRAB_ZONE {
            return Some(i);
        }
        edge_x += 1; // column spacing
    }

    None
}

/// Check if a click is on a column border in the header row. If so, start resize.
/// Returns true if a resize was started.
fn try_start_column_resize(app: &mut App, x: u16, y: u16) -> bool {
    let layout = &app.view_state.mouse_layout;
    let area = layout.table_content_area;

    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return false;
    }

    let rel_x = x - area.x;
    let rel_y = (y - area.y) as usize;

    // Only the column letters row (row 0)
    if rel_y != 0 {
        return false;
    }

    // Walk column edges and check if click is near a right border
    // raw_widths[0] = gutter, raw_widths[1..] = data columns
    let mut edge_x: u16 = 0;
    for (i, &width) in layout.raw_widths.iter().enumerate() {
        edge_x += width;
        // Check if click is within RESIZE_GRAB_ZONE of this column's right edge
        if i > 0 && rel_x + RESIZE_GRAB_ZONE >= edge_x && rel_x <= edge_x + RESIZE_GRAB_ZONE {
            // i-1 is the display_cols index for this column
            if let Some(&col_idx) = layout.display_cols.get(i - 1) {
                app.view_state.mouse_layout.col_resize = Some((i, col_idx, x));
                set_pointer_shape("col-resize");
                return true;
            }
        }
        edge_x += 1; // column spacing
    }

    false
}

/// Handle dragging during a column resize operation.
fn handle_column_resize_drag(app: &mut App, x: u16) -> InputResult {
    let (raw_idx, col_idx, start_x) = match app.view_state.mouse_layout.col_resize {
        Some(v) => v,
        None => return InputResult::Continue,
    };

    // Get the current width from raw_widths
    let current_width = app
        .view_state
        .mouse_layout
        .raw_widths
        .get(raw_idx)
        .copied()
        .unwrap_or(8);

    // Compute new width: current + delta from start position
    let delta = x as i32 - start_x as i32;
    let new_width = (current_width as i32 + delta).max(4) as u16;

    // Update the stored start_x for next drag event
    app.view_state.mouse_layout.col_resize = Some((raw_idx, col_idx, x));

    // Apply the width via session (persists per-file)
    app.session.set_column_width(col_idx, new_width);

    InputResult::Continue
}

/// Resolve a click on the row number gutter to a row index, if any.
fn resolve_row_gutter(
    layout: &crate::ui::view_state::MouseLayout,
    area: ratatui::layout::Rect,
    x: u16,
    y: u16,
) -> Option<usize> {
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    let rel_x = x - area.x;
    let rel_y = (y - area.y) as usize;

    // Must be in the gutter column (rel_x < row_num_width)
    if rel_x >= layout.row_num_width {
        return None;
    }

    // Skip column letters row
    if rel_y == 0 {
        return None;
    }

    let frozen_count = layout.frozen_row_indices.len();
    let data_rel_y = rel_y - 1;

    if data_rel_y < frozen_count {
        layout.frozen_row_indices.get(data_rel_y).copied()
    } else {
        let scroll_rel = data_rel_y - frozen_count;
        layout.scrollable_indices.get(scroll_rel).copied()
    }
}

/// Resolve a click on the column letters row to a column index, if any.
fn resolve_column_header(
    layout: &crate::ui::view_state::MouseLayout,
    area: ratatui::layout::Rect,
    x: u16,
    y: u16,
) -> Option<usize> {
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    let rel_x = x - area.x;
    let rel_y = (y - area.y) as usize;

    // Only match the column letters row (row 0 in the table content area)
    if rel_y != 0 {
        return None;
    }

    // Map x to column (same logic as resolve_table_cell)
    let mut cumulative_x: u16 = 0;
    for (i, &width) in layout.raw_widths.iter().enumerate() {
        let col_end = cumulative_x + width;
        if rel_x < col_end {
            if i == 0 {
                return None; // row number gutter
            }
            return layout.display_cols.get(i - 1).copied();
        }
        cumulative_x = col_end + 1;
    }

    None
}

/// Resolve terminal coordinates to a (row_index, col_index) table cell, if any.
fn resolve_table_cell(
    layout: &crate::ui::view_state::MouseLayout,
    area: ratatui::layout::Rect,
    x: u16,
    y: u16,
) -> Option<(usize, usize)> {
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    let rel_x = x - area.x;
    let rel_y = (y - area.y) as usize;

    // Row 0 is the column letters row
    if rel_y == 0 {
        return None;
    }

    let frozen_count = layout.frozen_row_indices.len();
    let data_rel_y = rel_y - 1;

    let target_row = if data_rel_y < frozen_count {
        layout.frozen_row_indices.get(data_rel_y).copied()?
    } else {
        let scroll_rel = data_rel_y - frozen_count;
        layout.scrollable_indices.get(scroll_rel).copied()?
    };

    // Map x to column
    let mut cumulative_x: u16 = 0;
    for (i, &width) in layout.raw_widths.iter().enumerate() {
        let col_end = cumulative_x + width;
        if rel_x < col_end {
            if i == 0 {
                return None; // row number gutter
            }
            let col = *layout.display_cols.get(i - 1)?;
            return Some((target_row, col));
        }
        cumulative_x = col_end + 1;
    }

    None
}

/// Move the insert-mode edit cursor to the clicked position within the current cell.
fn move_insert_cursor(
    app: &mut App,
    layout: &crate::ui::view_state::MouseLayout,
    area: ratatui::layout::Rect,
    x: u16,
) {
    let rel_x = x - area.x;
    let selected_col = app.view_state.selected_column.get();

    // Find the x start and width of the selected column
    let mut col_start_x: u16 = 0;
    let mut col_width: u16 = 0;
    let mut found = false;

    for (i, &width) in layout.raw_widths.iter().enumerate() {
        if i > 0 {
            if let Some(&dc) = layout.display_cols.get(i - 1) {
                if dc == selected_col {
                    col_width = width;
                    found = true;
                    break;
                }
            }
        }
        col_start_x = col_start_x + width + 1;
    }

    if !found {
        return;
    }

    let char_offset = if rel_x >= col_start_x {
        (rel_x - col_start_x) as usize
    } else {
        0
    };

    if let Some(ref mut buf) = app.edit_buffer {
        let content_len = buf.content.chars().count();
        let max_width = col_width.saturating_sub(1) as usize;
        let display_len = content_len + 1; // +1 for the cursor '│'

        if display_len <= max_width {
            // No scrolling — account for the '│' indicator
            let new_cursor = if char_offset <= buf.cursor {
                char_offset
            } else {
                (char_offset - 1).min(content_len)
            };
            buf.cursor = new_cursor;
        } else {
            // Scrolled content — compute the visible window
            let cursor_in_display = buf.cursor;
            let half = max_width / 2;
            let window_start = if cursor_in_display <= half {
                0
            } else if cursor_in_display + half >= display_len {
                display_len.saturating_sub(max_width)
            } else {
                cursor_in_display - half
            };

            let pos_in_display = window_start + char_offset;
            let new_cursor = if pos_in_display <= buf.cursor {
                pos_in_display
            } else {
                (pos_in_display - 1).min(content_len)
            };
            buf.cursor = new_cursor;
        }
    }
}

/// Handle mouse scroll (wheel up/down).
fn handle_scroll(app: &mut App, up: bool) -> InputResult {
    match app.mode {
        Mode::Normal | Mode::VisualBlock | Mode::VisualLine | Mode::VisualColumn => {
            let count = 3;
            if up {
                crate::navigation::commands::move_up_by(app, count);
            } else {
                crate::navigation::commands::move_down_by(app, count);
            }
            InputResult::Continue
        }
        Mode::FileList => {
            // Scroll file list selection
            if up {
                app.view_state.file_list_selected =
                    app.view_state.file_list_selected.saturating_sub(3);
            } else {
                app.view_state.file_list_selected += 3;
                // Clamping will be handled by the file list rendering
            }
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

/// Map a click in the table area to a cell and navigate to it.
fn handle_table_click(app: &mut App, x: u16, y: u16) -> InputResult {
    let layout = &app.view_state.mouse_layout;
    let area = layout.table_content_area;

    let Some((target_row, target_col)) = resolve_table_cell(layout, area, x, y) else {
        return InputResult::Continue;
    };

    // Exit visual mode on a fresh click
    if matches!(
        app.mode,
        Mode::VisualBlock | Mode::VisualLine | Mode::VisualColumn
    ) {
        app.visual_selection = None;
        app.mode = Mode::Normal;
    }

    // Record drag anchor for potential drag selection
    app.view_state.mouse_layout.drag_anchor = Some((target_row, target_col));

    // Navigate to the target cell
    app.view_state.table_state.select(Some(target_row));
    app.view_state.selected_column = ColIndex::new(target_col);
    app.view_state.viewport_mode = ViewportMode::Auto;

    // Adjust horizontal scroll if needed
    if target_col >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count {
        app.view_state.column_scroll_offset = target_col - app.view_state.visible_cols_count + 1;
    } else if target_col < app.view_state.column_scroll_offset {
        app.view_state.column_scroll_offset = target_col;
    }

    InputResult::Continue
}

/// Handle mouse drag to extend a visual block selection.
fn handle_drag(app: &mut App, x: u16, y: u16) -> InputResult {
    // Only handle drag in normal or visual block mode
    if !matches!(app.mode, Mode::Normal | Mode::VisualBlock) {
        return InputResult::Continue;
    }

    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    let Some((target_row, target_col)) = resolve_table_cell(&layout, area, x, y) else {
        return InputResult::Continue;
    };

    let Some((anchor_row, anchor_col)) = layout.drag_anchor else {
        return InputResult::Continue;
    };

    // If still on the anchor cell, don't enter visual mode yet
    if target_row == anchor_row && target_col == anchor_col && app.mode == Mode::Normal {
        return InputResult::Continue;
    }

    // Enter visual block mode if not already in it
    if app.mode == Mode::Normal {
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(anchor_row),
            ColIndex::new(anchor_col),
            VisualMode::Block,
        ));
        app.mode = Mode::VisualBlock;
    }

    // Update the selection cursor to the dragged-to cell
    if let Some(ref mut sel) = app.visual_selection {
        sel.update_cursor(RowIndex::new(target_row), ColIndex::new(target_col));
    }

    // Move the navigation cursor to follow
    app.view_state.table_state.select(Some(target_row));
    app.view_state.selected_column = ColIndex::new(target_col);
    app.view_state.viewport_mode = ViewportMode::Auto;

    // Adjust horizontal scroll if needed
    if target_col >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count {
        app.view_state.column_scroll_offset = target_col - app.view_state.visible_cols_count + 1;
    } else if target_col < app.view_state.column_scroll_offset {
        app.view_state.column_scroll_offset = target_col;
    }

    InputResult::Continue
}

/// Handle a click in the file list mode.
fn handle_file_list_click(app: &mut App, _x: u16, y: u16) -> InputResult {
    // Compute the file list layout to find the current-column area.
    // The file manager uses: modal area -> split vertically (header 1, content, status 1)
    // content -> split horizontally (15% parent, 1 sep, 42% current, 1 sep, 42% preview)
    let frame_area = crossterm::terminal::size()
        .map(|(w, h)| ratatui::layout::Rect::new(0, 0, w, h))
        .unwrap_or_default();
    let modal_area = crate::ui::modal::large_modal_rect(frame_area);

    use ratatui::layout::{Constraint, Direction, Layout};
    let vlayout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(modal_area);
    let content = vlayout[1];

    let hlayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Length(1),
            Constraint::Percentage(42),
            Constraint::Length(1),
            Constraint::Percentage(42),
        ])
        .split(content);
    let current_area = hlayout[2];

    // Check if click is within the current column area
    if y >= current_area.y && y < current_area.y + current_area.height {
        let clicked_idx = (y - current_area.y) as usize;
        app.view_state.file_list_selected = clicked_idx;
    }

    InputResult::Continue
}

/// Handle right-click to open context menu.
fn handle_right_click(app: &mut App, x: u16, y: u16) -> InputResult {
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
fn handle_context_menu_click(app: &mut App, x: u16, y: u16) -> InputResult {
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
pub fn handle_context_menu_key(app: &mut App, key: crossterm::event::KeyEvent) -> InputResult {
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
fn execute_context_action(app: &mut App, item: ContextMenuItem) {
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

/// During a reorder drag, update the drop target for visual feedback.
fn handle_reorder_drag(app: &mut App, x: u16, y: u16) -> InputResult {
    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    if let Some((src, _)) = app.view_state.mouse_layout.col_reorder {
        // Update drop target column
        let target = resolve_column_header(&layout, area, x, y)
            .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(_, c)| c));
        if let Some(col_idx) = target {
            app.view_state.mouse_layout.col_reorder = Some((src, col_idx));
            app.view_state.selected_column = ColIndex::new(col_idx);
        }
    }

    if let Some((src, _)) = app.view_state.mouse_layout.row_reorder {
        // Update drop target row
        let target = resolve_row_gutter(&layout, area, x, y)
            .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(r, _)| r));
        if let Some(row_idx) = target {
            app.view_state.mouse_layout.row_reorder = Some((src, row_idx));
            app.view_state.table_state.select(Some(row_idx));
            app.view_state.viewport_mode = ViewportMode::Auto;
        }
    }

    InputResult::Continue
}

/// Finalize a column reorder when the mouse is released.
fn finalize_column_reorder(app: &mut App, src_col: usize, x: u16, y: u16) -> InputResult {
    use crate::input::StatusMessage;

    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    // Resolve the drop target column from the header or any cell
    let dst_col = resolve_column_header(&layout, area, x, y)
        .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(_, c)| c));

    let Some(dst_col) = dst_col else {
        return InputResult::Continue;
    };

    if src_col == dst_col {
        return InputResult::Continue;
    }

    let from_start = ColIndex::new(src_col);
    let from_end = ColIndex::new(src_col);
    // Insert before dst_col if moving left, after dst_col if moving right
    let to_before = if dst_col > src_col {
        dst_col + 1
    } else {
        dst_col
    };

    let actual_insert = app.document.move_columns(from_start, from_end, to_before);

    app.history.push(crate::history::EditCommand::MoveColumns {
        from_start,
        from_end,
        to_before,
        actual_insert,
    });

    app.view_state.selected_column = ColIndex::new(actual_insert);
    app.status_message = Some(StatusMessage::from(format!(
        "Moved column {} → {}",
        crate::ui::utils::column_to_excel_letter(src_col),
        crate::ui::utils::column_to_excel_letter(actual_insert),
    )));

    InputResult::Continue
}

/// Finalize a row reorder when the mouse is released.
fn finalize_row_reorder(app: &mut App, src_row: usize, x: u16, y: u16) -> InputResult {
    use crate::input::StatusMessage;

    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    // Resolve the drop target row from the gutter or any cell
    let dst_row = resolve_row_gutter(&layout, area, x, y)
        .or_else(|| resolve_table_cell(&layout, area, x, y).map(|(r, _)| r));

    let Some(dst_row) = dst_row else {
        return InputResult::Continue;
    };

    if src_row == dst_row {
        return InputResult::Continue;
    }

    // Move row via sequential swaps
    let from = src_row;
    let to = dst_row;
    if from < to {
        for i in from..to {
            app.document
                .swap_rows(RowIndex::new(i), RowIndex::new(i + 1));
        }
    } else {
        for i in (to..from).rev() {
            app.document
                .swap_rows(RowIndex::new(i), RowIndex::new(i + 1));
        }
    }

    app.history.push(crate::history::EditCommand::MoveRow {
        from: RowIndex::new(from),
        to: RowIndex::new(to),
    });

    app.view_state.table_state.select(Some(dst_row));
    app.view_state.viewport_mode = ViewportMode::Auto;
    app.status_message = Some(StatusMessage::from(format!(
        "Moved row {} → {}",
        from + 1,
        to + 1,
    )));

    InputResult::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use crate::session::FileConfig;
    use crate::ui::view_state::MouseLayout;
    use ratatui::layout::Rect;
    use std::path::PathBuf;

    /// Create a test App with a 3-column, 5-row CSV and a pre-populated MouseLayout.
    /// Layout: table content area at (0,2) 40x8.
    /// Row 0 of content = column letters row.
    /// Gutter width=3, then 3 columns of width 10 each (with 1px spacing between).
    fn create_test_app() -> App {
        let document = Document::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                vec!["a1".into(), "b1".into(), "c1".into()],
                vec!["a2".into(), "b2".into(), "c2".into()],
                vec!["a3".into(), "b3".into(), "c3".into()],
                vec!["a4".into(), "b4".into(), "c4".into()],
                vec!["a5".into(), "b5".into(), "c5".into()],
            ],
            "test.csv".into(),
        );
        let mut app = App::new(
            document,
            vec![PathBuf::from("test.csv")],
            0,
            FileConfig::new(),
        );

        app.view_state.mouse_layout = MouseLayout {
            table_content_area: Rect::new(0, 2, 40, 8),
            display_cols: vec![0, 1, 2],
            raw_widths: vec![3, 10, 10, 10],
            frozen_row_indices: vec![],
            scrollable_indices: vec![0, 1, 2, 3, 4],
            row_num_width: 3,
            file_list_area: Rect::default(),
            last_click: None,
            drag_anchor: None,
            col_resize: None,
            col_reorder: None,
            row_reorder: None,
            resize_hover_col: None,
        };

        app
    }

    // ── resolve_table_cell ─────────────────────────────────────────

    #[test]
    fn test_resolve_cell_first_data_cell() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        // x=4 (past gutter 3 + spacing 1), y=3 (area.y=2, rel_y=1)
        assert_eq!(resolve_table_cell(layout, area, 4, 3), Some((0, 0)));
    }

    #[test]
    fn test_resolve_cell_second_column() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        // After gutter(3)+spacing(1)+col0(10)+spacing(1) = 15
        assert_eq!(resolve_table_cell(layout, area, 15, 3), Some((0, 1)));
    }

    #[test]
    fn test_resolve_cell_gutter_returns_none() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_table_cell(layout, area, 1, 3), None);
    }

    #[test]
    fn test_resolve_cell_header_row_returns_none() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_table_cell(layout, area, 5, 2), None);
    }

    #[test]
    fn test_resolve_cell_outside_area() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_table_cell(layout, area, 5, 0), None);
    }

    #[test]
    fn test_resolve_cell_third_row() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        // rel_y=3 (y=5) → data_rel_y=2 → scrollable_indices[2]=2
        assert_eq!(resolve_table_cell(layout, area, 5, 5), Some((2, 0)));
    }

    #[test]
    fn test_resolve_cell_beyond_last_row() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_table_cell(layout, area, 5, 9), None);
    }

    // ── resolve_column_header ──────────────────────────────────────

    #[test]
    fn test_resolve_header_first_col() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_column_header(layout, area, 5, 2), Some(0));
    }

    #[test]
    fn test_resolve_header_second_col() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_column_header(layout, area, 15, 2), Some(1));
    }

    #[test]
    fn test_resolve_header_non_header_row() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_column_header(layout, area, 5, 3), None);
    }

    #[test]
    fn test_resolve_header_gutter_area() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_column_header(layout, area, 1, 2), None);
    }

    // ── resolve_row_gutter ─────────────────────────────────────────

    #[test]
    fn test_resolve_gutter_first_row() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_row_gutter(layout, area, 1, 3), Some(0));
    }

    #[test]
    fn test_resolve_gutter_third_row() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_row_gutter(layout, area, 1, 5), Some(2));
    }

    #[test]
    fn test_resolve_gutter_outside_width() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_row_gutter(layout, area, 5, 3), None);
    }

    #[test]
    fn test_resolve_gutter_header_row() {
        let app = create_test_app();
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;
        assert_eq!(resolve_row_gutter(layout, area, 1, 2), None);
    }

    // ── detect_resize_border ───────────────────────────────────────

    #[test]
    fn test_resize_border_at_edge() {
        let app = create_test_app();
        // Gutter(3)+col0(10)=13, edge at x=13. x=12 → 12+2>=13 ✓
        assert_eq!(detect_resize_border(&app, 12, 2), Some(1));
    }

    #[test]
    fn test_resize_border_far_from_edge() {
        let app = create_test_app();
        assert_eq!(detect_resize_border(&app, 7, 2), None);
    }

    #[test]
    fn test_resize_border_not_header_row() {
        let app = create_test_app();
        assert_eq!(detect_resize_border(&app, 12, 3), None);
    }

    // ── table click navigation ─────────────────────────────────────

    #[test]
    fn test_table_click_navigates() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(0));
        app.view_state.selected_column = ColIndex::new(0);

        handle_table_click(&mut app, 15, 5);

        assert_eq!(app.view_state.table_state.selected(), Some(2));
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));
    }

    #[test]
    fn test_table_click_clears_visual_mode() {
        let mut app = create_test_app();
        app.mode = Mode::VisualBlock;
        app.visual_selection = Some(VisualSelection::new(
            RowIndex::new(0),
            ColIndex::new(0),
            VisualMode::Block,
        ));

        handle_table_click(&mut app, 5, 3);

        assert_eq!(app.mode, Mode::Normal);
        assert!(app.visual_selection.is_none());
    }

    #[test]
    fn test_table_click_sets_drag_anchor() {
        let mut app = create_test_app();
        handle_table_click(&mut app, 5, 3);
        assert_eq!(app.view_state.mouse_layout.drag_anchor, Some((0, 0)));
    }

    // ── double-click ───────────────────────────────────────────────

    #[test]
    fn test_double_click_enters_insert() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(0));
        handle_double_click(&mut app, 5, 3);
        assert_eq!(app.mode, Mode::Insert);
        assert!(app.edit_buffer.is_some());
    }

    #[test]
    fn test_double_click_ignored_in_command_mode() {
        let mut app = create_test_app();
        app.mode = Mode::Command;
        handle_double_click(&mut app, 5, 3);
        assert_ne!(app.mode, Mode::Insert);
    }

    // ── insert mode click ──────────────────────────────────────────

    #[test]
    fn test_insert_click_outside_cancels() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(0));
        app.view_state.selected_column = ColIndex::new(0);
        enter_insert_mode(&mut app, CursorPosition::End, InitialContent::Keep);

        handle_insert_click(&mut app, 15, 5);

        assert_eq!(app.mode, Mode::Normal);
        assert!(app.edit_buffer.is_none());
        assert_eq!(app.view_state.table_state.selected(), Some(2));
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));
    }

    // ── drag selection ─────────────────────────────────────────────

    #[test]
    fn test_drag_enters_visual_block() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.drag_anchor = Some((0, 0));

        handle_drag(&mut app, 15, 5);

        assert_eq!(app.mode, Mode::VisualBlock);
        let sel = app.visual_selection.unwrap();
        assert_eq!(sel.anchor, (RowIndex::new(0), ColIndex::new(0)));
        assert_eq!(sel.cursor, (RowIndex::new(2), ColIndex::new(1)));
    }

    #[test]
    fn test_drag_same_cell_no_visual() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.drag_anchor = Some((0, 0));

        handle_drag(&mut app, 5, 3);

        assert_eq!(app.mode, Mode::Normal);
        assert!(app.visual_selection.is_none());
    }

    #[test]
    fn test_drag_extends_selection() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.drag_anchor = Some((0, 0));

        handle_drag(&mut app, 15, 4);
        handle_drag(&mut app, 25, 6);

        let sel = app.visual_selection.unwrap();
        assert_eq!(sel.cursor, (RowIndex::new(3), ColIndex::new(2)));
    }

    // ── scroll ─────────────────────────────────────────────────────

    #[test]
    fn test_scroll_down() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(0));
        handle_scroll(&mut app, false);
        assert_eq!(app.view_state.table_state.selected(), Some(3));
    }

    #[test]
    fn test_scroll_up() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(4));
        handle_scroll(&mut app, true);
        assert_eq!(app.view_state.table_state.selected(), Some(1));
    }

    #[test]
    fn test_scroll_up_clamps() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(1));
        handle_scroll(&mut app, true);
        assert_eq!(app.view_state.table_state.selected(), Some(0));
    }

    // ── right-click context menus ──────────────────────────────────

    #[test]
    fn test_right_click_cell_opens_cell_menu() {
        let mut app = create_test_app();
        handle_right_click(&mut app, 5, 3);
        let menu = app.context_menu.as_ref().unwrap();
        assert!(menu.items.contains(&ContextMenuItem::Cut));
        assert!(menu.items.contains(&ContextMenuItem::Copy));
        assert!(menu.items.contains(&ContextMenuItem::Paste));
        assert!(menu.items.contains(&ContextMenuItem::Clear));
    }

    #[test]
    fn test_right_click_header_opens_column_menu() {
        let mut app = create_test_app();
        handle_right_click(&mut app, 5, 2);
        let menu = app.context_menu.as_ref().unwrap();
        assert!(menu.items.contains(&ContextMenuItem::ColumnInsertBefore));
        assert!(menu.items.contains(&ContextMenuItem::ColumnInsertAfter));
        assert!(menu.items.contains(&ContextMenuItem::ColumnDelete));
        assert!(!menu.items.contains(&ContextMenuItem::Cut));
    }

    #[test]
    fn test_right_click_gutter_opens_row_menu() {
        let mut app = create_test_app();
        handle_right_click(&mut app, 1, 3);
        let menu = app.context_menu.as_ref().unwrap();
        assert!(menu.items.contains(&ContextMenuItem::RowInsertAbove));
        assert!(menu.items.contains(&ContextMenuItem::RowInsertBelow));
        assert!(menu.items.contains(&ContextMenuItem::RowDelete));
        assert!(!menu.items.contains(&ContextMenuItem::Cut));
    }

    #[test]
    fn test_right_click_navigates_to_cell() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(0));
        app.view_state.selected_column = ColIndex::new(0);
        handle_right_click(&mut app, 15, 5);
        assert_eq!(app.view_state.table_state.selected(), Some(2));
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));
    }

    #[test]
    fn test_right_click_outside_no_menu() {
        let mut app = create_test_app();
        handle_right_click(&mut app, 5, 0);
        assert!(app.context_menu.is_none());
    }

    // ── context menu keyboard ──────────────────────────────────────

    #[test]
    fn test_context_menu_down() {
        let mut app = create_test_app();
        app.context_menu = Some(ContextMenu {
            x: 0,
            y: 0,
            selected: 0,
            items: vec![
                ContextMenuItem::Cut,
                ContextMenuItem::Copy,
                ContextMenuItem::Paste,
            ],
        });
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        handle_context_menu_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.context_menu.as_ref().unwrap().selected, 1);
    }

    #[test]
    fn test_context_menu_skips_separator() {
        let mut app = create_test_app();
        app.context_menu = Some(ContextMenu {
            x: 0,
            y: 0,
            selected: 0,
            items: vec![
                ContextMenuItem::Copy,
                ContextMenuItem::Separator,
                ContextMenuItem::Clear,
            ],
        });
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        handle_context_menu_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.context_menu.as_ref().unwrap().selected, 2);
    }

    #[test]
    fn test_context_menu_escape() {
        let mut app = create_test_app();
        app.context_menu = Some(ContextMenu {
            x: 0,
            y: 0,
            selected: 0,
            items: vec![ContextMenuItem::Copy],
        });
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        handle_context_menu_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.context_menu.is_none());
    }

    // ── context menu actions ───────────────────────────────────────

    #[test]
    fn test_action_copy_cell() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(0);
        execute_context_action(&mut app, ContextMenuItem::Copy);
        assert_eq!(app.clipboard.cell().unwrap(), "a1");
    }

    #[test]
    fn test_action_cut_cell() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(0);
        execute_context_action(&mut app, ContextMenuItem::Cut);
        assert_eq!(app.clipboard.cell().unwrap(), "a1");
        assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "");
    }

    #[test]
    fn test_action_clear_cell() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(0);
        execute_context_action(&mut app, ContextMenuItem::Clear);
        assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "");
    }

    #[test]
    fn test_action_paste_cell() {
        let mut app = create_test_app();
        app.clipboard.yank_cell("pasted".into());
        app.view_state.table_state.select(Some(2));
        app.view_state.selected_column = ColIndex::new(0);
        execute_context_action(&mut app, ContextMenuItem::Paste);
        assert_eq!(
            app.document.cell(RowIndex::new(2), ColIndex::new(0)),
            "pasted"
        );
    }

    #[test]
    fn test_action_column_delete() {
        let mut app = create_test_app();
        let orig = app.document.column_count();
        app.view_state.selected_column = ColIndex::new(1);
        execute_context_action(&mut app, ContextMenuItem::ColumnDelete);
        assert_eq!(app.document.column_count(), orig - 1);
    }

    #[test]
    fn test_action_column_insert_before() {
        let mut app = create_test_app();
        let orig = app.document.column_count();
        app.view_state.selected_column = ColIndex::new(1);
        execute_context_action(&mut app, ContextMenuItem::ColumnInsertBefore);
        assert_eq!(app.document.column_count(), orig + 1);
    }

    #[test]
    fn test_action_column_insert_after() {
        let mut app = create_test_app();
        let orig = app.document.column_count();
        app.view_state.selected_column = ColIndex::new(1);
        execute_context_action(&mut app, ContextMenuItem::ColumnInsertAfter);
        assert_eq!(app.document.column_count(), orig + 1);
    }

    #[test]
    fn test_action_row_delete() {
        let mut app = create_test_app();
        let orig = app.document.row_count();
        app.view_state.table_state.select(Some(1));
        execute_context_action(&mut app, ContextMenuItem::RowDelete);
        assert_eq!(app.document.row_count(), orig - 1);
    }

    #[test]
    fn test_action_row_insert_below() {
        let mut app = create_test_app();
        let orig = app.document.row_count();
        app.view_state.table_state.select(Some(1));
        execute_context_action(&mut app, ContextMenuItem::RowInsertBelow);
        assert_eq!(app.document.row_count(), orig + 1);
        assert_eq!(app.view_state.table_state.selected(), Some(2));
    }

    #[test]
    fn test_action_row_insert_above() {
        let mut app = create_test_app();
        let orig = app.document.row_count();
        app.view_state.table_state.select(Some(2));
        execute_context_action(&mut app, ContextMenuItem::RowInsertAbove);
        assert_eq!(app.document.row_count(), orig + 1);
    }

    // ── column resize ──────────────────────────────────────────────

    #[test]
    fn test_resize_drag_increases_width() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.col_resize = Some((1, 0, 13));
        handle_column_resize_drag(&mut app, 18);
        assert_eq!(app.session.column_width(0).unwrap(), 15);
    }

    #[test]
    fn test_resize_drag_minimum_width() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.col_resize = Some((1, 0, 13));
        handle_column_resize_drag(&mut app, 0);
        assert_eq!(app.session.column_width(0).unwrap(), 4);
    }

    // ── column reorder ─────────────────────────────────────────────

    #[test]
    fn test_column_reorder() {
        let mut app = create_test_app();
        assert_eq!(app.document.header(ColIndex::new(0)), "A");
        assert_eq!(app.document.header(ColIndex::new(1)), "B");
        assert_eq!(app.document.header(ColIndex::new(2)), "C");

        // Move column A (0) to after column C (2)
        finalize_column_reorder(&mut app, 0, 26, 2);

        assert_eq!(app.document.header(ColIndex::new(0)), "B");
        assert_eq!(app.document.header(ColIndex::new(1)), "C");
        assert_eq!(app.document.header(ColIndex::new(2)), "A");
    }

    #[test]
    fn test_column_reorder_same_noop() {
        let mut app = create_test_app();
        finalize_column_reorder(&mut app, 0, 5, 2);
        assert_eq!(app.document.header(ColIndex::new(0)), "A");
    }

    // ── row reorder ────────────────────────────────────────────────

    #[test]
    fn test_row_reorder_down() {
        let mut app = create_test_app();
        // Move row 1 (a1) to row 3 position
        finalize_row_reorder(&mut app, 1, 1, 6);
        assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "a2");
        assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "a3");
        assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "a1");
    }

    #[test]
    fn test_row_reorder_up() {
        let mut app = create_test_app();
        // Move row 3 (a3) to row 1 position
        finalize_row_reorder(&mut app, 3, 1, 4);
        assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "a3");
        assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "a1");
        assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "a2");
    }

    #[test]
    fn test_row_reorder_same_noop() {
        let mut app = create_test_app();
        finalize_row_reorder(&mut app, 1, 1, 4);
        assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "a1");
    }

    // ── frozen rows ────────────────────────────────────────────────

    #[test]
    fn test_resolve_with_frozen_rows() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.frozen_row_indices = vec![0];
        app.view_state.mouse_layout.scrollable_indices = vec![1, 2, 3, 4];
        let layout = &app.view_state.mouse_layout;
        let area = layout.table_content_area;

        // rel_y=1 → frozen row 0
        assert_eq!(resolve_table_cell(layout, area, 5, 3), Some((0, 0)));
        // rel_y=2 → scrollable[0] = row 1
        assert_eq!(resolve_table_cell(layout, area, 5, 4), Some((1, 0)));
    }

    // ── reorder drag visual state ──────────────────────────────────

    #[test]
    fn test_col_reorder_drag_updates_target() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.col_reorder = Some((0, 0));

        handle_reorder_drag(&mut app, 26, 2);

        let (src, dst) = app.view_state.mouse_layout.col_reorder.unwrap();
        assert_eq!(src, 0);
        assert_eq!(dst, 2);
    }

    #[test]
    fn test_row_reorder_drag_updates_target() {
        let mut app = create_test_app();
        app.view_state.mouse_layout.row_reorder = Some((0, 0));

        handle_reorder_drag(&mut app, 1, 6);

        let (src, dst) = app.view_state.mouse_layout.row_reorder.unwrap();
        assert_eq!(src, 0);
        assert_eq!(dst, 3);
    }
}
