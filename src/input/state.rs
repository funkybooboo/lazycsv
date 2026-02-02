//! Input state management for multi-key commands and count prefixes.
//!
//! This module tracks the state of pending multi-key commands (like 'gg', 'zz')
//! and count prefixes (like '5j' to move down 5 rows).

use super::actions::PendingCommand;
use super::handler::{MAX_COMMAND_COUNT, MULTI_KEY_TIMEOUT_MS};
use std::num::NonZeroUsize;
use std::time::Instant;

/// State for multi-key input handling
#[derive(Debug, Default)]
pub struct InputState {
    /// Pending multi-key command (e.g., waiting for second key after 'g' or 'z')
    pub pending_command: Option<PendingCommand>,

    /// Count prefix for vim commands (e.g., 5 for "5j")
    pub command_count: Option<NonZeroUsize>,

    /// Time when pending command was set (for timeout)
    pub pending_command_time: Option<Instant>,
}

impl InputState {
    /// Create a new InputState with no pending commands
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if there's a pending command
    pub fn has_pending_command(&self) -> bool {
        self.pending_command.is_some()
    }

    /// Clear the pending command state
    pub fn clear_pending_command(&mut self) {
        self.pending_command = None;
        self.pending_command_time = None;
    }

    /// Set a pending command
    pub fn set_pending_command(&mut self, cmd: PendingCommand) {
        self.pending_command = Some(cmd);
        self.pending_command_time = Some(Instant::now());
    }

    /// Check if the pending command has timed out (1 second)
    pub fn is_pending_command_timed_out(&self) -> bool {
        if let Some(time) = self.pending_command_time {
            time.elapsed().as_millis() > MULTI_KEY_TIMEOUT_MS
        } else {
            false
        }
    }

    /// Get the current command count, or 1 if none is set
    pub fn get_count_or_default(&self) -> usize {
        self.command_count.map(|c| c.get()).unwrap_or(1)
    }

    /// Clear the command count
    pub fn clear_count(&mut self) {
        self.command_count = None;
    }

    /// Add a digit to the command count
    pub fn add_count_digit(&mut self, digit: u32) {
        let digit_value = digit as usize;
        self.command_count = match self.command_count.take() {
            None => NonZeroUsize::new(digit_value),
            Some(existing) => {
                let new_value = existing.get() * 10 + digit_value;
                // Limit to reasonable size to prevent overflow
                if new_value < MAX_COMMAND_COUNT {
                    NonZeroUsize::new(new_value)
                } else {
                    Some(existing)
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_default() {
        let state = InputState::new();
        assert!(!state.has_pending_command());
        assert_eq!(state.get_count_or_default(), 1);
    }

    #[test]
    fn test_pending_command() {
        let mut state = InputState::new();

        state.set_pending_command(PendingCommand::G);
        assert!(state.has_pending_command());
        assert_eq!(state.pending_command, Some(PendingCommand::G));

        state.clear_pending_command();
        assert!(!state.has_pending_command());
        assert_eq!(state.pending_command, None);
    }

    #[test]
    fn test_command_count() {
        let mut state = InputState::new();

        state.add_count_digit(5);
        assert_eq!(state.get_count_or_default(), 5);

        state.add_count_digit(3);
        assert_eq!(state.get_count_or_default(), 53);

        state.clear_count();
        assert_eq!(state.get_count_or_default(), 1);
    }

    #[test]
    fn test_count_overflow_protection() {
        let mut state = InputState::new();

        // Build a very large count
        for _ in 0..10 {
            state.add_count_digit(9);
        }

        // Should be clamped
        assert!(state.get_count_or_default() < 100000);
    }

    #[test]
    fn test_pending_command_timeout() {
        let mut state = InputState::new();
        state.set_pending_command(PendingCommand::G);

        // Verify NOT timed out immediately
        assert!(!state.is_pending_command_timed_out());

        // Sleep for timeout duration + buffer
        std::thread::sleep(std::time::Duration::from_millis(
            MULTI_KEY_TIMEOUT_MS as u64 + 100,
        ));

        // Verify IS timed out after sleep
        assert!(state.is_pending_command_timed_out());
    }

    #[test]
    fn test_pending_command_no_timeout_when_none() {
        let state = InputState::new();
        // No pending command set
        assert!(!state.is_pending_command_timed_out());
    }
}
