//! Search functionality (/, n, N, *, search highlighting)

use super::VimEditor;

impl VimEditor {
    // ============================================================================
    // Search Operations
    // ============================================================================

    /// Search forward for pattern (/)
    ///
    /// Sets the search pattern, finds all matches, and jumps to the first match
    /// AFTER the current cursor position (like Vim's `/` command).
    pub fn search_forward(&mut self, pattern: String) {
        if pattern.is_empty() {
            return;
        }
        self.search_pattern = Some(pattern);
        self.find_all_matches();

        // Jump to first match AFTER cursor (or wrap to first match)
        if !self.search_matches.is_empty() {
            let current_pos = self.cursor;
            let next_idx = self
                .search_matches
                .iter()
                .position(|&pos| pos > current_pos)
                .unwrap_or(0); // Wrap to first match

            self.current_match = Some(next_idx);
            let (line, col) = self.search_matches[next_idx];
            self.cursor = (line, col);
            self.clamp_cursor();
        }
    }

    /// Search for word under cursor (*)
    ///
    /// Gets the word under the cursor and searches for all occurrences.
    pub fn search_word_under_cursor(&mut self) {
        if let Some(word) = self.word_under_cursor() {
            self.search_forward(word);
        }
    }

    /// Jump to next search match (n)
    pub fn jump_to_next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        // Find next match after cursor
        let current_pos = self.cursor;
        let next_idx = self
            .search_matches
            .iter()
            .position(|&pos| pos > current_pos)
            .unwrap_or(0); // Wrap to first match

        self.current_match = Some(next_idx);
        let (line, col) = self.search_matches[next_idx];
        self.cursor = (line, col);
        self.clamp_cursor();
    }

    /// Jump to previous search match (N)
    pub fn jump_to_prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        // Find previous match before cursor
        let current_pos = self.cursor;
        let prev_idx = self
            .search_matches
            .iter()
            .rposition(|&pos| pos < current_pos)
            .unwrap_or(self.search_matches.len() - 1); // Wrap to last match

        self.current_match = Some(prev_idx);
        let (line, col) = self.search_matches[prev_idx];
        self.cursor = (line, col);
        self.clamp_cursor();
    }

    /// Clear search (:noh or :nohlsearch)
    pub fn clear_search(&mut self) {
        self.search_pattern = None;
        self.search_matches.clear();
        self.current_match = None;
    }

    // ============================================================================
    // Search Query Methods
    // ============================================================================

    /// Get search pattern
    pub fn search_pattern(&self) -> Option<&str> {
        self.search_pattern.as_deref()
    }

    /// Get search matches for UI highlighting
    pub fn search_matches(&self) -> &[(usize, usize)] {
        &self.search_matches
    }

    /// Get current match index
    pub fn current_match_index(&self) -> Option<usize> {
        self.current_match
    }

    /// Get number of search matches
    pub fn search_match_count(&self) -> usize {
        self.search_matches.len()
    }

    // ============================================================================
    // Internal Helpers
    // ============================================================================

    /// Find all matches of current search pattern
    fn find_all_matches(&mut self) {
        self.search_matches.clear();
        self.current_match = None;

        if let Some(pattern) = &self.search_pattern {
            for (line_idx, line) in self.lines.iter().enumerate() {
                // Use char indices to handle multi-byte characters correctly
                let chars: Vec<char> = line.chars().collect();
                let mut char_pos = 0;

                while char_pos < chars.len() {
                    let remaining: String = chars[char_pos..].iter().collect();
                    if let Some(match_pos) = remaining.find(pattern) {
                        // Convert byte position to char position
                        let match_char_pos = remaining[..match_pos].chars().count();
                        self.search_matches
                            .push((line_idx, char_pos + match_char_pos));
                        // Move past this match (by at least 1 char to avoid infinite loop)
                        char_pos += match_char_pos + pattern.chars().count().max(1);
                    } else {
                        break;
                    }
                }
            }
        }
    }

    /// Get word under cursor for * search or display
    pub fn word_under_cursor(&self) -> Option<String> {
        let line = self.current_line();
        let chars: Vec<char> = line.chars().collect();
        if self.cursor.1 >= chars.len() {
            return None;
        }

        // Find word boundaries (alphanumeric + underscore)
        let mut start = self.cursor.1;
        let mut end = self.cursor.1;

        // Expand left to word start
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Expand right to word end
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }
}
