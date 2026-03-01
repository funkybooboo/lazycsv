//! Magnifier mode UI rendering
//!
//! Renders a centered popup overlay for the magnifier mode, which provides
//! a full vim-like text editor for complex multi-line cell editing.

use crate::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
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
///
/// Displays a centered popup with:
/// - Title bar showing cell position and mode
/// - Line numbers (right-aligned, dim)
/// - Text content with cursor
/// - Bottom help bar with commands
pub fn render_magnifier(frame: &mut Frame, app: &App, area: Rect) {
    let magnifier = match &app.magnifier_state {
        Some(m) => m,
        None => return, // No magnifier active
    };

    // Create centered popup (80% width, 80% height)
    let popup_area = centered_rect(80, 80, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Split popup into title bar, content area, and help bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),    // Content area
            Constraint::Length(1), // Help bar
        ])
        .split(popup_area);

    // Title bar: "Editing A5" on left, mode and cursor position on right
    let (row, col) = magnifier.cell_position();
    let cell_pos = format!(
        "{}{}",
        crate::ui::utils::column_to_excel_letter(col.get()),
        row.get()
    );
    let mode_str = match magnifier.mode() {
        crate::magnifier::MagnifierMode::Normal => "NORMAL",
        crate::magnifier::MagnifierMode::Insert => "INSERT",
    };
    let (cursor_line, cursor_col) = magnifier.cursor();
    let cursor_pos = format!("{}:{}", cursor_line + 1, cursor_col + 1);

    let title_left = format!(" Editing {}", cell_pos);
    let title_right = format!("[{}]  {}  ", mode_str, cursor_pos);
    let title_padding = (popup_area.width as usize)
        .saturating_sub(title_left.len())
        .saturating_sub(title_right.len());
    let title_text = format!("{}{}{}", title_left, " ".repeat(title_padding), title_right);

    let title_bar = Paragraph::new(title_text)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default()),
        );
    frame.render_widget(title_bar, chunks[0]);

    // Content area: line numbers + text
    render_content(frame, magnifier, chunks[1]);

    // Help bar
    let help_text = " :w save | :wq quit | Ctrl+h/j/k/l navigate ";
    let help_bar = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::DIM))
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default()),
        );
    frame.render_widget(help_bar, chunks[2]);
}

/// Render the content area with line numbers and text
fn render_content(frame: &mut Frame, magnifier: &crate::magnifier::MagnifierState, area: Rect) {
    // Calculate line number column width (max 4 digits)
    let line_count = magnifier.lines().len();
    let line_num_width = if line_count == 0 {
        2
    } else {
        (line_count.to_string().len() as u16).clamp(2, 4)
    };

    // Split into line numbers and text content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(line_num_width + 2), // +2 for padding and separator
            Constraint::Min(0),
        ])
        .split(area);

    // Render line numbers
    render_line_numbers(frame, magnifier, chunks[0], line_num_width);

    // Render text content with cursor
    render_text_content(frame, magnifier, chunks[1]);
}

/// Render line numbers (right-aligned, dim)
fn render_line_numbers(
    frame: &mut Frame,
    magnifier: &crate::magnifier::MagnifierState,
    area: Rect,
    width: u16,
) {
    let line_count = magnifier.lines().len();
    let visible_height = area.height as usize;

    // Calculate scroll offset to keep cursor visible
    let (cursor_line, _) = magnifier.cursor();
    let scroll_offset = if cursor_line < visible_height / 2 {
        0
    } else if cursor_line >= line_count.saturating_sub(visible_height / 2) {
        line_count.saturating_sub(visible_height)
    } else {
        cursor_line.saturating_sub(visible_height / 2)
    };

    let end_line = (scroll_offset + visible_height).min(line_count);

    let mut line_nums = String::new();
    for i in scroll_offset..end_line {
        let line_num = format!("{:>width$} │", i + 1, width = width as usize);
        line_nums.push_str(&line_num);
        if i < end_line - 1 {
            line_nums.push('\n');
        }
    }

    let line_num_widget = Paragraph::new(line_nums)
        .style(Style::default().add_modifier(Modifier::DIM))
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::BOTTOM)
                .border_style(Style::default()),
        );

    frame.render_widget(line_num_widget, area);
}

/// Render text content with cursor
fn render_text_content(
    frame: &mut Frame,
    magnifier: &crate::magnifier::MagnifierState,
    area: Rect,
) {
    let line_count = magnifier.lines().len();
    let visible_height = area.height as usize;

    // Calculate scroll offset to keep cursor visible
    let (cursor_line, cursor_col) = magnifier.cursor();
    let scroll_offset = if cursor_line < visible_height / 2 {
        0
    } else if cursor_line >= line_count.saturating_sub(visible_height / 2) {
        line_count.saturating_sub(visible_height)
    } else {
        cursor_line.saturating_sub(visible_height / 2)
    };

    let end_line = (scroll_offset + visible_height).min(line_count);

    // Build text with cursor
    let mut content = String::new();
    for (idx, i) in (scroll_offset..end_line).enumerate() {
        let line = magnifier
            .lines()
            .get(i)
            .map(|s: &String| s.as_str())
            .unwrap_or("");

        // Add cursor if this is the cursor line
        if i == cursor_line {
            let line_with_cursor = insert_cursor(line, cursor_col, magnifier.mode());
            content.push_str(&line_with_cursor);
        } else {
            content.push_str(line);
        }

        if idx < (end_line - scroll_offset) - 1 {
            content.push('\n');
        }
    }

    // If buffer is empty, show cursor on empty line
    if line_count == 0 {
        content = insert_cursor("", 0, magnifier.mode());
    }

    let text_widget = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default()),
    );

    frame.render_widget(text_widget, area);
}

/// Insert cursor character at the given position
fn insert_cursor(line: &str, col: usize, mode: crate::magnifier::MagnifierMode) -> String {
    let cursor_char = match mode {
        crate::magnifier::MagnifierMode::Normal => '█', // Block cursor
        crate::magnifier::MagnifierMode::Insert => '│', // Pipe cursor
    };

    let chars: Vec<char> = line.chars().collect();
    let mut result = String::new();

    for (i, &ch) in chars.iter().enumerate() {
        if i == col {
            result.push(cursor_char);
        }
        result.push(ch);
    }

    // If cursor is at end of line
    if col >= chars.len() {
        result.push(cursor_char);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::position::{ColIndex, RowIndex};
    use crate::magnifier::{MagnifierMode, MagnifierState};

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
    fn test_insert_cursor_normal_mode_start() {
        let line = "Hello, world!";
        let result = insert_cursor(line, 0, MagnifierMode::Normal);
        assert_eq!(result, "█Hello, world!");
    }

    #[test]
    fn test_insert_cursor_normal_mode_middle() {
        let line = "Hello, world!";
        let result = insert_cursor(line, 7, MagnifierMode::Normal);
        assert_eq!(result, "Hello, █world!");
    }

    #[test]
    fn test_insert_cursor_normal_mode_end() {
        let line = "Hello";
        let result = insert_cursor(line, 5, MagnifierMode::Normal);
        assert_eq!(result, "Hello█");
    }

    #[test]
    fn test_insert_cursor_insert_mode() {
        let line = "Hello";
        let result = insert_cursor(line, 2, MagnifierMode::Insert);
        assert_eq!(result, "He│llo");
    }

    #[test]
    fn test_insert_cursor_empty_line() {
        let line = "";
        let result = insert_cursor(line, 0, MagnifierMode::Normal);
        assert_eq!(result, "█");
    }

    #[test]
    fn test_insert_cursor_beyond_end() {
        let line = "Hi";
        let result = insert_cursor(line, 10, MagnifierMode::Normal);
        // Cursor should be at end
        assert_eq!(result, "Hi█");
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
