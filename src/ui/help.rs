//! Help overlay rendering with keybinding reference.
//!
//! Displays a modal help overlay showing all available keybindings and
//! navigation commands when triggered by '?'. Supports scrolling on small
//! screens.

use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
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
    vec![
        Line::from(Span::styled(
            "LazyCSV v0.6.0 - Keyboard Shortcuts",
            super::modal::bold_style(),
        )),
        Line::from(""),
        Line::from(Span::styled("NAVIGATION", super::modal::bold_style())),
        Line::from("  hjkl / arrows      Move cursor (with count: 5j, 10h)"),
        Line::from("  w / b / e          Next/prev/last non-empty cell"),
        Line::from("  gg                 First data row (row 1 if header mode ON)"),
        Line::from("  gh                 Go to header row (row 0)"),
        Line::from("  gd                 Go to first data row (row 1)"),
        Line::from("  G                  Last row"),
        Line::from("  5g                 Jump to row 5"),
        Line::from("  0 / $              First/last column"),
        Line::from("  Ctrl+d / Ctrl+u    Page down/up"),
        Line::from(""),
        Line::from(Span::styled(
            "COLUMN NAVIGATION",
            super::modal::bold_style(),
        )),
        Line::from("  :cA                Jump to column A"),
        Line::from("  :cB                Jump to column B"),
        Line::from("  :cAA               Jump to column AA"),
        Line::from("  :c1                Jump to column 1 (A)"),
        Line::from("  :c27               Jump to column 27 (AA)"),
        Line::from(""),
        Line::from(Span::styled("COMMAND MODE", super::modal::bold_style())),
        Line::from("  :                  Enter command mode"),
        Line::from("  :q / :wq / :q!     Quit (save/force)"),
        Line::from("  :w / :W            Save file / all files"),
        Line::from("  :ht                Toggle header mode"),
        Line::from("  :delim ;           Set delimiter to semicolon"),
        Line::from("  :new A,B,C         Create new CSV with headers"),
        Line::from("  :f <name>          Rename current file"),
        Line::from("  :sort <col,...>    Sort ascending by column(s)"),
        Line::from("  :sort! <col,...>   Sort descending by column(s)"),
        Line::from("  Esc                Cancel command"),
        Line::from(""),
        Line::from(Span::styled("RANGE OPERATIONS", super::modal::bold_style())),
        Line::from("  :5,10d             Delete rows 5-10"),
        Line::from("  :5,10y             Yank rows 5-10"),
        Line::from("  :%d / :%y          Delete/yank all data rows"),
        Line::from("  :.d / :.y          Delete/yank current row"),
        Line::from("  :$d / :$y          Delete/yank last row"),
        Line::from(""),
        Line::from(Span::styled("INSERT MODE", super::modal::bold_style())),
        Line::from("  i / a              Edit cell (cursor at end)"),
        Line::from("  I                  Edit cell (cursor at start)"),
        Line::from("  A                  Edit cell (cursor at end)"),
        Line::from("  s                  Replace cell (clear + edit)"),
        Line::from("  F2                 Edit cell"),
        Line::from("  Delete             Clear cell (stay in Normal)"),
        Line::from(""),
        Line::from(Span::styled(
            "INSERT MODE EDITING",
            super::modal::bold_style(),
        )),
        Line::from("  Enter              Commit, move down"),
        Line::from("  Shift+Enter        Commit, move up"),
        Line::from("  Tab                Commit, move right"),
        Line::from("  Shift+Tab          Commit, move left"),
        Line::from("  Esc                Cancel edit"),
        Line::from("  Backspace          Delete char before cursor"),
        Line::from("  Ctrl+w             Delete word backward"),
        Line::from("  Ctrl+u             Delete to start"),
        Line::from(""),
        Line::from(Span::styled("MAGNIFIER MODE", super::modal::bold_style())),
        Line::from("  Space+m            Open magnifier (full vim editor)"),
        Line::from("  ZZ / :wq           Save and close magnifier"),
        Line::from("  :q!                Close without saving"),
        Line::from("  Alt+h/j/k/l        Navigate to adjacent cells"),
        Line::from("  i/a/o/O            Enter insert mode"),
        Line::from("  hjkl / w/b/e       Vim motions"),
        Line::from("  dd / yy / p        Delete/yank/paste lines"),
        Line::from("  x / s              Delete/substitute char"),
        Line::from(""),
        Line::from(Span::styled("ROW OPERATIONS", super::modal::bold_style())),
        Line::from("  o                  Insert row below, enter Insert"),
        Line::from("  O                  Insert row above, enter Insert"),
        Line::from("  dd                 Delete row"),
        Line::from("  yy                 Yank (copy) row"),
        Line::from("  p                  Paste row below"),
        Line::from(""),
        Line::from(Span::styled("VIEWPORT & FILES", super::modal::bold_style())),
        Line::from("  zt / zz / zb       Row at top/center/bottom"),
        Line::from("  Space+f            Open file menu"),
        Line::from(""),
        Line::from(Span::styled("FILE MENU", super::modal::bold_style())),
        Line::from("  j/k                Navigate down/up"),
        Line::from("  gg / G             Jump to top/bottom"),
        Line::from("  h                  Go to parent directory"),
        Line::from("  l / Enter          Enter directory or open CSV"),
        Line::from("  /                  Search/filter"),
        Line::from("  r                  Rename file/directory"),
        Line::from("  d                  Delete file/directory"),
        Line::from("  m                  Move file/directory"),
        Line::from("  y                  Copy file/directory"),
        Line::from("  n                  Create new file"),
        Line::from("  q / Esc            Close file menu"),
        Line::from(""),
        Line::from(Span::styled("SQL EDITOR", super::modal::bold_style())),
        Line::from("  Space+q            Open SQL query editor"),
        Line::from("  Enter              Execute query (results in output.csv)"),
        Line::from("  Shift+Enter        Insert newline in query"),
        Line::from("  Esc                Close editor without executing"),
        Line::from("  Ctrl+u             Clear query buffer"),
        Line::from(""),
        Line::from(Span::styled("GLOBAL", super::modal::bold_style())),
        Line::from("  :q / :q!           Quit (force quit)"),
        Line::from("  ?                  Toggle this help"),
        Line::from(""),
        Line::from(Span::styled("HELP NAVIGATION", super::modal::bold_style())),
        Line::from("  j/k                Scroll down/up one line"),
        Line::from("  Ctrl+d / Ctrl+u    Scroll down/up one page"),
        Line::from("  Ctrl+f / Ctrl+b    Scroll down/up one page"),
        Line::from("  g / G              Jump to top/bottom"),
        Line::from("  /                  Search help text"),
        Line::from("  n / N              Next/previous search match"),
        Line::from("  Esc / ?            Close help"),
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
pub fn render_help_overlay(frame: &mut Frame, scroll_offset: u16, search_query: Option<&str>) {
    // Create centered area using standard large modal size
    let area = super::modal::large_modal_rect(frame.area());

    let help_text = build_help_text();

    // Simplified title - navigation hints moved to status bar
    let title = if let Some(query) = search_query {
        format!(" Help: /{} ", query)
    } else {
        " Help ".to_string()
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);

    // Clear background and render border
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // Split layout for content and status bar
    let (content_area, status_area) = super::modal::split_with_status_bar(inner);

    // Render help content
    let paragraph = Paragraph::new(help_text).scroll((scroll_offset, 0));
    frame.render_widget(paragraph, content_area);

    // Render status bar with navigation hints
    render_help_status_bar(frame, search_query, status_area);
}

/// Render the help status bar with navigation hints
fn render_help_status_bar(
    frame: &mut Frame,
    search_query: Option<&str>,
    area: ratatui::layout::Rect,
) {
    let status_text = if let Some(pattern) = search_query {
        // TODO: Add match tracking later (e.g., "match 5/12")
        format!("/{}  | n/N: next/prev | Esc: close", pattern)
    } else {
        "j/k: scroll | /: search | Esc: close".to_string()
    };

    frame.render_widget(Paragraph::new(status_text), area);
}

// centered_rect() moved to src/ui/modal.rs
// Use super::modal::centered_rect() or super::modal::large_modal_rect() instead
