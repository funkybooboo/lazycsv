//! Visual mode operations for LazyCSV
//!
//! This module handles all visual mode operations (delete, yank, paste) across
//! the three visual modes: Block, Line, and Column.
//!
//! # Visual Modes
//!
//! - **Block Mode** (`v`): Rectangular selection of cells
//! - **Line Mode** (`V`): Whole row selection  
//! - **Column Mode** (`,v`): Whole column selection
//!
//! # Triple Clipboard System
//!
//! Each visual mode uses its own clipboard buffer:
//! - Block mode → Region buffer (2D array of cells)
//! - Line mode → Row buffer (rows with all columns)
//! - Column mode → Column buffer (columns with all rows including header)

mod delete;
mod handler;
mod paste;
mod yank;

pub use delete::handle_visual_delete;
pub use handler::handle;
pub use paste::handle_visual_paste;
pub use yank::handle_visual_yank;
