//! Main handler for Normal mode input
//!
//! This module is the primary dispatcher for Normal mode keyboard input.

use crate::app::{messages, App, Mode, VisualMode, VisualSelection};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::handler::{
    enter_insert_mode, handle_file_switch, CursorPosition, InitialContent,
};
use crate::input::{InputResult, PendingCommand, StatusMessage};
use crate::navigation;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::num::NonZeroUsize;

/// Maximum command count to prevent overflow
const MAX_COMMAND_COUNT: usize = 100000;

/// Returns true if navigation commands are allowed (help overlay is closed)
fn is_navigation_allowed(app: &App) -> bool {
    !app.view_state.help_overlay_visible
}

/// Toggle help overlay visibility
fn handle_help_toggle(app: &mut App) {
    app.view_state.help_overlay_visible = !app.view_state.help_overlay_visible;
}

/// Handle keyboard input in Normal mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Clear transient messages on keypress
    if let Some(ref msg) = app.status_message {
        if msg.should_clear_on_keypress() {
            app.status_message = None;
        }
    }

    // Handle external modification prompt — intercept keys before normal processing
    if app.external_modification_pending {
        match key.code {
            KeyCode::Char('r') => {
                app.external_modification_pending = false;
                app.status_message = None;
                return Ok(InputResult::ReloadFile);
            }
            KeyCode::Esc => {
                app.external_modification_pending = false;
                // Record current disk mtime so we don't re-prompt until next change
                let path = app.current_file().clone();
                app.session.record_file_mtime(&path);
                app.status_message = None;
                return Ok(InputResult::Continue);
            }
            _ => {
                // Other keys dismiss the prompt silently (ignore the change)
                app.external_modification_pending = false;
                let path = app.current_file().clone();
                app.session.record_file_mtime(&path);
                app.status_message = None;
                // Fall through to normal handling
            }
        }
    }

    // Note: No timeout on pending commands (vim-like behavior - wait indefinitely)

    // Handle pending multi-key sequences
    if let Some(pending) = app.input_state.pending_command.clone() {
        return crate::input::normal_mode::multi_key::handle(app, pending, key.code);
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

        // Clear search highlighting with Esc
        KeyCode::Esc if app.search_state.is_some() => {
            app.search_state = None;
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

        // G command: go to last row, or {count}G to go to specific line
        KeyCode::Char('G') if is_navigation_allowed(app) => {
            if let Some(count) = app.input_state.command_count.take() {
                navigation::commands::goto_line(app, count.get());
            } else {
                navigation::commands::goto_last_row(app);
            }
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

        // Enter search mode
        KeyCode::Char('/') if is_navigation_allowed(app) => {
            app.search_buffer.clear();
            app.mode = Mode::Search;
            return Ok(InputResult::Continue);
        }

        // Next search match
        KeyCode::Char('n') if is_navigation_allowed(app) => {
            let cursor_row = app.selected_row().unwrap_or(RowIndex::new(0));
            let cursor_col = app.view_state.selected_column;
            if let Some(ref mut state) = app.search_state {
                if let Some(((row, col), wrapped)) = state.jump_to_next(cursor_row, cursor_col) {
                    app.view_state.table_state.select(Some(row.get()));
                    app.view_state.selected_column = col;
                    let pos = state.display_position();
                    let pattern = state.pattern.clone();
                    if wrapped {
                        app.status_message = Some(StatusMessage::new_owned(format!(
                            "search hit BOTTOM, continuing at TOP  /{} {}",
                            pattern, pos
                        )));
                    } else {
                        app.status_message =
                            Some(StatusMessage::new_owned(format!("/{} {}", pattern, pos)));
                    }
                }
            }
            return Ok(InputResult::Continue);
        }

        // Previous search match
        KeyCode::Char('N') if is_navigation_allowed(app) => {
            let cursor_row = app.selected_row().unwrap_or(RowIndex::new(0));
            let cursor_col = app.view_state.selected_column;
            if let Some(ref mut state) = app.search_state {
                if let Some(((row, col), wrapped)) = state.jump_to_prev(cursor_row, cursor_col) {
                    app.view_state.table_state.select(Some(row.get()));
                    app.view_state.selected_column = col;
                    let pos = state.display_position();
                    let pattern = state.pattern.clone();
                    if wrapped {
                        app.status_message = Some(StatusMessage::new_owned(format!(
                            "search hit TOP, continuing at BOTTOM  /{} {}",
                            pattern, pos
                        )));
                    } else {
                        app.status_message =
                            Some(StatusMessage::new_owned(format!("/{} {}", pattern, pos)));
                    }
                }
            }
            return Ok(InputResult::Continue);
        }

        // Search for current cell content (vim *)
        KeyCode::Char('*') if is_navigation_allowed(app) => {
            let cursor_row = app.selected_row().unwrap_or(RowIndex::new(0));
            let cursor_col = app.view_state.selected_column;
            let cell_content = app.document.cell(cursor_row, cursor_col).to_string();

            if !cell_content.is_empty() {
                let matches = crate::search::find_matches(&app.document, &cell_content);
                if !matches.is_empty() {
                    let mut state = crate::search::SearchState::new(cell_content.clone(), matches);
                    // Jump to next match (skips the current cell)
                    if let Some(((row, col), _wrapped)) = state.jump_to_next(cursor_row, cursor_col)
                    {
                        app.view_state.table_state.select(Some(row.get()));
                        app.view_state.selected_column = col;
                        app.status_message = Some(StatusMessage::new_owned(format!(
                            "/{} {}",
                            cell_content,
                            state.display_position()
                        )));
                    }
                    app.search_state = Some(state);
                } else {
                    app.status_message = Some(StatusMessage::new_owned(format!(
                        "Pattern not found: {}",
                        cell_content
                    )));
                }
            }
            return Ok(InputResult::Continue);
        }

        // Enter Visual Block mode (v)
        KeyCode::Char('v') if is_navigation_allowed(app) => {
            let row = app.selected_row().unwrap_or(RowIndex::new(0));
            let col = app.view_state.selected_column;
            app.visual_selection = Some(VisualSelection::new(row, col, VisualMode::Block));
            app.mode = Mode::VisualBlock;
            return Ok(InputResult::Continue);
        }

        // Enter Visual Line mode (V)
        KeyCode::Char('V') if is_navigation_allowed(app) => {
            let row = app.selected_row().unwrap_or(RowIndex::new(0));
            let col = app.view_state.selected_column;
            app.visual_selection = Some(VisualSelection::new(row, col, VisualMode::Line));
            app.mode = Mode::VisualLine;
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
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Insert mode: 'a' - edit cell, cursor at end (same as 'i' for cells)
        KeyCode::Char('a') if is_navigation_allowed(app) => {
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Insert mode: 'I' - edit cell, cursor at start
        KeyCode::Char('I') if is_navigation_allowed(app) => {
            enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
        }

        // Insert mode: 'A' - edit cell, cursor at end (same as 'i')
        KeyCode::Char('A') if is_navigation_allowed(app) => {
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Insert mode: 's' - replace cell (clear + edit)
        KeyCode::Char('s') if is_navigation_allowed(app) => {
            enter_insert_mode(app, CursorPosition::Start, InitialContent::Clear);
        }

        // Insert mode: F2 - edit cell (same as 'i')
        KeyCode::F(2) if is_navigation_allowed(app) => {
            enter_insert_mode(app, CursorPosition::End, InitialContent::Keep);
        }

        // Magnifier mode: 'm' - open magnifier for complex cell editing
        KeyCode::Char('m') if is_navigation_allowed(app) => {
            app.open_magnifier();
        }

        // Row operations: 'o' - add row below and enter Insert mode
        KeyCode::Char('o') if is_navigation_allowed(app) => {
            if let Some(row_idx) = app.selected_row() {
                let new_row_idx = RowIndex::new(row_idx.get() + 1);
                app.document.insert_row(new_row_idx);
                app.view_state.table_state.select(Some(new_row_idx.get()));
                enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
            }
        }

        // Row operations: 'O' - add row above and enter Insert mode
        KeyCode::Char('O') if is_navigation_allowed(app) => {
            if let Some(row_idx) = app.selected_row() {
                app.document.insert_row(row_idx);
                // Selection stays at current index which is now the new row
                enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
            }
        }

        // Comma leader - start column command sequence
        KeyCode::Char(',') if is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Comma);
            return Ok(InputResult::Continue);
        }

        // Row operations: 'P' - paste row(s) above
        KeyCode::Char('P') if is_navigation_allowed(app) => {
            if let Some(rows) = app.clipboard.rows() {
                if let Some(row_idx) = app.selected_row() {
                    let pasted_count = rows.len();
                    for (i, clipboard_row) in rows.iter().enumerate() {
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
            if let Some(rows) = app.clipboard.rows() {
                if let Some(row_idx) = app.selected_row() {
                    let pasted_count = rows.len();
                    for (i, clipboard_row) in rows.iter().enumerate() {
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
            if let Some(row_idx) = app.selected_row() {
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

/// Handle count prefix (numeric digits for commands like 5j, 10G)
fn handle_count_prefix(app: &mut App, digit: char) -> Result<InputResult> {
    let digit_value = digit
        .to_digit(10)
        .expect("digit validated by is_ascii_digit") as usize;

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
