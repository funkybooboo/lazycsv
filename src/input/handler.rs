//! Input handling and keyboard event processing

use crate::app::{messages, App, EditBuffer, Mode};
use crate::domain::position::{ColIndex, RowIndex};
use crate::navigation;
use crate::ui::ViewportMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::num::NonZeroUsize;

use super::{InputResult, PendingCommand, StatusMessage};

/// Timeout for multi-key commands (no longer used in handler, but still exported for state)
pub const MULTI_KEY_TIMEOUT_MS: u128 = 1000;

/// Maximum command count to prevent overflow
pub const MAX_COMMAND_COUNT: usize = 100000;

/// Format a KeyCode in a user-friendly way (not Rust debug format)
fn format_keycode(code: &KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        other => format!("{:?}", other),
    }
}

/// Format a PendingCommand in a user-friendly way
fn format_pending_command(cmd: &PendingCommand) -> String {
    match cmd {
        PendingCommand::G => "g".to_string(),
        PendingCommand::Z => "z".to_string(),
        PendingCommand::GotoColumn(letters) => format!("g{}", letters),
        PendingCommand::D => "d".to_string(),
        PendingCommand::Y => "y".to_string(),
        PendingCommand::C => "c".to_string(),
        PendingCommand::Comma => ",".to_string(),
        PendingCommand::CommaD => ",d".to_string(),
        PendingCommand::CommaY => ",y".to_string(),
    }
}

/// Handle keyboard input events
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Command => handle_command_mode(app, key),
        Mode::Insert => handle_insert_mode(app, key),
        Mode::FileList => handle_file_list_mode(app, key),
        Mode::SqlEditor => handle_sql_editor_mode(app, key),
        Mode::Magnifier => handle_magnifier_mode(app, key),
        // TODO: Implement handlers for new modes in future versions
        Mode::HeaderEdit | Mode::Visual => {
            // For now, Esc returns to Normal mode
            if key.code == KeyCode::Esc {
                app.mode = Mode::Normal;
                app.edit_buffer = None;
            }
            Ok(InputResult::Continue)
        }
    }
}

/// Returns true if navigation commands are allowed (help overlay is closed)
fn is_navigation_allowed(app: &App) -> bool {
    !app.view_state.help_overlay_visible
}

/// Toggle help overlay visibility
fn handle_help_toggle(app: &mut App) {
    app.view_state.help_overlay_visible = !app.view_state.help_overlay_visible;
}

/// Handle file switching between next and previous files
fn handle_file_switch(app: &mut App, next: bool) -> InputResult {
    if !app.session.has_multiple_files() {
        return InputResult::Continue;
    }

    let switched = if next {
        app.session.next_file()
    } else {
        app.session.prev_file()
    };

    if switched {
        InputResult::ReloadFile
    } else {
        InputResult::Continue
    }
}

/// Enter Insert mode for cell editing
///
/// # Arguments
/// * `cursor_at_start` - If true, cursor is at start of content; otherwise at end
/// * `clear_content` - If true, clear the cell content (for 's' command)
fn enter_insert_mode(app: &mut App, cursor_at_start: bool, clear_content: bool) {
    let row_idx = app.get_selected_row().unwrap_or(RowIndex::new(0));
    let col_idx = app.view_state.selected_column;

    let current_value = app.document.get_cell(row_idx, col_idx).to_string();

    let (content, cursor) = if clear_content {
        (String::new(), 0)
    } else if cursor_at_start {
        (current_value.clone(), 0)
    } else {
        // Use character count, not byte length, for cursor position
        let char_count = current_value.chars().count();
        (current_value.clone(), char_count)
    };

    app.edit_buffer = Some(EditBuffer {
        content,
        cursor,
        original: current_value,
    });
    app.mode = Mode::Insert;
}

/// Commit the current edit and return to Normal mode
fn commit_edit(app: &mut App) {
    if let Some(buffer) = app.edit_buffer.take() {
        if let Some(row_idx) = app.get_selected_row() {
            let col_idx = app.view_state.selected_column;

            // Only mark dirty if content changed
            if buffer.content != buffer.original {
                app.document.set_cell(row_idx, col_idx, buffer.content);
                app.last_edit_position = Some((row_idx, col_idx));
            }
        }
    }
    app.mode = Mode::Normal;
}

/// Handle keyboard input in Normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Clear transient messages on keypress
    if let Some(ref msg) = app.status_message {
        if msg.should_clear_on_keypress() {
            app.status_message = None;
        }
    }

    // Note: No timeout on pending commands (vim-like behavior - wait indefinitely)

    // Handle pending multi-key sequences
    if let Some(pending) = app.input_state.pending_command.clone() {
        return handle_multi_key_command(app, pending, key.code);
    }

    // Handle numeric prefixes only when navigation is allowed
    if is_navigation_allowed(app) {
        if let KeyCode::Char(c) = key.code {
            if c.is_numeric() && (c != '0' || app.input_state.command_count.is_some()) {
                return handle_count_prefix(app, c);
            }
        }
    }

    match key.code {
        // Open SQL query editor
        KeyCode::Char('q') if is_navigation_allowed(app) => {
            app.sql_cursor = app.sql_buffer.chars().count();
            app.mode = Mode::SqlEditor;
            return Ok(InputResult::Continue);
        }

        // Toggle help overlay
        KeyCode::Char('?') => {
            handle_help_toggle(app);
        }

        // Close help overlay with Esc
        KeyCode::Esc if app.view_state.help_overlay_visible => {
            app.view_state.hide_help();
        }

        // Help overlay scrolling: j/k for line, Ctrl+d/u for page
        KeyCode::Char('j') | KeyCode::Down if app.view_state.help_overlay_visible => {
            // Use HELP_CONTENT_LINES (52) as safe max scroll
            app.view_state.scroll_help_down(52);
        }

        KeyCode::Char('k') | KeyCode::Up if app.view_state.help_overlay_visible => {
            app.view_state.scroll_help_up();
        }

        KeyCode::Char('d')
            if app.view_state.help_overlay_visible
                && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            // Page down (10 lines)
            app.view_state.scroll_help_page_down(10, 52);
        }

        KeyCode::Char('u')
            if app.view_state.help_overlay_visible
                && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            // Page up (10 lines)
            app.view_state.scroll_help_page_up(10);
        }

        // Clear pending command with Esc
        KeyCode::Esc if app.input_state.pending_command.is_some() => {
            app.input_state.clear_pending_command();
            app.status_message = Some(StatusMessage::from(messages::CMD_CANCELLED));
        }

        // File switching
        KeyCode::Char('[') if is_navigation_allowed(app) => {
            return Ok(handle_file_switch(app, false));
        }

        KeyCode::Char(']') if is_navigation_allowed(app) => {
            return Ok(handle_file_switch(app, true));
        }

        // Start multi-key sequences
        KeyCode::Char('g') if is_navigation_allowed(app) => {
            // Check if we have a count prefix (e.g., 5 from 5g)
            if let Some(count) = app.input_state.command_count.take() {
                // Row jump: 5g → jump to row 5
                navigation::commands::goto_line(app, count.get());
                return Ok(InputResult::Continue);
            }

            // No count - start gg sequence
            app.input_state.set_pending_command(PendingCommand::G);
            return Ok(InputResult::Continue);
        }

        KeyCode::Char('z') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Z);
            return Ok(InputResult::Continue);
        }

        // Enter command mode
        KeyCode::Char(':') if is_navigation_allowed(app) => {
            app.mode = Mode::Command;
            app.input_state.clear_command_buffer();
            return Ok(InputResult::Continue);
        }

        // Start 'd' pending command (for dd - delete row)
        KeyCode::Char('d') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::D);
            return Ok(InputResult::Continue);
        }

        // Start 'y' pending command (for yy - yank row)
        KeyCode::Char('y') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Y);
            return Ok(InputResult::Continue);
        }

        // Start 'c' pending command (for cc - clear row)
        KeyCode::Char('c') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::C);
            return Ok(InputResult::Continue);
        }

        // Insert mode: 'i' - edit cell, cursor at end
        KeyCode::Char('i') if is_navigation_allowed(app) => {
            enter_insert_mode(app, false, false);
        }

        // Insert mode: 'a' - edit cell, cursor at end (same as 'i' for cells)
        KeyCode::Char('a') if is_navigation_allowed(app) => {
            enter_insert_mode(app, false, false);
        }

        // Insert mode: 'I' - edit cell, cursor at start
        KeyCode::Char('I') if is_navigation_allowed(app) => {
            enter_insert_mode(app, true, false);
        }

        // Insert mode: 'A' - edit cell, cursor at end (same as 'i')
        KeyCode::Char('A') if is_navigation_allowed(app) => {
            enter_insert_mode(app, false, false);
        }

        // Insert mode: 's' - replace cell (clear + edit)
        KeyCode::Char('s') if is_navigation_allowed(app) => {
            enter_insert_mode(app, true, true);
        }

        // Insert mode: F2 - edit cell (same as 'i')
        KeyCode::F(2) if is_navigation_allowed(app) => {
            enter_insert_mode(app, false, false);
        }

        // Magnifier mode: 'm' - open magnifier for complex cell editing
        KeyCode::Char('m') if is_navigation_allowed(app) => {
            app.open_magnifier();
        }

        // Row operations: 'o' - add row below and enter Insert mode
        KeyCode::Char('o') if is_navigation_allowed(app) => {
            if let Some(row_idx) = app.get_selected_row() {
                let new_row_idx = RowIndex::new(row_idx.get() + 1);
                app.document.insert_row(new_row_idx);
                app.view_state.table_state.select(Some(new_row_idx.get()));
                enter_insert_mode(app, true, false);
            }
        }

        // Row operations: 'O' - add row above and enter Insert mode
        KeyCode::Char('O') if is_navigation_allowed(app) => {
            if let Some(row_idx) = app.get_selected_row() {
                app.document.insert_row(row_idx);
                // Selection stays at current index which is now the new row
                enter_insert_mode(app, true, false);
            }
        }

        // Comma leader - start column command sequence
        KeyCode::Char(',') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Comma);
            return Ok(InputResult::Continue);
        }

        // Row operations: 'P' - paste row(s) above
        KeyCode::Char('P') if is_navigation_allowed(app) => {
            if let Some(region) = app.clipboard.as_region() {
                if let Some(row_idx) = app.get_selected_row() {
                    let pasted_count = region.len();
                    for (i, clipboard_row) in region.iter().enumerate() {
                        let insert_idx = RowIndex::new(row_idx.get() + i);
                        app.document.insert_row(insert_idx);
                        for (col_idx, value) in clipboard_row.iter().enumerate() {
                            if col_idx < app.document.column_count() {
                                app.document.set_cell(
                                    insert_idx,
                                    ColIndex::new(col_idx),
                                    value.clone(),
                                );
                            }
                        }
                    }
                    // Selection stays at current index (the first pasted row)
                    app.status_message = Some(StatusMessage::new_owned(format!(
                        "Pasted {} row(s)",
                        pasted_count
                    )));
                }
            } else {
                app.status_message = Some(StatusMessage::from("Nothing to paste"));
            }
        }

        // Row operations: 'p' - paste row(s) below
        KeyCode::Char('p') if is_navigation_allowed(app) => {
            if let Some(region) = app.clipboard.as_region() {
                if let Some(row_idx) = app.get_selected_row() {
                    let pasted_count = region.len();
                    for (i, clipboard_row) in region.iter().enumerate() {
                        let insert_idx = RowIndex::new(row_idx.get() + 1 + i);
                        app.document.insert_row(insert_idx);
                        for (col_idx, value) in clipboard_row.iter().enumerate() {
                            if col_idx < app.document.column_count() {
                                app.document.set_cell(
                                    insert_idx,
                                    ColIndex::new(col_idx),
                                    value.clone(),
                                );
                            }
                        }
                    }
                    // Move selection to last pasted row
                    let last_pasted = row_idx.get() + pasted_count;
                    app.view_state.table_state.select(Some(last_pasted));
                    app.status_message = Some(StatusMessage::new_owned(format!(
                        "Pasted {} row(s)",
                        pasted_count
                    )));
                }
            } else {
                app.status_message = Some(StatusMessage::from("Nothing to paste"));
            }
        }

        // Delete key - clear current cell
        KeyCode::Delete if is_navigation_allowed(app) => {
            if let Some(row_idx) = app.get_selected_row() {
                let col_idx = app.view_state.selected_column;
                app.document.set_cell(row_idx, col_idx, String::new());
                app.status_message = Some(StatusMessage::from("Cell cleared"));
            }
        }

        // Enter key - move down one row (like j)
        KeyCode::Enter if is_navigation_allowed(app) => {
            navigation::commands::move_down_by(app, 1);
        }

        // Page navigation: Ctrl+d - page down
        KeyCode::Char('d')
            if is_navigation_allowed(app) && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let count = app
                .input_state
                .command_count
                .take()
                .map(|n| n.get())
                .unwrap_or(1);
            for _ in 0..count {
                let current = app.view_state.table_state.selected().unwrap_or(0);
                let target = (current + navigation::PAGE_SIZE)
                    .min(app.document.row_count().saturating_sub(1));
                app.view_state.table_state.select(Some(target));
            }
        }

        // Page navigation: Ctrl+u - page up
        KeyCode::Char('u')
            if is_navigation_allowed(app) && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
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

        // Navigation commands
        _ if is_navigation_allowed(app) => {
            navigation::handle_navigation(app, key.code)?;
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Handle multi-key command sequences (gg, zz, zt, zb, g<letters>, etc.)
fn handle_multi_key_command(
    app: &mut App,
    first: PendingCommand,
    second: KeyCode,
) -> Result<InputResult> {
    match (&first, second) {
        // gg - Go to first row
        (PendingCommand::G, KeyCode::Char('g')) => {
            app.input_state.clear_pending_command();
            navigation::goto_first_row(app);
            app.status_message = Some(StatusMessage::from(messages::JUMPED_TO_FIRST_ROW));
        }

        // g + letter - Start column jump (e.g., gA, gB)
        (PendingCommand::G, KeyCode::Char(c)) if c.is_ascii_alphabetic() => {
            let new_pending = first.append_letter(c);
            app.input_state.set_pending_command(new_pending);
            return Ok(InputResult::Continue);
        }

        // g + letter + more letters - Continue buffering (e.g., gB -> gBC)
        (PendingCommand::GotoColumn(_), KeyCode::Char(c)) if c.is_ascii_alphabetic() => {
            let new_pending = first.append_letter(c);
            app.input_state.set_pending_command(new_pending);
            return Ok(InputResult::Continue);
        }

        // g + letter(s) + Enter or non-letter - Execute column jump
        (PendingCommand::GotoColumn(_), KeyCode::Enter)
        | (PendingCommand::GotoColumn(_), KeyCode::Char(_)) => {
            app.input_state.clear_pending_command();
            if let Some(letters) = first.get_column_letters() {
                navigation::commands::goto_column(app, letters);
            }
        }

        // zt - Top of screen
        (PendingCommand::Z, KeyCode::Char('t')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Top;
            app.status_message = Some(StatusMessage::from(messages::VIEW_TOP));
        }

        // zz - Center of screen
        (PendingCommand::Z, KeyCode::Char('z')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Center;
            app.status_message = Some(StatusMessage::from(messages::VIEW_CENTER));
        }

        // zb - Bottom of screen
        (PendingCommand::Z, KeyCode::Char('b')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Bottom;
            app.status_message = Some(StatusMessage::from(messages::VIEW_BOTTOM));
        }

        // dd - Delete row(s) with optional count prefix
        (PendingCommand::D, KeyCode::Char('d')) => {
            app.input_state.clear_pending_command();
            let count = app
                .input_state
                .command_count
                .take()
                .map(|n| n.get())
                .unwrap_or(1);
            if let Some(row_idx) = app.get_selected_row() {
                // If deleting row 0 (header), turn header_mode OFF
                if row_idx.get() == 0 {
                    app.document.header_mode = false;
                    app.session.set_header_mode(false);
                }

                let end_idx = RowIndex::new(row_idx.get() + count - 1);
                let deleted = app.document.delete_rows(row_idx, end_idx);
                let deleted_count = deleted.len();
                if deleted_count > 0 {
                    app.clipboard.yank_region(deleted);
                    // Adjust selection if needed
                    let row_count = app.document.row_count();
                    if row_count == 0 {
                        app.view_state.table_state.select(None);
                    } else if row_idx.get() >= row_count {
                        app.view_state.table_state.select(Some(row_count - 1));
                    }

                    if row_idx.get() == 0 {
                        app.status_message =
                            Some(StatusMessage::from("Header row deleted, header mode OFF"));
                    } else {
                        app.status_message = Some(StatusMessage::new_owned(format!(
                            "{} row(s) deleted",
                            deleted_count
                        )));
                    }
                }
            }
        }

        // yy - Yank (copy) row(s) with optional count prefix
        (PendingCommand::Y, KeyCode::Char('y')) => {
            app.input_state.clear_pending_command();
            let count = app
                .input_state
                .command_count
                .take()
                .map(|n| n.get())
                .unwrap_or(1);
            if let Some(row_idx) = app.get_selected_row() {
                let end_idx = RowIndex::new(
                    (row_idx.get() + count - 1).min(app.document.row_count().saturating_sub(1)),
                );
                let rows = app.document.get_rows(row_idx, end_idx);
                let yanked_count = rows.len();
                if yanked_count > 0 {
                    app.clipboard.yank_region(rows);
                    app.status_message = Some(StatusMessage::new_owned(format!(
                        "{} row(s) yanked",
                        yanked_count
                    )));
                }
            }
        }

        // cc - Clear row and enter insert mode
        (PendingCommand::C, KeyCode::Char('c')) => {
            app.input_state.clear_pending_command();
            if let Some(row_idx) = app.get_selected_row() {
                // Clear all cells in the row
                let col_count = app.document.column_count();
                for col in 0..col_count {
                    app.document
                        .set_cell(row_idx, ColIndex::new(col), String::new());
                }
                // Move cursor to first column
                app.view_state.selected_column = ColIndex::new(0);
                // Enter insert mode
                enter_insert_mode(app, true, false);
                app.status_message = Some(StatusMessage::from("Row cleared"));
            }
        }

        // ,d - transition to CommaD (for ,dd)
        (PendingCommand::Comma, KeyCode::Char('d')) => {
            app.input_state.set_pending_command(PendingCommand::CommaD);
            return Ok(InputResult::Continue);
        }

        // ,y - transition to CommaY (for ,yy)
        (PendingCommand::Comma, KeyCode::Char('y')) => {
            app.input_state.set_pending_command(PendingCommand::CommaY);
            return Ok(InputResult::Continue);
        }

        // ,p - paste column(s) to the right of current column
        (PendingCommand::Comma, KeyCode::Char('p')) => {
            app.input_state.clear_pending_command();
            if let Some(columns) = app.clipboard.as_columns() {
                let col_idx = app.view_state.selected_column;
                let pasted_count = columns.len();
                for (i, col_data) in columns.into_iter().enumerate() {
                    let insert_at = ColIndex::new(col_idx.get() + 1 + i);
                    app.document.insert_column(insert_at, col_data);
                }
                // Move selection to first pasted column
                app.view_state.selected_column = ColIndex::new(col_idx.get() + 1);
                app.status_message = Some(StatusMessage::new_owned(format!(
                    "Pasted {} column(s)",
                    pasted_count
                )));
            } else {
                app.status_message = Some(StatusMessage::from("Nothing to paste"));
            }
        }

        // ,P - paste column(s) to the left of current column
        (PendingCommand::Comma, KeyCode::Char('P')) => {
            app.input_state.clear_pending_command();
            if let Some(columns) = app.clipboard.as_columns() {
                let col_idx = app.view_state.selected_column;
                let pasted_count = columns.len();
                for (i, col_data) in columns.into_iter().enumerate() {
                    let insert_at = ColIndex::new(col_idx.get() + i);
                    app.document.insert_column(insert_at, col_data);
                }
                // Selection stays at current index (first pasted column)
                app.status_message = Some(StatusMessage::new_owned(format!(
                    "Pasted {} column(s)",
                    pasted_count
                )));
            } else {
                app.status_message = Some(StatusMessage::from("Nothing to paste"));
            }
        }

        // ,o - insert empty column to the right
        (PendingCommand::Comma, KeyCode::Char('o')) => {
            app.input_state.clear_pending_command();
            let col_idx = app.view_state.selected_column;
            let insert_at = ColIndex::new(col_idx.get() + 1);
            app.document.insert_empty_column(insert_at);
            app.view_state.selected_column = insert_at;
            app.status_message = Some(StatusMessage::from("Inserted empty column"));
        }

        // ,O - insert empty column to the left
        (PendingCommand::Comma, KeyCode::Char('O')) => {
            app.input_state.clear_pending_command();
            let col_idx = app.view_state.selected_column;
            app.document.insert_empty_column(col_idx);
            app.status_message = Some(StatusMessage::from("Inserted empty column"));
        }

        // ,dd - delete column(s) with optional count prefix
        (PendingCommand::CommaD, KeyCode::Char('d')) => {
            app.input_state.clear_pending_command();
            let count = app
                .input_state
                .command_count
                .take()
                .map(|n| n.get())
                .unwrap_or(1);
            let col_idx = app.view_state.selected_column;
            let end_idx = ColIndex::new(col_idx.get() + count - 1);
            let deleted = app.document.delete_columns(col_idx, end_idx);
            let deleted_count = deleted.len();
            if deleted_count > 0 {
                app.clipboard.yank_columns(deleted);
                // Adjust selection if needed
                let col_count = app.document.column_count();
                if col_count == 0 {
                    // No columns left — nothing to select
                } else if col_idx.get() >= col_count {
                    app.view_state.selected_column = ColIndex::new(col_count - 1);
                }
                app.status_message = Some(StatusMessage::new_owned(format!(
                    "{} column(s) deleted",
                    deleted_count
                )));
            }
        }

        // ,yy - yank column(s) with optional count prefix
        (PendingCommand::CommaY, KeyCode::Char('y')) => {
            app.input_state.clear_pending_command();
            let count = app
                .input_state
                .command_count
                .take()
                .map(|n| n.get())
                .unwrap_or(1);
            let col_idx = app.view_state.selected_column;
            let end_idx = ColIndex::new(
                (col_idx.get() + count - 1).min(app.document.column_count().saturating_sub(1)),
            );
            let columns = app.document.get_columns(col_idx, end_idx);
            let yanked_count = columns.len();
            if yanked_count > 0 {
                app.clipboard.yank_columns(columns);
                app.status_message = Some(StatusMessage::new_owned(format!(
                    "{} column(s) yanked",
                    yanked_count
                )));
            }
        }

        _ => {
            app.input_state.clear_pending_command();
            app.status_message = Some(StatusMessage::from(messages::unknown_command(
                &format_pending_command(&first),
                &format_keycode(&second),
            )));
        }
    }

    Ok(InputResult::Continue)
}

/// Handle count prefix (numeric digits for commands like 5j, 10G)
fn handle_count_prefix(app: &mut App, digit: char) -> Result<InputResult> {
    let digit_value = digit.to_digit(10).unwrap() as usize;

    app.input_state.command_count = match app.input_state.command_count.take() {
        None => NonZeroUsize::new(digit_value),
        Some(existing) => {
            let new_value = existing.get() * 10 + digit_value;
            // Limit to reasonable size to prevent overflow
            if new_value < MAX_COMMAND_COUNT {
                NonZeroUsize::new(new_value)
            } else {
                Some(existing)
            }
        }
    };

    Ok(InputResult::Continue)
}

/// Handle keyboard input in Command mode
fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Clear transient messages on keypress
    if let Some(ref msg) = app.status_message {
        if msg.should_clear_on_keypress() {
            app.status_message = None;
        }
    }

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.input_state.clear_command_buffer();
            app.status_message = Some(StatusMessage::from(messages::CMD_CANCELLED));
        }

        KeyCode::Enter => {
            execute_command(app)?;
            // Only return to Normal if command didn't change mode
            // (Some commands like :files switch to a different mode)
            if app.mode == Mode::Command {
                app.mode = Mode::Normal;
            }
            app.input_state.clear_command_buffer();
        }

        KeyCode::Backspace => {
            app.input_state.pop_command_char();
        }

        KeyCode::Char(c) => {
            app.input_state.push_command_char(c);
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Parse and execute range commands like :5,10d, :%d, :.d, :$d
/// Returns Some(Result) if this is a range command, None otherwise
fn parse_and_execute_range_command(app: &mut App, cmd: &str) -> Option<Result<()>> {
    use crate::RowIndex;

    // Check if command contains a range pattern or special markers
    // Patterns: 5,10d, %d, .d, $d, 5,10y, %y, etc.

    // Match special range markers
    if let Some(operation) = cmd.strip_prefix('%') {
        // %d = delete all rows, %y = yank all rows
        match operation {
            "d" => {
                // Delete all data rows (excluding header)
                let row_count = app.document.data_row_count();
                if row_count == 0 {
                    app.status_message = Some(StatusMessage::from("No data rows to delete"));
                    return Some(Ok(()));
                }

                // Delete rows 1 to row_count (all data rows, preserving header at row 0)
                let deleted = app
                    .document
                    .delete_rows(RowIndex::new(1), RowIndex::new(row_count));
                app.status_message = Some(StatusMessage::from(format!(
                    "Deleted {} row(s)",
                    deleted.len()
                )));

                // Move cursor to row 1 (or row 0 if no data rows left)
                if app.document.data_row_count() > 0 {
                    app.view_state.table_state.select(Some(1));
                } else {
                    app.view_state.table_state.select(Some(0));
                }

                return Some(Ok(()));
            }
            "y" => {
                // Yank all data rows
                let row_count = app.document.data_row_count();
                if row_count == 0 {
                    app.status_message = Some(StatusMessage::from("No data rows to yank"));
                    return Some(Ok(()));
                }

                let yanked = app
                    .document
                    .get_rows(RowIndex::new(1), RowIndex::new(row_count));
                // TODO: Store in clipboard when clipboard system is implemented
                app.status_message = Some(StatusMessage::from(format!(
                    "Yanked {} row(s)",
                    yanked.len()
                )));

                return Some(Ok(()));
            }
            _ => {
                app.status_message = Some(StatusMessage::from(format!(
                    "Unknown range operation: :{}",
                    cmd
                )));
                return Some(Ok(()));
            }
        }
    }

    // Match .d or .y (current row)
    if let Some(operation) = cmd.strip_prefix('.') {
        match operation {
            "d" => {
                // Delete current row
                if let Some(row_idx) = app.get_selected_row() {
                    if let Some(_deleted) = app.document.delete_row(row_idx) {
                        app.status_message = Some(StatusMessage::from("Deleted 1 row"));

                        // Adjust cursor position
                        let new_row_count = app.document.data_row_count();
                        let current_pos = row_idx.get();

                        if new_row_count == 0 {
                            // No data rows left, move to header
                            app.view_state.table_state.select(Some(0));
                        } else if current_pos > new_row_count {
                            // Cursor past end, move to last row
                            app.view_state.table_state.select(Some(new_row_count));
                        }
                        // Otherwise keep cursor at same position
                    } else {
                        app.status_message = Some(StatusMessage::from("Failed to delete row"));
                    }
                } else {
                    app.status_message = Some(StatusMessage::from("No row selected"));
                }
                return Some(Ok(()));
            }
            "y" => {
                // Yank current row
                if let Some(row_idx) = app.get_selected_row() {
                    let yanked = app.document.get_rows(row_idx, row_idx);
                    if !yanked.is_empty() {
                        // TODO: Store in clipboard when clipboard system is implemented
                        app.status_message = Some(StatusMessage::from("Yanked 1 row"));
                    } else {
                        app.status_message = Some(StatusMessage::from("Failed to yank row"));
                    }
                } else {
                    app.status_message = Some(StatusMessage::from("No row selected"));
                }
                return Some(Ok(()));
            }
            _ => {
                app.status_message = Some(StatusMessage::from(format!(
                    "Unknown range operation: :{}",
                    cmd
                )));
                return Some(Ok(()));
            }
        }
    }

    // Match $d or $y (last row)
    if let Some(operation) = cmd.strip_prefix('$') {
        match operation {
            "d" => {
                // Delete last row
                let row_count = app.document.data_row_count();
                if row_count == 0 {
                    app.status_message = Some(StatusMessage::from("No data rows to delete"));
                    return Some(Ok(()));
                }

                if let Some(_deleted) = app.document.delete_row(RowIndex::new(row_count)) {
                    app.status_message = Some(StatusMessage::from("Deleted 1 row"));

                    // Move cursor to new last row if cursor was on deleted row
                    if let Some(current_row) = app.get_selected_row() {
                        if current_row.get() > app.document.data_row_count() {
                            app.view_state
                                .table_state
                                .select(Some(app.document.data_row_count()));
                        }
                    }
                } else {
                    app.status_message = Some(StatusMessage::from("Failed to delete row"));
                }
                return Some(Ok(()));
            }
            "y" => {
                // Yank last row
                let row_count = app.document.data_row_count();
                if row_count == 0 {
                    app.status_message = Some(StatusMessage::from("No data rows to yank"));
                    return Some(Ok(()));
                }

                let yanked = app
                    .document
                    .get_rows(RowIndex::new(row_count), RowIndex::new(row_count));
                if !yanked.is_empty() {
                    // TODO: Store in clipboard when clipboard system is implemented
                    app.status_message = Some(StatusMessage::from("Yanked 1 row"));
                } else {
                    app.status_message = Some(StatusMessage::from("Failed to yank row"));
                }
                return Some(Ok(()));
            }
            _ => {
                app.status_message = Some(StatusMessage::from(format!(
                    "Unknown range operation: :{}",
                    cmd
                )));
                return Some(Ok(()));
            }
        }
    }

    // Match numeric ranges: 5,10d or 5,10y
    if let Some(comma_pos) = cmd.find(',') {
        let start_str = &cmd[0..comma_pos];
        let rest = &cmd[comma_pos + 1..];

        // Parse start number
        if let Ok(start_num) = start_str.parse::<usize>() {
            // Find where the operation starts (last letter)
            if let Some(last_char) = rest.chars().last() {
                let operation = last_char;
                let end_str = &rest[0..rest.len() - 1];

                // Parse end number
                if let Ok(end_num) = end_str.parse::<usize>() {
                    match operation {
                        'd' => {
                            // Delete range
                            if start_num == 0 || end_num == 0 {
                                app.status_message = Some(StatusMessage::from(
                                    "Row numbers must be >= 1 (row 0 is header)",
                                ));
                                return Some(Ok(()));
                            }

                            if start_num > end_num {
                                app.status_message = Some(StatusMessage::from(
                                    "Invalid range: start must be <= end",
                                ));
                                return Some(Ok(()));
                            }

                            let deleted = app
                                .document
                                .delete_rows(RowIndex::new(start_num), RowIndex::new(end_num));

                            if deleted.is_empty() {
                                app.status_message = Some(StatusMessage::from(
                                    "No rows deleted (range out of bounds)",
                                ));
                            } else {
                                app.status_message = Some(StatusMessage::from(format!(
                                    "Deleted {} row(s)",
                                    deleted.len()
                                )));

                                // Adjust cursor position
                                if let Some(current_row) = app.get_selected_row() {
                                    let new_row_count = app.document.data_row_count();
                                    if new_row_count == 0 {
                                        app.view_state.table_state.select(Some(0));
                                    } else if current_row.get() > new_row_count {
                                        app.view_state.table_state.select(Some(new_row_count));
                                    }
                                }
                            }

                            return Some(Ok(()));
                        }
                        'y' => {
                            // Yank range
                            if start_num == 0 || end_num == 0 {
                                app.status_message = Some(StatusMessage::from(
                                    "Row numbers must be >= 1 (row 0 is header)",
                                ));
                                return Some(Ok(()));
                            }

                            if start_num > end_num {
                                app.status_message = Some(StatusMessage::from(
                                    "Invalid range: start must be <= end",
                                ));
                                return Some(Ok(()));
                            }

                            let yanked = app
                                .document
                                .get_rows(RowIndex::new(start_num), RowIndex::new(end_num));

                            if yanked.is_empty() {
                                app.status_message = Some(StatusMessage::from(
                                    "No rows yanked (range out of bounds)",
                                ));
                            } else {
                                // TODO: Store in clipboard when clipboard system is implemented
                                app.status_message = Some(StatusMessage::from(format!(
                                    "Yanked {} row(s)",
                                    yanked.len()
                                )));
                            }

                            return Some(Ok(()));
                        }
                        _ => {
                            app.status_message = Some(StatusMessage::from(format!(
                                "Unknown range operation: {}",
                                operation
                            )));
                            return Some(Ok(()));
                        }
                    }
                }
            }
        }
        // Check for column range: B,Dd or A,Ey or B,D m A
        else if rest.chars().last().is_some() {
            // Check for move command: "D m A" or "D m 0"
            if start_str.chars().all(|c| c.is_ascii_alphabetic()) {
                let words: Vec<&str> = rest.split_whitespace().collect();
                if words.len() == 3 && words[1] == "m" {
                    use crate::ui::utils::excel_letter_to_column;

                    let end_col_str = words[0];
                    let dest_str = words[2];

                    if !end_col_str.chars().all(|c| c.is_ascii_alphabetic()) {
                        app.status_message =
                            Some(StatusMessage::from("Invalid end column in move command"));
                        return Some(Ok(()));
                    }

                    let start_col = match excel_letter_to_column(&start_str.to_uppercase()) {
                        Ok(c) => c,
                        Err(e) => {
                            app.status_message = Some(StatusMessage::from(e));
                            return Some(Ok(()));
                        }
                    };
                    let end_col = match excel_letter_to_column(&end_col_str.to_uppercase()) {
                        Ok(c) => c,
                        Err(e) => {
                            app.status_message = Some(StatusMessage::from(e));
                            return Some(Ok(()));
                        }
                    };

                    if start_col > end_col {
                        app.status_message = Some(StatusMessage::from(
                            "Invalid range: start column must be <= end column",
                        ));
                        return Some(Ok(()));
                    }

                    let max_col = app.document.column_count();
                    if start_col >= max_col {
                        app.status_message = Some(StatusMessage::from(format!(
                            "Column {} does not exist (max: {})",
                            start_str.to_uppercase(),
                            crate::ui::utils::column_to_excel_letter(max_col.saturating_sub(1))
                        )));
                        return Some(Ok(()));
                    }

                    // Parse destination
                    let to_before = if dest_str == "0" {
                        0usize
                    } else if dest_str.chars().all(|c| c.is_ascii_alphabetic()) {
                        match excel_letter_to_column(&dest_str.to_uppercase()) {
                            Ok(dest_col) => dest_col + 1, // "after" that column
                            Err(e) => {
                                app.status_message = Some(StatusMessage::from(e));
                                return Some(Ok(()));
                            }
                        }
                    } else {
                        app.status_message = Some(StatusMessage::from(
                            "Invalid destination: use a column letter or 0",
                        ));
                        return Some(Ok(()));
                    };

                    // Check if destination is inside source range (no-op)
                    if to_before >= start_col && to_before <= end_col + 1 {
                        app.status_message = Some(StatusMessage::from(
                            "Columns already in position (destination inside source range)",
                        ));
                        return Some(Ok(()));
                    }

                    let count = end_col - start_col + 1;
                    let result = app.document.move_columns(
                        ColIndex::new(start_col),
                        ColIndex::new(end_col),
                        to_before,
                    );

                    app.view_state.selected_column = ColIndex::new(result);
                    app.status_message =
                        Some(StatusMessage::from(format!("Moved {} column(s)", count)));

                    return Some(Ok(()));
                }
            }

            let last_char = rest.chars().last().unwrap();
            let operation = last_char;
            let end_str = &rest[0..rest.len() - 1];

            // Check if both start and end are letters (column names) and end_str is not empty
            if !end_str.is_empty()
                && start_str.chars().all(|c| c.is_ascii_alphabetic())
                && end_str.chars().all(|c| c.is_ascii_alphabetic())
            {
                use crate::ui::utils::excel_letter_to_column;

                // Convert column letters to indices
                match (
                    excel_letter_to_column(&start_str.to_uppercase()),
                    excel_letter_to_column(&end_str.to_uppercase()),
                ) {
                    (Ok(start_col), Ok(end_col)) => {
                        match operation {
                            'd' => {
                                // Delete column range
                                if start_col > end_col {
                                    app.status_message = Some(StatusMessage::from(
                                        "Invalid range: start column must be <= end column",
                                    ));
                                    return Some(Ok(()));
                                }

                                let max_col = app.document.column_count();
                                if start_col >= max_col {
                                    app.status_message = Some(StatusMessage::from(format!(
                                        "Column {} does not exist (max: {})",
                                        start_str.to_uppercase(),
                                        crate::ui::utils::column_to_excel_letter(
                                            max_col.saturating_sub(1)
                                        )
                                    )));
                                    return Some(Ok(()));
                                }

                                let deleted = app.document.delete_columns(
                                    ColIndex::new(start_col),
                                    ColIndex::new(end_col),
                                );

                                if deleted.is_empty() {
                                    app.status_message = Some(StatusMessage::from(
                                        "No columns deleted (range out of bounds)",
                                    ));
                                } else {
                                    app.status_message = Some(StatusMessage::from(format!(
                                        "Deleted {} column(s)",
                                        deleted.len()
                                    )));

                                    // Adjust cursor position
                                    let current_col = app.view_state.selected_column.get();
                                    let new_col_count = app.document.column_count();

                                    if new_col_count == 0 {
                                        // No columns left, shouldn't happen but handle it
                                        app.view_state.selected_column = ColIndex::new(0);
                                    } else if current_col >= end_col {
                                        // Cursor at or after deleted range
                                        let new_pos = current_col.saturating_sub(deleted.len());
                                        app.view_state.selected_column =
                                            ColIndex::new(new_pos.min(new_col_count - 1));
                                    } else if current_col >= start_col {
                                        // Cursor in deleted range, move to start
                                        app.view_state.selected_column =
                                            ColIndex::new(start_col.min(new_col_count - 1));
                                    }
                                }

                                return Some(Ok(()));
                            }
                            'y' => {
                                // Yank column range
                                if start_col > end_col {
                                    app.status_message = Some(StatusMessage::from(
                                        "Invalid range: start column must be <= end column",
                                    ));
                                    return Some(Ok(()));
                                }

                                let yanked = app
                                    .document
                                    .get_columns(ColIndex::new(start_col), ColIndex::new(end_col));

                                if yanked.is_empty() {
                                    app.status_message = Some(StatusMessage::from(
                                        "No columns yanked (range out of bounds)",
                                    ));
                                } else {
                                    // TODO: Store in clipboard when clipboard system is implemented
                                    app.status_message = Some(StatusMessage::from(format!(
                                        "Yanked {} column(s)",
                                        yanked.len()
                                    )));
                                }

                                return Some(Ok(()));
                            }
                            _ => {
                                app.status_message = Some(StatusMessage::from(format!(
                                    "Unknown range operation: {}",
                                    operation
                                )));
                                return Some(Ok(()));
                            }
                        }
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        app.status_message = Some(StatusMessage::from(e));
                        return Some(Ok(()));
                    }
                }
            }
        }
    }

    // Not a range command
    None
}

/// Execute command from command buffer
fn execute_command(app: &mut App) -> Result<()> {
    let cmd = app.input_state.command_buffer.trim().to_string();

    if cmd.is_empty() {
        return Ok(());
    }

    // Special handling for :c command (column jump)
    // Support both `:cA` (no space) and `:c A` (with space)
    if cmd.starts_with('c') || cmd.starts_with('C') {
        let rest = &cmd[1..]; // Get everything after 'c'

        // Check if rest starts with a letter or digit (column specifier)
        // AND doesn't contain a comma (to avoid conflicting with column range commands like :C,Cd)
        if !rest.is_empty() && !rest.starts_with(' ') && !rest.contains(',') {
            // This is :cA, :cB, :c1, etc. (no space)
            let column_input = rest.trim();

            // Check if it's a numeric input (e.g., :c1, :c27)
            if let Ok(col_num) = column_input.parse::<usize>() {
                // Numeric column jump (1-indexed: 1=A, 2=B, 27=AA)
                navigation::commands::goto_column_by_number(app, col_num);
            } else {
                // Letter column jump (e.g., :cA, :cB, :cAA)
                // Validate it's only letters
                let column_letters = column_input.to_uppercase();
                if column_letters.chars().all(|c| c.is_ascii_alphabetic()) {
                    navigation::commands::goto_column(app, &column_letters);
                } else {
                    app.status_message = Some(StatusMessage::from(
                        "Invalid column name. Use letters (e.g., :cA, :cAA) or numbers (e.g., :c1, :c27)"
                    ));
                }
            }
            return Ok(());
        }
    }

    // Special handling for range operations: :5,10d, :%d, :.d, :$d, etc.
    if let Some(range_result) = parse_and_execute_range_command(app, &cmd) {
        return range_result;
    }

    // Split command into parts for commands with arguments
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let cmd_name_original = parts[0]; // Keep original case
    let cmd_name_lower = cmd_name_original.to_lowercase();
    let _arg = parts.get(1).map(|s| s.trim());

    // Check case-sensitive commands first
    match cmd_name_original {
        "W" => {
            // Save all dirty files
            match app.save_all_files() {
                Ok(paths) => {
                    if paths.is_empty() {
                        app.status_message = Some(StatusMessage::from("No files to save"));
                    } else {
                        app.status_message = Some(StatusMessage::from(format!(
                            "{} file(s) written",
                            paths.len()
                        )));
                    }
                }
                Err(e) => {
                    app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
                }
            }
            return Ok(());
        }
        "Wq" => {
            // Save all dirty files and quit
            match app.save_all_files() {
                Ok(_) => {
                    app.should_quit = true;
                }
                Err(e) => {
                    app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
                }
            }
            return Ok(());
        }
        _ => {} // Fall through to case-insensitive commands
    }

    // Case-insensitive commands
    match cmd_name_lower.as_str() {
        "q" | "quit" => {
            if app.document.is_dirty {
                app.status_message = Some(StatusMessage::from(
                    "No write since last change (add ! to override)",
                ));
            } else {
                app.should_quit = true;
            }
            return Ok(());
        }
        "q!" => {
            // Force quit - clear cache and quit
            app.session.clear_cache();
            app.should_quit = true;
            return Ok(());
        }
        "w" | "write" => {
            // Save current file only
            match app.save_current_file() {
                Ok(path) => {
                    app.status_message = Some(StatusMessage::from(format!(
                        "\"{}\" written",
                        path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
                    )));
                }
                Err(e) => {
                    app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
                }
            }
            return Ok(());
        }
        "wq" | "x" => {
            // Save current file and quit
            match app.save_current_file() {
                Ok(_) => {
                    app.should_quit = true;
                }
                Err(e) => {
                    app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
                }
            }
            return Ok(());
        }
        "h" | "help" => {
            app.status_message = Some(StatusMessage::from("Press ? for help"));
            return Ok(());
        }
        "ht" => {
            // Toggle header mode
            app.document.toggle_header_mode();
            app.session.set_header_mode(app.document.header_mode);

            // Adjust cursor position if needed
            if let Some(row_idx) = app.get_selected_row() {
                if app.document.header_mode && row_idx.get() == 0 {
                    // Header mode turned ON and cursor is on row 0 - move to row 1
                    app.view_state.table_state.select(Some(1));
                }
            }

            let mode_str = if app.document.header_mode {
                "ON"
            } else {
                "OFF"
            };
            app.status_message = Some(StatusMessage::from(format!("Header mode: {}", mode_str)));
            return Ok(());
        }
        "delim" => {
            // Change CSV delimiter for current file and reload
            if let Some(arg) = _arg {
                if arg.len() == 1 {
                    let new_delim = arg.chars().next().unwrap();

                    // Track in session for current file
                    let current_file = app.get_current_file().clone();
                    app.session.set_delimiter(current_file.clone(), new_delim);

                    // Reload file with new delimiter
                    match app.reload_current_file_with_delimiter(new_delim) {
                        Ok(_) => {
                            app.status_message = Some(StatusMessage::from(format!(
                                "Delimiter changed to '{}' and file reloaded",
                                new_delim
                            )));
                        }
                        Err(e) => {
                            app.status_message =
                                Some(StatusMessage::from(format!("Reload failed: {}", e)));
                        }
                    }
                } else {
                    app.status_message =
                        Some(StatusMessage::from("Delimiter must be a single character"));
                }
            } else {
                app.status_message = Some(StatusMessage::from(
                    "Usage: :delim <char> (e.g., :delim ; or :delim |)",
                ));
            }
            return Ok(());
        }
        "new" => {
            // Create a new CSV document with optional headers
            let headers = if let Some(arg) = _arg {
                // Parse comma-separated headers
                arg.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>()
            } else {
                // Default: single column named "Column 1"
                vec!["Column 1".to_string()]
            };

            // Create new document with headers only (0 data rows)
            let filename = app.document.filename.clone();
            let delimiter = app.document.delimiter;

            app.document = crate::csv::Document::new(headers.clone(), vec![], filename);
            app.document.delimiter = delimiter; // Preserve current delimiter
            app.document.is_dirty = true;
            app.document.header_mode = true;

            // Mark current file as dirty in session
            let current_file = app.get_current_file().clone();
            app.session.mark_dirty(&current_file);

            // Reset view state and position cursor
            app.view_state = crate::ui::ViewState::default();
            // Position at row 1 (first data row position) if header_mode is ON
            // Since we have 0 data rows, this will be out of bounds but selection will still work
            if app.document.header_mode {
                app.view_state.table_state.select(Some(1));
            } else {
                app.view_state.table_state.select(Some(0));
            }

            app.status_message = Some(StatusMessage::from(format!(
                "New CSV created with {} column(s)",
                headers.len()
            )));
            return Ok(());
        }
        "c" => {
            // Column jump: :cA, :cB, :cAA, :c1, etc.
            if let Some(arg) = _arg {
                let column_input = arg.trim();

                // Check if it's a numeric input (e.g., :c1, :c27)
                if let Ok(col_num) = column_input.parse::<usize>() {
                    // Numeric column jump (1-indexed: 1=A, 2=B, 27=AA)
                    navigation::commands::goto_column_by_number(app, col_num);
                } else {
                    // Letter column jump (e.g., :cA, :cB, :cAA)
                    // Validate it's only letters
                    let column_letters = column_input.to_uppercase();
                    if column_letters.chars().all(|c| c.is_ascii_alphabetic()) {
                        navigation::commands::goto_column(app, &column_letters);
                    } else {
                        app.status_message = Some(StatusMessage::from(
                            "Invalid column name. Use letters (e.g., :cA, :cAA) or numbers (e.g., :c1, :c27)"
                        ));
                    }
                }
            } else {
                app.status_message = Some(StatusMessage::from(
                    "Usage: :c<column> (e.g., :cA, :cB, :cAA, :c1, :c27)",
                ));
            }
            return Ok(());
        }
        "f" => {
            // :f (no arg) shows current filename, :f <name> renames
            if let Some(arg) = _arg {
                let new_name = arg.to_string();
                app.document.filename = new_name.clone();
                let new_path = std::path::PathBuf::from(&new_name);
                app.session.rename_current_file(new_path.clone());
                app.document.is_dirty = true;
                app.session.mark_dirty(&new_path);
                app.status_message =
                    Some(StatusMessage::from(format!("Renamed to \"{}\"", new_name)));
            } else {
                let current = app.document.filename.clone();
                app.status_message = Some(StatusMessage::from(format!("\"{}\"", current)));
            }
            return Ok(());
        }
        "files" => {
            // Enter FileList mode to show file picker
            app.mode = Mode::FileList;
            app.input_state.clear_file_filter();
            app.status_message = Some(StatusMessage::from(
                "Type to filter, number to select, Enter for first, Esc to cancel",
            ));
            return Ok(());
        }
        _ => {}
    }

    // Unknown command
    app.status_message = Some(StatusMessage::from(format!("Unknown command: :{}", cmd)));
    Ok(())
}

/// Handle keyboard input in FileList mode
fn handle_file_list_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match key.code {
        KeyCode::Esc => {
            // Cancel file picker
            app.mode = Mode::Normal;
            app.status_message = None;
            app.input_state.clear_file_filter();
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }
        KeyCode::Backspace => {
            // Delete character from filter
            app.input_state.pop_file_filter_char();
            // Reset selection to 0 when filter changes
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            // Move selection up
            if app.view_state.file_list_selected > 0 {
                app.view_state.file_list_selected -= 1;
            }
            Ok(InputResult::Continue)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            // Move selection down
            let filter = app.input_state.file_filter_buffer.to_lowercase();
            let files = app.session.files();

            let filtered_count = files
                .iter()
                .filter(|path| {
                    if filter.is_empty() {
                        true
                    } else {
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_lowercase().contains(&filter))
                            .unwrap_or(false)
                    }
                })
                .count();

            if app.view_state.file_list_selected + 1 < filtered_count {
                app.view_state.file_list_selected += 1;
            }
            Ok(InputResult::Continue)
        }
        KeyCode::Enter => {
            // Select current file
            let filter = app.input_state.file_filter_buffer.to_lowercase();
            let files = app.session.files();

            // Get filtered file list
            let filtered_files: Vec<(usize, &std::path::PathBuf)> = files
                .iter()
                .enumerate()
                .filter(|(_, path)| {
                    if filter.is_empty() {
                        true
                    } else {
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_lowercase().contains(&filter))
                            .unwrap_or(false)
                    }
                })
                .collect();

            if filtered_files.is_empty() {
                app.status_message = Some(StatusMessage::from("No matching files"));
                return Ok(InputResult::Continue);
            }

            // Get the actual file index from filtered list
            let selected_idx = app
                .view_state
                .file_list_selected
                .min(filtered_files.len() - 1);
            let target_index = filtered_files[selected_idx].0;

            // Switch to selected file
            let current = app.session.active_file_index();
            if target_index != current {
                let file_count = app.session.file_count();
                let diff = if target_index > current {
                    target_index - current
                } else {
                    file_count - current + target_index
                };

                for _ in 0..diff {
                    app.session.next_file();
                }
            }

            app.mode = Mode::Normal;
            app.input_state.clear_file_filter();
            app.view_state.file_list_selected = 0;

            if target_index != current {
                Ok(InputResult::ReloadFile)
            } else {
                Ok(InputResult::Continue)
            }
        }
        KeyCode::Char(c) => {
            // Add character to filter
            app.input_state.push_file_filter_char(c);
            // Reset selection to 0 when filter changes
            app.view_state.file_list_selected = 0;
            Ok(InputResult::Continue)
        }
        _ => {
            // Ignore other keys
            Ok(InputResult::Continue)
        }
    }
}

/// Handle keyboard input in Insert mode
fn handle_insert_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // If no edit buffer, return to Normal mode (shouldn't happen)
    if app.edit_buffer.is_none() {
        app.mode = Mode::Normal;
        return Ok(InputResult::Continue);
    }

    match (key.code, key.modifiers) {
        // Exit: Save and move down
        (KeyCode::Enter, KeyModifiers::NONE) => {
            commit_edit(app);
            navigation::commands::move_down_by(app, 1);
        }

        // Exit: Save and move up
        (KeyCode::Enter, KeyModifiers::SHIFT) => {
            commit_edit(app);
            navigation::commands::move_up_by(app, 1);
        }

        // Exit: Save and move right
        (KeyCode::Tab, KeyModifiers::NONE) => {
            commit_edit(app);
            navigation::commands::move_right_by(app, 1);
        }

        // Exit: Save and move left
        (KeyCode::Tab, KeyModifiers::SHIFT) | (KeyCode::BackTab, _) => {
            commit_edit(app);
            navigation::commands::move_left_by(app, 1);
        }

        // Exit: Cancel
        (KeyCode::Esc, _) => {
            app.edit_buffer = None;
            app.mode = Mode::Normal;
        }

        // Text editing: Type character
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                // Convert char cursor position to byte position for insert
                let byte_pos = buffer
                    .content
                    .char_indices()
                    .nth(buffer.cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(buffer.content.len());
                buffer.content.insert(byte_pos, c);
                buffer.cursor += 1;
            }
        }

        // Text editing: Backspace
        (KeyCode::Backspace, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                if buffer.cursor > 0 {
                    buffer.cursor -= 1;
                    // Convert char cursor position to byte position for remove
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
            }
        }

        // Text editing: Ctrl+h (vim-style backspace)
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                if buffer.cursor > 0 {
                    buffer.cursor -= 1;
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
            }
        }

        // Text editing: Delete
        (KeyCode::Delete, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                let char_count = buffer.content.chars().count();
                if buffer.cursor < char_count {
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
            }
        }

        // Cursor movement: Left
        (KeyCode::Left, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                buffer.cursor = buffer.cursor.saturating_sub(1);
            }
        }

        // Cursor movement: Right
        (KeyCode::Right, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                let char_count = buffer.content.chars().count();
                buffer.cursor = (buffer.cursor + 1).min(char_count);
            }
        }

        // Cursor movement: Home
        (KeyCode::Home, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                buffer.cursor = 0;
            }
        }

        // Cursor movement: End
        (KeyCode::End, _) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                buffer.cursor = buffer.content.chars().count();
            }
        }

        // Vim-style: Ctrl+w - delete word backward
        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                // Delete trailing spaces first
                while buffer.cursor > 0
                    && buffer.content.chars().nth(buffer.cursor - 1) == Some(' ')
                {
                    buffer.cursor -= 1;
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
                // Delete word characters
                while buffer.cursor > 0
                    && buffer.content.chars().nth(buffer.cursor - 1) != Some(' ')
                {
                    buffer.cursor -= 1;
                    let byte_pos = buffer
                        .content
                        .char_indices()
                        .nth(buffer.cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.content.remove(byte_pos);
                }
            }
        }

        // Vim-style: Ctrl+u - delete to start of line
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            if let Some(ref mut buffer) = app.edit_buffer {
                // Convert char cursor position to byte position for slicing
                let byte_pos = buffer
                    .content
                    .char_indices()
                    .nth(buffer.cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(buffer.content.len());
                buffer.content = buffer.content[byte_pos..].to_string();
                buffer.cursor = 0;
            }
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Generate a unique output filename that doesn't conflict with existing session files.
/// Returns "output.csv", "output1.csv", "output2.csv", etc.
fn generate_output_filename(app: &App) -> String {
    let existing: std::collections::HashSet<String> = app
        .session
        .files()
        .iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect();

    let base = "output.csv".to_string();
    if !existing.contains(&base) {
        return base;
    }
    let mut i = 1;
    loop {
        let name = format!("output{}.csv", i);
        if !existing.contains(&name) {
            return name;
        }
        i += 1;
    }
}

/// Move the SQL cursor up one line within the sql_buffer.
fn move_sql_cursor_up(buffer: &str, cursor: usize) -> usize {
    let chars: Vec<char> = buffer.chars().collect();
    // Find position of start of current line
    let mut line_start = cursor;
    while line_start > 0 && chars[line_start - 1] != '\n' {
        line_start -= 1;
    }
    if line_start == 0 {
        return cursor; // Already on first line
    }
    let col = cursor - line_start;
    // Find start of previous line
    let prev_line_end = line_start - 1; // the '\n' char
    let mut prev_line_start = prev_line_end;
    while prev_line_start > 0 && chars[prev_line_start - 1] != '\n' {
        prev_line_start -= 1;
    }
    let prev_line_len = prev_line_end - prev_line_start;
    prev_line_start + col.min(prev_line_len)
}

/// Move the SQL cursor down one line within the sql_buffer.
fn move_sql_cursor_down(buffer: &str, cursor: usize) -> usize {
    let chars: Vec<char> = buffer.chars().collect();
    let total = chars.len();
    // Find start of current line
    let mut line_start = cursor;
    while line_start > 0 && chars[line_start - 1] != '\n' {
        line_start -= 1;
    }
    let col = cursor - line_start;
    // Find end of current line
    let mut line_end = cursor;
    while line_end < total && chars[line_end] != '\n' {
        line_end += 1;
    }
    if line_end >= total {
        return cursor; // Already on last line
    }
    let next_line_start = line_end + 1;
    // Find end of next line
    let mut next_line_end = next_line_start;
    while next_line_end < total && chars[next_line_end] != '\n' {
        next_line_end += 1;
    }
    let next_line_len = next_line_end - next_line_start;
    next_line_start + col.min(next_line_len)
}

/// Handle keyboard input in SQL editor mode
fn handle_sql_editor_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match (key.code, key.modifiers) {
        // Escape → return to Normal mode
        (KeyCode::Esc, _) => {
            app.mode = Mode::Normal;
        }

        // Shift+Enter → insert newline
        (KeyCode::Enter, KeyModifiers::SHIFT) => {
            let byte_pos = app
                .sql_buffer
                .char_indices()
                .nth(app.sql_cursor)
                .map(|(i, _)| i)
                .unwrap_or(app.sql_buffer.len());
            app.sql_buffer.insert(byte_pos, '\n');
            app.sql_cursor += 1;
        }

        // Enter → execute query
        (KeyCode::Enter, KeyModifiers::NONE) => {
            let query = app.sql_buffer.trim().to_string();
            if query.is_empty() {
                app.status_message = Some(StatusMessage::new_owned("Empty query".to_string()));
                app.mode = Mode::Normal;
                return Ok(InputResult::Continue);
            }

            // Build SQLite connection and load all session CSVs
            let conn = rusqlite::Connection::open_in_memory()
                .map_err(|e| anyhow::anyhow!("Failed to open SQLite: {}", e))?;

            // Load each session file into SQLite
            for file_path in app.session.files().to_vec() {
                let table_name = crate::query::table_name_from_path(&file_path);

                // Check if this is the current document
                let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let doc = if filename == app.document.filename {
                    // Use current in-memory document (may have unsaved edits)
                    app.document.clone()
                } else if let Some(cached) = app.session.get_cached_document(&file_path) {
                    // Use cached (dirty) document
                    cached.clone()
                } else if file_path.exists() {
                    // Load from disk
                    let config = app.session.config();
                    match crate::csv::Document::from_file(
                        &file_path,
                        config.delimiter,
                        config.no_headers,
                        config.encoding.clone(),
                    ) {
                        Ok(d) => d,
                        Err(_) => continue,
                    }
                } else {
                    continue; // Skip virtual files that don't exist on disk
                };

                if doc.rows.is_empty() || doc.rows[0].is_empty() {
                    continue;
                }

                if crate::query::load_csv_into_sqlite(&conn, &doc, &table_name).is_err() {
                    continue; // Skip files that fail to load
                }
            }

            // Execute query
            match crate::query::execute_query_to_document(&conn, &query, "output.csv".to_string()) {
                Ok(mut doc) => {
                    // Determine output filename using reuse logic:
                    // Look for an existing query output sheet (may have been renamed)
                    let reuse_name = app.session.find_query_output_file().and_then(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_string())
                    });

                    let output_name = reuse_name.unwrap_or_else(|| generate_output_filename(app));
                    doc.filename = output_name;

                    app.sql_error = None;
                    app.mode = Mode::Normal;
                    return Ok(InputResult::SwitchToDocument(doc));
                }
                Err(e) => {
                    app.sql_error = Some(format!("SQL error: {}", e));
                    // Stay in SqlEditor mode so user can fix the query
                }
            }
        }

        // Type character
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            app.sql_error = None;
            let byte_pos = app
                .sql_buffer
                .char_indices()
                .nth(app.sql_cursor)
                .map(|(i, _)| i)
                .unwrap_or(app.sql_buffer.len());
            app.sql_buffer.insert(byte_pos, c);
            app.sql_cursor += 1;
        }

        // Backspace
        (KeyCode::Backspace, _) => {
            if app.sql_cursor > 0 {
                app.sql_cursor -= 1;
                let byte_pos = app
                    .sql_buffer
                    .char_indices()
                    .nth(app.sql_cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                app.sql_buffer.remove(byte_pos);
            }
        }

        // Delete
        (KeyCode::Delete, _) => {
            let char_count = app.sql_buffer.chars().count();
            if app.sql_cursor < char_count {
                let byte_pos = app
                    .sql_buffer
                    .char_indices()
                    .nth(app.sql_cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                app.sql_buffer.remove(byte_pos);
            }
        }

        // Ctrl+u → clear buffer
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.sql_buffer.clear();
            app.sql_cursor = 0;
        }

        // Left arrow
        (KeyCode::Left, _) => {
            app.sql_cursor = app.sql_cursor.saturating_sub(1);
        }

        // Right arrow
        (KeyCode::Right, _) => {
            let char_count = app.sql_buffer.chars().count();
            app.sql_cursor = (app.sql_cursor + 1).min(char_count);
        }

        // Up arrow → move cursor up one line
        (KeyCode::Up, _) => {
            app.sql_cursor = move_sql_cursor_up(&app.sql_buffer, app.sql_cursor);
        }

        // Down arrow → move cursor down one line
        (KeyCode::Down, _) => {
            app.sql_cursor = move_sql_cursor_down(&app.sql_buffer, app.sql_cursor);
        }

        // Home → start of current line
        (KeyCode::Home, _) => {
            let chars: Vec<char> = app.sql_buffer.chars().collect();
            let mut pos = app.sql_cursor;
            while pos > 0 && chars[pos - 1] != '\n' {
                pos -= 1;
            }
            app.sql_cursor = pos;
        }

        // End → end of current line
        (KeyCode::End, _) => {
            let chars: Vec<char> = app.sql_buffer.chars().collect();
            let total = chars.len();
            let mut pos = app.sql_cursor;
            while pos < total && chars[pos] != '\n' {
                pos += 1;
            }
            app.sql_cursor = pos;
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

// ============================================================================
// Magnifier Mode Handler (Phase 5)
// ============================================================================

/// Handle keyboard input in Magnifier mode
fn handle_magnifier_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    use crate::magnifier::MagnifierMode;

    let mag = match app.magnifier_state.as_mut() {
        Some(m) => m,
        None => {
            // No magnifier state - return to normal mode
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
    };

    // Check for Ctrl+hjkl navigation (works in both Normal and Insert modes within magnifier)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('h') => {
                return handle_magnifier_navigate(app, Direction::Left);
            }
            KeyCode::Char('j') => {
                return handle_magnifier_navigate(app, Direction::Down);
            }
            KeyCode::Char('k') => {
                return handle_magnifier_navigate(app, Direction::Up);
            }
            KeyCode::Char('l') => {
                return handle_magnifier_navigate(app, Direction::Right);
            }
            _ => {}
        }
    }

    match mag.mode() {
        MagnifierMode::Normal => handle_magnifier_normal(app, key),
        MagnifierMode::Insert => handle_magnifier_insert(app, key),
        MagnifierMode::Command => handle_magnifier_command(app, key),
        MagnifierMode::Visual | MagnifierMode::VisualLine => handle_magnifier_visual(app, key),
    }
}

/// Direction for cell navigation in magnifier
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Handle navigation to adjacent cells from magnifier
fn handle_magnifier_navigate(app: &mut App, direction: Direction) -> Result<InputResult> {
    // Check if magnifier has unsaved changes
    if app.magnifier_is_dirty() {
        // TODO: Show save prompt dialog
        // For now, just show a message and don't navigate
        app.status_message = Some(StatusMessage::from(
            "Unsaved changes! Use :w to save, :q! to discard",
        ));
        return Ok(InputResult::Continue);
    }

    // Close magnifier without saving (it's clean)
    app.close_magnifier_discard();

    // Navigate to adjacent cell
    match direction {
        Direction::Left => {
            if app.view_state.selected_column.get() > 0 {
                app.view_state.selected_column =
                    ColIndex::new(app.view_state.selected_column.get() - 1);
            }
        }
        Direction::Right => {
            if app.view_state.selected_column.get() < app.document.column_count().saturating_sub(1)
            {
                app.view_state.selected_column =
                    ColIndex::new(app.view_state.selected_column.get() + 1);
            }
        }
        Direction::Up => {
            if let Some(current_row) = app.view_state.table_state.selected() {
                if current_row > 0 {
                    app.view_state.table_state.select(Some(current_row - 1));
                }
            }
        }
        Direction::Down => {
            if let Some(current_row) = app.view_state.table_state.selected() {
                if current_row < app.document.row_count().saturating_sub(1) {
                    app.view_state.table_state.select(Some(current_row + 1));
                }
            }
        }
    }

    // Reopen magnifier on new cell
    app.open_magnifier();

    Ok(InputResult::Continue)
}

/// Handle keys in magnifier Normal mode
fn handle_magnifier_normal(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    use crate::magnifier::PendingCommand;

    let mag = match app.magnifier_state.as_mut() {
        Some(m) => m,
        None => return Ok(InputResult::Continue),
    };

    // Handle pending commands first
    if let Some(pending) = mag.take_pending() {
        match (pending, key.code) {
            // Multi-key sequences
            (PendingCommand::G, KeyCode::Char('g')) => mag.move_to_first_line(),
            (PendingCommand::D, KeyCode::Char('d')) => {
                mag.push_undo();
                mag.delete_line();
            }
            (PendingCommand::Y, KeyCode::Char('y')) => mag.yank_line(),
            (PendingCommand::C, KeyCode::Char('c')) => mag.change_line(),
            (PendingCommand::Z, KeyCode::Char('Z')) => {
                app.save_and_close_magnifier();
                return Ok(InputResult::Continue);
            }
            (PendingCommand::IndentRight, KeyCode::Char('>')) => mag.indent_line(),
            (PendingCommand::IndentLeft, KeyCode::Char('<')) => mag.dedent_line(),

            // Character find commands
            (PendingCommand::FindForward, KeyCode::Char(c)) => mag.find_char_forward(c),
            (PendingCommand::FindBackward, KeyCode::Char(c)) => mag.find_char_backward(c),
            (PendingCommand::TillForward, KeyCode::Char(c)) => mag.till_char_forward(c),
            (PendingCommand::TillBackward, KeyCode::Char(c)) => mag.till_char_backward(c),
            (PendingCommand::Replace, KeyCode::Char(c)) => mag.replace_char(c),

            _ => {
                // Invalid sequence, clear pending
            }
        }
        return Ok(InputResult::Continue);
    }

    // Handle Ctrl+r for redo
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char('r') = key.code {
            mag.redo();
            return Ok(InputResult::Continue);
        }
    }

    match key.code {
        // Basic motions
        KeyCode::Char('h') | KeyCode::Left => mag.move_left(),
        KeyCode::Char('j') | KeyCode::Down => mag.move_down(),
        KeyCode::Char('k') | KeyCode::Up => mag.move_up(),
        KeyCode::Char('l') | KeyCode::Right => mag.move_right(),

        // Line motions
        KeyCode::Char('0') => mag.move_to_line_start(),
        KeyCode::Char('$') => mag.move_to_line_end(),
        KeyCode::Char('^') => mag.move_to_first_non_blank(),

        // Word motions
        KeyCode::Char('w') => mag.move_next_word(),
        KeyCode::Char('b') => mag.move_prev_word(),
        KeyCode::Char('e') => mag.move_end_word(),

        // Buffer motions
        KeyCode::Char('G') => mag.move_to_last_line(),

        // Simple operators
        KeyCode::Char('x') => {
            mag.push_undo();
            mag.delete_char();
        }
        KeyCode::Char('p') => mag.paste_below(),
        KeyCode::Char('P') => mag.paste_above(),
        KeyCode::Char('J') => mag.join_lines(),

        // Undo/redo
        KeyCode::Char('u') => mag.undo(),

        // Enter insert mode
        KeyCode::Char('i') => mag.insert_before(),
        KeyCode::Char('a') => mag.insert_after(),
        KeyCode::Char('A') => {
            mag.move_to_line_end();
            mag.insert_after();
        }
        KeyCode::Char('I') => {
            mag.move_to_first_non_blank();
            mag.insert_before();
        }
        KeyCode::Char('o') => mag.insert_line_below(),
        KeyCode::Char('O') => mag.insert_line_above(),
        KeyCode::Char('s') => mag.substitute_char(),
        KeyCode::Char('C') => mag.change_to_eol(),

        // Visual mode
        KeyCode::Char('v') => mag.enter_visual_mode(),
        KeyCode::Char('V') => mag.enter_visual_line_mode(),

        // Search
        KeyCode::Char('/') => mag.enter_command_mode_with("/"),
        KeyCode::Char('n') => mag.jump_to_next_match(),
        KeyCode::Char('N') => mag.jump_to_prev_match(),
        KeyCode::Char('*') => {
            if let Some(word) = mag.get_word_under_cursor() {
                mag.search_forward(word);
            }
        }

        // Multi-key command initiators
        KeyCode::Char('g') => mag.set_pending(PendingCommand::G),
        KeyCode::Char('d') => mag.set_pending(PendingCommand::D),
        KeyCode::Char('y') => mag.set_pending(PendingCommand::Y),
        KeyCode::Char('c') => mag.set_pending(PendingCommand::C),
        KeyCode::Char('Z') => mag.set_pending(PendingCommand::Z),
        KeyCode::Char('f') => mag.set_pending(PendingCommand::FindForward),
        KeyCode::Char('F') => mag.set_pending(PendingCommand::FindBackward),
        KeyCode::Char('t') => mag.set_pending(PendingCommand::TillForward),
        KeyCode::Char('T') => mag.set_pending(PendingCommand::TillBackward),
        KeyCode::Char('r') => mag.set_pending(PendingCommand::Replace),
        KeyCode::Char('>') => mag.set_pending(PendingCommand::IndentRight),
        KeyCode::Char('<') => mag.set_pending(PendingCommand::IndentLeft),

        // Repeat find
        KeyCode::Char(';') => mag.repeat_find(),
        KeyCode::Char(',') => mag.repeat_find_reverse(),

        // Command mode
        KeyCode::Char(':') => mag.enter_command_mode(),

        // Escape - close if clean, warn if dirty
        KeyCode::Esc => {
            if app.magnifier_is_dirty() {
                app.status_message = Some(StatusMessage::from(
                    "Unsaved changes! Use :wq to save, :q! to discard",
                ));
            } else {
                app.close_magnifier_discard();
            }
        }

        // Count prefix
        KeyCode::Char(c) if c.is_numeric() => {
            if let Some(digit) = c.to_digit(10) {
                mag.set_count_prefix(digit as usize);
            }
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Handle keys in magnifier Insert mode
fn handle_magnifier_insert(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    let mag = match app.magnifier_state.as_mut() {
        Some(m) => m,
        None => return Ok(InputResult::Continue),
    };

    match key.code {
        // Exit insert mode
        KeyCode::Esc => {
            mag.exit_insert_mode();
        }

        // Text input
        KeyCode::Char(c) => {
            mag.insert_char(c);
        }

        // Backspace
        KeyCode::Backspace => {
            mag.delete_char_before();
        }

        // Delete
        KeyCode::Delete => {
            mag.delete_char_at();
        }

        // Enter - newline
        KeyCode::Enter => {
            mag.newline();
        }

        // Arrow keys for navigation in insert mode
        KeyCode::Left => mag.move_left(),
        KeyCode::Right => mag.move_right(),
        KeyCode::Up => mag.move_up(),
        KeyCode::Down => mag.move_down(),

        // Home/End
        KeyCode::Home => mag.move_to_line_start(),
        KeyCode::End => mag.move_to_line_end(),

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Handle keys in magnifier Command mode
fn handle_magnifier_command(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    let mag = match app.magnifier_state.as_mut() {
        Some(m) => m,
        None => return Ok(InputResult::Continue),
    };

    match key.code {
        KeyCode::Esc => {
            mag.exit_command_mode();
        }
        KeyCode::Enter => {
            let cmd = mag.command_buffer().to_string();
            mag.exit_command_mode();

            // Handle search
            if let Some(pattern) = cmd.strip_prefix('/') {
                mag.search_forward(pattern.to_string());
                return Ok(InputResult::Continue);
            }

            // Handle ex commands
            match cmd.as_str() {
                "w" => {
                    // Save to cell
                    app.save_magnifier_content();
                    app.status_message = Some(StatusMessage::from("Saved"));
                }
                "q" => {
                    if app.magnifier_is_dirty() {
                        app.status_message = Some(StatusMessage::from(
                            "Unsaved changes! Use :wq to save, :q! to discard",
                        ));
                    } else {
                        app.close_magnifier_discard();
                    }
                }
                "wq" => {
                    app.save_and_close_magnifier();
                }
                "q!" => {
                    app.close_magnifier_discard();
                }
                "noh" => {
                    if let Some(m) = app.magnifier_state.as_mut() {
                        m.clear_search();
                    }
                }
                _ => {
                    app.status_message =
                        Some(StatusMessage::from(format!("Unknown command: {}", cmd)));
                }
            }
        }
        KeyCode::Char(c) => {
            mag.command_insert_char(c);
        }
        KeyCode::Backspace => {
            mag.command_backspace();
            // If buffer is empty, exit command mode
            if mag.command_buffer().is_empty() {
                mag.exit_command_mode();
            }
        }
        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Handle keys in magnifier Visual mode
fn handle_magnifier_visual(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    let mag = match app.magnifier_state.as_mut() {
        Some(m) => m,
        None => return Ok(InputResult::Continue),
    };

    match key.code {
        // Motions extend selection
        KeyCode::Char('h') | KeyCode::Left => mag.move_left(),
        KeyCode::Char('j') | KeyCode::Down => mag.move_down(),
        KeyCode::Char('k') | KeyCode::Up => mag.move_up(),
        KeyCode::Char('l') | KeyCode::Right => mag.move_right(),

        // Word motions
        KeyCode::Char('w') => mag.move_next_word(),
        KeyCode::Char('b') => mag.move_prev_word(),
        KeyCode::Char('e') => mag.move_end_word(),

        // Line motions
        KeyCode::Char('0') => mag.move_to_line_start(),
        KeyCode::Char('$') => mag.move_to_line_end(),
        KeyCode::Char('^') => mag.move_to_first_non_blank(),

        // Buffer motions
        KeyCode::Char('g') => mag.move_to_first_line(),
        KeyCode::Char('G') => mag.move_to_last_line(),

        // Operators on selection
        KeyCode::Char('d') => mag.delete_selection(),
        KeyCode::Char('y') => mag.yank_selection(),
        KeyCode::Char('c') => mag.change_selection(),

        // Exit visual mode
        KeyCode::Esc => mag.exit_visual_mode(),

        _ => {}
    }

    Ok(InputResult::Continue)
}
