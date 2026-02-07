//! UI view state management including viewport control and scroll offsets.
//!
//! This module manages the state of the user interface including the current
//! selection, scroll position, and viewport positioning modes.

use crate::domain::position::ColIndex;
use ratatui::widgets::TableState;

/// Viewport positioning mode for view commands (zt, zz, zb)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportMode {
    Auto,   // Auto-center when possible (default)
    Top,    // Selected row at top (zt)
    Center, // Selected row centered (zz)
    Bottom, // Selected row at bottom (zb)
}

/// Holds state for the UI/View layer
#[derive(Debug)]
pub struct ViewState {
    /// Ratatui table widget state (tracks row selection)
    pub table_state: TableState,

    /// Currently selected column
    pub selected_column: ColIndex,

    /// Column scroll offset (how many columns to skip on the left)
    pub column_scroll_offset: usize,

    /// Whether the help overlay is currently shown
    pub help_overlay_visible: bool,

    /// Current viewport positioning mode
    pub viewport_mode: ViewportMode,

    /// File list horizontal scroll offset (for wide file lists)
    pub file_list_scroll_offset: usize,

    /// Help overlay vertical scroll offset
    pub help_scroll_offset: u16,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            table_state: TableState::default(),
            selected_column: ColIndex::new(0),
            column_scroll_offset: 0,
            help_overlay_visible: false,
            viewport_mode: ViewportMode::Auto,
            file_list_scroll_offset: 0,
            help_scroll_offset: 0,
        }
    }
}

impl ViewState {
    /// Create a new ViewState with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle the help overlay visibility
    pub fn toggle_help(&mut self) {
        self.help_overlay_visible = !self.help_overlay_visible;
    }

    /// Show the help overlay
    pub fn show_help(&mut self) {
        self.help_overlay_visible = true;
    }

    /// Hide the help overlay
    pub fn hide_help(&mut self) {
        self.help_overlay_visible = false;
        self.help_scroll_offset = 0; // Reset scroll when closing
    }

    /// Check if help overlay is visible
    pub fn is_help_visible(&self) -> bool {
        self.help_overlay_visible
    }

    /// Scroll help overlay down
    pub fn scroll_help_down(&mut self, max_scroll: u16) {
        if self.help_scroll_offset < max_scroll {
            self.help_scroll_offset += 1;
        }
    }

    /// Scroll help overlay up
    pub fn scroll_help_up(&mut self) {
        self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
    }

    /// Scroll help overlay down by a page
    pub fn scroll_help_page_down(&mut self, page_size: u16, max_scroll: u16) {
        self.help_scroll_offset = (self.help_scroll_offset + page_size).min(max_scroll);
    }

    /// Scroll help overlay up by a page
    pub fn scroll_help_page_up(&mut self, page_size: u16) {
        self.help_scroll_offset = self.help_scroll_offset.saturating_sub(page_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_state_default() {
        let state = ViewState::new();
        assert_eq!(state.selected_column, ColIndex::new(0));
        assert_eq!(state.column_scroll_offset, 0);
        assert!(!state.help_overlay_visible);
        assert_eq!(state.viewport_mode, ViewportMode::Auto);
    }

    #[test]
    fn test_toggle_help() {
        let mut state = ViewState::new();

        assert!(!state.is_help_visible());

        state.toggle_help();
        assert!(state.is_help_visible());

        state.toggle_help();
        assert!(!state.is_help_visible());
    }

    #[test]
    fn test_show_hide_help() {
        let mut state = ViewState::new();

        state.show_help();
        assert!(state.is_help_visible());

        state.hide_help();
        assert!(!state.is_help_visible());
    }

    #[test]
    fn test_viewport_mode() {
        let mut state = ViewState::new();
        assert_eq!(state.viewport_mode, ViewportMode::Auto);

        state.viewport_mode = ViewportMode::Center;
        assert_eq!(state.viewport_mode, ViewportMode::Center);
    }
}
