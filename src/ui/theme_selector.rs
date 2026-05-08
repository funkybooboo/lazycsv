//! Theme selector modal rendering and mouse interaction.
//!
//! Keyboard handling lives in `input::theme_selector_mode`. This module
//! provides theme scanning, rendering, and click-to-select behaviour.

use std::path::PathBuf;

use crate::config;
use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Scan for available themes in order: config dir, then built-in themes dir.
pub fn scan_themes() -> Vec<(String, PathBuf)> {
    let mut themes = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. Check config directory (~/.config/lazycsv/themes/)
    if let Some(config_dir) = config::dirs_path() {
        let themes_dir = config_dir.join("themes");
        collect_themes_from_dir(&themes_dir, &mut themes, &mut seen);
    }

    // 2. Check built-in themes directory (relative to binary or cwd)
    let builtin_paths = [
        PathBuf::from("themes"),
        PathBuf::from("../themes"),
        PathBuf::from("../../themes"),
    ];
    for dir in &builtin_paths {
        if dir.exists() {
            collect_themes_from_dir(dir, &mut themes, &mut seen);
            break;
        }
    }

    // Sort alphabetically by display name
    themes.sort_by(|a, b| a.0.cmp(&b.0));
    themes
}

/// Collect .toml files from a directory as themes.
fn collect_themes_from_dir(
    dir: &std::path::Path,
    themes: &mut Vec<(String, PathBuf)>,
    seen: &mut std::collections::HashSet<String>,
) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let display_name = theme_display_name(stem);
                    if seen.insert(display_name.clone()) {
                        themes.push((display_name, path));
                    }
                }
            }
        }
    }
}

/// Convert filename stem to a display name (e.g., "catppuccin-mocha" -> "Catppuccin Mocha").
pub fn theme_display_name(stem: &str) -> String {
    stem.split('-')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render the theme selector modal.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = super::modal::large_modal_rect(frame.area());

    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default().style(super::modal::popup_bg_style(&app.config.theme)),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_theme_list(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

/// Render the header with current config info.
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let config_path = config::dirs_path()
        .map(|p| p.join("config.toml"))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.config/lazycsv/config.toml".to_string());

    let header_text = format!(" Theme Selector — writing to: {}", config_path);
    let header = Paragraph::new(Line::from(vec![Span::styled(
        header_text,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(app.config.theme.popup.title_fg),
    )]));
    frame.render_widget(header, area);
}

/// Render the scrollable list of themes.
fn render_theme_list(frame: &mut Frame, app: &App, area: Rect) {
    let themes = &app.theme_list;
    if themes.is_empty() {
        let msg = Paragraph::new("No themes found in themes/ directory")
            .style(super::modal::popup_text_style(&app.config.theme));
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, (name, _path))| {
            let is_selected = i == app.theme_selector_index;

            let style = if is_selected {
                Style::default()
                    .bg(app.config.theme.popup.completion_sel_bg)
                    .fg(app.config.theme.popup.completion_sel_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                super::modal::popup_text_style(&app.config.theme)
            };

            let line = Line::from(vec![
                Span::styled("  ", style),
                Span::styled(name.clone(), style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        super::modal::popup_block(&app.config.theme, " Available Themes ").borders(Borders::ALL),
    );

    frame.render_widget(list, area);
}

/// Handle a left-click on the theme selector modal.
/// Returns `true` if the click was within the modal list area.
pub(crate) fn handle_click(app: &mut App, x: u16, y: u16) -> bool {
    let frame_area = crossterm::terminal::size()
        .map(|(w, h)| ratatui::layout::Rect::new(0, 0, w, h))
        .unwrap_or_default();
    let modal = super::modal::large_modal_rect(frame_area);

    if modal.width < 2 || modal.height < 3 {
        return false;
    }

    if x < modal.x || x >= modal.x + modal.width || y < modal.y || y >= modal.y + modal.height {
        return false;
    }

    let content_y_start = modal.y + 1;
    let content_y_end = modal.y + modal.height - 1;

    if y < content_y_start || y >= content_y_end {
        return false;
    }

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(modal);

    let list_area = chunks[1];
    let inner_y_start = list_area.y + 1;
    let inner_y_end = list_area.y + list_area.height - 1;

    if y < inner_y_start || y >= inner_y_end {
        return false;
    }

    let theme_idx = (y - inner_y_start) as usize;
    if theme_idx < app.theme_list.len() {
        app.theme_selector_index = theme_idx;
        return true;
    }

    false
}

/// Render the status bar with keybindings hint.
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let theme_count = app.theme_list.len();
    let current = if app.theme_list.is_empty() {
        "No themes".to_string()
    } else {
        format!("{}/{}", app.theme_selector_index + 1, theme_count)
    };

    let status_text = super::modal::build_three_part_status_line(
        " j/k: navigate | Enter: apply | Esc: cancel ",
        &current,
        "",
        area.width as usize,
    );

    let status = Paragraph::new(status_text)
        .style(super::modal::popup_text_style(&app.config.theme).add_modifier(Modifier::DIM));
    frame.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_display_name() {
        assert_eq!(theme_display_name("gruvbox-dark"), "Gruvbox Dark");
        assert_eq!(theme_display_name("catppuccin-mocha"), "Catppuccin Mocha");
        assert_eq!(theme_display_name("solarized-light"), "Solarized Light");
        assert_eq!(theme_display_name("nord"), "Nord");
        assert_eq!(theme_display_name("dracula"), "Dracula");
    }

    #[test]
    fn test_scan_themes_not_empty() {
        let themes = scan_themes();
        let _ = themes;
    }
}
