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

    /// Command buffer for command mode (stores text after ":")
    pub command_buffer: String,

    /// Cursor position within command buffer (char index)
    pub command_cursor: usize,

    /// File filter buffer for FileList mode (search/filter files)
    pub file_filter_buffer: String,

    /// Whether file list is in search mode (/ pressed)
    pub file_list_search_active: bool,
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
    pub fn count_or_default(&self) -> usize {
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

    /// Clear the command buffer and reset cursor
    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
        self.command_cursor = 0;
    }

    /// Insert a character at the cursor position
    pub fn push_command_char(&mut self, c: char) {
        let byte_pos = self
            .command_buffer
            .char_indices()
            .nth(self.command_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.command_buffer.len());
        self.command_buffer.insert(byte_pos, c);
        self.command_cursor += 1;
    }

    /// Delete the character before the cursor
    pub fn pop_command_char(&mut self) {
        if self.command_cursor > 0 {
            self.command_cursor -= 1;
            let byte_pos = self
                .command_buffer
                .char_indices()
                .nth(self.command_cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.command_buffer.len());
            self.command_buffer.remove(byte_pos);
        }
    }

    /// Delete the character at the cursor (Delete key)
    pub fn delete_command_char(&mut self) {
        let char_count = self.command_buffer.chars().count();
        if self.command_cursor < char_count {
            let byte_pos = self
                .command_buffer
                .char_indices()
                .nth(self.command_cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.command_buffer.len());
            self.command_buffer.remove(byte_pos);
        }
    }

    /// Move command cursor left
    pub fn command_cursor_left(&mut self) {
        if self.command_cursor > 0 {
            self.command_cursor -= 1;
        }
    }

    /// Move command cursor right
    pub fn command_cursor_right(&mut self) {
        let char_count = self.command_buffer.chars().count();
        if self.command_cursor < char_count {
            self.command_cursor += 1;
        }
    }

    /// Move command cursor to start
    pub fn command_cursor_home(&mut self) {
        self.command_cursor = 0;
    }

    /// Move command cursor to end
    pub fn command_cursor_end(&mut self) {
        self.command_cursor = self.command_buffer.chars().count();
    }

    /// Clear the file filter buffer
    pub fn clear_file_filter(&mut self) {
        self.file_filter_buffer.clear();
    }

    /// Push a character to the file filter buffer
    pub fn push_file_filter_char(&mut self, c: char) {
        self.file_filter_buffer.push(c);
    }

    /// Pop a character from the file filter buffer
    pub fn pop_file_filter_char(&mut self) {
        self.file_filter_buffer.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_default() {
        let state = InputState::new();
        assert!(!state.has_pending_command());
        assert_eq!(state.count_or_default(), 1);
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
        assert_eq!(state.count_or_default(), 5);

        state.add_count_digit(3);
        assert_eq!(state.count_or_default(), 53);

        state.clear_count();
        assert_eq!(state.count_or_default(), 1);
    }

    #[test]
    fn test_count_overflow_protection() {
        let mut state = InputState::new();

        // Build a very large count
        for _ in 0..10 {
            state.add_count_digit(9);
        }

        // Should be clamped
        assert!(state.count_or_default() < 100000);
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

    #[test]
    fn test_count_prefix_very_large_number() {
        let mut state = InputState::new();

        // Build a very large count: 999999
        for _ in 0..6 {
            state.add_count_digit(9);
        }

        let count = state.count_or_default();

        // Should be clamped just below MAX_COMMAND_COUNT
        // The implementation uses `new_value < MAX_COMMAND_COUNT` so it stops at 99999
        assert!(count <= MAX_COMMAND_COUNT);
        assert!(count >= 99999); // Should be at or near the limit
    }

    #[test]
    fn test_count_prefix_zero_behavior() {
        let mut state = InputState::new();

        // Add a zero
        state.add_count_digit(0);

        let count = state.count_or_default();

        // Zero should be treated as "no count" (default to 1)
        // or could be special case for "go to first column" (0)
        // Document current behavior
        assert!(count == 0 || count == 1);
    }

    #[test]
    fn test_count_prefix_cleared_after_retrieval() {
        let mut state = InputState::new();

        // Build count: 5
        state.add_count_digit(5);
        assert_eq!(state.count_or_default(), 5);

        // Clear it
        state.clear_count();

        // Should be back to default (1)
        assert_eq!(state.count_or_default(), 1);
    }

    #[test]
    fn test_multiple_count_digits_accumulation() {
        let mut state = InputState::new();

        // Build count: 123
        state.add_count_digit(1);
        state.add_count_digit(2);
        state.add_count_digit(3);

        assert_eq!(state.count_or_default(), 123);
    }

    // ── Command buffer editing tests ──────────────────────────────

    #[test]
    fn test_push_char_appends_at_end() {
        let mut state = InputState::new();
        state.push_command_char('s');
        state.push_command_char('o');
        state.push_command_char('r');
        state.push_command_char('t');
        assert_eq!(state.command_buffer, "sort");
        assert_eq!(state.command_cursor, 4);
    }

    #[test]
    fn test_push_char_inserts_at_cursor() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.push_command_char('c');
        // cursor is at 2 ("ac|"), move left
        state.command_cursor_left();
        // cursor is at 1 ("a|c"), insert 'b'
        state.push_command_char('b');
        assert_eq!(state.command_buffer, "abc");
        assert_eq!(state.command_cursor, 2);
    }

    #[test]
    fn test_backspace_at_end() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.push_command_char('b');
        state.push_command_char('c');
        state.pop_command_char();
        assert_eq!(state.command_buffer, "ab");
        assert_eq!(state.command_cursor, 2);
    }

    #[test]
    fn test_backspace_in_middle() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.push_command_char('b');
        state.push_command_char('c');
        state.command_cursor_left(); // cursor at 2 ("ab|c")
        state.pop_command_char(); // delete 'b'
        assert_eq!(state.command_buffer, "ac");
        assert_eq!(state.command_cursor, 1);
    }

    #[test]
    fn test_backspace_at_start_does_nothing() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.command_cursor_home();
        state.pop_command_char();
        assert_eq!(state.command_buffer, "a");
        assert_eq!(state.command_cursor, 0);
    }

    #[test]
    fn test_delete_at_cursor() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.push_command_char('b');
        state.push_command_char('c');
        state.command_cursor_home(); // cursor at 0 ("|abc")
        state.delete_command_char(); // delete 'a'
        assert_eq!(state.command_buffer, "bc");
        assert_eq!(state.command_cursor, 0);
    }

    #[test]
    fn test_delete_at_end_does_nothing() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.delete_command_char();
        assert_eq!(state.command_buffer, "a");
        assert_eq!(state.command_cursor, 1);
    }

    #[test]
    fn test_cursor_left_right() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.push_command_char('b');
        state.push_command_char('c');
        assert_eq!(state.command_cursor, 3);

        state.command_cursor_left();
        assert_eq!(state.command_cursor, 2);
        state.command_cursor_left();
        assert_eq!(state.command_cursor, 1);
        state.command_cursor_right();
        assert_eq!(state.command_cursor, 2);
    }

    #[test]
    fn test_cursor_left_at_zero() {
        let mut state = InputState::new();
        state.command_cursor_left();
        assert_eq!(state.command_cursor, 0);
    }

    #[test]
    fn test_cursor_right_at_end() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.command_cursor_right(); // already at end
        assert_eq!(state.command_cursor, 1);
    }

    #[test]
    fn test_home_end() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.push_command_char('b');
        state.push_command_char('c');

        state.command_cursor_home();
        assert_eq!(state.command_cursor, 0);

        state.command_cursor_end();
        assert_eq!(state.command_cursor, 3);
    }

    #[test]
    fn test_clear_resets_cursor() {
        let mut state = InputState::new();
        state.push_command_char('a');
        state.push_command_char('b');
        state.clear_command_buffer();
        assert_eq!(state.command_buffer, "");
        assert_eq!(state.command_cursor, 0);
    }

    #[test]
    fn test_insert_in_middle_of_word() {
        let mut state = InputState::new();
        // Type "srt"
        state.push_command_char('s');
        state.push_command_char('r');
        state.push_command_char('t');
        // Move left twice to position after 's'
        state.command_cursor_left();
        state.command_cursor_left();
        // Insert 'o' to make "sort"
        state.push_command_char('o');
        assert_eq!(state.command_buffer, "sort");
        assert_eq!(state.command_cursor, 2);
    }
}
