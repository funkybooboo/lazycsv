//! Insert Mode input handling
//!
//! This module provides quick inline cell editing with vim-style keybindings.
//! Insert mode allows users to edit cell contents with:
//!
//! - Character insertion and deletion
//! - Cursor movement (arrow keys, Home, End)
//! - Vim-style editing commands (Ctrl+w, Ctrl+u, Ctrl+h)
//! - Directional commit (Enter, Tab with modifiers)
//! - Cancel with Escape
//! - Formula completion popup (triggered by '=' at start of cell)
//!
//! ## Module Organization
//!
//! The insert mode handler is split into focused submodules:
//!
//! - `commit_cancel`: Commit edits and move, or cancel changes
//! - `text_editing`: Character input, backspace, delete operations
//! - `cursor_movement`: Arrow key navigation, Home/End
//! - `vim_commands`: Vim-style editing (Ctrl+w, Ctrl+u, Ctrl+h)
//! - `formula_completion`: Formula function completion popup
//!
//! ## Unicode Handling
//!
//! All cursor operations work with character positions (not byte positions)
//! to correctly handle multi-byte UTF-8 sequences like emoji and CJK characters.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::input::InputResult;

mod commit_cancel;
mod cursor_movement;
pub mod formula_completion;
mod text_editing;
mod vim_commands;

use commit_cancel::handle_commit_cancel;
use cursor_movement::handle_cursor_movement;
use text_editing::handle_text_editing;
use vim_commands::handle_vim_commands;

/// Handle keyboard input in Insert mode
pub fn handle_insert_mode(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    // If no edit buffer, return to Normal mode (shouldn't happen)
    if app.edit_buffer.is_none() {
        app.mode = crate::app::Mode::Normal;
        return Ok(InputResult::Continue);
    }

    // If formula completion popup is active, handle it first
    if app.formula_completion.is_some() {
        return handle_formula_completion_key(app, key);
    }

    match (key.code, key.modifiers) {
        // Vim-style commands (check first to intercept Ctrl combinations)
        (KeyCode::Char('h' | 'w' | 'u'), KeyModifiers::CONTROL) => {
            handle_vim_commands(app, key);
        }

        // Commit/cancel operations
        (KeyCode::Enter, _) | (KeyCode::Tab, _) | (KeyCode::BackTab, _) | (KeyCode::Esc, _) => {
            handle_commit_cancel(app, key);
        }

        // Text editing operations (regular character input)
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            handle_text_editing(app, key);

            // After typing '=', check if it's at position 1 (just typed '=' as first char)
            if c == '=' {
                if let Some(ref buffer) = app.edit_buffer {
                    if buffer.content == "=" && buffer.cursor == 1 {
                        formula_completion::open_formula_completion(app, "");
                    }
                }
            }
        }
        (KeyCode::Backspace, _) | (KeyCode::Delete, _) => {
            handle_text_editing(app, key);
        }

        // Cursor movement operations
        (KeyCode::Left, _) | (KeyCode::Right, _) | (KeyCode::Home, _) | (KeyCode::End, _) => {
            handle_cursor_movement(app, key);
        }

        _ => {}
    }

    Ok(InputResult::Continue)
}

/// Handle keystrokes when the formula completion popup is active.
fn handle_formula_completion_key(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    match (key.code, key.modifiers) {
        // Navigate completion list
        (KeyCode::Down, _) => {
            if let Some(ref mut comp) = app.formula_completion {
                comp.move_down();
            }
        }
        (KeyCode::Up, _) => {
            if let Some(ref mut comp) = app.formula_completion {
                comp.move_up();
            }
        }

        // Accept selected item
        (KeyCode::Enter, _) | (KeyCode::Tab, _) => {
            formula_completion::accept_completion(app);
        }

        // Dismiss popup
        (KeyCode::Esc, _) => {
            formula_completion::close_formula_completion(app);
        }

        // Backspace: update filter or dismiss
        (KeyCode::Backspace, _) => {
            let should_close = if let Some(ref mut comp) = app.formula_completion {
                if comp.filter.is_empty() {
                    true
                } else {
                    comp.pop_filter();
                    comp.filtered_items().is_empty()
                }
            } else {
                true
            };
            if should_close {
                formula_completion::close_formula_completion(app);
                // Also process backspace in the edit buffer
                handle_text_editing(app, key);
            }
        }

        // Character input: filter the completion list + type into buffer
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            // If they type '(' it means they want to manually type the formula — close popup
            if c == '(' || c == ')' || c == ':' {
                formula_completion::close_formula_completion(app);
                handle_text_editing(app, key);
            } else {
                // Update filter
                let should_close = if let Some(ref mut comp) = app.formula_completion {
                    comp.push_filter(c);
                    comp.filtered_items().is_empty()
                } else {
                    true
                };
                if should_close {
                    formula_completion::close_formula_completion(app);
                }
                // Also type the character into the edit buffer
                handle_text_editing(app, key);
            }
        }

        // Any other key dismisses the popup and processes normally
        _ => {
            formula_completion::close_formula_completion(app);
        }
    }

    Ok(InputResult::Continue)
}
