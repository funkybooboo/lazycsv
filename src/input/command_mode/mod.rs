//! Command mode (ex commands) handling
//!
//! This module handles vim-style ex commands (prefixed with `:`) for file operations,
//! navigation, and CSV manipulation. Commands provide a text-based interface for
//! operations that don't have dedicated keybindings.
//!
//! # Supported Commands
//!
//! ## File Operations
//! - `:w [file]` - Write (save) current or all files
//! - `:q` - Quit (with unsaved changes prompt)
//! - `:wq` - Write and quit
//! - `:e file` - Edit (open) a file
//!
//! ## Navigation
//! - `:123` or `:goto 123` - Jump to row 123
//! - `:goto A` - Jump to column A
//! - `:files` - Show file list
//!
//! ## CSV Operations
//! - `:sort A` - Sort by column A
//! - `:filter A=value` - Filter rows
//! - `:delete 5` or `:5delete` - Delete row 5
//! - `:delete A` - Delete column A
//!
//! # Module Organization
//!
//! - `handler`: Command line input handling (character entry, backspace, enter)
//! - `executor`: Command parsing and execution dispatch
//! - `range_commands`: Range-based operations (delete, yank, etc.)

mod executor;
mod handler;
mod range_commands;
pub mod stats;

pub use handler::handle;
