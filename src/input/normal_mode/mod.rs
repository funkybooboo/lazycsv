//! Normal mode input handling
//!
//! This module handles keyboard input in Normal mode, the default mode for navigation
//! and commands in LazyCSV. Normal mode provides vim-style keybindings for efficient
//! CSV navigation and manipulation.
//!
//! # Key Features
//!
//! - **Navigation**: h/j/k/l, gg/G, w/b, 0/$, Ctrl+f/b, etc.
//! - **Visual mode**: v (block), V (line), ,v (column)
//! - **Editing**: i/a/I/A/s (insert), d/y/p (delete/yank/paste), u (undo), Ctrl+r (redo)
//! - **Commands**: : (command mode), / (search), q (SQL query)
//! - **Files**: gf (jump to file), Ctrl+^ (switch files), :w/:q (save/quit)
//! - **Multi-key**: gg, dd, yy, cc, and more
//!
//! # Module Organization
//!
//! - `handler`: Main input handler dispatching to appropriate actions
//! - `multi_key`: Multi-keystroke command handling (gg, dd, etc.)

mod handler;
mod multi_key;

pub use handler::handle;
