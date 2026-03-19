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
