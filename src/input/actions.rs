use crossterm::event::KeyCode;
use std::borrow::Cow;

/// Result of processing user input
#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    /// Continue normal operation
    Continue,
    /// Reload the current file (after file switching)
    ReloadFile,
    /// Quit the application
    Quit,
}

/// High-level user actions that can be performed
#[derive(Debug, Clone, PartialEq)]
pub enum UserAction {
    /// Navigate within the CSV data
    Navigate(NavigateAction),
    /// Control viewport positioning
    ViewportControl(ViewportAction),
    /// Toggle the help overlay
    ToggleHelp,
    /// Quit the application
    Quit { force: bool },
    /// Switch to a different file
    SwitchFile(FileDirection),
    /// Cancel the current pending command
    CancelCommand,
}

/// Navigation actions within the CSV data
#[derive(Debug, Clone, PartialEq)]
pub enum NavigateAction {
    /// Move up by count rows
    Up { count: usize },
    /// Move down by count rows
    Down { count: usize },
    /// Move left by count columns
    Left { count: usize },
    /// Move right by count columns
    Right { count: usize },
    /// Jump to first row
    FirstRow,
    /// Jump to last row
    LastRow,
    /// Jump to first column
    FirstColumn,
    /// Jump to last column
    LastColumn,
    /// Jump to specific line (1-indexed)
    GotoLine { line: usize },
    /// Page down
    PageDown,
    /// Page up
    PageUp,
}

/// Viewport positioning actions (vim's zt, zz, zb)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportAction {
    /// Position selected row at top of screen (zt)
    Top,
    /// Position selected row at center of screen (zz)
    Center,
    /// Position selected row at bottom of screen (zb)
    Bottom,
    /// Auto-positioning (default behavior)
    Auto,
}

/// Direction for file switching
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileDirection {
    /// Switch to next file
    Next,
    /// Switch to previous file
    Previous,
}

/// Pending multi-key command state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PendingCommand {
    /// Waiting for second 'g' (for gg - go to first row)
    G,
    /// Waiting for second key after 'z' (zt, zz, zb)
    Z,
}

impl PendingCommand {
    /// Create from KeyCode if it starts a multi-key sequence
    pub fn from_key_code(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Char('g') => Some(Self::G),
            KeyCode::Char('z') => Some(Self::Z),
            _ => None,
        }
    }
}

/// Newtype wrapper for status messages
#[derive(Debug, Clone, PartialEq)]
pub struct StatusMessage(Cow<'static, str>);

impl StatusMessage {
    /// Create a new status message from a static string
    pub const fn new_static(msg: &'static str) -> Self {
        Self(Cow::Borrowed(msg))
    }

    /// Create a new status message from an owned String
    pub fn new_owned(msg: String) -> Self {
        Self(Cow::Owned(msg))
    }

    /// Get the message as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to Cow<'static, str> for backwards compatibility
    pub fn into_cow(self) -> Cow<'static, str> {
        self.0
    }
}

impl From<&'static str> for StatusMessage {
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

impl From<String> for StatusMessage {
    fn from(s: String) -> Self {
        Self::new_owned(s)
    }
}

impl AsRef<str> for StatusMessage {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_result() {
        assert_eq!(InputResult::Continue, InputResult::Continue);
        assert_ne!(InputResult::Continue, InputResult::Quit);
    }

    #[test]
    fn test_navigate_action() {
        let action = NavigateAction::Up { count: 5 };
        match action {
            NavigateAction::Up { count } => assert_eq!(count, 5),
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_viewport_action() {
        assert_eq!(ViewportAction::Top, ViewportAction::Top);
        assert_ne!(ViewportAction::Top, ViewportAction::Center);
    }

    #[test]
    fn test_file_direction() {
        assert_eq!(FileDirection::Next, FileDirection::Next);
        assert_ne!(FileDirection::Next, FileDirection::Previous);
    }

    #[test]
    fn test_pending_command_from_key() {
        assert_eq!(
            PendingCommand::from_key_code(KeyCode::Char('g')),
            Some(PendingCommand::G)
        );
        assert_eq!(
            PendingCommand::from_key_code(KeyCode::Char('z')),
            Some(PendingCommand::Z)
        );
        assert_eq!(PendingCommand::from_key_code(KeyCode::Char('j')), None);
    }

    #[test]
    fn test_status_message_static() {
        let msg = StatusMessage::new_static("Test message");
        assert_eq!(msg.as_str(), "Test message");
    }

    #[test]
    fn test_status_message_owned() {
        let msg = StatusMessage::new_owned("Test".to_string());
        assert_eq!(msg.as_str(), "Test");
    }

    #[test]
    fn test_status_message_from_str() {
        let msg: StatusMessage = "Hello".into();
        assert_eq!(msg.as_str(), "Hello");
    }

    #[test]
    fn test_status_message_from_string() {
        let msg: StatusMessage = String::from("World").into();
        assert_eq!(msg.as_str(), "World");
    }
}
