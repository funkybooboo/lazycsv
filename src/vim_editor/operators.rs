//! Operator commands (x, dd, yy, p, i, a, o, O, r, J, etc.)

use super::VimEditor;

impl VimEditor {
    // ============================================================================
    // Insert Mode Text Input
    // ============================================================================

    /// Insert character at cursor position (in Insert mode)
    pub fn insert_char(&mut self, c: char) {
        let char_col = self.cursor.1;
        let line = self.current_line_mut();
        let char_count = line.chars().count();
        let char_col = char_col.min(char_count);
        // Convert char position to byte position for String::insert
        let byte_pos = line.char_indices()
            .nth(char_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(byte_pos, c);
        self.cursor.1 = char_col + 1;
    }

    /// Delete character before cursor (Backspace in Insert mode)
    pub fn delete_char_before(&mut self) {
        if self.cursor.1 > 0 {
            let char_col = self.cursor.1 - 1;
            let line = self.current_line_mut();
            let char_count = line.chars().count();
            if char_col < char_count {
                // Convert char position to byte position for String::remove
                if let Some((byte_pos, _)) = line.char_indices().nth(char_col) {
                    line.remove(byte_pos);
                }
            }
            self.cursor.1 = char_col;
        } else if self.cursor.0 > 0 {
            // At start of line - join with previous line
            let current_line = self.lines.remove(self.cursor.0);
            self.cursor.0 -= 1;
            let prev_line_chars = self.lines[self.cursor.0].chars().count();
            self.lines[self.cursor.0].push_str(&current_line);
            self.cursor.1 = prev_line_chars;
        }
    }

    /// Delete character at cursor (Delete key in Insert mode)
    pub fn delete_char_at(&mut self) {
        let char_col = self.cursor.1;
        let line_idx = self.cursor.0;

        if line_idx < self.lines.len() {
            let line = &mut self.lines[line_idx];
            let char_count = line.chars().count();
            if char_col < char_count {
                if let Some((byte_pos, _)) = line.char_indices().nth(char_col) {
                    line.remove(byte_pos);
                }
                return;
            }
        }

        // At end of line - join with next line
        if line_idx < self.lines.len() - 1 {
            let next_line = self.lines.remove(line_idx + 1);
            self.lines[line_idx].push_str(&next_line);
        }
    }

    /// Insert newline at cursor (Enter in Insert mode)
    pub fn newline(&mut self) {
        let char_col = self.cursor.1;
        let line_idx = self.cursor.0;

        let byte_pos = self.lines[line_idx].char_indices()
            .nth(char_col)
            .map(|(i, _)| i)
            .unwrap_or(self.lines[line_idx].len());
        let rest = self.lines[line_idx].split_off(byte_pos);
        self.cursor.0 = line_idx + 1;
        self.lines.insert(self.cursor.0, rest);
        self.cursor.1 = 0;
    }

    // ============================================================================
    // Normal Mode Basic Operators
    // ============================================================================

    /// Delete character under cursor (x in Normal mode)
    pub fn delete_char(&mut self) {
        let char_col = self.cursor.1;
        let line = self.current_line_mut();
        let char_count = line.chars().count();
        if char_col < char_count {
            if let Some((byte_pos, _)) = line.char_indices().nth(char_col) {
                line.remove(byte_pos);
            }
        }
        self.clamp_cursor();
    }

    /// Delete current line (dd in Normal mode)
    pub fn delete_line(&mut self) {
        if self.lines.len() == 1 {
            // Last line - just clear it
            self.clipboard = vec![self.lines[0].clone()];
            self.lines[0].clear();
            self.cursor.1 = 0;
        } else {
            // Remove line and store in clipboard
            let deleted = self.lines.remove(self.cursor.0);
            self.clipboard = vec![deleted];

            // Adjust cursor if we deleted the last line
            if self.cursor.0 >= self.lines.len() {
                self.cursor.0 = self.lines.len().saturating_sub(1);
            }
        }
        self.clamp_cursor();
    }

    /// Delete from cursor to end of line (D in Normal mode)
    pub fn delete_to_eol(&mut self) {
        let cursor_col = self.cursor.1;
        let line = self.current_line_mut();
        if cursor_col < line.len() {
            let deleted: String = line.drain(cursor_col..).collect();
            self.clipboard = vec![deleted];
        }
        self.clamp_cursor();
    }

    /// Yank (copy) current line (yy in Normal mode)
    pub fn yank_line(&mut self) {
        let line = self.current_line().to_string();
        self.clipboard = vec![line];
    }

    /// Paste clipboard below current line (p in Normal mode)
    pub fn paste_below(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }

        for (i, line) in self.clipboard.iter().enumerate() {
            self.lines.insert(self.cursor.0 + 1 + i, line.clone());
        }

        // Move cursor to first pasted line
        self.cursor.0 += 1;
        self.cursor.1 = 0;
        self.clamp_cursor();
    }

    /// Paste clipboard above current line (P in Normal mode)
    pub fn paste_above(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }

        for (i, line) in self.clipboard.iter().enumerate() {
            self.lines.insert(self.cursor.0 + i, line.clone());
        }

        // Cursor stays on same line (which is now pushed down)
        self.cursor.1 = 0;
        self.clamp_cursor();
    }

    /// Substitute character (s in Normal mode) - delete char and enter insert
    pub fn substitute_char(&mut self) {
        self.delete_char();
        self.enter_insert_mode();
    }

    // ============================================================================
    // Enter Insert Mode Variations
    // ============================================================================

    /// Enter insert mode before cursor (i)
    pub fn insert_before(&mut self) {
        self.enter_insert_mode();
        // Cursor stays at current position
    }

    /// Enter insert mode after cursor (a)
    pub fn insert_after(&mut self) {
        self.enter_insert_mode();
        // Move cursor one position right
        if self.cursor.1 < self.current_line().len() {
            self.cursor.1 += 1;
        }
    }

    /// Insert new line below and enter insert mode (o)
    pub fn insert_line_below(&mut self) {
        self.cursor.0 += 1;
        self.lines.insert(self.cursor.0, String::new());
        self.cursor.1 = 0;
        self.enter_insert_mode();
    }

    /// Insert new line above and enter insert mode (O)
    pub fn insert_line_above(&mut self) {
        self.lines.insert(self.cursor.0, String::new());
        self.cursor.1 = 0;
        self.enter_insert_mode();
    }

    /// Enter insert mode at start of line (I)
    pub fn insert_at_line_start(&mut self) {
        self.move_to_first_non_blank();
        self.enter_insert_mode();
    }

    /// Enter insert mode at end of line (A)
    pub fn insert_at_line_end(&mut self) {
        let line_len = self.current_line().len();
        self.cursor.1 = line_len;
        self.enter_insert_mode();
    }

    // ============================================================================
    // Change Operators
    // ============================================================================

    /// Change character (cl or s)
    pub fn change_char(&mut self) {
        self.delete_char();
        self.enter_insert_mode();
    }

    /// Change entire line (cc)
    pub fn change_line(&mut self) {
        let line = self.current_line_mut();
        line.clear();
        self.cursor.1 = 0;
        self.enter_insert_mode();
    }

    /// Change to end of line (C)
    pub fn change_to_eol(&mut self) {
        let cursor_col = self.cursor.1;
        let line = self.current_line_mut();
        line.truncate(cursor_col);
        self.enter_insert_mode();
    }

    // ============================================================================
    // Replace & Join
    // ============================================================================

    /// Replace single character (r)
    pub fn replace_char(&mut self, c: char) {
        let cursor_col = self.cursor.1;
        let line = self.current_line_mut();
        let chars: Vec<char> = line.chars().collect();
        if cursor_col < chars.len() {
            let mut new_chars = chars;
            new_chars[cursor_col] = c;
            *line = new_chars.into_iter().collect();
        }
    }

    /// Join current line with next (J)
    pub fn join_lines(&mut self) {
        let line_idx = self.cursor.0;
        if line_idx + 1 < self.lines.len() {
            let next_line = self.lines.remove(line_idx + 1);
            let current = self.current_line_mut();
            if !current.is_empty() && !next_line.is_empty() {
                current.push(' ');
            }
            current.push_str(&next_line);
        }
    }

    // ============================================================================
    // Indent/Dedent
    // ============================================================================

    /// Indent line (>>)
    pub fn indent_line(&mut self) {
        self.current_line_mut().insert_str(0, "  ");
        self.cursor.1 += 2;
    }

    /// Dedent line (<<)
    pub fn dedent_line(&mut self) {
        let line = self.current_line_mut();
        if line.starts_with("  ") {
            line.drain(0..2);
            self.cursor.1 = self.cursor.1.saturating_sub(2);
        } else if line.starts_with('\t') {
            line.remove(0);
            self.cursor.1 = self.cursor.1.saturating_sub(1);
        }
    }
}
