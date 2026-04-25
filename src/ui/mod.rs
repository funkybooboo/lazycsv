pub mod conditional;
pub mod file_manager;
pub mod file_switcher;
pub mod help;
pub mod magnifier;
pub mod modal;
pub mod sql_editor;
// mod sql_editor_helpers; // DEPRECATED: Old SQL editor code, removed in v0.11.0
pub mod stats_overlay;
pub mod status_bar;
pub mod table;
pub mod utils;
pub mod view_state;

use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
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
    // Paint a theme-colored base layer over the entire frame so any
    // un-painted gaps (right of last column, below the data, etc.)
    // pick up the configured ui background instead of the terminal default.
    let base = ratatui::widgets::Block::default().style(
        Style::default()
            .fg(app.config.theme.ui.fg)
            .bg(app.config.theme.ui.bg),
    );
    frame.render_widget(base, frame.area());

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
        help::render_help_overlay(
            frame,
            app.view_state.help_scroll_offset,
            search_query,
            &app.config.theme,
        );
    }

    // Render SQL editor overlay if active
    if app.mode == crate::app::Mode::SqlEditor {
        if let Some(ref vim_editor) = app.sql_vim_editor {
            let history_popup = app
                .sql_history_popup
                .as_ref()
                .map(|p| (p, app.sql_history.as_slice()));
            sql_editor::render_sql_editor_vim(
                frame,
                vim_editor,
                app.sql_error.as_deref(),
                app.sql_completion.as_ref(),
                &app.sql_diagnostics,
                history_popup,
                &app.config.theme,
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
        if app.input_state.file_list_shell_active {
            render_shell_prompt(frame, app);
        }
    }

    // Render file operation prompt if active
    if app.mode == crate::app::Mode::FileOperationPrompt {
        render_file_operation_prompt(frame, app);
    }

    // Render formula completion popup if active (in insert mode)
    if app.formula_completion.is_some() && app.mode == crate::app::Mode::Insert {
        render_formula_completion(frame, app);
    }

    // Render statistics overlay if active
    if app.view_state.stats_overlay_visible {
        if let Some(ref data) = app.view_state.stats_overlay_data {
            stats_overlay::render(frame, data, &app.config.theme);
        }
    }

    // Render context menu if active
    if let Some(ref menu) = app.context_menu {
        render_context_menu(frame, menu, &app.config.theme);
    }

    // Render the shell-command stderr popup last so it sits above everything.
    if let Some(ref popup) = app.shell_error_popup {
        render_shell_error_popup(frame, app, popup);
    }
}

/// Render the scrollable popup that surfaces multi-line shell-command stderr.
fn render_shell_error_popup(
    frame: &mut ratatui::Frame,
    app: &crate::app::App,
    popup: &crate::app::ShellErrorPopup,
) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        text::Line,
        widgets::{Clear, Paragraph},
    };

    let area = modal::large_modal_rect(frame.area());
    frame.render_widget(Clear, area);

    let block = modal::popup_block(&app.config.theme, &popup.title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into content + status bar (1 line at the bottom).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let lines: Vec<Line> = popup.body.lines().map(Line::from).collect();
    let body = Paragraph::new(lines)
        .scroll((popup.scroll, 0))
        .style(modal::popup_text_style(&app.config.theme));
    frame.render_widget(body, chunks[0]);

    let hint = Paragraph::new("j/k: scroll  |  Esc: close").style(
        modal::popup_text_style(&app.config.theme).add_modifier(ratatui::style::Modifier::DIM),
    );
    frame.render_widget(hint, chunks[1]);
}

/// Render file operation prompt overlay
fn render_file_operation_prompt(frame: &mut ratatui::Frame, app: &crate::app::App) {
    use ratatui::{
        text::Line,
        widgets::{Clear, Paragraph},
    };

    // Small centered prompt using standard small modal size
    let area = modal::small_modal_rect(frame.area());
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

    let block = modal::popup_block(&app.config.theme, &title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(prompt),
        Line::from(format!("> {}", app.file_operation_buffer)),
        Line::from(""),
        Line::from("Enter: confirm | Esc: cancel"),
    ];

    let paragraph = Paragraph::new(text).style(modal::popup_text_style(&app.config.theme));
    frame.render_widget(paragraph, inner);
}

/// Render the file-menu shell-command prompt ("Shell (block):").
fn render_shell_prompt(frame: &mut ratatui::Frame, app: &crate::app::App) {
    use ratatui::{
        layout::{Constraint, Direction, Layout, Rect},
        text::{Line, Span},
        widgets::{Clear, Paragraph},
    };

    // 45% wide × 3 lines (border + 1 input row + border), centered.
    let frame_area = frame.area();
    let popup_width = (frame_area.width as u32 * 45 / 100).max(40) as u16;
    let popup_width = popup_width.min(frame_area.width.saturating_sub(2));
    let popup_height: u16 = 3;
    let x = frame_area.x + frame_area.width.saturating_sub(popup_width) / 2;
    let y = frame_area.y + frame_area.height.saturating_sub(popup_height) / 2;
    let area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, area);
    let block = modal::popup_block(&app.config.theme, "Shell (block):");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let buffer = &app.input_state.shell_buffer;
    let cursor = app.input_state.shell_cursor;
    let style = modal::popup_text_style(&app.config.theme);
    let cursor_style = modal::cursor_style(&app.config.theme);

    // Render text with an inline cursor block at the cursor position.
    let mut spans: Vec<Span> = Vec::new();
    let mut chars = buffer.chars();
    for _ in 0..cursor {
        if let Some(c) = chars.next() {
            spans.push(Span::styled(c.to_string(), style));
        }
    }
    match chars.next() {
        Some(c) => spans.push(Span::styled(c.to_string(), cursor_style)),
        None => spans.push(Span::styled(" ", cursor_style)),
    }
    for c in chars {
        spans.push(Span::styled(c.to_string(), style));
    }

    // Single-row layout inside the bordered block.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1)])
        .split(inner);
    frame.render_widget(Paragraph::new(Line::from(spans)).style(style), rows[0]);
}

/// Render formula completion popup anchored near the selected cell
fn render_formula_completion(frame: &mut ratatui::Frame, app: &crate::app::App) {
    use crate::app::{CompletionItem, COMPLETION_MAX_VISIBLE};
    use ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Clear, Paragraph},
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
            let base_style = if is_selected {
                modal::completion_selected_style(&app.config.theme)
            } else {
                modal::completion_unselected_style(&app.config.theme)
            };
            let bg = if is_selected {
                Color::Blue
            } else {
                app.config.theme.popup.bg
            };
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
    let title = if !completion.filter.is_empty() {
        format!(" /{} ", completion.filter)
    } else {
        " Formulas ".to_string()
    };
    let popup_block = modal::popup_block(&app.config.theme, &title);
    let popup_inner = popup_block.inner(popup_rect);
    frame.render_widget(popup_block, popup_rect);

    let paragraph = Paragraph::new(lines).style(modal::popup_text_style(&app.config.theme));
    frame.render_widget(paragraph, popup_inner);
}

/// Render right-click context menu popup
fn render_context_menu(
    frame: &mut ratatui::Frame,
    menu: &crate::app::ContextMenu,
    theme: &crate::config::Theme,
) {
    use ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Clear,
    };

    let item_count = menu.items.len() as u16;
    let max_label = menu
        .items
        .iter()
        .map(|i| i.label().len())
        .max()
        .unwrap_or(4);
    let popup_width: u16 = (max_label as u16 + 4).max(14); // label + padding + borders
    let popup_height = item_count + 2; // +2 for borders

    let frame_area = frame.area();

    // Position near the click, clamped to screen
    let x = menu.x.min(frame_area.right().saturating_sub(popup_width));
    let y = menu.y.min(frame_area.bottom().saturating_sub(popup_height));

    let popup_rect = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_rect);

    let block = modal::popup_block(theme, "");
    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let inner_width = inner.width as usize;
    let lines: Vec<Line<'static>> = menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if *item == crate::app::ContextMenuItem::Separator {
                let sep = "─".repeat(inner_width);
                return Line::from(Span::styled(
                    sep,
                    Style::default().bg(theme.popup.bg).fg(Color::DarkGray),
                ));
            }
            let is_selected = i == menu.selected;
            let label = format!(" {:<width$}", item.label(), width = inner_width - 1);
            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(theme.popup.bg).fg(Color::White)
            };
            Line::from(Span::styled(label, style))
        })
        .collect();

    let paragraph = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

// Re-export public utilities and types
pub use utils::column_to_excel_letter;
pub use view_state::{ViewState, ViewportMode};
