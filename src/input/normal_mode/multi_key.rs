//! Multi-key command handlers for normal mode
//!
//! Handles sequences like:
//! - gg (go to first row)
//! - gA, gAA (go to column A, AA)
//! - zt, zz, zb (viewport positioning)
//! - dd, yy, cc (delete, yank, change row)
//! - ,dd, ,yy (delete, yank column)
//! - ,p, ,P (paste column)
//! - ,o, ,O (insert empty column)
//! - ,v (visual column mode)

use crate::app::{App, Mode, VisualMode, VisualSelection};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::actions::{InputResult, StatusMessage};
use crate::input::PendingCommand;
use crate::navigation;
use crate::ui::ViewportMode;
use anyhow::Result;
use crossterm::event::KeyCode;

use super::super::handler::{enter_insert_mode, CursorPosition, InitialContent};

/// Handle multi-key command sequences
pub fn handle(app: &mut App, first: PendingCommand, second: KeyCode) -> Result<InputResult> {
    match (&first, second) {
        // gg - Go to first row
        (PendingCommand::G, KeyCode::Char('g')) => {
            app.input_state.clear_pending_command();
            navigation::goto_first_row(app);
            app.status_message = Some(StatusMessage::from("Jumped to first row"));
        }

        // gh - Go to header row (row 0)
        (PendingCommand::G, KeyCode::Char('h')) => {
            app.input_state.clear_pending_command();
            if !app.document.header_mode {
                app.status_message = Some(StatusMessage::from(
                    "Header mode is OFF (use :ht to enable)",
                ));
            } else if app.document.row_count() == 0 {
                app.status_message = Some(StatusMessage::from("Empty document"));
            } else {
                // Move to row 0 (header row), keeping current column
                app.view_state.table_state.select(Some(0));
                app.status_message = Some(StatusMessage::from("Moved to header row"));
            }
        }

        // gd - Go to first data row (row 1)
        (PendingCommand::G, KeyCode::Char('d')) => {
            app.input_state.clear_pending_command();
            if app.document.row_count() <= 1 {
                app.status_message = Some(StatusMessage::from("No data rows"));
            } else {
                // Move to row 1 (first data row)
                app.view_state.table_state.select(Some(1));
                app.status_message = Some(StatusMessage::from("Moved to first data row"));
            }
        }

        // gv - Reselect last visual selection
        (PendingCommand::G, KeyCode::Char('v')) => {
            app.input_state.clear_pending_command();
            if let Some(last_sel) = app.last_visual_selection {
                // Restore the selection
                app.visual_selection = Some(last_sel);
                // Enter the appropriate visual mode
                app.mode = match last_sel.mode {
                    VisualMode::Block => Mode::VisualBlock,
                    VisualMode::Line => Mode::VisualLine,
                    VisualMode::Column => Mode::VisualColumn,
                };
                // Move cursor to the selection cursor position
                app.view_state
                    .table_state
                    .select(Some(last_sel.cursor.0.get()));
                app.view_state.selected_column = last_sel.cursor.1;
                app.status_message = Some(StatusMessage::from("Reselected"));
            } else {
                app.status_message = Some(StatusMessage::from("No previous visual selection"));
            }
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
            if let Some(letters) = first.column_letters() {
                navigation::commands::goto_column(app, letters);
            }
        }

        // zt - Top of screen
        (PendingCommand::Z, KeyCode::Char('t')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Top;
            app.status_message = Some(StatusMessage::from("Viewport: top"));
        }

        // zz - Center of screen
        (PendingCommand::Z, KeyCode::Char('z')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Center;
            app.status_message = Some(StatusMessage::from("Viewport: center"));
        }

        // zb - Bottom of screen
        (PendingCommand::Z, KeyCode::Char('b')) => {
            app.input_state.clear_pending_command();
            app.view_state.viewport_mode = ViewportMode::Bottom;
            app.status_message = Some(StatusMessage::from("Viewport: bottom"));
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
            if let Some(row_idx) = app.selected_row() {
                // If deleting row 0 (header), turn header_mode OFF
                if row_idx.get() == 0 {
                    app.document.header_mode = false;
                    app.session.set_header_mode(false);
                }

                let end_idx = RowIndex::new(row_idx.get() + count - 1);
                let deleted = app.document.delete_rows(row_idx, end_idx);
                let deleted_count = deleted.len();
                if deleted_count > 0 {
                    app.clipboard.yank_rows(deleted);
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
            if let Some(row_idx) = app.selected_row() {
                let end_idx = RowIndex::new(
                    (row_idx.get() + count - 1).min(app.document.row_count().saturating_sub(1)),
                );
                let rows = app.document.rows_range(row_idx, end_idx);
                let yanked_count = rows.len();
                if yanked_count > 0 {
                    app.clipboard.yank_rows(rows);
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
            if let Some(row_idx) = app.selected_row() {
                // Clear all cells in the row
                let col_count = app.document.column_count();
                for col in 0..col_count {
                    app.document
                        .set_cell(row_idx, ColIndex::new(col), String::new());
                }
                // Move cursor to first column
                app.view_state.selected_column = ColIndex::new(0);
                // Enter insert mode
                enter_insert_mode(app, CursorPosition::Start, InitialContent::Keep);
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

        // ,v - enter Visual Column mode
        (PendingCommand::Comma, KeyCode::Char('v')) => {
            app.input_state.clear_pending_command();
            let row = app.selected_row().unwrap_or(RowIndex::new(0));
            let col = app.view_state.selected_column;
            app.visual_selection = Some(VisualSelection::new(row, col, VisualMode::Column));
            app.mode = Mode::VisualColumn;
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
            let columns = app.document.columns_range(col_idx, end_idx);
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
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown command: {}{}",
                format_pending_command(&first),
                format_keycode(&second)
            )));
        }
    }

    Ok(InputResult::Continue)
}

/// Format a KeyCode in a user-friendly way
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
