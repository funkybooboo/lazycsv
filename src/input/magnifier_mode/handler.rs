//! Main keyboard input handlers for magnifier mode
//!
//! Dispatches input events to the appropriate magnifier sub-mode handler.

use crate::app::{App, Mode};
use crate::domain::position::ColIndex;
use crate::input::actions::{InputResult, StatusMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Main entry point for magnifier mode input handling
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    use crate::magnifier::MagnifierMode;

    let mag = match app.magnifier_state.as_mut() {
        Some(m) => m,
        None => {
            // No magnifier state - return to normal mode
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
    };

    // Check for Alt+hjkl and Alt+arrows navigation (works in both Normal and Insert modes within magnifier)
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => {
                return handle_navigate(app, Direction::Left);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                return handle_navigate(app, Direction::Down);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                return handle_navigate(app, Direction::Up);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                return handle_navigate(app, Direction::Right);
            }
            _ => {}
        }
    }

    match mag.mode() {
        MagnifierMode::Normal => handle_normal(app, key),
        MagnifierMode::Insert => handle_insert(app, key),
        MagnifierMode::Command => handle_command(app, key),
        MagnifierMode::Visual | MagnifierMode::VisualLine => handle_visual(app, key),
    }
}

/// Direction for cell navigation in magnifier
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Handle navigation to adjacent cells from magnifier (Alt+hjkl or Alt+arrows)
fn handle_navigate(app: &mut App, direction: Direction) -> Result<InputResult> {
    // Check if magnifier has unsaved changes
    if app.magnifier_is_dirty() {
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
fn handle_normal(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    use crate::input::magnifier_mode::{mode_changes, motions, operators, pending, search};

    let mag = match app.magnifier_state.as_mut() {
        Some(m) => m,
        None => return Ok(InputResult::Continue),
    };

    // Handle pending commands first
    if let Some(pending_cmd) = mag.take_pending() {
        let should_close = handle_pending_command(mag, pending_cmd, key.code);
        if should_close {
            app.save_and_close_magnifier();
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

    // Try helper modules for common operations
    if motions::handle_motion_command(mag, key.code)
        || operators::handle_operator_command(mag, key.code)
        || mode_changes::handle_mode_change_command(mag, key.code)
        || search::handle_search_command(mag, key.code)
        || pending::handle_pending_setup(mag, key.code)
    {
        return Ok(InputResult::Continue);
    }

    // Handle remaining special commands
    match key.code {
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

/// Handle pending command completion (multi-key sequences)
/// Returns true if the magnifier should be closed (ZZ command)
fn handle_pending_command(
    mag: &mut crate::magnifier::MagnifierState,
    pending: crate::magnifier::PendingCommand,
    key_code: KeyCode,
) -> bool {
    use crate::magnifier::PendingCommand;

    match (pending, key_code) {
        // Multi-key sequences
        (PendingCommand::G, KeyCode::Char('g')) => {
            mag.move_to_first_line();
            false
        }
        (PendingCommand::D, KeyCode::Char('d')) => {
            mag.push_undo();
            mag.delete_line();
            false
        }
        (PendingCommand::Y, KeyCode::Char('y')) => {
            mag.yank_line();
            false
        }
        (PendingCommand::C, KeyCode::Char('c')) => {
            mag.change_line();
            false
        }
        (PendingCommand::Z, KeyCode::Char('Z')) => {
            // Signal to close magnifier
            true
        }
        (PendingCommand::IndentRight, KeyCode::Char('>')) => {
            mag.indent_line();
            false
        }
        (PendingCommand::IndentLeft, KeyCode::Char('<')) => {
            mag.dedent_line();
            false
        }

        // Character find commands
        (PendingCommand::FindForward, KeyCode::Char(c)) => {
            mag.find_char_forward(c);
            false
        }
        (PendingCommand::FindBackward, KeyCode::Char(c)) => {
            mag.find_char_backward(c);
            false
        }
        (PendingCommand::TillForward, KeyCode::Char(c)) => {
            mag.till_char_forward(c);
            false
        }
        (PendingCommand::TillBackward, KeyCode::Char(c)) => {
            mag.till_char_backward(c);
            false
        }
        (PendingCommand::Replace, KeyCode::Char(c)) => {
            mag.replace_char(c);
            false
        }

        _ => {
            // Invalid sequence, clear pending
            false
        }
    }
}

/// Handle keys in magnifier Insert mode
fn handle_insert(app: &mut App, key: KeyEvent) -> Result<InputResult> {
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
fn handle_command(app: &mut App, key: KeyEvent) -> Result<InputResult> {
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
fn handle_visual(app: &mut App, key: KeyEvent) -> Result<InputResult> {
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
