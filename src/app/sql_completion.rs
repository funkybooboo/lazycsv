//! SQL completion popup types and fuzzy matching.

/// Maximum visible rows in the completion popup
pub const COMPLETION_MAX_VISIBLE: usize = 10;

/// Category of a completion item
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionKind {
    Keyword,
    Function,
    Column,
    Table,
}

impl CompletionKind {
    /// Short tag shown in the completion popup
    pub fn tag(self) -> &'static str {
        match self {
            CompletionKind::Keyword => "[K]",
            CompletionKind::Function => "[F]",
            CompletionKind::Column => "[C]",
            CompletionKind::Table => "[T]",
        }
    }

    pub fn color(self) -> ratatui::style::Color {
        match self {
            CompletionKind::Keyword => ratatui::style::Color::Cyan,
            CompletionKind::Function => ratatui::style::Color::Yellow,
            CompletionKind::Column => ratatui::style::Color::Green,
            CompletionKind::Table => ratatui::style::Color::Magenta,
        }
    }
}

/// SQL query history popup state
#[derive(Debug, Clone, Default)]
pub struct SqlHistoryPopup {
    /// Index of the currently highlighted entry (0 = most recent)
    pub selected: usize,
    /// First visible entry index (for scrolling)
    pub scroll_offset: usize,
    /// True when the first `d` of a `dd` delete has been pressed
    pub pending_d: bool,
}

impl SqlHistoryPopup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clamp selected/scroll_offset after the history list shrinks.
    pub fn clamp(&mut self, len: usize, visible: usize) {
        if len == 0 {
            self.selected = 0;
            self.scroll_offset = 0;
            return;
        }
        if self.selected >= len {
            self.selected = len - 1;
        }
        let max_scroll = len.saturating_sub(visible);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
    }

    pub fn move_up(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn move_down(&mut self, len: usize, visible: usize) {
        if len == 0 {
            return;
        }
        if self.selected + 1 < len {
            self.selected += 1;
            if self.selected >= self.scroll_offset + visible {
                self.scroll_offset = self.selected + 1 - visible;
            }
        }
    }
}

/// A single item in the completion popup
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub text: String,
    pub kind: CompletionKind,
    /// If set, selecting this item replaces the entire editor content
    /// (used for query templates).
    pub template: Option<String>,
    /// If set, these steps are executed after the template is inserted.
    /// Each step is either literal text or a table-pick prompt.
    pub template_steps: Vec<TemplateStep>,
}

/// SQL completion popup state
#[derive(Debug, Clone)]
pub struct SqlCompletion {
    /// All available items (unfiltered)
    pub all_items: Vec<CompletionItem>,
    /// Current filter/search string
    pub filter: String,
    /// Number of characters of the partial word that were already typed before
    /// the popup was opened. Used to replace the prefix on accept.
    pub prefix_len: usize,
    /// Currently selected index (within filtered list)
    pub selected: usize,
    /// Scroll offset for the visible window
    pub scroll_offset: usize,
}

impl SqlCompletion {
    pub fn new(items: Vec<CompletionItem>, prefix: &str) -> Self {
        Self {
            all_items: items,
            filter: prefix.to_string(),
            prefix_len: prefix.chars().count(),
            selected: 0,
            scroll_offset: 0,
        }
    }

    /// Get the filtered list of items matching the current filter.
    /// Uses fuzzy matching: characters must appear in order but not contiguously.
    /// Results are sorted by match quality (prefix > substring > fuzzy).
    pub fn filtered_items(&self) -> Vec<&CompletionItem> {
        if self.filter.is_empty() {
            return self.all_items.iter().collect();
        }

        let filter_lower = self.filter.to_lowercase();
        let mut scored: Vec<(i32, &CompletionItem)> = self
            .all_items
            .iter()
            .filter_map(|item| {
                let name_lower = item.text.to_lowercase();
                fuzzy_match_score(&name_lower, &filter_lower).map(|score| (score, item))
            })
            .collect();

        // Sort by score descending (higher = better match)
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, item)| item).collect()
    }

    pub fn move_down(&mut self) {
        let count = self.filtered_items().len();
        if count > 0 {
            self.selected = (self.selected + 1).min(count - 1);
            if self.selected >= self.scroll_offset + COMPLETION_MAX_VISIBLE {
                self.scroll_offset = self.selected + 1 - COMPLETION_MAX_VISIBLE;
            }
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    /// Append a character to the filter and reset selection.
    pub fn push_filter(&mut self, ch: char) {
        self.filter.push(ch);
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Remove the last character from the filter.
    pub fn pop_filter(&mut self) {
        self.filter.pop();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn selected_item(&self) -> Option<&CompletionItem> {
        let filtered = self.filtered_items();
        filtered.get(self.selected).copied()
    }
}

/// Fuzzy match a name against a filter pattern.
///
/// Returns a score if the filter characters appear in order within the name.
/// Higher scores indicate better matches:
/// - 100: exact match
/// - 90: prefix match
/// - 80: substring (contiguous) match
/// - 50-79: fuzzy match (bonus for consecutive chars and early matches)
/// - None: no match
fn fuzzy_match_score(name: &str, filter: &str) -> Option<i32> {
    if filter.is_empty() {
        return Some(0);
    }
    if name == filter {
        return Some(100);
    }
    if name.starts_with(filter) {
        return Some(90);
    }
    if name.contains(filter) {
        return Some(80);
    }

    // Fuzzy: each filter char must appear in order
    let mut name_chars = name.chars().peekable();
    let mut score: i32 = 50;
    let mut last_match_pos = 0usize;

    for (fi, fc) in filter.chars().enumerate() {
        let mut found = false;
        let mut pos = last_match_pos;
        for nc in name_chars.by_ref() {
            if nc == fc {
                // Bonus for consecutive matches
                if fi > 0 && pos == last_match_pos {
                    score += 3;
                }
                // Bonus for matching near the start
                if pos < 3 {
                    score += 2;
                }
                last_match_pos = pos + 1;
                found = true;
                break;
            }
            pos += 1;
        }
        if !found {
            return None;
        }
    }

    Some(score)
}

/// A step in a multi-part query template.
#[derive(Debug, Clone)]
pub enum TemplateStep {
    /// Insert literal text at cursor.
    Text(String),
    /// Prompt the user to pick a table name via completion popup.
    PickTable,
    /// Prompt the user to pick a column from the table aliased as `alias`
    /// in the current query. The alias is resolved at execution time.
    /// Use `"*"` to pick from all referenced tables.
    PickColumn(String),
    /// Insert the same column name that was last picked via `PickColumn`.
    RepeatLastColumn,
    /// Replace the entire editor content with a format string.
    /// `{table}` is replaced with the last picked table name,
    /// `{column}` is replaced with the last picked column name.
    Assemble(String),
}
