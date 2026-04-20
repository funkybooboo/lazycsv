//! File manager modal for browsing, searching, and managing CSV files.
//!
//! Provides a yazi-like file manager interface with:
//! - File search/filtering
//! - Navigation (j/k/gg/G)
//! - File operations (rename, delete, copy, create)
//! - Visual feedback for current file and dirty files

use crate::config::Theme;
use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Get the 8 preview column colors from the theme
fn column_colors(theme: &Theme) -> [Color; 8] {
    [
        theme.file_menu_preview_col_1,
        theme.file_menu_preview_col_2,
        theme.file_menu_preview_col_3,
        theme.file_menu_preview_col_4,
        theme.file_menu_preview_col_5,
        theme.file_menu_preview_col_6,
        theme.file_menu_preview_col_7,
        theme.file_menu_preview_col_8,
    ]
}

/// Render the file manager modal
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = super::modal::large_modal_rect(frame.area());

    // Clear background
    frame.render_widget(Clear, area);

    // Just render the file list - no extra sections
    render_file_list(frame, app, area);

    // Render file details popup if visible
    if app.view_state.file_spot_visible {
        render_spot_popup(frame, app, area);
    }
}

/// Render the file list with 3-column layout
fn render_file_list(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::input::file_list_mode::{scan_directory_filtered, BrowserEntry};

    // Split area into: header (1 line) + content + status bar (1 line)
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header / breadcrumb
            Constraint::Min(0),    // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    let header_area = layout[0];
    let content = layout[1];
    let status = layout[2];

    // Render breadcrumb header
    render_breadcrumb_header(frame, app, header_area);

    // Split content into 5 parts: parent | sep | current | sep | preview
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15), // parent
            Constraint::Length(1),      // separator
            Constraint::Percentage(42), // current
            Constraint::Length(1),      // separator
            Constraint::Percentage(42), // preview
        ])
        .split(content);

    let parent_area = columns[0];
    let sep1_area = columns[1];
    let current_area = columns[2];
    let sep2_area = columns[3];
    let preview_area = columns[4];

    // Render vertical separators
    let sep_style = Style::default().fg(app.config.theme.file_menu_separator_fg);
    let sep_block = Block::default()
        .borders(Borders::LEFT)
        .border_style(sep_style);
    frame.render_widget(sep_block.clone(), sep1_area);
    frame.render_widget(sep_block, sep2_area);

    // Scan current directory for entries
    let show_hidden = app.view_state.show_hidden_files;
    let entries = match scan_directory_filtered(&app.view_state.current_directory, show_hidden) {
        Ok(e) => e,
        Err(err) => {
            let error_msg = format!("Error reading directory: {}", err);
            let error_para = Paragraph::new(error_msg).style(Style::default());
            frame.render_widget(error_para, current_area);
            return;
        }
    };

    // Apply filter
    let filter = &app.input_state.file_filter_buffer;
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

    // Stash column areas so mouse clicks can be mapped back to list indices.
    app.view_state.mouse_layout.file_list_parent_area = parent_area;
    app.view_state.mouse_layout.file_list_area = current_area;

    // Render left column - parent directory
    render_parent_column(frame, app, parent_area);

    // Render middle column - current directory
    render_current_column(frame, app, &filtered_entries, current_area);

    // Render right column - preview
    render_preview_column(frame, app, &filtered_entries, preview_area);

    // Render status bar
    render_file_manager_status_bar(frame, app, &filtered_entries, status);
}

/// Render the breadcrumb header showing current path (yazi-style)
fn render_breadcrumb_header(frame: &mut Frame, app: &App, area: Rect) {
    let path = &app.view_state.current_directory;

    // Build path display similar to yazi: ~/projects/lazycsv/test_data
    let display_path = if let Some(home) = std::env::var_os("HOME") {
        let home_path = std::path::Path::new(&home);
        if let Ok(stripped) = path.strip_prefix(home_path) {
            format!("~/{}", stripped.display())
        } else {
            path.display().to_string()
        }
    } else {
        path.display().to_string()
    };

    // Add filter info if active
    let filter = &app.input_state.file_filter_buffer;
    let header_text = if app.input_state.file_list_search_active {
        format!("{} (find: {})", display_path, filter)
    } else {
        display_path
    };

    let header = Paragraph::new(Line::from(vec![Span::styled(
        header_text,
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    frame.render_widget(header, area);
}

/// Render the left column showing parent directory contents
fn render_parent_column(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::input::file_list_mode::{scan_directory_filtered, BrowserEntry};

    // Get parent directory
    let parent_dir = match app.view_state.current_directory.parent() {
        Some(p) => p,
        None => {
            // At root - show nothing
            return;
        }
    };

    // Scan parent directory
    let entries = match scan_directory_filtered(parent_dir, app.view_state.show_hidden_files) {
        Ok(e) => e,
        Err(_) => return, // Silently fail for parent
    };

    // Get current directory name for highlighting
    let current_dir_name = app
        .view_state
        .current_directory
        .file_name()
        .and_then(|n| n.to_str());

    // Build list items - skip ".." entry and entries with no filename
    let items: Vec<ListItem> = entries
        .iter()
        .filter(|entry| {
            match entry.filename() {
                Some(name) => name != "..",
                None => false, // Skip entries with no valid filename
            }
        })
        .map(|entry| {
            let name = entry.filename().unwrap_or_default();
            let is_current = Some(name) == current_dir_name;

            let display_name = name.to_string();

            let style = match entry {
                BrowserEntry::Directory(_) => {
                    if is_current {
                        Style::default()
                            .fg(app.config.theme.file_menu_highlight_fg)
                            .bg(app.config.theme.file_menu_highlight_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(app.config.theme.file_menu_dir_fg)
                    }
                }
                BrowserEntry::CsvFile(_) => {
                    if is_current {
                        Style::default()
                            .fg(app.config.theme.file_menu_highlight_fg)
                            .bg(app.config.theme.file_menu_highlight_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().add_modifier(Modifier::DIM)
                    }
                }
            };

            // Truncate long names to fit column
            let max_width = area.width.saturating_sub(1) as usize;
            let truncated = if display_name.len() > max_width && max_width > 3 {
                format!("{}...", &display_name[..max_width - 3])
            } else {
                display_name
            };

            ListItem::new(Line::from(Span::styled(truncated, style)))
        })
        .collect();

    // Find index of current directory for scroll position (adjusted for skipped "..")
    let current_idx = entries
        .iter()
        .filter(|entry| match entry.filename() {
            Some(name) => name != "..",
            None => false,
        })
        .position(|entry| entry.filename() == current_dir_name);

    let list = List::new(items);
    let mut state = ListState::default().with_selected(current_idx);
    frame.render_stateful_widget(list, area, &mut state);
    app.view_state.mouse_layout.file_list_parent_offset = state.offset();
}

/// Render the middle column showing current directory contents
fn render_current_column(
    frame: &mut Frame,
    app: &mut App,
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

            // Icon/type indicator
            match entry {
                BrowserEntry::Directory(path) => {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("..");

                    let style = if is_selected {
                        Style::default()
                            .fg(app.config.theme.file_menu_highlight_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(app.config.theme.file_menu_dir_fg)
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
                        let bullet_style = if is_selected {
                            Style::default().fg(app.config.theme.file_menu_highlight_fg)
                        } else {
                            Style::default().fg(app.config.theme.file_menu_active_indicator_fg)
                        };
                        spans.push(Span::styled("● ", bullet_style));
                    }

                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

                    let is_dirty = app.session.is_dirty(path);

                    let style = if is_selected {
                        Style::default()
                            .fg(app.config.theme.file_menu_highlight_fg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    spans.push(Span::styled(filename.to_string(), style));
                    if is_dirty {
                        let dirty_style = if is_selected {
                            Style::default().fg(app.config.theme.file_menu_highlight_fg)
                        } else {
                            Style::default().fg(app.config.theme.dirty_indicator_fg)
                        };
                        spans.push(Span::styled("*", dirty_style));
                    }
                }
            }

            let line = Line::from(spans);
            if is_selected {
                ListItem::new(line).style(
                    Style::default()
                        .bg(app.config.theme.file_menu_highlight_bg)
                        .fg(app.config.theme.file_menu_highlight_fg),
                )
            } else {
                ListItem::new(line)
            }
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
        let list = List::new(items)
            .highlight_symbol("") // No extra symbol, highlight is via bg color
            .scroll_padding(5);
        let mut state = ListState::default().with_selected(Some(selected_idx));
        frame.render_stateful_widget(list, area, &mut state);
        app.view_state.mouse_layout.file_list_offset = state.offset();
    }
}

/// Render the right column showing preview of selected item
fn render_preview_column(
    frame: &mut Frame,
    app: &App,
    filtered_entries: &[&crate::input::file_list_mode::BrowserEntry],
    area: Rect,
) {
    use crate::input::file_list_mode::{scan_directory_filtered, BrowserEntry};

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
            let entries = match scan_directory_filtered(path, app.view_state.show_hidden_files) {
                Ok(e) => e,
                Err(_) => return, // Silently fail for preview
            };

            let max_entries = area.height as usize;
            let items: Vec<ListItem> = entries
                .iter()
                .filter(|entry| entry.filename().is_some())
                .take(max_entries)
                .map(|entry| {
                    let name = entry.filename().unwrap_or_default();
                    let (display_name, style) = match entry {
                        BrowserEntry::Directory(_) => {
                            let dn = if name == ".." {
                                "../".to_string()
                            } else {
                                name.to_string()
                            };
                            (
                                dn,
                                Style::default()
                                    .fg(app.config.theme.file_menu_dir_fg)
                                    .add_modifier(Modifier::DIM),
                            )
                        }
                        BrowserEntry::CsvFile(_) => (
                            name.to_string(),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                    };

                    ListItem::new(Line::from(Span::styled(display_name, style)))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, area);
        }
        BrowserEntry::CsvFile(path) => {
            // Preview CSV file contents - fill the available height
            let max_lines = area.height as usize;
            let preview_lines = read_csv_preview(path, max_lines);
            let col_colors = column_colors(&app.config.theme);

            let items: Vec<ListItem> = preview_lines
                .iter()
                .enumerate()
                .map(|(idx, line)| {
                    let line_num = format!("{},", idx);
                    let mut spans =
                        vec![Span::styled(line_num, Style::default().fg(Color::DarkGray))];

                    // Parse CSV fields and colorize by column index
                    for (fi, field) in line.split(',').enumerate() {
                        if fi > 0 {
                            spans.push(Span::styled(",", Style::default()));
                        }
                        let color = col_colors[fi % col_colors.len()];
                        spans.push(Span::styled(field, Style::default().fg(color)));
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, area);
        }
    }
}

/// Render the yazi-style Spot popup with file details
fn render_spot_popup(frame: &mut Frame, app: &App, area: Rect) {
    use crate::input::file_list_mode::{scan_directory_filtered, BrowserEntry};
    use std::fs;

    // Get the currently selected entry
    let show_hidden = app.view_state.show_hidden_files;
    let entries = match scan_directory_filtered(&app.view_state.current_directory, show_hidden) {
        Ok(e) => e,
        Err(_) => return,
    };

    let filter = &app.input_state.file_filter_buffer;
    let filter_lower = filter.to_lowercase();
    let filtered: Vec<&BrowserEntry> = entries
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

    if filtered.is_empty() {
        return;
    }

    let idx = app.view_state.file_list_selected.min(filtered.len() - 1);
    let entry = filtered[idx];
    let path = entry.path();

    // Gather file metadata
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };

    let created = metadata
        .created()
        .ok()
        .map(format_system_time)
        .unwrap_or_else(|| "-".to_string());

    let modified = metadata
        .modified()
        .ok()
        .map(format_system_time)
        .unwrap_or_else(|| "-".to_string());

    let mimetype = if metadata.is_dir() {
        "inode/directory".to_string()
    } else {
        match path.extension().and_then(|e| e.to_str()) {
            Some("csv") => "text/csv".to_string(),
            Some("xlsx") | Some("xls") => "application/vnd.ms-excel".to_string(),
            Some("ods") => "application/vnd.oasis.opendocument.spreadsheet".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    };

    // Count CSV rows (only for CSV files, skip for directories)
    let row_count = match entry {
        BrowserEntry::CsvFile(p) => count_csv_rows(p)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string()),
        BrowserEntry::Directory(_) => "-".to_string(),
    };

    // Build popup content
    let label_style = Style::default().add_modifier(Modifier::DIM);
    let value_style = Style::default().fg(Color::Yellow);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("    Created:   ", label_style),
            Span::styled(&created, value_style),
        ]),
        Line::from(vec![
            Span::styled("    Modified:  ", label_style),
            Span::styled(&modified, value_style),
        ]),
        Line::from(vec![
            Span::styled("    Mimetype:  ", label_style),
            Span::raw(&mimetype),
        ]),
        Line::from(vec![
            Span::styled("    Rows:      ", label_style),
            Span::styled(&row_count, value_style),
        ]),
        Line::from(""),
    ];

    // Size and position: centered in the lower half
    let popup_width = 44u16;
    let popup_height = 8u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + area.height.saturating_sub(popup_height + 2);
    let popup_area = Rect::new(x, y, popup_width.min(area.width), popup_height);

    // Clear and draw
    frame.render_widget(Clear, popup_area);

    let title = " Info ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(ratatui::layout::Alignment::Center)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

/// Format a SystemTime as "YYYY-MM-DD HH:MM:SS"
fn format_system_time(time: std::time::SystemTime) -> String {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple UTC-based formatting (good enough for display)
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate date from days since epoch
    let (year, month, day) = days_to_date(days as i64);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day)
fn days_to_date(days: i64) -> (i64, u32, u32) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Count the number of rows in a CSV file (excluding header)
fn count_csv_rows(path: &std::path::Path) -> Option<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let count = reader.lines().count().saturating_sub(1);
    Some(format_with_commas(count))
}

/// Format a number with comma separators (e.g., 100000 -> "100,000")
fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
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

/// Render the yazi-style status bar
fn render_file_manager_status_bar(
    frame: &mut Frame,
    app: &App,
    filtered_entries: &[&crate::input::file_list_mode::BrowserEntry],
    area: Rect,
) {
    use crate::input::file_list_mode::BrowserEntry;

    if app.input_state.file_list_search_active {
        // Show filter pattern when searching
        let search_line = Line::from(vec![
            Span::styled(
                " / ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(&app.input_state.file_filter_buffer),
        ]);
        let para = Paragraph::new(search_line)
            .style(Style::default().bg(app.config.theme.file_menu_status_bg));
        frame.render_widget(para, area);
        return;
    }

    let selected_idx = app.view_state.file_list_selected;
    let total = filtered_entries.len();

    // Get selected entry info
    let (filename, file_size) = if !filtered_entries.is_empty() {
        let idx = selected_idx.min(total - 1);
        let entry = filtered_entries[idx];
        let name = entry.filename().unwrap_or("").to_string();
        let size = match entry {
            BrowserEntry::CsvFile(p) | BrowserEntry::Directory(p) => {
                std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
            }
        };
        (name, size)
    } else {
        ("".to_string(), 0u64)
    };

    // Format file size
    let size_str = format_file_size(file_size);

    // Position indicator
    let position = if total > 0 {
        let idx = selected_idx.min(total - 1);
        let pct = if total <= 1 {
            100
        } else {
            (idx * 100) / (total - 1)
        };
        format!(" {}% {}/{} ", pct, idx + 1, total)
    } else {
        " 0/0 ".to_string()
    };

    // Build yazi-style status bar
    let mode_span = Span::styled(
        " NOR ",
        Style::default()
            .bg(app.config.theme.file_menu_status_mode_bg)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let size_span = Span::styled(
        format!(" {} ", size_str),
        Style::default()
            .bg(app.config.theme.file_menu_status_accent_bg)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let filename_span = Span::styled(
        format!("  {} ", filename),
        Style::default().bg(app.config.theme.file_menu_status_bg),
    );

    // Right side: position
    let right_text = position;
    let used_width = 5 + size_str.len() + 2 + filename.len() + 3 + right_text.len();
    let padding = (area.width as usize).saturating_sub(used_width);

    let pad_span = Span::styled(
        " ".repeat(padding),
        Style::default().bg(app.config.theme.file_menu_status_bg),
    );

    let position_span = Span::styled(
        right_text,
        Style::default()
            .bg(app.config.theme.file_menu_status_accent_bg)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let status_line = Line::from(vec![
        mode_span,
        size_span,
        filename_span,
        pad_span,
        position_span,
    ]);

    let para = Paragraph::new(status_line);
    frame.render_widget(para, area);
}

/// Format file size in human-readable form
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.0}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
