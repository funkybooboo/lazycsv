//! Ex command parsing and execution (:w, :q, :wq, :q!, :noh)

use super::modes::VimMode;
use super::VimEditor;

/// Ex command result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExCommand {
    /// Save content
    Write,
    /// Quit
    Quit,
    /// Save and quit
    WriteQuit,
    /// Force quit without saving
    ForceQuit,
    /// Clear search highlighting
    NoHighlight,
    /// Unknown/invalid command
    Unknown(String),
}

impl VimEditor {
    /// Enter command mode
    pub fn enter_command_mode(&mut self) {
        self.mode = VimMode::Command;
        self.command_buffer.clear();
    }

    /// Enter command mode with a prefix (e.g., "/")
    pub fn enter_command_mode_with(&mut self, prefix: &str) {
        self.mode = VimMode::Command;
        self.command_buffer = prefix.to_string();
    }

    /// Exit command mode
    pub fn exit_command_mode(&mut self) {
        self.mode = VimMode::Normal;
        self.command_buffer.clear();
    }

    /// Get the command buffer
    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Insert a character into the command buffer
    pub fn command_insert_char(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    /// Delete the last character from the command buffer
    pub fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    /// Parse and return the ex command
    pub fn parse_command(&self) -> ExCommand {
        let cmd = self.command_buffer.trim();

        match cmd {
            "w" => ExCommand::Write,
            "q" => ExCommand::Quit,
            "wq" | "x" => ExCommand::WriteQuit,
            "q!" => ExCommand::ForceQuit,
            "noh" | "nohlsearch" => ExCommand::NoHighlight,
            _ => ExCommand::Unknown(cmd.to_string()),
        }
    }
}
