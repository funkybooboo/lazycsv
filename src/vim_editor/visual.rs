//! Visual mode selection and operations (v, V, visual delete/yank/change)

use super::{Selection, VimEditor, VimMode};

impl VimEditor {
    // ============================================================================
    // Visual Mode Entry/Exit
    // ============================================================================

    /// Enter visual mode (character-wise selection)
    pub fn enter_visual_mode(&mut self) {
        self.mode = VimMode::Visual;
        self.visual_anchor = Some(self.cursor);
    }

    /// Enter visual line mode (line-wise selection)
    pub fn enter_visual_line_mode(&mut self) {
        self.mode = VimMode::VisualLine;
        self.visual_anchor = Some((self.cursor.0, 0));
    }

    /// Exit visual mode (return to normal mode)
    pub fn exit_visual_mode(&mut self) {
        self.mode = VimMode::Normal;
        self.visual_anchor = None;
    }

    /// Toggle visual mode (enter if not in visual, exit if in visual)
    pub fn toggle_visual_mode(&mut self) {
        if matches!(self.mode, VimMode::Visual) {
            self.exit_visual_mode();
        } else {
            self.enter_visual_mode();
        }
    }

    /// Toggle visual line mode
    pub fn toggle_visual_line_mode(&mut self) {
        if matches!(self.mode, VimMode::VisualLine) {
            self.exit_visual_mode();
        } else {
            self.enter_visual_line_mode();
        }
    }

    // ============================================================================
    // Visual Selection Query
    // ============================================================================

    /// Get the current visual selection
    ///
    /// Returns None if not in visual mode, otherwise returns the selection
    /// with start and end positions normalized (start <= end).
    pub fn visual_selection(&self) -> Option<Selection> {
        let anchor = self.visual_anchor?;
        let cursor = self.cursor;

        match self.mode {
            VimMode::Visual => {
                let (start, end) = if anchor <= cursor {
                    (anchor, cursor)
                } else {
                    (cursor, anchor)
                };
                Some(Selection::CharWise { start, end })
            }
            VimMode::VisualLine => {
                let (start_line, end_line) = if anchor.0 <= cursor.0 {
                    (anchor.0, cursor.0)
                } else {
                    (cursor.0, anchor.0)
                };
                Some(Selection::LineWise {
                    start_line,
                    end_line,
                })
            }
            _ => None,
        }
    }

    /// Get visual anchor position (where selection started)
    pub fn visual_anchor(&self) -> Option<(usize, usize)> {
        self.visual_anchor
    }

    // ============================================================================
    // Visual Mode Operations
    // ============================================================================

    /// Delete visual selection
    pub fn delete_selection(&mut self) {
        if let Some(selection) = self.visual_selection() {
            match selection {
                Selection::CharWise { start, end } => {
                    // Delete characters from start to end
                    if start.0 == end.0 {
                        // Single line selection
                        let line = &mut self.lines[start.0];
                        let chars: Vec<char> = line.chars().collect();
                        let start_col = start.1.min(chars.len());
                        let end_col = (end.1 + 1).min(chars.len());
                        let new_line: String = chars
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i < start_col || *i >= end_col)
                            .map(|(_, c)| c)
                            .collect();
                        *line = new_line;
                        self.cursor = start;
                    } else {
                        // Multi-line selection - delete entire lines
                        self.lines.drain(start.0..=end.0);
                        if self.lines.is_empty() {
                            self.lines.push(String::new());
                        }
                        self.cursor = (start.0.min(self.lines.len() - 1), 0);
                    }
                }
                Selection::LineWise {
                    start_line,
                    end_line,
                } => {
                    // Delete entire lines and save to clipboard
                    let deleted: Vec<String> = self.lines.drain(start_line..=end_line).collect();
                    self.clipboard = deleted;
                    if self.lines.is_empty() {
                        self.lines.push(String::new());
                    }
                    self.cursor = (start_line.min(self.lines.len() - 1), 0);
                }
            }
            self.exit_visual_mode();
            self.clamp_cursor();
        }
    }

    /// Yank (copy) visual selection
    pub fn yank_selection(&mut self) {
        if let Some(selection) = self.visual_selection() {
            match selection {
                Selection::CharWise { start, end } => {
                    if start.0 == end.0 {
                        // Single line - yank substring
                        let line = &self.lines[start.0];
                        let chars: Vec<char> = line.chars().collect();
                        let start_col = start.1.min(chars.len());
                        let end_col = (end.1 + 1).min(chars.len());
                        let yanked: String = chars[start_col..end_col].iter().collect();
                        self.clipboard = vec![yanked];
                    } else {
                        // Multi-line - yank entire lines
                        self.clipboard = self.lines[start.0..=end.0].to_vec();
                    }
                }
                Selection::LineWise {
                    start_line,
                    end_line,
                } => {
                    self.clipboard = self.lines[start_line..=end_line].to_vec();
                }
            }
            self.exit_visual_mode();
        }
    }

    /// Change visual selection (delete and enter insert mode)
    pub fn change_selection(&mut self) {
        self.delete_selection();
        self.enter_insert_mode();
    }

    /// Indent visual selection (shift right)
    pub fn indent_selection(&mut self) {
        if let Some(selection) = self.visual_selection() {
            let (start_line, end_line) = match selection {
                Selection::CharWise { start, end } => (start.0, end.0),
                Selection::LineWise {
                    start_line,
                    end_line,
                } => (start_line, end_line),
            };

            // Indent all lines in selection
            for line_idx in start_line..=end_line {
                if line_idx < self.lines.len() {
                    self.lines[line_idx].insert_str(0, "  ");
                }
            }

            self.exit_visual_mode();
        }
    }

    /// Dedent visual selection (shift left)
    pub fn dedent_selection(&mut self) {
        if let Some(selection) = self.visual_selection() {
            let (start_line, end_line) = match selection {
                Selection::CharWise { start, end } => (start.0, end.0),
                Selection::LineWise {
                    start_line,
                    end_line,
                } => (start_line, end_line),
            };

            // Dedent all lines in selection
            for line_idx in start_line..=end_line {
                if line_idx < self.lines.len() {
                    let line = &mut self.lines[line_idx];
                    if line.starts_with("  ") {
                        line.drain(0..2);
                    } else if line.starts_with('\t') {
                        line.remove(0);
                    }
                }
            }

            self.exit_visual_mode();
        }
    }
}
