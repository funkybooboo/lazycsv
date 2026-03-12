//! Magnifier mode input handling
//!
//! This module handles keyboard input when the magnifier (full vim editor) is active.
//! The magnifier provides a multi-line text editor for viewing and editing large cell
//! contents with full vim motion and operator support.
//!
//! # Module Organization
//!
//! - `handler`: Main input handler dispatching to appropriate subhandlers
//! - `motions`: Vim motion commands (h, j, k, l, w, b, etc.)
//! - `operators`: Vim operators (d, y, c, etc.)
//! - `mode_changes`: Mode transitions (i, v, Esc, etc.)
//! - `search`: Search within magnifier content (/, n, N)
//! - `pending`: Pending operator state management

mod handler;
mod mode_changes;
mod motions;
mod operators;
mod pending;
mod search;

pub use handler::handle;
