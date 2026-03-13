//! File manager modal for browsing, searching, and managing CSV files.
//!
//! Provides a yazi-like file manager interface with:
//! - File search/filtering
//! - Navigation (j/k/gg/G)
//! - File operations (rename, delete, copy, create)
//! - Visual feedback for current file and dirty files

use crate::App;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Width percentage for file manager modal
const FILE_MANAGER_WIDTH_PERCENT: u16 = 80;

/// Height percentage for file manager modal
const FILE_MANAGER_HEIGHT_PERCENT: u16 = 80;

/// Render the file manager modal
pub fn render(frame: &mut Frame, app: &App) {
    let area = super::help::centered_rect(
        FILE_MANAGER_WIDTH_PERCENT,
        FILE_MANAGER_HEIGHT_PERCENT,
        frame.area(),
    );

    // Clear background
    frame.render_widget(Clear, area);

    // Just render the file list - no extra sections
    render_file_list(frame, app, area);
}

/// Render the file list
fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    use crate::input::file_list_mode::{scan_directory, BrowserEntry};

    // Build title with current directory and search indicator
    let filter = &app.input_state.file_filter_buffer;
    let dir_name = app
        .view_state
        .current_directory
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");

    let title = if app.input_state.file_list_search_active {
        format!(" File Menu [{}]: /{} ", dir_name, filter)
    } else {
        format!(" File Menu [{}] ", dir_name)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Scan current directory for entries
    let entries = match scan_directory(&app.view_state.current_directory) {
        Ok(e) => e,
        Err(err) => {
            let error_msg = format!("Error reading directory: {}", err);
            let error_para = Paragraph::new(error_msg).style(Style::default());
            frame.render_widget(error_para, inner);
            return;
        }
    };

    // Apply filter
    let filter_lower = filter.to_lowercase();
    let filtered_entries: Vec<&BrowserEntry> = entries
        .iter()
        .filter(|entry| {
            if filter.is_empty() {
                true
            } else if let Some(name) = entry.filename() {
                name.to_lowercase().contains(&filter_lower)
            } else {
                false
            }
        })
        .collect();

    let selected_idx = app.view_state.file_list_selected;
    let active_file = app.session.files().get(app.session.active_file_index());

    // Build list items
    let items: Vec<ListItem> = filtered_entries
        .iter()
        .enumerate()
        .map(|(display_idx, entry)| {
            let is_selected = display_idx == selected_idx;
            let is_active = active_file.map(|f| f == entry.path()).unwrap_or(false);

            let mut spans = Vec::new();

            // Cursor indicator
            if is_selected {
                spans.push(Span::raw("> "));
            } else {
                spans.push(Span::raw("  "));
            }

            // Icon/type indicator
            match entry {
                BrowserEntry::Directory(path) => {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                    let icon = if name == ".." { "↑ " } else { "📁 " };
                    spans.push(Span::raw(icon));

                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(name, style));
                }
                BrowserEntry::CsvFile(path) => {
                    // Active file indicator
                    if is_active {
                        spans.push(Span::raw("● "));
                    } else {
                        spans.push(Span::raw("  "));
                    }

                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    let dirty = if app.session.is_dirty(path) { "*" } else { "" };

                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    spans.push(Span::styled(format!("{}{}", filename, dirty), style));
                }
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    // Show empty message if no items
    if items.is_empty() {
        let msg = if filter.is_empty() {
            "No items in directory"
        } else {
            "No matching items"
        };
        let empty = Paragraph::new(msg).style(Style::default());
        frame.render_widget(empty, inner);
    } else {
        let list = List::new(items);
        frame.render_widget(list, inner);
    }
}
