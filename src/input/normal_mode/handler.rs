//! Main handler for Normal mode input
//!
//! This module is the primary dispatcher for Normal mode keyboard input.

use crate::app::{messages, App};
use crate::input::{InputResult, PendingCommand, StatusMessage};
use crate::navigation;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::num::NonZeroUsize;

use super::{editing, help, mode_transitions, navigation as nav, search, visual_mode};

/// Maximum command count to prevent overflow
const MAX_COMMAND_COUNT: usize = 100000;

/// Handle keyboard input in Normal mode.
///
/// Does the keymap pre-pass first (so user-defined bindings in
/// `keys.toml` win), then falls through to the legacy match-based handler
/// for chords and anything not bound in the keymap.
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // Clear transient messages on keypress
    if let Some(ref msg) = app.status_message {
        if msg.should_clear_on_keypress() {
            app.status_message = None;
        }
    }

    // Try the keymap before running any of the legacy match arms — but
    // skip it when we're already in the middle of something that needs
    // priority routing (pending chord, count prefix, external-mod prompt,
    // help overlay blocking).
    if !app.external_modification_pending
        && app.input_state.pending_command.is_none()
        && app.input_state.command_count.is_none()
        && super::help::is_navigation_allowed(app)
    {
        if let Some(result) = crate::input::keymap_dispatch::try_keymap(
            app,
            key,
            crate::config::keys::KeymapScope::Normal,
            handle_raw,
        )? {
            return Ok(result);
        }
    }

    handle_raw(app, key)
}

/// The legacy match-based normal-mode handler. Called by [`handle`] after
/// the keymap pre-pass, and re-entered by `keymap_dispatch::execute` when
/// it synthesizes a key for an action.
pub fn handle_raw(app: &mut App, key: KeyEvent) -> Result<InputResult> {
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

    // Handle help overlay keys first (they take precedence)
    // This blocks all non-vim-navigation keys when help is visible
    if help::handle_key(app, key.code, key.modifiers) {
        return Ok(InputResult::Continue);
    }

    // Handle numeric prefixes only when navigation is allowed
    if help::is_navigation_allowed(app) {
        if let KeyCode::Char(c) = key.code {
            if c.is_numeric() && (c != '0' || app.input_state.command_count.is_some()) {
                return handle_count_prefix(app, c);
            }
        }
    }

    match key.code {
        // Space - start Space+key sequence
        KeyCode::Char(' ') if help::is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Space);
            return Ok(InputResult::Continue);
        }

        // Toggle help overlay (always allow, even when help is visible)
        KeyCode::Char('?') => {
            help::toggle(app);
        }

        // Clear pending command with Esc
        KeyCode::Esc if app.input_state.pending_command.is_some() => {
            app.input_state.clear_pending_command();
            app.status_message = Some(StatusMessage::from(messages::CMD_CANCELLED));
        }

        // Clear search highlighting with Esc
        KeyCode::Esc if app.search_state.is_some() => {
            search::clear_search(app);
        }

        // Start multi-key sequences
        KeyCode::Char('g') if help::is_navigation_allowed(app) => {
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
        KeyCode::Char('G') if help::is_navigation_allowed(app) => {
            if let Some(count) = app.input_state.command_count.take() {
                navigation::commands::goto_line(app, count.get());
            } else {
                navigation::commands::goto_last_row(app);
            }
            return Ok(InputResult::Continue);
        }

        KeyCode::Char('z') if help::is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Z);
            return Ok(InputResult::Continue);
        }

        // Enter command mode
        KeyCode::Char(':') if help::is_navigation_allowed(app) => {
            mode_transitions::enter_command_mode(app);
            return Ok(InputResult::Continue);
        }

        // Enter search mode
        KeyCode::Char('/') if help::is_navigation_allowed(app) => {
            mode_transitions::enter_search_mode(app);
            return Ok(InputResult::Continue);
        }

        // Next search match
        KeyCode::Char('n') if help::is_navigation_allowed(app) => {
            search::next_match(app);
            return Ok(InputResult::Continue);
        }

        // Previous search match
        KeyCode::Char('N') if help::is_navigation_allowed(app) => {
            search::prev_match(app);
            return Ok(InputResult::Continue);
        }

        // Search for current cell content (vim *)
        KeyCode::Char('*') if help::is_navigation_allowed(app) => {
            search::search_current_cell(app);
            return Ok(InputResult::Continue);
        }

        // Enter Visual Block mode (v)
        KeyCode::Char('v') if help::is_navigation_allowed(app) => {
            visual_mode::enter_block_mode(app);
            return Ok(InputResult::Continue);
        }

        // Enter Visual Line mode (V)
        KeyCode::Char('V') if help::is_navigation_allowed(app) => {
            visual_mode::enter_line_mode(app);
            return Ok(InputResult::Continue);
        }

        // Start 'd' pending command (for dd - delete row)
        KeyCode::Char('d') if help::is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::D);
            return Ok(InputResult::Continue);
        }

        // Start 'y' pending command (for yy - yank row)
        KeyCode::Char('y') if help::is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Y);
            return Ok(InputResult::Continue);
        }

        // Start 'c' pending command (for cc - clear row, cw - copy cell)
        KeyCode::Char('c') if help::is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::C);
            return Ok(InputResult::Continue);
        }

        // Insert mode: 'i' - edit cell, cursor at end
        KeyCode::Char('i') if help::is_navigation_allowed(app) => {
            mode_transitions::insert_at_end(app);
        }

        // Insert mode: 'a' - edit cell, cursor at end (same as 'i' for cells)
        KeyCode::Char('a') if help::is_navigation_allowed(app) => {
            mode_transitions::append_at_end(app);
        }

        // Insert mode: 'I' - edit cell, cursor at start
        KeyCode::Char('I') if help::is_navigation_allowed(app) => {
            mode_transitions::insert_at_start(app);
        }

        // Insert mode: 'A' - edit cell, cursor at end (same as 'i')
        KeyCode::Char('A') if help::is_navigation_allowed(app) => {
            mode_transitions::append_at_line_end(app);
        }

        // Insert mode: 's' or 'r' - replace cell (clear + edit)
        KeyCode::Char('s') | KeyCode::Char('r')
            if help::is_navigation_allowed(app)
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            mode_transitions::substitute_cell(app);
        }

        // Insert mode: F2 - edit cell (same as 'i')
        KeyCode::F(2) if help::is_navigation_allowed(app) => {
            mode_transitions::edit_with_f2(app);
        }

        // Row operations: 'o' - add row below and enter Insert mode
        KeyCode::Char('o') if help::is_navigation_allowed(app) => {
            editing::insert_row_below(app);
        }

        // Row operations: 'O' - add row above and enter Insert mode
        KeyCode::Char('O') if help::is_navigation_allowed(app) => {
            editing::insert_row_above(app);
        }

        // Column width: '+' - increase current column width by 2
        KeyCode::Char('+') if help::is_navigation_allowed(app) => {
            let col = app.view_state.selected_column.get();
            let current = app.session.column_width(col).unwrap_or(15);
            app.session
                .set_column_width(col, current.saturating_add(2).min(200));
            crate::config::views::save_current_views(app);
            return Ok(InputResult::Continue);
        }

        // Column width: '-' - decrease current column width by 2
        KeyCode::Char('-') if help::is_navigation_allowed(app) => {
            let col = app.view_state.selected_column.get();
            let current = app.session.column_width(col).unwrap_or(15);
            app.session
                .set_column_width(col, current.saturating_sub(2).max(4));
            crate::config::views::save_current_views(app);
            return Ok(InputResult::Continue);
        }

        // Comma leader - start column command sequence
        KeyCode::Char(',') if help::is_navigation_allowed(app) => {
            app.input_state.set_pending_command(PendingCommand::Comma);
            return Ok(InputResult::Continue);
        }

        // Row operations: 'P' - paste row(s) above
        KeyCode::Char('P') if help::is_navigation_allowed(app) => {
            editing::paste_rows_above(app);
        }

        // Paste: 'p' - paste cell value (if cw was used) or row(s) below
        KeyCode::Char('p') if help::is_navigation_allowed(app) => {
            if let Some(value) = app.clipboard.cell().map(|s| s.to_string()) {
                // Cell paste
                if let Some(row_idx) = app.selected_row() {
                    let col_idx = app.view_state.selected_column;
                    let old_value = app.document.cell(row_idx, col_idx);
                    app.document.set_cell(row_idx, col_idx, value.clone());
                    app.history.push(crate::history::EditCommand::SetCell {
                        row: row_idx,
                        col: col_idx,
                        old_value,
                        new_value: value.clone(),
                    });
                    app.status_message = Some(StatusMessage::from(format!("Pasted: {}", value)));
                }
            } else {
                editing::paste_rows_below(app);
            }
        }

        // ~ - Toggle case of current cell
        KeyCode::Char('~') if help::is_navigation_allowed(app) => {
            if let Some(row) = app.selected_row() {
                let col = app.view_state.selected_column;
                let old = app.document.cell(row, col);
                let new_value = crate::transforms::toggle_case(&old);
                if new_value != old {
                    app.document.set_cell(row, col, new_value.clone());
                    app.history.push(crate::history::EditCommand::SetCell {
                        row,
                        col,
                        old_value: old,
                        new_value,
                    });
                }
            }
        }

        // Delete key - clear current cell
        KeyCode::Delete if help::is_navigation_allowed(app) => {
            editing::clear_cell(app);
        }

        // Enter key - move down one row (like j)
        KeyCode::Enter if help::is_navigation_allowed(app) => {
            nav::move_down(app);
        }

        // Page navigation: Ctrl+d - page down
        KeyCode::Char('d')
            if help::is_navigation_allowed(app)
                && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            nav::page_down(app);
        }

        // Dot repeat: . — repeat last cell edit at current cursor
        KeyCode::Char('.') if help::is_navigation_allowed(app) => {
            if let Some(ref last) = app.last_edit.clone() {
                match last {
                    crate::history::EditCommand::SetCell { new_value, .. } => {
                        if let Some(row) = app.selected_row() {
                            let col = app.view_state.selected_column;
                            app.commit_cell_value(row, col, new_value.clone());
                            app.status_message =
                                Some(StatusMessage::from("Repeated last edit".to_string()));
                        }
                    }
                    crate::history::EditCommand::InsertRow { .. } => {
                        editing::insert_row_below(app);
                    }
                    crate::history::EditCommand::DeleteRow { .. }
                    | crate::history::EditCommand::DeleteRows { .. } => {
                        super::commands::delete_rows(app);
                    }
                    _ => {
                        app.status_message =
                            Some(StatusMessage::from("No repeatable edit".to_string()));
                    }
                }
            } else {
                app.status_message = Some(StatusMessage::from("No previous edit".to_string()));
            }
        }

        // Undo: u
        KeyCode::Char('u')
            if help::is_navigation_allowed(app)
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            if app.history.undo(&mut app.document) {
                app.status_message = Some(StatusMessage::from("Undone".to_string()));
            } else {
                app.status_message =
                    Some(StatusMessage::from("Already at oldest change".to_string()));
            }
        }

        // Redo: Ctrl+r
        KeyCode::Char('r')
            if help::is_navigation_allowed(app)
                && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            if app.history.redo(&mut app.document) {
                app.status_message = Some(StatusMessage::from("Redone".to_string()));
            } else {
                app.status_message =
                    Some(StatusMessage::from("Already at newest change".to_string()));
            }
        }

        // Page navigation: Ctrl+u - page up
        KeyCode::Char('u')
            if help::is_navigation_allowed(app)
                && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            nav::page_up(app);
        }

        // Macro: `q` — toggles recording (stops if active, otherwise prompts for register)
        KeyCode::Char('q')
            if help::is_navigation_allowed(app)
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            if app.macros.is_recording() {
                if let Some(reg) = app.macros.stop_recording() {
                    app.status_message =
                        Some(StatusMessage::from(format!("Recorded into @{}", reg)));
                }
            } else {
                app.input_state
                    .set_pending_command(crate::input::PendingCommand::Q);
                app.status_message = Some(StatusMessage::from(
                    "Recording... press a-z to choose register",
                ));
            }
            return Ok(InputResult::Continue);
        }

        // Macro: `@` — replay (waits for register letter or `@` for last)
        KeyCode::Char('@') if help::is_navigation_allowed(app) => {
            app.input_state
                .set_pending_command(crate::input::PendingCommand::At);
            return Ok(InputResult::Continue);
        }

        // Shift+Arrow: start/extend visual block selection
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
            if help::is_navigation_allowed(app) && key.modifiers.contains(KeyModifiers::SHIFT) =>
        {
            // Enter visual block mode with current cell as anchor
            visual_mode::enter_block_mode(app);
            // Delegate the arrow key to the visual mode handler to extend selection
            return crate::input::visual_mode::handle(app, key);
        }

        // Navigation commands
        _ if help::is_navigation_allowed(app) => {
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
