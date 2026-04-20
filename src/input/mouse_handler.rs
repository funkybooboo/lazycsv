//! Mouse event handling for the TUI.
//!
//! Maps terminal mouse coordinates to logical cells, file list items,
//! and scroll actions using layout information captured during rendering.

use crate::app::{App, Mode, VisualMode, VisualSelection};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::handler::{enter_insert_mode, CursorPosition, InitialContent};
use crate::input::mouse_context_menu;
use crate::input::mouse_coords::{
    move_insert_cursor, resolve_column_header, resolve_row_gutter, resolve_table_cell,
};
use crate::input::mouse_reorder;
use crate::input::InputResult;
use crate::ui::ViewportMode;
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
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
/// Returns (InputResult, needs_redraw).
pub fn handle_mouse(app: &mut App, event: MouseEvent) -> (InputResult, bool) {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let (x, y) = (event.column, event.row);

            // If context menu is open, handle click on it or dismiss
            if app.context_menu.is_some() {
                return (
                    mouse_context_menu::handle_context_menu_click(app, x, y),
                    true,
                );
            }

            // Check for column resize grab on header row border
            if try_start_column_resize(app, x, y) {
                return (InputResult::Continue, false);
            }

            // Multi-click detection: increment count if same spot within threshold.
            let click_count = match app.view_state.mouse_layout.last_click {
                Some((t, px, py, c))
                    if px == x && py == y && t.elapsed().as_millis() < DOUBLE_CLICK_MS =>
                {
                    c + 1
                }
                _ => 1,
            };
            app.view_state.mouse_layout.last_click = Some((Instant::now(), x, y, click_count));

            let result = match click_count {
                1 => handle_left_click(app, x, y),
                2 => handle_double_click(app, x, y),
                _ => handle_triple_click(app, x, y),
            };
            (result, true)
        }
        MouseEventKind::Down(MouseButton::Right) => (
            mouse_context_menu::handle_right_click(app, event.column, event.row),
            true,
        ),
        MouseEventKind::Drag(MouseButton::Left) => {
            // If a column resize is active, handle it
            if app.view_state.mouse_layout.col_resize.is_some() {
                return (handle_column_resize_drag(app, event.column), true);
            }
            // Column or row reorder drags
            if app.view_state.mouse_layout.col_reorder.is_some()
                || app.view_state.mouse_layout.row_reorder.is_some()
            {
                return (
                    mouse_reorder::handle_reorder_drag(app, event.column, event.row),
                    true,
                );
            }
            // Selection drag — only redraw when selected cell actually changes
            let prev_row = app.view_state.table_state.selected();
            let prev_col = app.view_state.selected_column;
            let result = handle_drag(app, event.column, event.row);
            let changed = app.view_state.table_state.selected() != prev_row
                || app.view_state.selected_column != prev_col;
            (result, changed)
        }
        MouseEventKind::Up(MouseButton::Left) => {
            let had_drag = app.view_state.mouse_layout.col_reorder.is_some()
                || app.view_state.mouse_layout.row_reorder.is_some()
                || app.view_state.mouse_layout.col_resize.is_some();

            if let Some((src_col, _)) = app.view_state.mouse_layout.col_reorder.take() {
                reset_pointer();
                return (
                    mouse_reorder::finalize_column_reorder(app, src_col, event.column, event.row),
                    true,
                );
            }
            if let Some((src_row, _)) = app.view_state.mouse_layout.row_reorder.take() {
                reset_pointer();
                return (
                    mouse_reorder::finalize_row_reorder(app, src_row, event.column, event.row),
                    true,
                );
            }
            app.view_state.mouse_layout.col_resize = None;
            app.view_state.mouse_layout.last_edge_scroll = None;
            if had_drag {
                reset_pointer();
            }
            (InputResult::Continue, had_drag)
        }
        MouseEventKind::Moved => (handle_mouse_move(app, event.column, event.row), false),
        MouseEventKind::ScrollUp => {
            // Shift+ScrollUp = scroll left (for terminals that don't send ScrollLeft)
            if event.modifiers.contains(KeyModifiers::SHIFT) {
                (handle_horizontal_scroll(app, true), true)
            } else {
                (handle_scroll(app, true), true)
            }
        }
        MouseEventKind::ScrollDown => {
            if event.modifiers.contains(KeyModifiers::SHIFT) {
                (handle_horizontal_scroll(app, false), true)
            } else {
                (handle_scroll(app, false), true)
            }
        }
        MouseEventKind::ScrollLeft => (handle_horizontal_scroll(app, true), true),
        MouseEventKind::ScrollRight => (handle_horizontal_scroll(app, false), true),
        _ => (InputResult::Continue, false),
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
        Mode::SqlEditor => {
            sql_editor_move_cursor(app, x, y);
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

/// Handle a double-click: navigate to the cell, then enter insert mode.
fn handle_double_click(app: &mut App, x: u16, y: u16) -> InputResult {
    // File browser: double-click activates the row (opens file / navigates into dir / `..` goes up).
    if app.mode == Mode::FileList {
        handle_file_list_click(app, x, y);
        return crate::input::file_list_mode::navigate_into_selected(app)
            .unwrap_or(InputResult::Continue);
    }

    // SQL editor: double-click selects the word under the cursor.
    if app.mode == Mode::SqlEditor {
        sql_editor_move_cursor(app, x, y);
        if let Some(ed) = app.sql_vim_editor.as_mut() {
            ed.select_word_at_cursor();
        }
        return InputResult::Continue;
    }

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

/// Handle a triple-click. Currently used only by the SQL editor to select the current line.
fn handle_triple_click(app: &mut App, x: u16, y: u16) -> InputResult {
    if app.mode == Mode::SqlEditor {
        sql_editor_move_cursor(app, x, y);
        if let Some(ed) = app.sql_vim_editor.as_mut() {
            ed.select_current_line();
        }
    }
    InputResult::Continue
}

/// Map a click inside the SQL editor modal to a (line, col) position
/// and move the vim editor's cursor there. No-op if the click is outside the
/// editor text area.
fn sql_editor_move_cursor(app: &mut App, x: u16, y: u16) {
    let Some(ed) = app.sql_vim_editor.as_mut() else {
        return;
    };

    let frame_area = crossterm::terminal::size()
        .map(|(w, h)| ratatui::layout::Rect::new(0, 0, w, h))
        .unwrap_or_default();
    let modal = crate::ui::modal::large_modal_rect(frame_area);
    // `standard_block` uses Borders::ALL → 1-char inset on each side.
    if modal.width < 2 || modal.height < 3 {
        return;
    }
    let inner_x = modal.x + 1;
    let inner_y = modal.y + 1;
    let inner_height = modal.height - 2;
    // split_editor_area reserves 1 line for the status bar at the bottom.
    let query_height = inner_height.saturating_sub(1);

    if x < inner_x || y < inner_y || y >= inner_y + query_height {
        return;
    }

    let line_num_width = format!("{}", ed.line_count()).len() as u16 + 1; // +1 trailing space
    let content_x = inner_x + line_num_width;
    if x < content_x {
        // Clicked in the line-number gutter → place cursor at column 0 of that line.
        let line = (y - inner_y) as usize;
        ed.set_cursor(line, 0);
        return;
    }

    let line = (y - inner_y) as usize;
    let col = (x - content_x) as usize;
    ed.set_cursor(line, col);
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
/// Returns the display column index (1-based into col_positions) if hovering on a border.
fn detect_resize_border(app: &App, x: u16, y: u16) -> Option<usize> {
    let layout = &app.view_state.mouse_layout;
    let area = layout.table_content_area;

    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }

    if (y - area.y) != 0 {
        return None;
    }

    // Check proximity to each column right edge using col_positions.
    let positions = &layout.col_positions;
    for i in 1..positions.len().saturating_sub(1) {
        let right_edge = positions[i + 1];
        if x + RESIZE_GRAB_ZONE >= right_edge && x <= right_edge + RESIZE_GRAB_ZONE {
            return Some(i);
        }
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

/// Handle horizontal mouse scroll (trackpad two-finger left/right).
fn handle_horizontal_scroll(app: &mut App, left: bool) -> InputResult {
    match app.mode {
        Mode::Normal | Mode::VisualBlock | Mode::VisualLine | Mode::VisualColumn => {
            let count = 3;
            if left {
                crate::navigation::commands::move_left_by(app, count);
            } else {
                crate::navigation::commands::move_right_by(app, count);
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

    // Navigate to the target cell, preserving the current scroll position.
    // The clicked cell is already visible on screen (resolve_table_cell succeeded),
    // so lock the viewport to the current scroll offset to prevent recentering.
    let scroll_offset = app.view_state.mouse_layout.last_scroll_offset;
    app.view_state.table_state.select(Some(target_row));
    app.view_state.selected_column = ColIndex::new(target_col);
    app.view_state.viewport_mode = ViewportMode::Fixed(scroll_offset);

    InputResult::Continue
}

/// Handle mouse drag to extend a visual block selection.
/// Supports auto-scrolling when dragging past the visible edges.
fn handle_drag(app: &mut App, x: u16, y: u16) -> InputResult {
    // Only handle drag in normal or visual block mode
    if !matches!(app.mode, Mode::Normal | Mode::VisualBlock) {
        return InputResult::Continue;
    }

    let layout = app.view_state.mouse_layout.clone();
    let area = layout.table_content_area;

    // Try to resolve the cell under the cursor, or auto-scroll if past edges
    let (target_row, target_col) = if let Some(cell) = resolve_table_cell(&layout, area, x, y) {
        cell
    } else if layout.drag_anchor.is_some() {
        // Mouse is outside the table area — throttle edge scrolling to ~20 rows/sec
        let now = Instant::now();
        let elapsed = app
            .view_state
            .mouse_layout
            .last_edge_scroll
            .map(|last| now.duration_since(last).as_millis());
        if let Some(ms) = elapsed {
            if ms < 50 {
                return InputResult::Continue;
            }
        }
        app.view_state.mouse_layout.last_edge_scroll = Some(now);

        let row_count = app.document.row_count();
        let col_count = app.document.column_count();
        let current_row = app.view_state.table_state.selected().unwrap_or(0);
        let current_col = app.view_state.selected_column.get();

        // Vertical: above header row → scroll up, below table → scroll down
        let target_row = if y <= area.y + 1 {
            current_row.saturating_sub(1)
        } else if y >= area.y + area.height {
            (current_row + 1).min(row_count.saturating_sub(1))
        } else {
            current_row
        };

        // Horizontal: left of data area → scroll left, right of table → scroll right
        let gutter_end = layout.col_positions.get(1).copied().unwrap_or(0);
        let target_col = if x < gutter_end {
            current_col.saturating_sub(1)
        } else if x >= area.x + area.width {
            (current_col + 1).min(col_count.saturating_sub(1))
        } else {
            current_col
        };

        (target_row, target_col)
    } else {
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

    // Move the navigation cursor to follow.
    // Use Fixed viewport to prevent Auto from recentering the view.
    // Only adjust scroll by 1 row when the target is outside the current viewport.
    let scroll_offset = app.view_state.mouse_layout.last_scroll_offset;
    let scrollable_height = (area.height as usize)
        .saturating_sub(1) // column letters row
        .saturating_sub(layout.frozen_row_indices.len());

    let new_scroll = if target_row >= scroll_offset + scrollable_height {
        // Target is below the visible area — scroll down by 1
        scroll_offset + 1
    } else if target_row < scroll_offset {
        // Target is above the visible area — scroll up by 1
        scroll_offset.saturating_sub(1)
    } else {
        scroll_offset
    };

    app.view_state.table_state.select(Some(target_row));
    app.view_state.selected_column = ColIndex::new(target_col);
    app.view_state.viewport_mode = ViewportMode::Fixed(new_scroll);

    // Adjust horizontal scroll if needed
    if target_col >= app.view_state.column_scroll_offset + app.view_state.visible_cols_count {
        app.view_state.column_scroll_offset = target_col - app.view_state.visible_cols_count + 1;
    } else if target_col < app.view_state.column_scroll_offset {
        app.view_state.column_scroll_offset = target_col;
    }

    InputResult::Continue
}

/// Handle a click in the file list mode.
/// Uses areas and scroll offsets stashed by the file manager renderer.
fn handle_file_list_click(app: &mut App, x: u16, y: u16) -> InputResult {
    let layout = &app.view_state.mouse_layout;
    let current = layout.file_list_area;
    let parent = layout.file_list_parent_area;

    // Click in the current-directory column → move selection to that row.
    if point_in_rect(x, y, current) {
        let row = (y - current.y) as usize;
        let clicked_idx = layout.file_list_offset + row;
        app.view_state.file_list_selected = clicked_idx;
        return InputResult::Continue;
    }

    // Click in the parent column → navigate to that sibling directory.
    if point_in_rect(x, y, parent) {
        let row = (y - parent.y) as usize;
        let clicked_idx = layout.file_list_parent_offset + row;
        crate::input::file_list_mode::navigate_to_parent_column_index(app, clicked_idx);
        return InputResult::Continue;
    }

    InputResult::Continue
}

fn point_in_rect(x: u16, y: u16, r: ratatui::layout::Rect) -> bool {
    r.width > 0 && r.height > 0 && x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{ContextMenu, ContextMenuItem};
    use crate::csv::Document;
    use crate::input::handler::{enter_insert_mode, CursorPosition, InitialContent};
    use crate::input::mouse_context_menu::*;
    use crate::input::mouse_reorder::*;
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

        // col_positions: gutter(0), col0(4), col1(15), col2(26), trailing(36)
        app.view_state.mouse_layout = MouseLayout {
            table_content_area: Rect::new(0, 2, 40, 8),
            display_cols: vec![0, 1, 2],
            raw_widths: vec![3, 10, 10, 10],
            col_positions: vec![0, 4, 15, 26, 36],
            frozen_row_indices: vec![],
            scrollable_indices: vec![0, 1, 2, 3, 4],
            row_num_width: 3,
            file_list_area: Rect::default(),
            file_list_parent_area: Rect::default(),
            file_list_offset: 0,
            file_list_parent_offset: 0,
            last_click: None,
            drag_anchor: None,
            col_resize: None,
            col_reorder: None,
            row_reorder: None,
            resize_hover_col: None,
            last_scroll_offset: 0,
            last_edge_scroll: None,
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
        // col_positions: [0, 4, 15, 26, 36]. Right edge of col0 is positions[2]=15.
        // x=14 → 14+2>=15 ✓
        assert_eq!(detect_resize_border(&app, 14, 2), Some(1));
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
        handle_drag(&mut app, 26, 6); // col2 starts at x=26

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
        mouse_reorder::finalize_column_reorder(&mut app, 0, 26, 2);

        assert_eq!(app.document.header(ColIndex::new(0)), "B");
        assert_eq!(app.document.header(ColIndex::new(1)), "C");
        assert_eq!(app.document.header(ColIndex::new(2)), "A");
    }

    #[test]
    fn test_column_reorder_same_noop() {
        let mut app = create_test_app();
        mouse_reorder::finalize_column_reorder(&mut app, 0, 5, 2);
        assert_eq!(app.document.header(ColIndex::new(0)), "A");
    }

    // ── row reorder ────────────────────────────────────────────────

    #[test]
    fn test_row_reorder_down() {
        let mut app = create_test_app();
        // Move row 1 (a1) to row 3 position
        mouse_reorder::finalize_row_reorder(&mut app, 1, 1, 6);
        assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "a2");
        assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "a3");
        assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "a1");
    }

    #[test]
    fn test_row_reorder_up() {
        let mut app = create_test_app();
        // Move row 3 (a3) to row 1 position
        mouse_reorder::finalize_row_reorder(&mut app, 3, 1, 4);
        assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "a3");
        assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "a1");
        assert_eq!(app.document.cell(RowIndex::new(3), ColIndex::new(0)), "a2");
    }

    #[test]
    fn test_row_reorder_same_noop() {
        let mut app = create_test_app();
        mouse_reorder::finalize_row_reorder(&mut app, 1, 1, 4);
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
