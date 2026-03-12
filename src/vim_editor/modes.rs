//! Vim mode definitions and transitions

/// Vim editing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    /// Normal mode - navigation and commands
    Normal,
    /// Insert mode - text input
    Insert,
    /// Command mode - ex commands (:w, :q, etc)
    Command,
    /// Visual mode - character-wise selection
    Visual,
    /// Visual Line mode - line-wise selection
    VisualLine,
}

impl VimMode {
    /// Get the display name for this mode
    pub fn display_name(&self) -> &'static str {
        match self {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Command => "COMMAND",
            VimMode::Visual => "VISUAL",
            VimMode::VisualLine => "VISUAL LINE",
        }
    }
}

/// Pending command for multi-key sequences
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingCommand {
    /// Waiting for second 'g' (gg)
    G,
    /// Waiting for second 'd' (dd)
    D,
    /// Waiting for second 'y' (yy)
    Y,
    /// Waiting for second 'c' (cc)
    C,
    /// Waiting for second 'Z' (ZZ)
    Z,
    /// Waiting for character after 'f'
    FindForward,
    /// Waiting for character after 'F'
    FindBackward,
    /// Waiting for character after 't'
    TillForward,
    /// Waiting for character after 'T'
    TillBackward,
    /// Waiting for character to replace with 'r'
    Replace,
    /// Waiting for second '>' (>>)
    IndentRight,
    /// Waiting for second '<' (<<)
    IndentLeft,
}

/// Last find command for repeating with ; and ,
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindCommand {
    Forward(char),
    Backward(char),
    TillForward(char),
    TillBackward(char),
}

/// Selection range for visual mode operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// Character-wise selection (like vim's `v`)
    CharWise {
        start: (usize, usize),
        end: (usize, usize),
    },
    /// Line-wise selection (like vim's `V`)
    LineWise { start_line: usize, end_line: usize },
}
