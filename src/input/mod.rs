//! Input handling and user action processing
//!
//! Manages keyboard input, multi-key command sequences, and state for
//! pending commands (like 'g' waiting for second key in 'gg').

pub mod actions;
pub mod handler;
pub mod insert_mode;
pub mod magnifier_mode;
pub mod state;
pub mod visual_mode;

pub use actions::{
    FileDirection, InputResult, NavigateAction, PendingCommand, StatusMessage, UserAction,
    ViewportAction,
};
pub use handler::{handle_key, MULTI_KEY_TIMEOUT_MS};
pub use state::InputState;
