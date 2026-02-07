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
#[derive(Debug, Clone, PartialEq)]
pub enum PendingCommand {
    /// Waiting for second 'g' (for gg - go to first row)
    G,
    /// Waiting for second key after 'z' (zt, zz, zb)
    Z,
    /// Buffering column letters (e.g., after 'g', receiving 'B' then 'C' for column BC)
    GotoColumn(String),
    /// Waiting for second 'd' (for dd - delete row)
    D,
    /// Waiting for second 'y' (for yy - yank row)
    Y,
}

impl PendingCommand {
    /// Create from KeyCode if it starts a multi-key sequence
    pub fn from_key_code(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Char('g') => Some(Self::G),
            KeyCode::Char('z') => Some(Self::Z),
            KeyCode::Char('d') => Some(Self::D),
            KeyCode::Char('y') => Some(Self::Y),
            _ => None,
        }
    }

    /// Append a letter to GotoColumn buffer, or create new GotoColumn from G
    pub fn append_letter(self, letter: char) -> Self {
        match self {
            PendingCommand::G => PendingCommand::GotoColumn(letter.to_string()),
            PendingCommand::GotoColumn(mut s) => {
                s.push(letter);
                PendingCommand::GotoColumn(s)
            }
            other => other,
        }
    }

    /// Get the buffered column letters if this is a GotoColumn command
    pub fn get_column_letters(&self) -> Option<&str> {
        match self {
            PendingCommand::GotoColumn(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

/// Newtype wrapper for status messages
#[derive(Debug, Clone, PartialEq)]
pub struct StatusMessage {
    content: Cow<'static, str>,
    clear_on_keypress: bool,
}

impl StatusMessage {
    /// Create a new status message from a static string (clears on keypress by default)
    pub const fn new_static(msg: &'static str) -> Self {
        Self {
            content: Cow::Borrowed(msg),
            clear_on_keypress: true,
        }
    }

    /// Create a new status message from an owned String (clears on keypress by default)
    pub fn new_owned(msg: String) -> Self {
        Self {
            content: Cow::Owned(msg),
            clear_on_keypress: true,
        }
    }

    /// Create a persistent message that won't clear on keypress
    pub fn new_persistent(msg: String) -> Self {
        Self {
            content: Cow::Owned(msg),
            clear_on_keypress: false,
        }
    }

    /// Get the message as a string slice
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Check if this message should clear on next keypress
    pub fn should_clear_on_keypress(&self) -> bool {
        self.clear_on_keypress
    }

    /// Convert to Cow<'static, str> for backwards compatibility
    pub fn into_cow(self) -> Cow<'static, str> {
        self.content
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
