//! Input handling and keyboard event processing

use crate::app::{messages, App, EditBuffer, Mode};
use crate::domain::position::RowIndex;
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
    }
}

/// Handle keyboard input events
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Command => handle_command_mode(app, key),
        Mode::Insert => handle_insert_mode(app, key),
        // TODO: Implement handlers for new modes in v0.5.0+
        Mode::Magnifier | Mode::HeaderEdit | Mode::Visual => {
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

/// Handle quit command with unsaved changes check
fn handle_quit(app: &mut App) {
    if app.document.is_dirty {
        app.status_message = Some(StatusMessage::from(messages::UNSAVED_CHANGES));
    } else {
        app.should_quit = true;
    }
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
        // Quit command
        KeyCode::Char('q') if is_navigation_allowed(app) => {
            handle_quit(app);
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

        // Row operations: 'p' - paste row below
        KeyCode::Char('p') if is_navigation_allowed(app) => {
            if let Some(clipboard) = app.row_clipboard.clone() {
                if let Some(row_idx) = app.get_selected_row() {
                    let new_row_idx = RowIndex::new(row_idx.get() + 1);
                    app.document.insert_row(new_row_idx);
                    // Copy clipboard content into the new row
                    for (col_idx, value) in clipboard.iter().enumerate() {
                        if col_idx < app.document.column_count() {
                            app.document.set_cell(
                                new_row_idx,
                                crate::domain::position::ColIndex::new(col_idx),
                                value.clone(),
                            );
                        }
                    }
                    app.view_state.table_state.select(Some(new_row_idx.get()));
                    app.status_message = Some(StatusMessage::from("Pasted 1 row"));
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

        // dd - Delete row
        (PendingCommand::D, KeyCode::Char('d')) => {
            app.input_state.clear_pending_command();
            if let Some(row_idx) = app.get_selected_row() {
                if let Some(deleted) = app.document.delete_row(row_idx) {
                    app.row_clipboard = Some(deleted);
                    // Adjust selection if needed
                    let row_count = app.document.row_count();
                    if row_count == 0 {
                        // No rows left
                        app.view_state.table_state.select(None);
                    } else if row_idx.get() >= row_count {
                        // Was at last row, move selection up
                        app.view_state.table_state.select(Some(row_count - 1));
                    }
                    // Otherwise selection stays at same index (which is now the next row)
                    app.status_message = Some(StatusMessage::from("1 row deleted"));
                }
            }
        }

        // yy - Yank (copy) row
        (PendingCommand::Y, KeyCode::Char('y')) => {
            app.input_state.clear_pending_command();
            if let Some(row_idx) = app.get_selected_row() {
                if let Some(row) = app.document.rows.get(row_idx.get()) {
                    app.row_clipboard = Some(row.clone());
                    app.status_message = Some(StatusMessage::from("1 row yanked"));
                }
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
            app.mode = Mode::Normal;
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

/// Execute command from command buffer
fn execute_command(app: &mut App) -> Result<()> {
    let cmd = app.input_state.command_buffer.trim().to_string();

    if cmd.is_empty() {
        return Ok(());
    }

    // Split command into parts for commands with arguments
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let cmd_name = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim());

    // Reserved commands (take priority)
    match cmd_name.as_str() {
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
            app.should_quit = true;
            return Ok(());
        }
        "w" | "write" => {
            // TODO: Implement save in v0.7.0
            app.status_message = Some(StatusMessage::from("Save not yet implemented"));
            return Ok(());
        }
        "wq" | "x" => {
            // TODO: Implement save and quit in v0.7.0
            app.status_message = Some(StatusMessage::from("Save not yet implemented"));
            return Ok(());
        }
        "h" | "help" => {
            app.status_message = Some(StatusMessage::from("Press ? for help"));
            return Ok(());
        }
        "c" => {
            // Column jump: :c A, :c 17, :c AA
            if let Some(col_arg) = arg {
                if let Ok(col_num) = col_arg.parse::<usize>() {
                    // Numeric column (1-indexed)
                    if col_num == 0 {
                        app.status_message =
                            Some(StatusMessage::from("Column number must be >= 1"));
                    } else {
                        navigation::commands::goto_column_by_number(app, col_num);
                    }
                } else if col_arg.chars().all(|c| c.is_ascii_alphabetic()) {
                    // Letter column (A, B, AA, etc.)
                    navigation::commands::goto_column(app, col_arg);
                } else {
                    app.status_message =
                        Some(StatusMessage::from(format!("Invalid column: {}", col_arg)));
                }
            } else {
                app.status_message =
                    Some(StatusMessage::from("Usage: :c <column> (e.g., :c A, :c 5)"));
            }
            return Ok(());
        }
        _ => {}
    }

    // Try to parse entire command as number (row jump: :15)
    if let Ok(line_num) = cmd.parse::<usize>() {
        navigation::commands::goto_line(app, line_num);
        app.status_message = Some(StatusMessage::from(format!("Jumped to row {}", line_num)));
        return Ok(());
    }

    // Unknown command
    app.status_message = Some(StatusMessage::from(format!("Unknown command: :{}", cmd)));
    Ok(())
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
