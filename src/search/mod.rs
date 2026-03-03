use crate::{ColIndex, Document, RowIndex};
use regex::RegexBuilder;

#[derive(Debug)]
pub struct SearchState {
    pub pattern: String,
    pub matches: Vec<(RowIndex, ColIndex)>,
    pub current_match: Option<usize>,
}

impl SearchState {
    pub fn new(pattern: String, matches: Vec<(RowIndex, ColIndex)>) -> Self {
        Self {
            pattern,
            matches,
            current_match: None,
        }
    }

    /// Find the next match after the given cursor position, wrapping around.
    /// Returns the match position and whether the search wrapped.
    pub fn jump_to_next(
        &mut self,
        cursor_row: RowIndex,
        cursor_col: ColIndex,
    ) -> Option<((RowIndex, ColIndex), bool)> {
        if self.matches.is_empty() {
            return None;
        }

        // Find first match strictly after cursor position
        let pos = self
            .matches
            .iter()
            .position(|&(r, c)| r > cursor_row || (r == cursor_row && c > cursor_col));

        let (idx, wrapped) = match pos {
            Some(idx) => (idx, false),
            None => (0, true), // Wrap to first match
        };

        self.current_match = Some(idx);
        Some((self.matches[idx], wrapped))
    }

    /// Find the previous match before the given cursor position, wrapping around.
    /// Returns the match position and whether the search wrapped.
    pub fn jump_to_prev(
        &mut self,
        cursor_row: RowIndex,
        cursor_col: ColIndex,
    ) -> Option<((RowIndex, ColIndex), bool)> {
        if self.matches.is_empty() {
            return None;
        }

        // Find last match strictly before cursor position
        let pos = self
            .matches
            .iter()
            .rposition(|&(r, c)| r < cursor_row || (r == cursor_row && c < cursor_col));

        let (idx, wrapped) = match pos {
            Some(idx) => (idx, false),
            None => (self.matches.len() - 1, true), // Wrap to last match
        };

        self.current_match = Some(idx);
        Some((self.matches[idx], wrapped))
    }

    pub fn is_match(&self, row: RowIndex, col: ColIndex) -> bool {
        self.matches.iter().any(|&(r, c)| r == row && c == col)
    }

    pub fn is_current_match(&self, row: RowIndex, col: ColIndex) -> bool {
        if let Some(idx) = self.current_match {
            if let Some(&(r, c)) = self.matches.get(idx) {
                return r == row && c == col;
            }
        }
        false
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Returns "[current/total]" display string for the status bar.
    pub fn display_position(&self) -> String {
        match self.current_match {
            Some(idx) => format!("[{}/{}]", idx + 1, self.matches.len()),
            None => format!("[0/{}]", self.matches.len()),
        }
    }
}

/// Find all cells matching the pattern (case-insensitive).
/// Tries regex first; falls back to literal substring if the pattern is invalid regex.
/// Returns matches sorted by (row, col) from natural iteration order.
pub fn find_matches(document: &Document, pattern: &str) -> Vec<(RowIndex, ColIndex)> {
    let mut matches = Vec::new();

    // Try to compile as regex (case-insensitive). Fall back to literal substring.
    if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
        for (row_idx, row) in document.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if re.is_match(cell) {
                    matches.push((RowIndex::new(row_idx), ColIndex::new(col_idx)));
                }
            }
        }
    } else {
        let pattern_lower = pattern.to_lowercase();
        for (row_idx, row) in document.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.to_lowercase().contains(&pattern_lower) {
                    matches.push((RowIndex::new(row_idx), ColIndex::new(col_idx)));
                }
            }
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(rows: Vec<Vec<&str>>) -> Document {
        let string_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|r| r.into_iter().map(|s| s.to_string()).collect())
            .collect();
        let headers = string_rows[0].clone();
        let data = string_rows[1..].to_vec();
        Document::new(headers, data, "test.csv".to_string())
    }

    #[test]
    fn test_find_matches_basic() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Alice", "Portland"],
            vec!["Bob", "Boston"],
            vec!["Charlie", "Portland"],
        ]);
        let matches = find_matches(&doc, "Portland");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (RowIndex::new(1), ColIndex::new(1)));
        assert_eq!(matches[1], (RowIndex::new(3), ColIndex::new(1)));
    }

    #[test]
    fn test_find_matches_case_insensitive() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Alice", "PORTLAND"],
            vec!["Bob", "portland"],
        ]);
        let matches = find_matches(&doc, "Portland");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_find_matches_includes_headers() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Alice", "Portland"],
        ]);
        let matches = find_matches(&doc, "Name");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (RowIndex::new(0), ColIndex::new(0)));
    }

    #[test]
    fn test_find_matches_no_results() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Alice", "Portland"],
        ]);
        let matches = find_matches(&doc, "xyz_not_found");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_find_matches_substring() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Alice", "Portland"],
        ]);
        let matches = find_matches(&doc, "land");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (RowIndex::new(1), ColIndex::new(1)));
    }

    #[test]
    fn test_jump_to_next_basic() {
        let matches = vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(1)),
            (RowIndex::new(3), ColIndex::new(0)),
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        // Cursor at (0, 0), should find match at (1, 0)
        let result = state.jump_to_next(RowIndex::new(0), ColIndex::new(0));
        assert_eq!(result, Some(((RowIndex::new(1), ColIndex::new(0)), false)));
        assert_eq!(state.current_match, Some(0));
    }

    #[test]
    fn test_jump_to_next_wraps() {
        let matches = vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(0)),
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        // Cursor past all matches, should wrap to first
        let result = state.jump_to_next(RowIndex::new(5), ColIndex::new(0));
        assert_eq!(result, Some(((RowIndex::new(1), ColIndex::new(0)), true)));
    }

    #[test]
    fn test_jump_to_prev_basic() {
        let matches = vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(1)),
            (RowIndex::new(3), ColIndex::new(0)),
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        // Cursor at (3, 0), should find match at (2, 1)
        let result = state.jump_to_prev(RowIndex::new(3), ColIndex::new(0));
        assert_eq!(result, Some(((RowIndex::new(2), ColIndex::new(1)), false)));
        assert_eq!(state.current_match, Some(1));
    }

    #[test]
    fn test_jump_to_prev_wraps() {
        let matches = vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(0)),
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        // Cursor before all matches, should wrap to last
        let result = state.jump_to_prev(RowIndex::new(0), ColIndex::new(0));
        assert_eq!(result, Some(((RowIndex::new(2), ColIndex::new(0)), true)));
    }

    #[test]
    fn test_jump_empty_matches() {
        let mut state = SearchState::new("test".to_string(), vec![]);
        assert!(state.jump_to_next(RowIndex::new(0), ColIndex::new(0)).is_none());
        assert!(state.jump_to_prev(RowIndex::new(0), ColIndex::new(0)).is_none());
    }

    #[test]
    fn test_is_match() {
        let matches = vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(1)),
        ];
        let state = SearchState::new("test".to_string(), matches);

        assert!(state.is_match(RowIndex::new(1), ColIndex::new(0)));
        assert!(state.is_match(RowIndex::new(2), ColIndex::new(1)));
        assert!(!state.is_match(RowIndex::new(0), ColIndex::new(0)));
        assert!(!state.is_match(RowIndex::new(1), ColIndex::new(1)));
    }

    #[test]
    fn test_is_current_match() {
        let matches = vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(1)),
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        // No current match yet
        assert!(!state.is_current_match(RowIndex::new(1), ColIndex::new(0)));

        // Jump to first match
        state.jump_to_next(RowIndex::new(0), ColIndex::new(0));
        assert!(state.is_current_match(RowIndex::new(1), ColIndex::new(0)));
        assert!(!state.is_current_match(RowIndex::new(2), ColIndex::new(1)));
    }

    #[test]
    fn test_display_position() {
        let matches = vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(0)),
            (RowIndex::new(3), ColIndex::new(0)),
        ];
        let mut state = SearchState::new("test".to_string(), matches);

        assert_eq!(state.display_position(), "[0/3]");

        state.jump_to_next(RowIndex::new(0), ColIndex::new(0));
        assert_eq!(state.display_position(), "[1/3]");

        state.jump_to_next(RowIndex::new(1), ColIndex::new(0));
        assert_eq!(state.display_position(), "[2/3]");
    }

    #[test]
    fn test_match_count() {
        let state = SearchState::new("test".to_string(), vec![
            (RowIndex::new(1), ColIndex::new(0)),
            (RowIndex::new(2), ColIndex::new(0)),
        ]);
        assert_eq!(state.match_count(), 2);
    }

    #[test]
    fn test_find_matches_regex_anchor_start() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Portland", "East Portland"],
            vec!["Bob", "Portland"],
        ]);
        // ^Portland should match cells that START with "Portland"
        let matches = find_matches(&doc, "^Portland");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (RowIndex::new(1), ColIndex::new(0))); // "Portland"
        assert_eq!(matches[1], (RowIndex::new(2), ColIndex::new(1))); // "Portland"
        // "East Portland" should NOT match ^Portland
    }

    #[test]
    fn test_find_matches_regex_anchor_end() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Johnson", "Portland"],
            vec!["John", "Boston"],
        ]);
        // son$ should match cells that END with "son"
        let matches = find_matches(&doc, "son$");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (RowIndex::new(1), ColIndex::new(0))); // "Johnson"
    }

    #[test]
    fn test_find_matches_regex_pattern() {
        let doc = make_doc(vec![
            vec!["Name", "Age"],
            vec!["Alice", "25"],
            vec!["Bob", "130"],
            vec!["Charlie", "7"],
        ]);
        // Match 1-2 digit numbers only
        let matches = find_matches(&doc, r"^\d{1,2}$");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (RowIndex::new(1), ColIndex::new(1))); // "25"
        assert_eq!(matches[1], (RowIndex::new(3), ColIndex::new(1))); // "7"
    }

    #[test]
    fn test_find_matches_invalid_regex_falls_back_to_literal() {
        let doc = make_doc(vec![
            vec!["Name", "Value"],
            vec!["test[", "other"],
        ]);
        // "[" is invalid regex — should fall back to literal substring match
        let matches = find_matches(&doc, "[");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (RowIndex::new(1), ColIndex::new(0)));
    }

    #[test]
    fn test_find_matches_regex_case_insensitive() {
        let doc = make_doc(vec![
            vec!["Name", "City"],
            vec!["Alice", "PORTLAND"],
            vec!["Bob", "portland"],
        ]);
        let matches = find_matches(&doc, "^portland$");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_single_match_wraps_both_directions() {
        let matches = vec![(RowIndex::new(1), ColIndex::new(0))];
        let mut state = SearchState::new("test".to_string(), matches);

        // Next from after the match wraps
        let result = state.jump_to_next(RowIndex::new(1), ColIndex::new(0));
        assert_eq!(result, Some(((RowIndex::new(1), ColIndex::new(0)), true)));

        // Prev from before the match wraps
        let result = state.jump_to_prev(RowIndex::new(1), ColIndex::new(0));
        assert_eq!(result, Some(((RowIndex::new(1), ColIndex::new(0)), true)));
    }
}
