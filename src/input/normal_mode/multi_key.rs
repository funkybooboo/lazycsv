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

use crate::app::App;
use crate::input::actions::{InputResult, StatusMessage};
use crate::input::PendingCommand;
use crate::navigation;
use anyhow::Result;
use crossterm::event::KeyCode;

use super::commands;

/// Handle multi-key command sequences
pub fn handle(app: &mut App, first: PendingCommand, second: KeyCode) -> Result<InputResult> {
    match (&first, second) {
        // gg - Go to first row
        (PendingCommand::G, KeyCode::Char('g')) => {
            commands::goto_first_row(app);
        }

        // gv - Reselect last visual selection
        (PendingCommand::G, KeyCode::Char('v')) => {
            commands::reselect_visual(app);
        }

        // g~ - Title case current cell
        (PendingCommand::G, KeyCode::Char('~')) => {
            app.input_state.clear_pending_command();
            if let Some(row) = app.selected_row() {
                let col = app.view_state.selected_column;
                let old = app.document.cell(row, col);
                let new_value = crate::transforms::to_title(&old);
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

        // g. - Toggle boolean value
        (PendingCommand::G, KeyCode::Char('.')) => {
            app.input_state.clear_pending_command();
            if let Some(row) = app.selected_row() {
                let col = app.view_state.selected_column;
                let old = app.document.cell(row, col);
                if let Some(new_value) = crate::transforms::toggle_boolean(&old) {
                    app.document.set_cell(row, col, new_value.clone());
                    app.history.push(crate::history::EditCommand::SetCell {
                        row,
                        col,
                        old_value: old,
                        new_value,
                    });
                } else {
                    app.status_message = Some(StatusMessage::from(
                        "Not a boolean value (true/false, yes/no, 1/0, on/off)".to_string(),
                    ));
                }
            }
        }

        // gj - Swap current row with row below
        (PendingCommand::G, KeyCode::Char('j')) => {
            app.input_state.clear_pending_command();
            if let Some(row) = app.selected_row() {
                let next = crate::domain::position::RowIndex::new(row.get() + 1);
                if next.get() < app.document.row_count() {
                    app.document.swap_rows(row, next);
                    app.history
                        .push(crate::history::EditCommand::SwapRows { a: row, b: next });
                    app.view_state.table_state.select(Some(next.get()));
                }
            }
        }

        // gk - Swap current row with row above
        (PendingCommand::G, KeyCode::Char('k')) => {
            app.input_state.clear_pending_command();
            if let Some(row) = app.selected_row() {
                if row.get() > 1 {
                    // Don't swap with header (row 0)
                    let prev = crate::domain::position::RowIndex::new(row.get() - 1);
                    app.document.swap_rows(row, prev);
                    app.history
                        .push(crate::history::EditCommand::SwapRows { a: row, b: prev });
                    app.view_state.table_state.select(Some(prev.get()));
                }
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
            commands::viewport_top(app);
        }

        // zz - Center of screen
        (PendingCommand::Z, KeyCode::Char('z')) => {
            commands::viewport_center(app);
        }

        // zb - Bottom of screen
        (PendingCommand::Z, KeyCode::Char('b')) => {
            commands::viewport_bottom(app);
        }

        // dd - Delete row(s) with optional count prefix
        (PendingCommand::D, KeyCode::Char('d')) => {
            commands::delete_rows(app);
        }

        // yy - Yank (copy) row(s) with optional count prefix
        (PendingCommand::Y, KeyCode::Char('y')) => {
            commands::yank_rows(app);
        }

        // cc - Clear row and enter insert mode
        (PendingCommand::C, KeyCode::Char('c')) => {
            commands::change_row(app);
        }

        // cw - Copy cell value ("copy word") to internal + system clipboard
        (PendingCommand::C, KeyCode::Char('w')) => {
            app.input_state.clear_pending_command();
            if let Some(row_idx) = app.selected_row() {
                let col_idx = app.view_state.selected_column;
                let value = app.document.cell(row_idx, col_idx);
                app.clipboard.yank_cell(value.clone());
                // Also copy to system clipboard
                let _ = copy_to_system_clipboard(&value);
                app.status_message = Some(StatusMessage::from(format!("Copied: {}", value)));
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
            commands::enter_visual_column_mode(app);
            return Ok(InputResult::Continue);
        }

        // ,p - paste column(s) to the right of current column
        (PendingCommand::Comma, KeyCode::Char('p')) => {
            commands::paste_columns_after(app);
        }

        // ,P - paste column(s) to the left of current column
        (PendingCommand::Comma, KeyCode::Char('P')) => {
            commands::paste_columns_before(app);
        }

        // ,o - insert empty column to the right
        (PendingCommand::Comma, KeyCode::Char('o')) => {
            commands::insert_column_after(app);
        }

        // ,O - insert empty column to the left
        (PendingCommand::Comma, KeyCode::Char('O')) => {
            commands::insert_column_before(app);
        }

        // ,dd - delete column(s) with optional count prefix
        (PendingCommand::CommaD, KeyCode::Char('d')) => {
            commands::delete_columns(app);
        }

        // ,yy - yank column(s) with optional count prefix
        (PendingCommand::CommaY, KeyCode::Char('y')) => {
            commands::yank_columns(app);
        }

        // Space+f - Open files menu
        (PendingCommand::Space, KeyCode::Char('f')) => {
            use crate::app::Mode;
            app.input_state.clear_pending_command();
            app.mode = Mode::FileList;
            app.input_state.clear_file_filter();
            app.status_message = Some(StatusMessage::from(
                "Type to filter, number to select, Enter for first, Esc to cancel",
            ));
        }

        // Space+q - Open SQL query editor
        (PendingCommand::Space, KeyCode::Char('q')) => {
            app.input_state.clear_pending_command();
            super::mode_transitions::enter_sql_editor(app);
        }

        // Space+m - Open magnifier
        (PendingCommand::Space, KeyCode::Char('m')) => {
            app.input_state.clear_pending_command();
            super::mode_transitions::enter_magnifier(app);
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
        PendingCommand::Space => "Space".to_string(),
    }
}

/// Copy text to the system clipboard (best-effort, ignores errors).
fn copy_to_system_clipboard(text: &str) -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

    #[cfg(target_os = "linux")]
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
        .or_else(|_| {
            Command::new("xsel")
                .args(["--clipboard", "--input"])
                .stdin(Stdio::piped())
                .spawn()
        })?;

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Ok(());

    child
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Failed to open clipboard stdin"))?
        .write_all(text.as_bytes())?;
    child.wait()?;
    Ok(())
}
