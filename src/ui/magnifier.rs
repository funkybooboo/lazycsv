//! Magnifier mode UI rendering
//!
//! Provides a centered popup overlay with a full vim-like text editor
//! for complex multi-line cell editing.

use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Create a centered rectangle with the given percentage of width and height
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render the magnifier mode overlay
pub fn render_magnifier(frame: &mut Frame, app: &App, area: Rect) {
    let magnifier = match &app.magnifier_state {
        Some(m) => m,
        None => return,
    };

    // Create centered popup (70% width, 60% height)
    let popup_area = centered_rect(70, 60, area);
    frame.render_widget(Clear, popup_area);

    // Build title with cell position and helpful hints
    let (row, col) = magnifier.cell_position();
    let cell_name = format!(
        "{}{}",
        crate::ui::utils::column_to_excel_letter(col.get()),
        row.get()
    );

    let title = if magnifier.is_dirty() {
        format!(" {} [modified] (:wq save, :q! discard) ", cell_name)
    } else {
        format!(" {} (:wq save, :q! discard, Esc cancel) ", cell_name)
    };

    let main_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default());

    let inner = main_block.inner(popup_area);
    frame.render_widget(main_block, popup_area);

    // Split inner area: content + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    render_content(frame, magnifier, chunks[0]);

    // Build status bar
    let (cursor_line, cursor_col) = magnifier.cursor();
    let line_count = magnifier.lines().len();
    let line_percent = if line_count > 0 {
        ((cursor_line + 1) * 100) / line_count
    } else {
        100
    };

    let left_text = if magnifier.mode() == crate::magnifier::MagnifierMode::Command {
        format!(":{}", magnifier.command_buffer())
    } else {
        match magnifier.mode() {
            crate::magnifier::MagnifierMode::Insert => "-- INSERT --".to_string(),
            crate::magnifier::MagnifierMode::Visual => "-- VISUAL --".to_string(),
            crate::magnifier::MagnifierMode::VisualLine => "-- VISUAL LINE --".to_string(),
            crate::magnifier::MagnifierMode::Normal => {
                magnifier.pending_display().unwrap_or("").to_string()
            }
            _ => String::new(),
        }
    };

    let right_text = format!("{},{} {}%", cursor_line + 1, cursor_col + 1, line_percent);

    let middle_text = if let Some(pattern) = magnifier.search_pattern() {
        let matches = magnifier.search_matches();
        let current = magnifier.current_match_index();
        if matches.is_empty() {
            format!("/{} [0/0]", pattern)
        } else if let Some(idx) = current {
            format!("/{} [{}/{}]", pattern, idx + 1, matches.len())
        } else {
            format!("/{} [?/{}]", pattern, matches.len())
        }
    } else {
        String::new()
    };

    let available_width = chunks[1].width as usize;
    let padding_total = available_width
        .saturating_sub(left_text.len())
        .saturating_sub(middle_text.len())
        .saturating_sub(right_text.len());
    let padding_left = padding_total / 2;
    let padding_right = padding_total - padding_left;

    let status_text = format!(
        "{}{}{}{}{}",
        left_text,
        " ".repeat(padding_left),
        middle_text,
        " ".repeat(padding_right),
        right_text
    );

    let status_bar = Paragraph::new(status_text).style(Style::default());

    frame.render_widget(status_bar, chunks[1]);
}

/// Render content area with line numbers and text
fn render_content(frame: &mut Frame, magnifier: &crate::magnifier::MagnifierState, area: Rect) {
    let line_count = magnifier.lines().len();
    let line_num_width = if line_count == 0 {
        2
    } else {
        (line_count.to_string().len() as u16).clamp(2, 4)
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(line_num_width + 2), Constraint::Min(0)])
        .split(area);

    render_line_numbers(frame, magnifier, chunks[0], line_num_width);
    render_text_content(frame, magnifier, chunks[1]);
}

/// Render line numbers
fn render_line_numbers(
    frame: &mut Frame,
    magnifier: &crate::magnifier::MagnifierState,
    area: Rect,
    width: u16,
) {
    let line_count = magnifier.lines().len();
    let visible_height = area.height as usize;
    let (cursor_line, _) = magnifier.cursor();

    let scroll_offset = if cursor_line < visible_height / 2 {
        0
    } else if cursor_line >= line_count.saturating_sub(visible_height / 2) {
        line_count.saturating_sub(visible_height)
    } else {
        cursor_line.saturating_sub(visible_height / 2)
    };

    let end_line = (scroll_offset + visible_height).min(line_count);

    let mut lines: Vec<Line> = Vec::new();
    for i in scroll_offset..end_line {
        let line_num_text = format!("{:>width$}  ", i + 1, width = width as usize);
        let style = if i == cursor_line {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };
        lines.push(Line::from(Span::styled(line_num_text, style)));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

/// Render text content with cursor, selection, and search highlighting
fn render_text_content(
    frame: &mut Frame,
    magnifier: &crate::magnifier::MagnifierState,
    area: Rect,
) {
    let line_count = magnifier.lines().len();
    let visible_height = area.height as usize;
    let visible_width = area.width as usize;
    let (cursor_line, cursor_col) = magnifier.cursor();

    let scroll_offset = if cursor_line < visible_height / 2 {
        0
    } else if cursor_line >= line_count.saturating_sub(visible_height / 2) {
        line_count.saturating_sub(visible_height)
    } else {
        cursor_line.saturating_sub(visible_height / 2)
    };

    let h_scroll = if cursor_col < visible_width / 2 {
        0
    } else if cursor_col >= visible_width {
        cursor_col.saturating_sub(visible_width / 2)
    } else {
        0
    };

    let end_line = (scroll_offset + visible_height).min(line_count);
    let selection = magnifier.get_visual_selection();
    let search_matches = magnifier.search_matches();
    let current_match = magnifier.current_match_index();

    let mut lines: Vec<Line> = Vec::new();

    for i in scroll_offset..end_line {
        let line_text = magnifier.lines().get(i).map(|s| s.as_str()).unwrap_or("");
        let styled_line = render_line_with_highlights(
            line_text,
            i,
            cursor_line,
            cursor_col,
            h_scroll,
            visible_width,
            &selection,
            search_matches,
            current_match,
            magnifier.mode(),
        );
        lines.push(styled_line);
    }

    if line_count == 0 {
        let cursor_span = Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black));
        lines.push(Line::from(vec![cursor_span]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

/// Render a single line with all highlighting (cursor, selection, search)
#[allow(clippy::too_many_arguments)]
fn render_line_with_highlights(
    line_text: &str,
    line_idx: usize,
    cursor_line: usize,
    cursor_col: usize,
    h_scroll: usize,
    visible_width: usize,
    selection: &Option<crate::magnifier::Selection>,
    search_matches: &[(usize, usize)],
    current_match: Option<usize>,
    _mode: crate::magnifier::MagnifierMode,
) -> Line<'static> {
    let chars: Vec<char> = line_text.chars().collect();
    let mut spans: Vec<Span> = Vec::new();

    // Add left scroll indicator if needed
    if h_scroll > 0 {
        spans.push(Span::styled(
            "<",
            Style::default().add_modifier(Modifier::DIM),
        ));
    }

    // Calculate visible range
    let start_col = if h_scroll > 0 { h_scroll } else { 0 };
    let end_col = (start_col + visible_width.saturating_sub(if h_scroll > 0 { 2 } else { 1 }))
        .min(chars.len());

    // Render each character with appropriate styling
    for col in start_col..end_col {
        let ch = chars.get(col).copied().unwrap_or(' ');

        // Determine if this position should be highlighted
        let is_cursor = line_idx == cursor_line && col == cursor_col;
        let is_selected = is_position_selected(line_idx, col, selection);
        let search_highlight = get_search_highlight(line_idx, col, search_matches, current_match);

        let style = if is_cursor {
            // Cursor takes priority - use explicit colors for visibility
            Style::default().bg(Color::White).fg(Color::Black)
        } else if is_selected {
            // Visual selection - use explicit colors
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            // Search match or normal text
            search_highlight.unwrap_or_default()
        };

        let display_char = if ch == '\t' { ' ' } else { ch };
        spans.push(Span::styled(display_char.to_string(), style));
    }

    // Add cursor at end of line if needed
    if line_idx == cursor_line
        && cursor_col >= chars.len()
        && cursor_col >= start_col
        && cursor_col < end_col + 1
    {
        let cursor_style = Style::default().bg(Color::White).fg(Color::Black);
        spans.push(Span::styled(" ", cursor_style));
    }

    // Add right scroll indicator if needed
    if end_col < chars.len() {
        spans.push(Span::styled(
            ">",
            Style::default().add_modifier(Modifier::DIM),
        ));
    }

    Line::from(spans)
}

/// Check if a position is within the visual selection
fn is_position_selected(
    line: usize,
    col: usize,
    selection: &Option<crate::magnifier::Selection>,
) -> bool {
    use crate::magnifier::Selection;

    match selection {
        Some(Selection::CharWise { start, end }) => {
            let pos = (line, col);
            pos >= *start && pos <= *end
        }
        Some(Selection::LineWise {
            start_line,
            end_line,
        }) => line >= *start_line && line <= *end_line,
        None => false,
    }
}

/// Get search highlight style for a position
fn get_search_highlight(
    line: usize,
    col: usize,
    search_matches: &[(usize, usize)],
    current_match: Option<usize>,
) -> Option<Style> {
    for (idx, &(match_line, match_col)) in search_matches.iter().enumerate() {
        if match_line == line && match_col == col {
            return Some(if Some(idx) == current_match {
                // Current match - bold and underlined
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED)
            } else {
                // Other matches - just underlined
                Style::default().add_modifier(Modifier::UNDERLINED)
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::position::{ColIndex, RowIndex};
    use crate::magnifier::MagnifierState;

    #[test]
    fn test_centered_rect_80_percent() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(80, 80, area);

        // Should be centered with 80% width and height
        assert_eq!(centered.width, 80);
        assert_eq!(centered.height, 80);
        assert_eq!(centered.x, 10); // (100 - 80) / 2
        assert_eq!(centered.y, 10);
    }

    #[test]
    fn test_centered_rect_50_percent() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 50);
        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 25);
    }

    #[test]
    fn test_is_position_selected_charwise() {
        use crate::magnifier::Selection;

        let selection = Some(Selection::CharWise {
            start: (1, 2),
            end: (1, 5),
        });

        assert!(is_position_selected(1, 2, &selection));
        assert!(is_position_selected(1, 3, &selection));
        assert!(is_position_selected(1, 5, &selection));
        assert!(!is_position_selected(1, 1, &selection));
        assert!(!is_position_selected(1, 6, &selection));
        assert!(!is_position_selected(0, 3, &selection));
    }

    #[test]
    fn test_is_position_selected_linewise() {
        use crate::magnifier::Selection;

        let selection = Some(Selection::LineWise {
            start_line: 2,
            end_line: 4,
        });

        assert!(is_position_selected(2, 0, &selection));
        assert!(is_position_selected(3, 10, &selection));
        assert!(is_position_selected(4, 5, &selection));
        assert!(!is_position_selected(1, 0, &selection));
        assert!(!is_position_selected(5, 0, &selection));
    }

    #[test]
    fn test_get_search_highlight() {
        let matches = vec![(0, 5), (1, 3), (2, 8)];

        // Current match should be bold and underlined
        let style = get_search_highlight(1, 3, &matches, Some(1));
        assert!(style.is_some());

        // Other matches should be just underlined
        let style = get_search_highlight(0, 5, &matches, Some(1));
        assert!(style.is_some());

        // Non-matches should return None
        let style = get_search_highlight(0, 0, &matches, Some(1));
        assert!(style.is_none());
    }

    // Integration test: verify magnifier state can be created
    #[test]
    fn test_magnifier_state_creation() {
        let content = "Line 1\nLine 2\nLine 3";
        let position = (RowIndex::new(5), ColIndex::new(2));
        let magnifier = MagnifierState::new(content.to_string(), position);

        assert_eq!(magnifier.lines().len(), 3);
        assert_eq!(magnifier.lines()[0], "Line 1");
        assert_eq!(magnifier.lines()[1], "Line 2");
        assert_eq!(magnifier.lines()[2], "Line 3");
        assert_eq!(magnifier.cursor(), (0, 0));
        assert_eq!(magnifier.cell_position(), position);
    }

    #[test]
    fn test_magnifier_state_empty_content() {
        let content = "";
        let position = (RowIndex::new(1), ColIndex::new(0));
        let magnifier = MagnifierState::new(content.to_string(), position);

        assert_eq!(magnifier.lines().len(), 1);
        assert_eq!(magnifier.lines()[0], "");
    }

    #[test]
    fn test_magnifier_state_single_line() {
        let content = "Single line";
        let position = (RowIndex::new(3), ColIndex::new(1));
        let magnifier = MagnifierState::new(content.to_string(), position);

        assert_eq!(magnifier.lines().len(), 1);
        assert_eq!(magnifier.lines()[0], "Single line");
    }
}
