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
//!
//! ## Module Organization
//!
//! The insert mode handler is split into focused submodules:
//!
//! - `commit_cancel`: Commit edits and move, or cancel changes
//! - `text_editing`: Character input, backspace, delete operations
//! - `cursor_movement`: Arrow key navigation, Home/End
//! - `vim_commands`: Vim-style editing (Ctrl+w, Ctrl+u, Ctrl+h)
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
        (KeyCode::Char(_), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            handle_text_editing(app, key);
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
