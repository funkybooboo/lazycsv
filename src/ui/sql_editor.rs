//! SQL editor overlay rendering.
//!
//! Displays a centered modal popup for typing and executing SQL queries
//! against loaded CSV tables with full vim editing capabilities.

use crate::app::{DiagnosticSeverity, SqlCompletion, SqlDiagnostic, COMPLETION_MAX_VISIBLE};
use crate::vim_editor::{VimEditor, VimMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Width percentage for SQL editor overlay
const SQL_EDITOR_WIDTH_PERCENT: u16 = 80;

/// Height percentage for SQL editor overlay
const SQL_EDITOR_HEIGHT_PERCENT: u16 = 80;

/// Render the SQL editor overlay with vim editing
///
/// Displays a centered modal window where the user can edit SQL queries
/// using full vim modal editing (Normal, Insert, Visual modes).
pub fn render_sql_editor_vim(
    frame: &mut Frame,
    vim_editor: &VimEditor,
    sql_error: Option<&str>,
    completion: Option<&SqlCompletion>,
    diagnostics: &[SqlDiagnostic],
) {
    let area = super::help::centered_rect(
        SQL_EDITOR_WIDTH_PERCENT,
        SQL_EDITOR_HEIGHT_PERCENT,
        frame.area(),
    );

    // Clear background and render border with mode indicator
    frame.render_widget(Clear, area);

    let mode_str = vim_editor.mode().display_name();

    let title = format!(" SQL Query - {} ", mode_str);
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split area for query text, diagnostic tooltip, and status line
    let (query_area, status_area) = split_editor_area(inner);

    // Render query text with line numbers, cursor, and diagnostic squiggles
    let lines = build_vim_editor_lines_with_diagnostics(vim_editor, diagnostics);
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, query_area);

    // Build status line - show first diagnostic message if any, otherwise normal status
    let first_diag_at_cursor = find_diagnostic_at_cursor(vim_editor, diagnostics);
    let status_line = if let Some(diag) = first_diag_at_cursor {
        build_diagnostic_status_line(diag, status_area.width as usize)
    } else {
        build_status_line(vim_editor, sql_error, status_area.width as usize)
    };
    let status_paragraph = Paragraph::new(vec![status_line]);
    frame.render_widget(status_paragraph, status_area);

    // Render completion popup if active
    if let Some(comp) = completion {
        render_completion_popup(frame, vim_editor, comp, query_area);
    }
}

/// Render the completion popup anchored below the cursor
fn render_completion_popup(
    frame: &mut Frame,
    vim_editor: &VimEditor,
    completion: &SqlCompletion,
    query_area: Rect,
) {
    use crate::app::CompletionItem;

    let filtered: Vec<&CompletionItem> = completion.filtered_items();
    if filtered.is_empty() {
        return;
    }

    let (cursor_line, cursor_col) = vim_editor.cursor();
    let line_count = vim_editor.line_count();
    let line_num_width = format!("{}", line_count).len() + 1;

    let popup_x = query_area.x + line_num_width as u16 + cursor_col as u16;
    let popup_y = query_area.y + cursor_line as u16 + 1;

    // 4 = tag "[K] " prefix width
    let max_name_len = filtered.iter().map(|item| item.text.len()).max().unwrap_or(10);
    let title_width = if completion.filter.is_empty() {
        0
    } else {
        completion.filter.len() + 3
    };
    let popup_width = ((max_name_len + 4).max(title_width) + 4).min(50) as u16;
    let visible_count = filtered.len().min(COMPLETION_MAX_VISIBLE);
    let popup_height = visible_count as u16 + 2;

    let frame_area = frame.area();
    let popup_x = popup_x.min(frame_area.right().saturating_sub(popup_width));
    let popup_y = popup_y.min(frame_area.bottom().saturating_sub(popup_height));

    let popup_rect = Rect::new(popup_x, popup_y, popup_width, popup_height);
    let scroll_offset = completion.scroll_offset;
    let inner_width = popup_width.saturating_sub(2) as usize;
    let filter_lower = completion.filter.to_ascii_lowercase();

    let lines: Vec<Line<'static>> = filtered
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_count)
        .map(|(idx, item)| {
            let is_selected = idx == completion.selected;
            let bg = if is_selected { Color::Blue } else { Color::DarkGray };
            let base_style = Style::default().fg(Color::White).bg(bg);
            let tag_style = Style::default().fg(item.kind.color()).bg(bg);
            let highlight_style = base_style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

            let tag = format!("{} ", item.kind.tag());
            let name = &item.text;
            let name_budget = inner_width.saturating_sub(tag.len());

            let mut spans: Vec<Span<'static>> = vec![Span::styled(tag, tag_style)];

            // Highlight matching portion of the name
            if !filter_lower.is_empty() {
                if let Some(match_start) = name.to_ascii_lowercase().find(&filter_lower) {
                    let match_end = match_start + filter_lower.len();
                    let before = &name[..match_start];
                    let matched = &name[match_start..match_end];
                    let after = &name[match_end..];
                    spans.push(Span::styled(before.to_string(), base_style));
                    spans.push(Span::styled(matched.to_string(), highlight_style));
                    // Pad the rest
                    let remaining = name_budget.saturating_sub(name.len());
                    let padded_after = format!("{}{}", after, " ".repeat(remaining));
                    spans.push(Span::styled(padded_after, base_style));
                } else {
                    let padded = format!("{:<width$}", name, width = name_budget);
                    spans.push(Span::styled(padded, base_style));
                }
            } else {
                let padded = format!("{:<width$}", name, width = name_budget);
                spans.push(Span::styled(padded, if is_selected {
                    base_style.add_modifier(Modifier::BOLD)
                } else {
                    base_style
                }));
            }

            Line::from(spans)
        })
        .collect();

    frame.render_widget(Clear, popup_rect);
    let mut popup_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    if !completion.filter.is_empty() {
        popup_block = popup_block.title(format!(" /{} ", completion.filter));
    }
    let popup_inner = popup_block.inner(popup_rect);
    frame.render_widget(popup_block, popup_rect);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, popup_inner);
}

/// Build status line with mode/command/error and help tip
fn build_status_line<'a>(
    vim_editor: &VimEditor,
    sql_error: Option<&str>,
    width: usize,
) -> Line<'a> {
    let help_text = "? for help";

    if let Some(err) = sql_error {
        // Error message on left, help on right
        let error_text = format!("Error: {}", err);
        let padding = width.saturating_sub(error_text.len() + help_text.len() + 2);
        Line::from(vec![
            Span::styled(
                error_text,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat(padding)),
            Span::raw(help_text),
        ])
    } else if vim_editor.mode() == VimMode::Command {
        // Command buffer on left, help on right
        let cmd_text = format!(":{}", vim_editor.command_buffer());
        let padding = width.saturating_sub(cmd_text.len() + help_text.len() + 1);
        Line::from(vec![
            Span::styled(cmd_text, Style::default()),
            Span::raw(" ".repeat(padding)),
            Span::raw(help_text),
        ])
    } else {
        // Just help on right
        let padding = width.saturating_sub(help_text.len());
        Line::from(vec![Span::raw(" ".repeat(padding)), Span::raw(help_text)])
    }
}

/// Build display lines from vim editor with line numbers and cursor highlighting
fn build_vim_editor_lines(vim_editor: &VimEditor) -> Vec<Line<'static>> {
    let (cursor_line, cursor_col) = vim_editor.cursor();
    let line_count = vim_editor.line_count();
    let line_num_width = format!("{}", line_count).len();

    let mut display_lines = Vec::new();

    for (line_idx, line_text) in vim_editor.lines().iter().enumerate() {
        let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width);
        let line_num_span = Span::styled(line_num, Style::default().fg(Color::DarkGray));

        if line_idx == cursor_line {
            // This line contains the cursor - highlight cursor position
            let chars: Vec<char> = line_text.chars().collect();
            let mut spans = vec![line_num_span];

            // Text before cursor
            if cursor_col > 0 {
                let before: String = chars[..cursor_col.min(chars.len())].iter().collect();
                spans.push(Span::raw(before));
            }

            // Cursor character (inverted)
            if cursor_col < chars.len() {
                let cursor_char = chars[cursor_col].to_string();
                spans.push(Span::styled(
                    cursor_char,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // Cursor at end of line (show as space)
                spans.push(Span::styled(
                    " ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            // Text after cursor
            if cursor_col + 1 < chars.len() {
                let after: String = chars[cursor_col + 1..].iter().collect();
                spans.push(Span::raw(after));
            }

            display_lines.push(Line::from(spans));
        } else {
            // Regular line without cursor
            display_lines.push(Line::from(vec![
                line_num_span,
                Span::raw(line_text.clone()),
            ]));
        }
    }

    display_lines
}

/// Build display lines with diagnostic squiggly underlines.
///
/// For each source line that has diagnostics, an extra line with red `~` characters
/// is inserted below it to indicate the problematic span.
fn build_vim_editor_lines_with_diagnostics(
    vim_editor: &VimEditor,
    diagnostics: &[SqlDiagnostic],
) -> Vec<Line<'static>> {
    if diagnostics.is_empty() {
        return build_vim_editor_lines(vim_editor);
    }

    let (cursor_line, cursor_col) = vim_editor.cursor();
    let line_count = vim_editor.line_count();
    let line_num_width = format!("{}", line_count).len();

    let mut display_lines = Vec::new();

    for (line_idx, line_text) in vim_editor.lines().iter().enumerate() {
        let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width);
        let line_num_span = Span::styled(line_num.clone(), Style::default().fg(Color::DarkGray));

        if line_idx == cursor_line {
            let chars: Vec<char> = line_text.chars().collect();
            let mut spans = vec![line_num_span];

            if cursor_col > 0 {
                let before: String = chars[..cursor_col.min(chars.len())].iter().collect();
                spans.push(Span::raw(before));
            }

            if cursor_col < chars.len() {
                let cursor_char = chars[cursor_col].to_string();
                spans.push(Span::styled(
                    cursor_char,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    " ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            if cursor_col + 1 < chars.len() {
                let after: String = chars[cursor_col + 1..].iter().collect();
                spans.push(Span::raw(after));
            }

            display_lines.push(Line::from(spans));
        } else {
            display_lines.push(Line::from(vec![
                line_num_span,
                Span::raw(line_text.clone()),
            ]));
        }

        // Add squiggly underline line for diagnostics on this line
        let line_diags: Vec<&SqlDiagnostic> = diagnostics
            .iter()
            .filter(|d| d.line == line_idx)
            .collect();

        if !line_diags.is_empty() {
            let gutter = " ".repeat(line_num_width + 1);
            let squiggles = build_squiggle_line(&line_diags, line_text.len());
            let mut spans = vec![Span::raw(gutter)];
            spans.extend(squiggles);
            display_lines.push(Line::from(spans));
        }
    }

    display_lines
}

/// Build squiggly underline spans for a set of diagnostics on a single line.
///
/// Produces spans of spaces (for gaps) and `~` characters (for diagnostics)
/// colored by severity: red for errors, yellow for warnings.
fn build_squiggle_line(diagnostics: &[&SqlDiagnostic], _line_len: usize) -> Vec<Span<'static>> {
    // Sort diagnostics by column start
    let mut sorted: Vec<&&SqlDiagnostic> = diagnostics.iter().collect();
    sorted.sort_by_key(|d| d.col_start);

    let mut spans = Vec::new();
    let mut pos = 0;

    for diag in sorted {
        if diag.col_start > pos {
            spans.push(Span::raw(" ".repeat(diag.col_start - pos)));
        }
        let len = diag.col_end.saturating_sub(diag.col_start).max(1);
        let color = match diag.severity {
            DiagnosticSeverity::Error => Color::Red,
            DiagnosticSeverity::Warning => Color::Yellow,
        };
        spans.push(Span::styled(
            "~".repeat(len),
            Style::default().fg(color),
        ));
        pos = diag.col_end;
    }

    spans
}

/// Find the first diagnostic whose span covers the current cursor position.
fn find_diagnostic_at_cursor<'a>(
    vim_editor: &VimEditor,
    diagnostics: &'a [SqlDiagnostic],
) -> Option<&'a SqlDiagnostic> {
    let (cursor_line, cursor_col) = vim_editor.cursor();
    diagnostics
        .iter()
        .find(|d| d.line == cursor_line && cursor_col >= d.col_start && cursor_col < d.col_end)
}

/// Build a status line showing a diagnostic message.
fn build_diagnostic_status_line(diag: &SqlDiagnostic, width: usize) -> Line<'static> {
    let (prefix, color) = match diag.severity {
        DiagnosticSeverity::Error => ("Error: ", Color::Red),
        DiagnosticSeverity::Warning => ("Warning: ", Color::Yellow),
    };
    let msg = format!("{}{}", prefix, diag.message);
    let help_text = "? for help";
    let padding = width.saturating_sub(msg.len() + help_text.len() + 2);
    Line::from(vec![
        Span::styled(msg, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(padding)),
        Span::raw(help_text.to_string()),
    ])
}

/// Split the editor area into query text area and status area.
///
/// Returns (query_area, status_area).
fn split_editor_area(inner: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    (chunks[0], chunks[1])
}
