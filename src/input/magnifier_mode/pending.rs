//! Pending command setup for magnifier mode
//!
//! Handles setting up multi-key sequences (g, d, y, c, Z, f, F, t, T, r, >, <)

use crate::magnifier::{MagnifierState, PendingCommand};
use crossterm::event::KeyCode;

/// Handle setting up pending commands for multi-key sequences
pub fn handle_pending_setup(mag: &mut MagnifierState, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('g') => {
            mag.set_pending(PendingCommand::G);
            true
        }
        KeyCode::Char('d') => {
            mag.set_pending(PendingCommand::D);
            true
        }
        KeyCode::Char('y') => {
            mag.set_pending(PendingCommand::Y);
            true
        }
        KeyCode::Char('c') => {
            mag.set_pending(PendingCommand::C);
            true
        }
        KeyCode::Char('Z') => {
            mag.set_pending(PendingCommand::Z);
            true
        }
        KeyCode::Char('f') => {
            mag.set_pending(PendingCommand::FindForward);
            true
        }
        KeyCode::Char('F') => {
            mag.set_pending(PendingCommand::FindBackward);
            true
        }
        KeyCode::Char('t') => {
            mag.set_pending(PendingCommand::TillForward);
            true
        }
        KeyCode::Char('T') => {
            mag.set_pending(PendingCommand::TillBackward);
            true
        }
        KeyCode::Char('r') => {
            mag.set_pending(PendingCommand::Replace);
            true
        }
        KeyCode::Char('>') => {
            mag.set_pending(PendingCommand::IndentRight);
            true
        }
        KeyCode::Char('<') => {
            mag.set_pending(PendingCommand::IndentLeft);
            true
        }
        _ => false,
    }
}
