//! File manager modal for browsing, searching, and managing CSV files.
//!
//! Provides a yazi-like file manager interface with:
//! - File search/filtering
//! - Navigation (j/k/gg/G)
//! - File operations (rename, delete, copy, create)
//! - Visual feedback for current file and dirty files

use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
    Frame,
};

// Modal size constants moved to src/ui/modal.rs
// File manager now uses standard 80% × 80% size (MODAL_LARGE_WIDTH/HEIGHT)

/// Column layout percentages for 3-column view
const PARENT_COL_PERCENT: u16 = 30;
const CURRENT_COL_PERCENT: u16 = 40;
const PREVIEW_COL_PERCENT: u16 = 30;

/// Preview limits
const PREVIEW_MAX_DIR_ENTRIES: usize = 15;
const PREVIEW_MAX_CSV_LINES: usize = 10;

/// Render the file manager modal
pub fn render(frame: &mut Frame, app: &App) {
    let area = super::modal::large_modal_rect(frame.area());

    // Clear background
    frame.render_widget(Clear, area);

    // Just render the file list - no extra sections
    render_file_list(frame, app, area);
}

/// Render the file list with 3-column layout
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

    let block = super::modal::standard_block(&title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area into content + status bar
    let (content, status) = super::modal::split_with_status_bar(inner);

    // Split content into 3 columns: parent | current | preview
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(PARENT_COL_PERCENT),
            Constraint::Percentage(CURRENT_COL_PERCENT),
            Constraint::Percentage(PREVIEW_COL_PERCENT),
        ])
        .split(content);

    // Scan current directory for entries
    let entries = match scan_directory(&app.view_state.current_directory) {
        Ok(e) => e,
        Err(err) => {
            let error_msg = format!("Error reading directory: {}", err);
            let error_para = Paragraph::new(error_msg).style(Style::default());
            frame.render_widget(error_para, columns[1]);
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

    // Render left column - parent directory
    render_parent_column(frame, app, columns[0]);

    // Render middle column - current directory
    render_current_column(frame, app, &filtered_entries, columns[1]);

    // Render right column - preview
    render_preview_column(frame, app, &filtered_entries, columns[2]);

    // Render status bar
    render_file_manager_status_bar(frame, app, status);
}

/// Render the left column showing parent directory contents
fn render_parent_column(frame: &mut Frame, app: &App, area: Rect) {
    use crate::input::file_list_mode::{scan_directory, BrowserEntry};

    // Get parent directory
    let parent_dir = match app.view_state.current_directory.parent() {
        Some(p) => p,
        None => {
            // At root - show nothing
            return;
        }
    };

    // Scan parent directory
    let entries = match scan_directory(parent_dir) {
        Ok(e) => e,
        Err(_) => return, // Silently fail for parent
    };

    // Get current directory name for highlighting
    let current_dir_name = app
        .view_state
        .current_directory
        .file_name()
        .and_then(|n| n.to_str());

    // Build list items (limit to first PREVIEW_MAX_DIR_ENTRIES)
    let items: Vec<ListItem> = entries
        .iter()
        .take(PREVIEW_MAX_DIR_ENTRIES)
        .map(|entry| {
            let name = entry.filename().unwrap_or("unknown");
            let is_current = Some(name) == current_dir_name;

            let display_name = match entry {
                BrowserEntry::Directory(_) => {
                    if name == ".." {
                        "../".to_string()
                    } else {
                        format!("{}/", name)
                    }
                }
                BrowserEntry::CsvFile(_) => name.to_string(),
            };

            let style = if is_current {
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };

            ListItem::new(Line::from(Span::styled(display_name, style)))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area);
}

/// Render the middle column showing current directory contents
fn render_current_column(
    frame: &mut Frame,
    app: &App,
    filtered_entries: &[&crate::input::file_list_mode::BrowserEntry],
    area: Rect,
) {
    use crate::input::file_list_mode::BrowserEntry;

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
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    // Use "/" suffix for directories, "../" for parent
                    let display_name = if name == ".." {
                        "../".to_string()
                    } else {
                        format!("{}/", name)
                    };

                    spans.push(Span::styled(display_name, style));
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

                    let is_dirty = app.session.is_dirty(path);

                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    spans.push(Span::styled(filename.to_string(), style));
                    if is_dirty {
                        let dirty_style = Style::default().fg(app.config.theme.dirty_indicator_fg);
                        spans.push(Span::styled("*", dirty_style));
                    }
                }
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    // Show empty message if no items
    if items.is_empty() {
        let filter = &app.input_state.file_filter_buffer;
        let msg = if filter.is_empty() {
            "No items in directory"
        } else {
            "No matching items"
        };
        let empty = Paragraph::new(msg).style(Style::default());
        frame.render_widget(empty, area);
    } else {
        let list = List::new(items);
        frame.render_widget(list, area);
    }
}

/// Render the right column showing preview of selected item
fn render_preview_column(
    frame: &mut Frame,
    app: &App,
    filtered_entries: &[&crate::input::file_list_mode::BrowserEntry],
    area: Rect,
) {
    use crate::input::file_list_mode::{scan_directory, BrowserEntry};

    if filtered_entries.is_empty() {
        return;
    }

    let selected_idx = app
        .view_state
        .file_list_selected
        .min(filtered_entries.len() - 1);
    let selected_entry = filtered_entries[selected_idx];

    match selected_entry {
        BrowserEntry::Directory(path) => {
            // Preview directory contents
            let entries = match scan_directory(path) {
                Ok(e) => e,
                Err(_) => return, // Silently fail for preview
            };

            let items: Vec<ListItem> = entries
                .iter()
                .take(PREVIEW_MAX_DIR_ENTRIES)
                .map(|entry| {
                    let name = entry.filename().unwrap_or("unknown");
                    let display_name = match entry {
                        BrowserEntry::Directory(_) => {
                            if name == ".." {
                                "../".to_string()
                            } else {
                                format!("{}/", name)
                            }
                        }
                        BrowserEntry::CsvFile(_) => name.to_string(),
                    };

                    let style = Style::default().add_modifier(Modifier::DIM);
                    ListItem::new(Line::from(Span::styled(display_name, style)))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, area);
        }
        BrowserEntry::CsvFile(path) => {
            // Preview CSV file contents
            let preview_lines = read_csv_preview(path, PREVIEW_MAX_CSV_LINES);

            let items: Vec<ListItem> = preview_lines
                .iter()
                .enumerate()
                .map(|(idx, line)| {
                    let formatted = format!("{}: {}", idx + 1, line);
                    let style = Style::default().add_modifier(Modifier::DIM);
                    ListItem::new(Line::from(Span::styled(formatted, style)))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, area);
        }
    }
}

/// Read first N lines from a CSV file for preview
fn read_csv_preview(path: &std::path::Path, max_lines: usize) -> Vec<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let reader = BufReader::new(file);
    reader
        .lines()
        .take(max_lines)
        .filter_map(|line| line.ok())
        .collect()
}

/// Render the file manager status bar with navigation hints
fn render_file_manager_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_text = if app.input_state.file_list_search_active {
        // Show filter pattern when searching
        format!("/{}", app.input_state.file_filter_buffer)
    } else {
        // Show navigation hints
        "h/l: navigate | /: filter | r/d/m/y/n: operations | ?: help".to_string()
    };

    let status_para = Paragraph::new(status_text);
    frame.render_widget(status_para, area);
}
