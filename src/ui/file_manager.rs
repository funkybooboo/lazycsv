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
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Width percentage for file manager modal
const FILE_MANAGER_WIDTH_PERCENT: u16 = 60;

/// Height percentage for file manager modal
const FILE_MANAGER_HEIGHT_PERCENT: u16 = 70;

/// Render the file manager modal
pub fn render(frame: &mut Frame, app: &App) {
    let area = super::help::centered_rect(
        FILE_MANAGER_WIDTH_PERCENT,
        FILE_MANAGER_HEIGHT_PERCENT,
        frame.area(),
    );

    // Clear background
    frame.render_widget(Clear, area);

    // Create main layout: title + search + list + help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title + search bar
            Constraint::Min(5),    // File list
            Constraint::Length(3), // Help text
        ])
        .split(area);

    // Title and search bar
    render_title_and_search(frame, app, chunks[0]);

    // File list
    render_file_list(frame, app, chunks[1]);

    // Help text
    render_help(frame, chunks[2]);
}

/// Render title and search input
fn render_title_and_search(frame: &mut Frame, app: &App, area: Rect) {
    let filter = &app.input_state.file_filter_buffer;

    let title = if filter.is_empty() {
        " File Manager ".to_string()
    } else {
        format!(" File Manager: /{} ", filter)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default());

    // Get inner area before rendering block
    let inner = block.inner(area);

    // Render the block
    frame.render_widget(block, area);

    // Show search prompt inside
    let search_text = if filter.is_empty() {
        "Type to search files..."
    } else {
        ""
    };

    let search = Paragraph::new(search_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(search, inner);
}

/// Render the file list
fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Files ")
        .style(Style::default());

    let inner = block.inner(area);
    let block_clone = Block::default()
        .borders(Borders::ALL)
        .title(" Files ")
        .style(Style::default());

    frame.render_widget(block_clone, area);

    // Get filtered files
    let filter = app.input_state.file_filter_buffer.to_lowercase();
    let files = app.session.files();
    let active_idx = app.session.active_file_index();
    let selected_idx = app.view_state.file_list_selected;

    let filtered_files: Vec<(usize, &std::path::PathBuf)> = files
        .iter()
        .enumerate()
        .filter(|(_, path)| {
            if filter.is_empty() {
                true
            } else {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase().contains(&filter))
                    .unwrap_or(false)
            }
        })
        .collect();

    // Build list items
    let items: Vec<ListItem> = filtered_files
        .iter()
        .enumerate()
        .map(|(display_idx, (orig_idx, path))| {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let dirty = if app.session.is_dirty(path) { "*" } else { "" };
            let is_active = *orig_idx == active_idx;
            let is_selected = display_idx == selected_idx;

            // Build line with indicators
            let mut spans = Vec::new();

            // Cursor indicator
            if is_selected {
                spans.push(Span::styled(
                    "> ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw("  "));
            }

            // Active file indicator
            if is_active {
                spans.push(Span::styled("● ", Style::default().fg(Color::Green)));
            } else {
                spans.push(Span::raw("  "));
            }

            // Filename
            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            spans.push(Span::styled(format!("{}{}", filename, dirty), style));

            ListItem::new(Line::from(spans))
        })
        .collect();

    // Show empty message if no files match
    if items.is_empty() {
        let msg = if filter.is_empty() {
            "No CSV files found"
        } else {
            "No matching files"
        };
        let empty = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default());
        frame.render_widget(empty, inner);
    } else {
        let list = List::new(items);
        frame.render_widget(list, inner);
    }
}

/// Render help text
fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(":nav  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(":open  "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(":delete  "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(":rename  "),
        Span::styled("y", Style::default().fg(Color::Yellow)),
        Span::raw(":copy  "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(":new  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(":close"),
    ])];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .style(Style::default());

    let help = Paragraph::new(help_text).block(block);
    frame.render_widget(help, area);
}
