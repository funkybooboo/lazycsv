pub mod file_manager;
pub mod file_switcher;
pub mod help;
pub mod magnifier;
pub mod sql_editor;
// mod sql_editor_helpers; // DEPRECATED: Old SQL editor code, removed in v0.11.0
pub mod status_bar;
pub mod table;
pub mod utils;
pub mod view_state;

/// Maximum number of columns to display simultaneously
/// This prevents horizontal overflow on standard terminals
pub const MAX_VISIBLE_COLS: usize = 10;

use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Render a centered loading message (used before App exists, e.g. initial file load)
pub fn render_loading(frame: &mut Frame, message: &str) {
    use ratatui::layout::Alignment;
    use ratatui::widgets::Paragraph;

    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Percentage(50),
        ])
        .split(area);
    let paragraph = Paragraph::new(message.to_string()).alignment(Alignment::Center);
    frame.render_widget(paragraph, vertical[1]);
}

/// Main UI rendering function
pub fn render(frame: &mut Frame, app: &mut App) {
    // Split terminal into main area + status bar
    // Minimal layout: no heavy borders, just horizontal rules as separators
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Table area (includes title bar + rule)
            Constraint::Length(1), // Status bar (single line, vim-like)
        ])
        .split(frame.area());

    // Render table with row/column numbers
    table::render_table(frame, app, chunks[0]);

    // Render status bar
    status_bar::render(frame, app, chunks[1]);

    // Render help overlay if active
    if app.view_state.help_overlay_visible {
        let search_query = app.view_state.help_search_query.as_deref();
        help::render_help_overlay(frame, app.view_state.help_scroll_offset, search_query);
    }

    // Render SQL editor overlay if active
    if app.mode == crate::app::Mode::SqlEditor {
        if let Some(ref vim_editor) = app.sql_vim_editor {
            sql_editor::render_sql_editor_vim(
                frame,
                vim_editor,
                app.sql_error.as_deref(),
                app.sql_completion.as_ref(),
            );
        }
    }

    // Render magnifier overlay if active
    if app.magnifier_state.is_some() {
        magnifier::render_magnifier(frame, app, frame.area());
    }

    // Render file manager modal if active
    if app.mode == crate::app::Mode::FileList {
        file_manager::render(frame, app);
    }

    // Render file operation prompt if active
    if app.mode == crate::app::Mode::FileOperationPrompt {
        render_file_operation_prompt(frame, app);
    }

    // Render formula completion popup if active (in insert mode)
    if app.formula_completion.is_some() && app.mode == crate::app::Mode::Insert {
        render_formula_completion(frame, app);
    }
}

/// Render file operation prompt overlay
fn render_file_operation_prompt(frame: &mut ratatui::Frame, app: &crate::app::App) {
    use ratatui::{
        style::Style,
        text::Line,
        widgets::{Block, Borders, Clear, Paragraph},
    };

    // Small centered prompt (30% width, 3 lines height)
    let area = help::centered_rect(40, 20, frame.area());
    frame.render_widget(Clear, area);

    let (title, prompt) = match &app.file_operation {
        Some(crate::app::FileOperation::Rename(path)) => {
            let old_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            (format!(" Rename: {} ", old_name), "New name:")
        }
        Some(crate::app::FileOperation::Delete(path)) => {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            (format!(" Delete: {} ", name), "Type 'yes' to confirm:")
        }
        Some(crate::app::FileOperation::Move(_)) => (" Move ".to_string(), "Destination:"),
        Some(crate::app::FileOperation::Copy(path)) => {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            (format!(" Copy: {} ", name), "New name:")
        }
        Some(crate::app::FileOperation::Create) => (" Create New File ".to_string(), "Filename:"),
        None => (" File Operation ".to_string(), ""),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(prompt),
        Line::from(format!("> {}", app.file_operation_buffer)),
        Line::from(""),
        Line::from("Enter: confirm | Esc: cancel"),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}

/// Render formula completion popup anchored near the selected cell
fn render_formula_completion(frame: &mut ratatui::Frame, app: &crate::app::App) {
    use crate::app::{CompletionItem, COMPLETION_MAX_VISIBLE};
    use ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph},
    };

    let completion = match &app.formula_completion {
        Some(c) => c,
        None => return,
    };

    let filtered: Vec<&CompletionItem> = completion.filtered_items();
    if filtered.is_empty() {
        return;
    }

    // Position popup near the selected cell
    let frame_area = frame.area();

    // Place popup at a fixed position in the center-left of the screen.
    // This avoids complex scroll offset tracking while remaining visible.
    let popup_y = (frame_area.height / 3).max(2);
    let popup_x = 6_u16; // after row number column

    let max_name_len = filtered.iter().map(|item| item.text.len()).max().unwrap_or(10);
    let title_width = if completion.filter.is_empty() {
        0
    } else {
        completion.filter.len() + 3
    };
    let popup_width = ((max_name_len + 4).max(title_width) + 4).min(40) as u16;
    let visible_count = filtered.len().min(COMPLETION_MAX_VISIBLE);
    let popup_height = visible_count as u16 + 2;

    // Clamp to frame boundaries
    let popup_x = popup_x.min(frame_area.right().saturating_sub(popup_width));
    let popup_y = popup_y.min(frame_area.bottom().saturating_sub(popup_height));

    let popup_rect = Rect::new(popup_x, popup_y, popup_width, popup_height);
    let scroll_off = completion.scroll_offset;
    let inner_width = popup_width.saturating_sub(2) as usize;
    let filter_lower = completion.filter.to_ascii_lowercase();

    let lines: Vec<Line<'static>> = filtered
        .iter()
        .enumerate()
        .skip(scroll_off)
        .take(visible_count)
        .map(|(idx, item)| {
            let is_selected = idx == completion.selected;
            let bg = if is_selected {
                Color::Blue
            } else {
                Color::DarkGray
            };
            let base_style = Style::default().fg(Color::White).bg(bg);
            let tag_style = Style::default().fg(item.kind.color()).bg(bg);
            let highlight_style = base_style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

            let tag = format!("{} ", item.kind.tag());
            let name = &item.text;
            let name_budget = inner_width.saturating_sub(tag.len());

            let mut spans: Vec<Span<'static>> = vec![Span::styled(tag, tag_style)];

            if !filter_lower.is_empty() {
                if let Some(match_start) = name.to_ascii_lowercase().find(&filter_lower) {
                    let match_end = match_start + filter_lower.len();
                    let before = &name[..match_start];
                    let matched = &name[match_start..match_end];
                    let after = &name[match_end..];
                    spans.push(Span::styled(before.to_string(), base_style));
                    spans.push(Span::styled(matched.to_string(), highlight_style));
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
        .title(" Formulas ")
        .style(Style::default().bg(Color::DarkGray));
    if !completion.filter.is_empty() {
        popup_block = popup_block.title(format!(" /{} ", completion.filter));
    }
    let popup_inner = popup_block.inner(popup_rect);
    frame.render_widget(popup_block, popup_rect);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, popup_inner);
}

// Re-export public utilities and types
pub use utils::column_to_excel_letter;
pub use view_state::{ViewState, ViewportMode};
