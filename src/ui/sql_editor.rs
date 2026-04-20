//! SQL editor overlay rendering.
//!
//! Displays a centered modal popup for typing and executing SQL queries
//! against loaded CSV tables with full vim editing capabilities.

use crate::app::{
    DiagnosticSeverity, SqlCompletion, SqlDiagnostic, SqlHistoryPopup, COMPLETION_MAX_VISIBLE,
};
use crate::vim_editor::{Selection, VimEditor, VimMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

// Modal size constants moved to src/ui/modal.rs
// SQL editor now uses standard 80% × 80% size (MODAL_LARGE_WIDTH/HEIGHT)

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
    history_popup: Option<(&SqlHistoryPopup, &[String])>,
) {
    let area = super::modal::large_modal_rect(frame.area());

    // Clear background and render border
    frame.render_widget(Clear, area);

    // Title without mode (mode shown in status bar instead)
    let title = " SQL Query ";
    let block = super::modal::standard_block(title);
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

    // Render history popup if active (overlays the editor)
    if let Some((popup, history)) = history_popup {
        render_history_popup(frame, popup, history, area);
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
    let max_name_len = filtered
        .iter()
        .map(|item| item.text.len())
        .max()
        .unwrap_or(10);
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
            let base_style = if is_selected {
                super::modal::completion_selected_style()
            } else {
                super::modal::completion_unselected_style()
            };
            let bg = if is_selected {
                Color::Blue
            } else {
                super::modal::COLOR_POPUP_BG
            };
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
                spans.push(Span::styled(
                    padded,
                    if is_selected {
                        base_style.add_modifier(Modifier::BOLD)
                    } else {
                        base_style
                    },
                ));
            }

            Line::from(spans)
        })
        .collect();

    frame.render_widget(Clear, popup_rect);
    let mut popup_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(super::modal::COLOR_POPUP_BG));
    if !completion.filter.is_empty() {
        popup_block = popup_block.title(format!(" /{} ", completion.filter));
    }
    let popup_inner = popup_block.inner(popup_rect);
    frame.render_widget(popup_block, popup_rect);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, popup_inner);

    // Scroll indicators: render on the right border of the popup
    let total = filtered.len();
    if total > visible_count {
        let has_above = scroll_offset > 0;
        let has_below = scroll_offset + visible_count < total;
        let border_style = Style::default()
            .fg(Color::White)
            .bg(super::modal::COLOR_POPUP_BG);

        // Place arrows on the right border column (popup_rect edge)
        let border_x = popup_rect.right().saturating_sub(1);
        if has_above {
            let r = Rect::new(border_x, popup_rect.y + 1, 1, 1);
            frame.render_widget(Paragraph::new("▲").style(border_style), r);
        }
        if has_below {
            let r = Rect::new(border_x, popup_rect.bottom().saturating_sub(2), 1, 1);
            frame.render_widget(Paragraph::new("▼").style(border_style), r);
        }
    }
}

const HISTORY_MAX_VISIBLE: usize = 10;

/// Render the SQL history popup centered over the SQL editor area.
fn render_history_popup(
    frame: &mut Frame,
    popup: &SqlHistoryPopup,
    history: &[String],
    editor_area: Rect,
) {
    if history.is_empty() {
        return;
    }

    let visible = history.len().min(HISTORY_MAX_VISIBLE);
    let popup_height = (visible as u16 + 2).min(editor_area.height);
    let popup_width = (editor_area.width * 9 / 10).max(40).min(editor_area.width);
    let popup_x = editor_area.x + (editor_area.width.saturating_sub(popup_width)) / 2;
    let popup_y = editor_area.y + (editor_area.height.saturating_sub(popup_height)) / 2;
    let popup_rect = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_rect);
    let title = if popup.pending_d {
        " SQL History  (dd: confirm delete · Esc cancel) ".to_string()
    } else {
        " SQL History  (↑↓/jk · Enter select · dd delete · Esc close) ".to_string()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(super::modal::COLOR_POPUP_BG));
    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let inner_width = inner.width as usize;

    let lines: Vec<Line<'static>> = history
        .iter()
        .enumerate()
        .skip(popup.scroll_offset)
        .take(visible)
        .map(|(idx, query)| {
            let is_selected = idx == popup.selected;
            let style = if is_selected && popup.pending_d {
                Style::default().fg(Color::White).bg(Color::Red)
            } else if is_selected {
                super::modal::completion_selected_style()
            } else {
                super::modal::completion_unselected_style()
            };
            // Collapse multi-line queries to a single display line
            let one_line: String = query
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            // Truncate to fit
            let display = if one_line.chars().count() > inner_width {
                format!(
                    "{}…",
                    one_line
                        .chars()
                        .take(inner_width.saturating_sub(1))
                        .collect::<String>()
                )
            } else {
                format!("{:<width$}", one_line, width = inner_width)
            };
            Line::from(Span::styled(display, style))
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Build status line with mode/command/error and help tip
fn build_status_line<'a>(
    vim_editor: &VimEditor,
    sql_error: Option<&str>,
    width: usize,
) -> Line<'a> {
    let help_text = "Ctrl+Enter: execute | Ctrl+F: format | Ctrl+H: history | Ctrl+N: complete | Ctrl+/: help | Esc: exit";

    // Left side: Mode or command buffer or error
    let left = if let Some(err) = sql_error {
        format!("Error: {}", err)
    } else if vim_editor.mode() == VimMode::Command {
        format!(":{}", vim_editor.command_buffer())
    } else {
        // Show mode using standard format
        super::modal::format_mode_indicator(vim_editor.mode().display_name(), None)
    };

    // Build status line with mode/error on left, help on right
    let padding = width.saturating_sub(left.len() + help_text.len());

    if sql_error.is_some() {
        // Error in red - use centralized error style
        Line::from(vec![
            Span::styled(left, super::modal::error_style()),
            Span::raw(" ".repeat(padding)),
            Span::raw(help_text),
        ])
    } else {
        // Normal mode/command display
        Line::from(vec![
            Span::raw(left),
            Span::raw(" ".repeat(padding)),
            Span::raw(help_text),
        ])
    }
}

/// Build display lines from vim editor with line numbers, cursor, and visual-selection highlighting.
fn build_vim_editor_lines(vim_editor: &VimEditor) -> Vec<Line<'static>> {
    let line_count = vim_editor.line_count();
    let line_num_width = format!("{}", line_count).len();
    (0..line_count)
        .map(|i| build_source_line(vim_editor, i, line_num_width))
        .collect()
}

/// Build a single rendered source line: line number gutter + per-character spans
/// that apply cursor style, visual-selection style, or plain style as appropriate.
fn build_source_line(
    vim_editor: &VimEditor,
    line_idx: usize,
    line_num_width: usize,
) -> Line<'static> {
    let (cursor_line, cursor_col) = vim_editor.cursor();
    let selection = vim_editor.visual_selection();
    let line_text = vim_editor.line(line_idx).unwrap_or("");
    let chars: Vec<char> = line_text.chars().collect();

    let line_num = format!("{:>width$} ", line_idx + 1, width = line_num_width);
    let line_num_span = Span::styled(
        line_num,
        Style::default().fg(super::modal::COLOR_LINE_NUMBER),
    );
    let mut spans = vec![line_num_span];

    // If cursor is past line end, we still need to render a trailing cursor cell.
    let render_len = if line_idx == cursor_line {
        chars.len().max(cursor_col + 1)
    } else {
        chars.len()
    };

    for col in 0..render_len {
        let ch: String = if col < chars.len() {
            chars[col].to_string()
        } else {
            " ".to_string()
        };
        let is_cursor = line_idx == cursor_line && col == cursor_col;
        let is_selected = is_in_selection(selection, line_idx, col);
        if is_cursor {
            spans.push(Span::styled(ch, super::modal::cursor_style()));
        } else if is_selected {
            spans.push(Span::styled(ch, super::modal::visual_selection_style()));
        } else {
            spans.push(Span::raw(ch));
        }
    }

    Line::from(spans)
}

fn is_in_selection(sel: Option<Selection>, line: usize, col: usize) -> bool {
    match sel {
        None => false,
        Some(Selection::CharWise { start, end }) => {
            if line < start.0 || line > end.0 {
                false
            } else if start.0 == end.0 {
                col >= start.1 && col <= end.1
            } else if line == start.0 {
                col >= start.1
            } else if line == end.0 {
                col <= end.1
            } else {
                true
            }
        }
        Some(Selection::LineWise {
            start_line,
            end_line,
        }) => line >= start_line && line <= end_line,
    }
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

    let line_count = vim_editor.line_count();
    let line_num_width = format!("{}", line_count).len();

    let mut display_lines = Vec::new();
    for (line_idx, line_text) in vim_editor.lines().iter().enumerate() {
        display_lines.push(build_source_line(vim_editor, line_idx, line_num_width));

        let line_diags: Vec<&SqlDiagnostic> =
            diagnostics.iter().filter(|d| d.line == line_idx).collect();
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
        spans.push(Span::styled("~".repeat(len), Style::default().fg(color)));
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
