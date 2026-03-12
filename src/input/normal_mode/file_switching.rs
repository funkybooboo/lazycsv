//! File switching operations for Normal mode

use crate::app::App;
use crate::input::handler::handle_file_switch;
use crate::input::InputResult;

/// Switch to previous file in session
pub fn previous_file(app: &mut App) -> InputResult {
    handle_file_switch(app, false)
}

/// Switch to next file in session
pub fn next_file(app: &mut App) -> InputResult {
    handle_file_switch(app, true)
}
