//! Statistics overlay popup for visual mode selections.
//!
//! Displays per-column statistics (count, sum, avg, min, max, distinct)
//! in a scrollable modal overlay. Triggered by `gs` in visual mode or
//! `:stats` with no argument while in visual mode.

use crate::config::Theme;
use crate::input::command_mode::stats::format_number;
use crate::ui::view_state::StatsOverlayData;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

/// Render the statistics overlay popup.
pub fn render(frame: &mut Frame, data: &StatsOverlayData, theme: &Theme) {
    let area = super::modal::large_modal_rect(frame.area());

    let title = format!(" {} ", data.title);
    let block = super::modal::popup_block(theme, &title);
    let inner = block.inner(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let (content_area, status_area) = super::modal::split_with_status_bar(inner);

    // Build content lines
    let mut lines: Vec<Line<'_>> = Vec::new();

    // Header row
    let header_style = Style::default().add_modifier(Modifier::BOLD);
    lines.push(Line::from(vec![Span::styled(
        format!(
            "{:<20} {:>8} {:>8} {:>12} {:>12} {:>12} {:>12} {:>8}",
            "Column", "Count", "Numeric", "Sum", "Avg", "Min", "Max", "Distinct"
        ),
        header_style,
    )]));

    // Separator
    lines.push(Line::from(
        "\u{2500}".repeat(content_area.width.saturating_sub(1) as usize),
    ));

    // Data rows
    for col in &data.columns {
        let sum_str = col
            .sum
            .map(format_number)
            .unwrap_or_else(|| "-".to_string());
        let avg_str = col
            .avg
            .map(format_number)
            .unwrap_or_else(|| "-".to_string());
        let min_str = col
            .min
            .map(format_number)
            .unwrap_or_else(|| "-".to_string());
        let max_str = col
            .max
            .map(format_number)
            .unwrap_or_else(|| "-".to_string());

        lines.push(Line::from(format!(
            "{:<20} {:>8} {:>8} {:>12} {:>12} {:>12} {:>12} {:>8}",
            truncate_name(&col.name, 20),
            col.non_empty_count,
            col.numeric_count,
            sum_str,
            avg_str,
            min_str,
            max_str,
            col.distinct_count,
        )));
    }

    let paragraph = Paragraph::new(lines)
        .scroll((data.scroll_offset, 0))
        .style(super::modal::popup_text_style(theme));
    frame.render_widget(paragraph, content_area);

    // Status bar
    let status = Paragraph::new(" j/k: scroll | Esc: close ")
        .style(super::modal::popup_text_style(theme).add_modifier(Modifier::DIM));
    frame.render_widget(status, status_area);
}

/// Truncate a column name to fit within the given width.
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}
