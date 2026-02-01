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
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            table_state: TableState::default(),
            selected_column: ColIndex::new(0),
            column_scroll_offset: 0,
            help_overlay_visible: false,
            viewport_mode: ViewportMode::Auto,
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
    }

    /// Check if help overlay is visible
    pub fn is_help_visible(&self) -> bool {
        self.help_overlay_visible
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
