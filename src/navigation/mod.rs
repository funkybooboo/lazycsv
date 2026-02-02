//! Navigation commands and cursor movement
//!
//! Handles all cursor movement operations: up/down/left/right,
//! page navigation, and goto commands (gg, G, nG).

pub mod commands;

pub use commands::{
    goto_first_row, goto_last_row, goto_line, handle_navigation, move_down_by, move_left_by,
    move_right_by, move_up_by,
};

/// Rows per page for PageUp/PageDown navigation
pub use commands::PAGE_SIZE;
