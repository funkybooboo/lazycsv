//! Help overlay rendering with keybinding reference.
//!
//! Displays a modal help overlay showing all available keybindings and
//! navigation commands when triggered by '?'. Supports scrolling on small
//! screens.

use crate::config::Theme;
use ratatui::{
    style::Modifier,
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

// Modal size constants moved to src/ui/modal.rs
// Help overlay now uses standard 80% × 80% size (MODAL_LARGE_WIDTH/HEIGHT)

/// Build the help text lines
pub fn get_help_text() -> Vec<Line<'static>> {
    build_help_text()
}

/// Build the help text lines
fn build_help_text() -> Vec<Line<'static>> {
    let version = env!("CARGO_PKG_VERSION");
    vec![
        Line::from(Span::styled(
            format!("LazyCSV v{} - Keyboard Shortcuts & Commands", version),
            super::modal::bold_style(),
        )),
        Line::from(""),
        // ── Navigation ────────────────────────────────────────
        Line::from(Span::styled("NAVIGATION", super::modal::bold_style())),
        Line::from("  hjkl / arrows      Move cursor (with count: 5j, 10h)"),
        Line::from("  w / b / e          Next/prev/last non-empty cell"),
        Line::from("  gg                 First row (row 1)"),
        Line::from("  G                  Last row"),
        Line::from("  5g                 Jump to row 5"),
        Line::from("  0 / $              First/last column"),
        Line::from("  Ctrl+d / Ctrl+u    Page down/up"),
        Line::from("  :cA / :c1          Jump to column A (by letter or number)"),
        Line::from(""),
        // ── Editing ───────────────────────────────────────────
        Line::from(Span::styled("EDITING", super::modal::bold_style())),
        Line::from("  i / a              Edit cell (cursor at start/end)"),
        Line::from("  I / A              Edit cell (cursor at start/end)"),
        Line::from("  s                  Replace cell (clear + edit)"),
        Line::from("  r                  Replace cell content"),
        Line::from("  F2                 Edit cell"),
        Line::from("  x / Delete         Clear cell"),
        Line::from("  ~                  Toggle case"),
        Line::from("  g~                 Title case"),
        Line::from("  g.                 Toggle boolean"),
        Line::from("  cw                 Copy cell (internal + clipboard)"),
        Line::from("  u / Ctrl+r         Undo / redo"),
        Line::from("  .                  Repeat last edit"),
        Line::from(""),
        // ── Insert Mode ───────────────────────────────────────
        Line::from(Span::styled("INSERT MODE", super::modal::bold_style())),
        Line::from("  Enter              Commit, move down"),
        Line::from("  Shift+Enter        Commit, move up"),
        Line::from("  Tab / Shift+Tab    Commit, move right/left"),
        Line::from("  Esc                Cancel edit"),
        Line::from("  Backspace          Delete char before cursor"),
        Line::from("  Ctrl+w             Delete word backward"),
        Line::from("  Ctrl+u             Delete to start"),
        Line::from(""),
        // ── Row Operations ────────────────────────────────────
        Line::from(Span::styled("ROW OPERATIONS", super::modal::bold_style())),
        Line::from("  o / O              Insert row below/above"),
        Line::from("  dd                 Delete row (5dd for 5 rows)"),
        Line::from("  yy                 Yank (copy) row"),
        Line::from("  p / P              Paste row below/above"),
        Line::from("  gj / gk            Swap row down/up"),
        Line::from(""),
        // ── Column Operations ─────────────────────────────────
        Line::from(Span::styled(
            "COLUMN OPERATIONS (comma leader)",
            super::modal::bold_style(),
        )),
        Line::from("  ,dd                Delete column"),
        Line::from("  ,yy                Yank column"),
        Line::from("  ,p / ,P            Paste column right/left"),
        Line::from("  ,o / ,O            Insert column right/left"),
        Line::from(""),
        // ── Visual Mode ───────────────────────────────────────
        Line::from(Span::styled("VISUAL MODE", super::modal::bold_style())),
        Line::from("  v                  Visual block (rectangular select)"),
        Line::from("  V                  Visual line (whole rows)"),
        Line::from("  ,v                 Visual column (whole columns)"),
        Line::from("  hjkl / arrows      Extend selection"),
        Line::from("  d / y / p          Delete/yank/paste selection"),
        Line::from("  Y                  Copy selection to system clipboard (CSV)"),
        Line::from("  Esc                Cancel selection"),
        Line::from("  gv                 Re-select last selection"),
        Line::from("  gs                 Statistics popup for selection"),
        Line::from("  gg / G             Move to first/last row"),
        Line::from(""),
        Line::from("  Workflow: v -> hjkl to select -> Y to copy to clipboard"),
        Line::from("  Stats shown in status bar during selection"),
        Line::from(""),
        // ── Search ────────────────────────────────────────────
        Line::from(Span::styled("SEARCH", super::modal::bold_style())),
        Line::from("  /pattern           Search (regex supported)"),
        Line::from("  n / N              Next/previous match"),
        Line::from("  *                  Search current cell content"),
        Line::from("  :noh               Clear search highlighting"),
        Line::from(""),
        // ── Commands ──────────────────────────────────────────
        Line::from(Span::styled("COMMANDS", super::modal::bold_style())),
        Line::from("  :q / :wq / :q!     Quit (save/force)"),
        Line::from("  :w / :w!           Save file"),
        Line::from("  :h / :help         Show this help"),
        Line::from("  :f <name>          Rename current file"),
        Line::from("  :new A,B,C         Create new CSV with headers"),
        Line::from("  :delim ;           Set delimiter"),
        Line::from(""),
        // ── Sort & Data ───────────────────────────────────────
        Line::from(Span::styled("SORT & DATA", super::modal::bold_style())),
        Line::from("  :sort Col          Sort ascending by column(s)"),
        Line::from("  :sort! Col         Sort descending"),
        Line::from("  :stats [Col]       Column statistics (popup in visual mode)"),
        Line::from("  :sum / :avg Col    Sum/average of column"),
        Line::from("  :count / :distinct Column count/distinct values"),
        Line::from("  :footer            Toggle column totals footer row"),
        Line::from(""),
        // ── Find & Replace ────────────────────────────────────
        Line::from(Span::styled("FIND & REPLACE", super::modal::bold_style())),
        Line::from("  :s/old/new/        Replace in current cell"),
        Line::from("  :%s/old/new/g      Replace all in all cells"),
        Line::from("  :5,10s/old/new/g   Replace in row range"),
        Line::from("  :B,Ds/old/new/g    Replace in column range"),
        Line::from("  Flags: g (global), i (case-insensitive)"),
        Line::from(""),
        // ── Range Operations ──────────────────────────────────
        Line::from(Span::styled("RANGE OPERATIONS", super::modal::bold_style())),
        Line::from("  :5,10d / :5,10y    Delete/yank rows 5-10"),
        Line::from("  :B,Dd / :B,Dy      Delete/yank columns B-D"),
        Line::from("  :B,Dm A            Move columns B-D after A"),
        Line::from("  :%d / :%y          All data rows"),
        Line::from(""),
        // ── Column Width, Pin & Type ──────────────────────────
        Line::from(Span::styled(
            "COLUMN/ROW PIN & TYPE",
            super::modal::bold_style(),
        )),
        Line::from("  :width A 20        Set column A width to 20"),
        Line::from("  :width A auto      Auto-size column A"),
        Line::from("  :pin A,B           Pin columns (always visible)"),
        Line::from("  :pin 1,2           Pin rows (always visible)"),
        Line::from("  :unpin             Unpin all columns and rows"),
        Line::from("  :type A number     Set column type (number/date/boolean/text)"),
        Line::from("  :type A            Show column type"),
        Line::from("  :type A none       Clear column type"),
        Line::from(""),
        // ── Column Colors ────────────────────────────────────
        Line::from(Span::styled("COLUMN COLORS", super::modal::bold_style())),
        Line::from("  :bgcolor C red     Set column C background color"),
        Line::from("  :fgcolor C #ff0    Set column C foreground color"),
        Line::from("  :bgcolor C red = \"val\"  Conditional: equal to val"),
        Line::from("  :bgcolor C red > 100    Conditional: > < >= <= !="),
        Line::from("  :bgcolor C red ~ \"pat\"  Conditional: regex match"),
        Line::from("  :fgcolor C red > 32 && < 35  Compound AND"),
        Line::from("  :bgcolor C red = 1 || = 2    Compound OR"),
        Line::from("  :bgcolor C red (> 100 && < 200) || = 0  Grouped"),
        Line::from("  :bgcolor C list    List rules for column"),
        Line::from("  :bgcolor C remove 2  Remove rule #2"),
        Line::from("  :bgcolor C clear   Remove all column color rules"),
        Line::from(""),
        // ── Row Colors ───────────────────────────────────────
        Line::from(Span::styled("ROW COLORS", super::modal::bold_style())),
        Line::from("  :bgcolor 1 red     Set row 1 background color"),
        Line::from("  :fgcolor 1 blue    Set row 1 foreground color"),
        Line::from("  :bgcolor # red A > 100   Row conditional"),
        Line::from("  :bgcolor # red A > 1 && B = \"x\"  Multi-col"),
        Line::from("  :bgcolor # list    List row conditional rules"),
        Line::from("  :bgcolor # remove 2  Remove row conditional rule"),
        Line::from("  :bgcolor # clear   Remove all row conditionals"),
        Line::from("  :bgcolor 1 clear   Remove row 1 color"),
        Line::from("  :clearview         Clear all saved view settings"),
        Line::from(""),
        // ── Cell Transforms ───────────────────────────────────
        Line::from(Span::styled("CELL TRANSFORMS", super::modal::bold_style())),
        Line::from("  :upper / :lower    Uppercase/lowercase current cell"),
        Line::from("  :title             Title case current cell"),
        Line::from("  :trim              Trim whitespace from cell"),
        Line::from(""),
        // ── Clipboard ─────────────────────────────────────────
        Line::from(Span::styled("CLIPBOARD", super::modal::bold_style())),
        Line::from("  :copy              Copy CSV to system clipboard"),
        Line::from("  :paste             Paste from system clipboard"),
        Line::from(""),
        // ── Export ────────────────────────────────────────────
        Line::from(Span::styled("EXPORT", super::modal::bold_style())),
        Line::from("  :export json       Export to JSON (array of objects)"),
        Line::from("  :export tsv        Export to TSV"),
        Line::from("  :export md         Export to Markdown table"),
        Line::from("  :export xlsx       Export to XLSX spreadsheet"),
        Line::from("  :export parquet    Export to Parquet"),
        Line::from("  :export json path  Export to specific file"),
        Line::from("  Visual: select cells, then :export <format>"),
        Line::from(""),
        // ── Magnifier Mode ────────────────────────────────────
        Line::from(Span::styled("MAGNIFIER MODE", super::modal::bold_style())),
        Line::from("  Space+m            Open magnifier (full vim editor)"),
        Line::from("  ZZ / :wq           Save and close"),
        Line::from("  :q!                Close without saving"),
        Line::from("  Alt+h/j/k/l        Navigate to adjacent cells"),
        Line::from(""),
        // ── Viewport & Files ──────────────────────────────────
        Line::from(Span::styled("VIEWPORT & FILES", super::modal::bold_style())),
        Line::from("  zt / zz / zb       Row at top/center/bottom"),
        Line::from("  [ / ]              Previous/next file"),
        Line::from("  Space+f            Open file menu"),
        Line::from(""),
        // ── File Menu ─────────────────────────────────────────
        Line::from(Span::styled("FILE MENU", super::modal::bold_style())),
        Line::from("  j/k / arrows       Navigate up/down"),
        Line::from("  h / Left           Go to parent directory"),
        Line::from("  l / Right / Enter  Enter directory or open file"),
        Line::from("  gg / G             Jump to top/bottom"),
        Line::from("  /                  Search/filter files"),
        Line::from("  .                  Toggle hidden files"),
        Line::from("  Tab                File details (Spot)"),
        Line::from("  r/d/m/y/n          Rename/delete/move/copy/new"),
        Line::from("  q / Esc            Close file menu / popup"),
        Line::from(""),
        // ── SQL Editor ────────────────────────────────────────
        Line::from(Span::styled("SQL EDITOR", super::modal::bold_style())),
        Line::from("  Space+q / :sql     Open SQL query editor"),
        Line::from("  Enter              Execute query"),
        Line::from("  Shift+Enter        Insert newline"),
        Line::from("  Ctrl+H             Query history"),
        Line::from("  Ctrl+F             Format SQL"),
        Line::from("  Tab                Auto-complete"),
        Line::from("  Esc                Close editor"),
        Line::from(""),
        // ── Help Navigation ───────────────────────────────────
        Line::from(Span::styled("HELP NAVIGATION", super::modal::bold_style())),
        Line::from("  j/k                Scroll down/up"),
        Line::from("  Ctrl+d / Ctrl+u    Page down/up"),
        Line::from("  g / G              Jump to top/bottom"),
        Line::from("  /                  Search help text"),
        Line::from("  n / N              Next/previous match"),
        Line::from("  ? / Esc            Close help"),
        Line::from(""),
    ]
}

/// Render the help overlay with keybinding reference.
///
/// Displays a centered modal window showing all available keybindings
/// for navigation, editing, and other commands. The overlay covers
/// 70% of terminal width and 80% of height. Supports scrolling with
/// j/k keys on small screens.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `scroll_offset` - Vertical scroll offset for content
/// * `search_query` - Optional search query to display
pub fn render_help_overlay(
    frame: &mut Frame,
    scroll_offset: u16,
    search_query: Option<&str>,
    theme: &Theme,
) {
    // Create centered area using standard large modal size
    let area = super::modal::large_modal_rect(frame.area());

    let help_text = build_help_text();

    // Simplified title - navigation hints moved to status bar
    let title = if let Some(query) = search_query {
        format!(" Help: /{} ", query)
    } else {
        " Help ".to_string()
    };

    let block = super::modal::popup_block(theme, &title);
    let inner = block.inner(area);

    // Clear background and render border
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // Split layout for content and status bar
    let (content_area, status_area) = super::modal::split_with_status_bar(inner);

    // Render help content, highlighting search matches if active
    let display_text = if let Some(query) = search_query.filter(|q| !q.is_empty()) {
        highlight_matches(help_text, query, theme)
    } else {
        help_text
    };
    let paragraph = Paragraph::new(display_text)
        .scroll((scroll_offset, 0))
        .style(super::modal::popup_text_style(theme));
    frame.render_widget(paragraph, content_area);

    // Render status bar with navigation hints
    render_help_status_bar(frame, search_query, status_area, theme);
}

/// Highlight search query matches within help text lines.
/// Each line's spans are flattened to a string, then split around
/// case-insensitive matches with the matched portions highlighted.
fn highlight_matches<'a>(lines: Vec<Line<'a>>, query: &str, theme: &Theme) -> Vec<Line<'a>> {
    let query_lower = query.to_lowercase();
    let highlight_style = super::modal::search_match_style(theme).add_modifier(Modifier::BOLD);

    lines
        .into_iter()
        .map(|line| {
            // Collect all text and styles from existing spans
            let full_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            let text_lower = full_text.to_lowercase();

            // If no match in this line, return as-is
            if !text_lower.contains(&query_lower) {
                return line;
            }

            // Build new spans with highlights
            let mut spans: Vec<Span<'a>> = Vec::new();
            let mut pos = 0;
            let bytes = full_text.as_bytes();
            let query_bytes = query_lower.as_bytes();

            while pos < full_text.len() {
                // Find next match (case-insensitive)
                let remaining_lower = &text_lower[pos..];
                if let Some(match_offset) = remaining_lower.find(&query_lower) {
                    let match_start = pos + match_offset;
                    let match_end = match_start + query_bytes.len();

                    // Text before the match
                    if match_start > pos {
                        spans.push(Span::raw(
                            String::from_utf8_lossy(&bytes[pos..match_start]).into_owned(),
                        ));
                    }
                    // The matched text (preserve original case)
                    spans.push(Span::styled(
                        String::from_utf8_lossy(&bytes[match_start..match_end]).into_owned(),
                        highlight_style,
                    ));
                    pos = match_end;
                } else {
                    // No more matches — emit the rest
                    spans.push(Span::raw(
                        String::from_utf8_lossy(&bytes[pos..]).into_owned(),
                    ));
                    break;
                }
            }

            Line::from(spans)
        })
        .collect()
}

/// Render the help status bar with navigation hints
fn render_help_status_bar(
    frame: &mut Frame,
    search_query: Option<&str>,
    area: ratatui::layout::Rect,
    theme: &Theme,
) {
    let status_text = if let Some(pattern) = search_query {
        // TODO: Add match tracking later (e.g., "match 5/12")
        format!("/{}  | n/N: next/prev | Esc: close", pattern)
    } else {
        "j/k: scroll | /: search | Esc: close".to_string()
    };

    frame.render_widget(
        Paragraph::new(status_text).style(super::modal::popup_text_style(theme)),
        area,
    );
}

// centered_rect() moved to src/ui/modal.rs
// Use super::modal::centered_rect() or super::modal::large_modal_rect() instead
