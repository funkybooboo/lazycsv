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
//! - `help`: Help overlay display and scrolling
//! - `search`: Search operations (/, n, N, *)
//! - `visual_mode`: Visual mode entry (v, V, `,v`)
//! - `editing`: Row/cell editing operations (o, O, p, P, Delete)
//! - `mode_transitions`: Mode switching (i, a, :, /, q, m)
//! - `file_switching`: File navigation (`[`, `]`)
//! - `navigation`: Navigation helpers (Enter, Ctrl+d, Ctrl+u)

mod commands;
mod editing;
mod file_switching;
mod handler;
mod help;
mod mode_transitions;
mod multi_key;
mod navigation;
mod search;
mod visual_mode;

pub use handler::handle;
