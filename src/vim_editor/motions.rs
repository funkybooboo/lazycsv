//! Motion commands (hjkl, w/b/e, 0/$, gg/G, f/t, etc.)

use super::{FindCommand, VimEditor, VimMode};

impl VimEditor {
    // ============================================================================
    // Basic Directional Movement (hjkl)
    // ============================================================================

    /// Move cursor left (h)
    pub fn move_left(&mut self) {
        let count = self.take_count();
        self.cursor.1 = self.cursor.1.saturating_sub(count);
        self.clamp_cursor();
    }

    /// Move cursor right (l)
    pub fn move_right(&mut self) {
        let count = self.take_count();
        self.cursor.1 = self.cursor.1.saturating_add(count);
        self.clamp_cursor();
    }

    /// Move cursor up (k)
    pub fn move_up(&mut self) {
        let count = self.take_count();
        self.cursor.0 = self.cursor.0.saturating_sub(count);
        self.clamp_cursor();
    }

    /// Move cursor down (j)
    pub fn move_down(&mut self) {
        let count = self.take_count();
        self.cursor.0 = self.cursor.0.saturating_add(count);
        self.clamp_cursor();
    }

    // ============================================================================
    // Line-Level Movement (0, $, ^)
    // ============================================================================

    /// Move to start of line (0)
    pub fn move_to_line_start(&mut self) {
        self.cursor.1 = 0;
    }

    /// Move to end of line ($)
    pub fn move_to_line_end(&mut self) {
        let line_len = self.current_line().chars().count();
        self.cursor.1 = if self.mode == VimMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        self.clamp_cursor();
    }

    /// Move to first non-blank character (^)
    pub fn move_to_first_non_blank(&mut self) {
        let line = self.current_line();
        let first_non_blank = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
        self.cursor.1 = first_non_blank;
        self.clamp_cursor();
    }

    // ============================================================================
    // Document-Level Movement (gg, G, line number)
    // ============================================================================

    /// Move to first line (gg)
    pub fn move_to_first_line(&mut self) {
        self.cursor.0 = 0;
        self.clamp_cursor();
    }

    /// Move to last line (G)
    pub fn move_to_last_line(&mut self) {
        self.cursor.0 = self.lines.len().saturating_sub(1);
        self.clamp_cursor();
    }

    /// Move to specific line number (1-indexed for user, converted to 0-indexed)
    pub fn move_to_line(&mut self, line_number: usize) {
        // Convert 1-indexed to 0-indexed
        self.cursor.0 = line_number.saturating_sub(1);
        self.clamp_cursor();
    }

    // ============================================================================
    // Word Movement (w, b, e)
    // ============================================================================

    /// Move to next word (w)
    pub fn move_next_word(&mut self) {
        let count = self.take_count();
        for _ in 0..count {
            self.move_next_word_once();
        }
    }

    /// Move to previous word (b)
    pub fn move_prev_word(&mut self) {
        let count = self.take_count();
        for _ in 0..count {
            self.move_prev_word_once();
        }
    }

    /// Move to end of word (e)
    pub fn move_end_word(&mut self) {
        let count = self.take_count();
        for _ in 0..count {
            self.move_end_word_once();
        }
    }

    /// Helper: Move to next word once
    fn move_next_word_once(&mut self) {
        let line = self.current_line().to_string();
        let char_count = line.chars().count();
        let mut col = self.cursor.1;

        if col >= char_count {
            // At end of line, move to next line
            if self.cursor.0 < self.lines.len() - 1 {
                self.cursor.0 += 1;
                self.cursor.1 = 0;
                let new_line = self.current_line().to_string();
                let new_char_count = new_line.chars().count();
                // Skip leading whitespace
                while self.cursor.1 < new_char_count
                    && Self::is_whitespace_at(&new_line, self.cursor.1)
                {
                    self.cursor.1 += 1;
                }
            }
            self.clamp_cursor();
            return;
        }

        // Skip current word (non-whitespace)
        while col < char_count && !Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // Skip whitespace to next word
        while col < char_count && Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // If we reached end of line, move to next line
        if col >= char_count && self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
            let new_line = self.current_line().to_string();
            let new_char_count = new_line.chars().count();
            // Skip leading whitespace on new line
            while self.cursor.1 < new_char_count && Self::is_whitespace_at(&new_line, self.cursor.1)
            {
                self.cursor.1 += 1;
            }
        } else {
            self.cursor.1 = col;
        }

        self.clamp_cursor();
    }

    /// Helper: Move to previous word once
    fn move_prev_word_once(&mut self) {
        let mut col = self.cursor.1;

        // If at start of line, move to end of previous line
        if col == 0 {
            if self.cursor.0 > 0 {
                self.cursor.0 -= 1;
                let line = self.current_line().to_string();
                self.cursor.1 = line.chars().count().saturating_sub(1);
            }
            self.clamp_cursor();
            return;
        }

        let line = self.current_line().to_string();

        // Move back one position
        col = col.saturating_sub(1);

        // Skip whitespace backwards
        while col > 0 && Self::is_whitespace_at(&line, col) {
            col -= 1;
        }

        // Skip word backwards to find start
        while col > 0 && !Self::is_whitespace_at(&line, col.saturating_sub(1)) {
            col -= 1;
        }

        self.cursor.1 = col;
        self.clamp_cursor();
    }

    /// Helper: Move to end of word once
    fn move_end_word_once(&mut self) {
        let line = self.current_line().to_string();
        let char_count = line.chars().count();
        let mut col = self.cursor.1;

        // Move forward at least one character
        if col < char_count {
            col += 1;
        }

        // Skip whitespace
        while col < char_count && Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // Move to end of word (find next whitespace or end)
        while col < char_count && !Self::is_whitespace_at(&line, col) {
            col += 1;
        }

        // Position on last character of word (one before whitespace)
        if col > 0 && col <= char_count {
            col -= 1;
        }

        self.cursor.1 = col;
        self.clamp_cursor();
    }

    /// Check if character at position is whitespace
    fn is_whitespace_at(line: &str, pos: usize) -> bool {
        line.chars()
            .nth(pos)
            .map(|c| c.is_whitespace())
            .unwrap_or(false)
    }

    // ============================================================================
    // Find/Till Character Movement (f, F, t, T, ;, ,)
    // ============================================================================

    /// Find character forward (f)
    pub fn find_char_forward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        let start = self.cursor.1 + 1;

        for (i, &c) in chars.iter().enumerate().skip(start) {
            if c == ch {
                self.cursor.1 = i;
                self.last_find = Some(FindCommand::Forward(ch));
                return;
            }
        }
    }

    /// Find character backward (F)
    pub fn find_char_backward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();

        for i in (0..self.cursor.1).rev() {
            if chars[i] == ch {
                self.cursor.1 = i;
                self.last_find = Some(FindCommand::Backward(ch));
                return;
            }
        }
    }

    /// Till character forward (t)
    pub fn till_char_forward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        // For till, we search from cursor + 1, but we want to find the character
        // that's at least 2 positions away (so we skip the immediately adjacent one)
        let start = self.cursor.1 + 2;

        for (i, &c) in chars.iter().enumerate().skip(start) {
            if c == ch {
                self.cursor.1 = i.saturating_sub(1);
                self.last_find = Some(FindCommand::TillForward(ch));
                return;
            }
        }
    }

    /// Till character backward (T)
    pub fn till_char_backward(&mut self, ch: char) {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();

        for i in (0..self.cursor.1).rev() {
            if chars[i] == ch {
                self.cursor.1 = (i + 1).min(chars.len().saturating_sub(1));
                self.last_find = Some(FindCommand::TillBackward(ch));
                return;
            }
        }
    }

    /// Repeat last find (;)
    pub fn repeat_find(&mut self) {
        if let Some(find) = self.last_find {
            match find {
                FindCommand::Forward(ch) => self.find_char_forward(ch),
                FindCommand::Backward(ch) => self.find_char_backward(ch),
                FindCommand::TillForward(ch) => self.till_char_forward(ch),
                FindCommand::TillBackward(ch) => self.till_char_backward(ch),
            }
        }
    }

    /// Repeat last find in reverse (,)
    pub fn repeat_find_reverse(&mut self) {
        if let Some(find) = self.last_find {
            match find {
                FindCommand::Forward(ch) => self.find_char_backward(ch),
                FindCommand::Backward(ch) => self.find_char_forward(ch),
                FindCommand::TillForward(ch) => self.till_char_backward(ch),
                FindCommand::TillBackward(ch) => self.till_char_forward(ch),
            }
        }
    }
}
