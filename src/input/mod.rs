//! Input handling and user action processing
//!
//! Manages keyboard input, multi-key command sequences, and state for
//! pending commands (like 'g' waiting for second key in 'gg').

pub mod actions;
pub mod command_mode;
pub mod file_list_mode;
pub mod file_operation_mode;
pub mod handler;
pub mod insert_mode;
pub mod keybindings;
pub mod keymap_actions;
pub mod keymap_dispatch;
pub mod magnifier_mode;
pub(crate) mod mouse_context_menu;
pub(crate) mod mouse_coords;
pub mod mouse_handler;
pub(crate) mod mouse_reorder;
pub mod normal_mode;
pub mod search_mode;
pub mod sql_editor_mode;
pub mod state;
pub mod theme_selector_mode;
pub mod visual_mode;

pub use actions::{
    FileDirection, InputResult, NavigateAction, PendingCommand, StatusMessage, UserAction,
    ViewportAction,
};
pub use handler::{handle_key, MULTI_KEY_TIMEOUT_MS};
pub use state::InputState;
