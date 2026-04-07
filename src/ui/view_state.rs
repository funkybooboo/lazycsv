//! UI view state management including viewport control and scroll offsets.
//!
//! This module manages the state of the user interface including the current
//! selection, scroll position, and viewport positioning modes.

use crate::domain::position::ColIndex;
use crate::ui::conditional::{ColorRule, RowConditionalRule};
use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::time::Instant;

/// Layout information captured during rendering for mouse coordinate mapping.
#[derive(Debug, Clone, Default)]
pub struct MouseLayout {
    /// The table content area (chunks[2] from render_table)
    pub table_content_area: Rect,
    /// Column indices in display order (frozen first, then scrollable)
    pub display_cols: Vec<usize>,
    /// Width of each display column (including row number gutter at index 0)
    pub raw_widths: Vec<u16>,
    /// Resolved column x-start positions matching the Table widget layout.
    /// Index 0 = gutter, 1..N = data columns, N+1 = trailing right edge.
    pub col_positions: Vec<u16>,
    /// Row indices for frozen rows (displayed at top)
    pub frozen_row_indices: Vec<usize>,
    /// Row indices for scrollable rows (in display order)
    pub scrollable_indices: Vec<usize>,
    /// Width of the row number gutter
    pub row_num_width: u16,
    /// The file manager current-column area (for file list clicks)
    pub file_list_area: Rect,
    /// Last left-click position and time for double-click detection
    pub last_click: Option<(Instant, u16, u16)>,
    /// Drag anchor cell (row, col) set on mouse-down for drag selection
    pub drag_anchor: Option<(usize, usize)>,
    /// Active column resize: (display_col_index, column_index, initial_x)
    pub col_resize: Option<(usize, usize, u16)>,
    /// Active column reorder drag: (source column index, current drop target column index)
    pub col_reorder: Option<(usize, usize)>,
    /// Active row reorder drag: (source row index, current drop target row index)
    pub row_reorder: Option<(usize, usize)>,
    /// Display column index where a resize handle is being hovered (for visual indicator)
    pub resize_hover_col: Option<usize>,
    /// Last rendered vertical scroll offset (for mouse click viewport preservation)
    pub last_scroll_offset: usize,
    /// Timestamp of last edge-scroll during drag (for throttling)
    pub last_edge_scroll: Option<Instant>,
}

/// Viewport positioning mode for view commands (zt, zz, zb)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportMode {
    Auto,         // Auto-center when possible (default)
    Top,          // Selected row at top (zt)
    Center,       // Selected row centered (zz)
    Bottom,       // Selected row at bottom (zb)
    Fixed(usize), // Keep scroll at this exact offset (used by mouse clicks)
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

    /// Selected index in file list (for FileList mode)
    pub file_list_selected: usize,

    /// Help search query (None when not searching, Some for highlights + n/N navigation)
    pub help_search_query: Option<String>,

    /// Whether the user is actively typing in the help search prompt
    pub help_search_input_active: bool,

    /// Current match index in help search results
    pub help_search_match_index: usize,

    /// Current directory for file browser (yazi-style navigation)
    pub current_directory: std::path::PathBuf,

    /// Whether to show hidden files (dotfiles) in the file browser
    pub show_hidden_files: bool,

    /// Whether the file details popup (Spot) is visible
    pub file_spot_visible: bool,

    /// Per-column background color rules (column index -> rules, first match wins)
    pub column_bg_colors: HashMap<usize, Vec<ColorRule>>,

    /// Per-column foreground color rules (column index -> rules, first match wins)
    pub column_fg_colors: HashMap<usize, Vec<ColorRule>>,

    /// Per-row background color rules (row index -> rules, first match wins)
    pub row_bg_colors: HashMap<usize, Vec<ColorRule>>,

    /// Per-row foreground color rules (row index -> rules, first match wins)
    pub row_fg_colors: HashMap<usize, Vec<ColorRule>>,

    /// Row conditional bg rules (# syntax — apply to rows matching column conditions)
    pub row_cond_bg: Vec<RowConditionalRule>,

    /// Row conditional fg rules (# syntax — apply to rows matching column conditions)
    pub row_cond_fg: Vec<RowConditionalRule>,

    /// Number of columns that fit in the current terminal width (updated during rendering)
    pub visible_cols_count: usize,

    /// Layout info captured during rendering for mouse coordinate mapping
    pub mouse_layout: MouseLayout,
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
            file_list_selected: 0,
            help_search_query: None,
            help_search_input_active: false,
            help_search_match_index: 0,
            show_hidden_files: false,
            file_spot_visible: false,
            column_bg_colors: HashMap::new(),
            column_fg_colors: HashMap::new(),
            row_bg_colors: HashMap::new(),
            row_fg_colors: HashMap::new(),
            row_cond_bg: Vec::new(),
            row_cond_fg: Vec::new(),
            current_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from(".")),
            visible_cols_count: 10, // default, updated each render frame
            mouse_layout: MouseLayout::default(),
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
        self.help_scroll_offset = 0;
        self.help_search_query = None;
        self.help_search_input_active = false;
        self.help_search_match_index = 0;
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
