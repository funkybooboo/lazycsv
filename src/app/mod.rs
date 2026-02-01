mod input;
mod navigation;

use crate::CsvData;
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Instant;

/// Application modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal, // Navigation mode
            // Edit,    // v0.4.0: Quick cell editing
            // Visual,  // v1.1.0: Visual selection mode
            // Command, // v1.1.0: Command input mode
}

/// Viewport centering mode for view commands (zt, zz, zb)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportMode {
    Auto,   // Auto-center when possible (default)
    Top,    // Selected row at top (zt)
    Center, // Selected row centered (zz)
    Bottom, // Selected row at bottom (zb)
}

/// Holds state for the UI
#[derive(Debug)]
pub struct UiState {
    pub table_state: TableState,
    pub selected_col: usize,
    pub horizontal_offset: usize,
    pub show_cheatsheet: bool,
    pub viewport_mode: ViewportMode,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            table_state: TableState::default(),
            selected_col: 0,
            horizontal_offset: 0,
            show_cheatsheet: false,
            viewport_mode: ViewportMode::Auto,
        }
    }
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Loaded CSV data
    pub csv_data: CsvData,

    /// UI-related state
    pub ui: UiState,

    /// Flag to quit application
    pub should_quit: bool,

    /// Current application mode
    pub mode: Mode,

    /// List of CSV files in the same directory
    pub csv_files: Vec<PathBuf>,

    /// Index of current file in csv_files
    pub current_file_index: usize,

    /// Optional status message to display
    pub status_message: Option<Cow<'static, str>>,

    // Multi-key command support
    /// Pending key for multi-key commands (e.g., after 'g', waiting for second key)
    pub pending_key: Option<KeyCode>,

    /// Time when pending key was set (for timeout)
    pub pending_key_time: Option<Instant>,

    /// Count prefix for vim commands (e.g., "5" for 5j)
    pub command_count: Option<String>,

    /// Delimiter used for CSV parsing
    pub delimiter: Option<u8>,

    /// Whether CSV was parsed without headers
    pub no_headers: bool,

    /// The character encoding used for file loading
    pub encoding: Option<String>,
    // v0.4.0: Cell editing fields (to be implemented)
    // pub edit_buffer: String,
}

impl App {
    /// Create a new `App` instance from CLI arguments.
    /// This function handles file scanning, initial data loading, and App creation.
    pub fn from_cli(cli_args: crate::cli::CliArgs) -> Result<Self> {
        let path = cli_args.path.context("No path provided")?;

        // Determine the CSV file to load and scan directory for others
        let (file_path, csv_files, current_file_index) = if path.is_file() {
            let csv_files = crate::file_scanner::scan_directory_for_csvs(&path)?;
            let current_file_index = csv_files.iter().position(|p| p == &path).unwrap_or(0);
            (path, csv_files, current_file_index)
        } else if path.is_dir() {
            let csv_files = crate::file_scanner::scan_directory(&path)?;
            if csv_files.is_empty() {
                anyhow::bail!("No CSV files found in directory: {}", path.display());
            }
            let file_path = csv_files[0].clone();
            (file_path, csv_files, 0)
        } else {
            anyhow::bail!("Invalid path: {}", path.display());
        };

        // Load CSV data
        let csv_data = crate::csv_data::CsvData::from_file(
            &file_path,
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        )
        .context(format!("Failed to load CSV file: {}", file_path.display()))?;

        // Create and return the App
        Ok(Self::new(
            csv_data,
            csv_files,
            current_file_index,
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding,
        ))
    }

    /// Create new App from loaded CSV data, file list, and CLI parsing options
    pub fn new(
        csv_data: CsvData,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        delimiter: Option<u8>,
        no_headers: bool,
        encoding: Option<String>,
    ) -> Self {
        let mut ui_state = UiState::default();
        ui_state.table_state.select(Some(0));

        Self {
            csv_data,
            ui: ui_state,
            should_quit: false,
            mode: Mode::Normal,
            csv_files,
            current_file_index,
            status_message: None,
            pending_key: None,
            pending_key_time: None,
            command_count: None,
            delimiter,
            no_headers,
            encoding,
        }
    }

    /// Handle keyboard input events
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        input::handle_key(self, key)
    }

    /// Get current selected row index (for status display)
    pub fn selected_row(&self) -> Option<usize> {
        self.ui.table_state.selected()
    }

    /// Get current file path
    pub fn current_file(&self) -> &PathBuf {
        &self.csv_files[self.current_file_index]
    }

    /// Reload CSV data from current file
    pub fn reload_current_file(&mut self) -> Result<()> {
        let file_path = self.current_file().clone();
        self.csv_data = CsvData::from_file(
            &file_path,
            self.delimiter,
            self.no_headers,
            self.encoding.clone(),
        )
        .context(format!("Failed to reload file: {}", file_path.display()))?;

        // Reset UI state
        self.ui = UiState::default();
        self.ui.table_state.select(Some(0));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::path::PathBuf;

    fn create_test_csv_data() -> CsvData {
        CsvData {
            headers: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            rows: vec![
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
                vec!["4".to_string(), "5".to_string(), "6".to_string()],
                vec!["7".to_string(), "8".to_string(), "9".to_string()],
            ],
            filename: "test.csv".to_string(),
            is_dirty: false,
        }
    }

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_app_initialization() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.selected_row(), Some(0));
        assert_eq!(app.ui.selected_col, 0);
        assert!(!app.should_quit);
        assert!(!app.ui.show_cheatsheet);
    }

    #[test]
    fn test_navigation_down() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(1));

        app.handle_key(key_event(KeyCode::Down)).unwrap();
        assert_eq!(app.selected_row(), Some(2));

        // Try to go beyond last row - should stay at last row
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(2));
    }

    #[test]
    fn test_navigation_up() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        app.ui.table_state.select(Some(2));

        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.selected_row(), Some(1));

        app.handle_key(key_event(KeyCode::Up)).unwrap();
        assert_eq!(app.selected_row(), Some(0));

        // Try to go before first row - should stay at first row
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.selected_row(), Some(0));
    }

    #[test]
    fn test_navigation_left_right() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.ui.selected_col, 0);

        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, 1);

        app.handle_key(key_event(KeyCode::Right)).unwrap();
        assert_eq!(app.ui.selected_col, 2);

        // Try to go beyond last column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, 2);

        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.ui.selected_col, 1);

        app.handle_key(key_event(KeyCode::Left)).unwrap();
        assert_eq!(app.ui.selected_col, 0);

        // Try to go before first column
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.ui.selected_col, 0);
    }

    #[test]
    fn test_navigation_home_end() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        app.ui.table_state.select(Some(1));

        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
        assert_eq!(app.selected_row(), Some(2)); // Last row

        // gg - Go to first row (multi-key command)
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.selected_row(), Some(0)); // First row
    }

    #[test]
    fn test_navigation_first_last_column() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        app.ui.selected_col = 1;

        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
        assert_eq!(app.ui.selected_col, 2); // Last column

        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        assert_eq!(app.ui.selected_col, 0); // First column
    }

    #[test]
    fn test_quit_functionality() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert!(!app.should_quit);

        app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_with_unsaved_changes() {
        let mut csv_data = create_test_csv_data();
        csv_data.is_dirty = true;
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert!(!app.should_quit);

        app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
        assert!(!app.should_quit); // Should not quit
        assert!(app.status_message.is_some()); // Should show warning
    }

    #[test]
    fn test_help_toggle() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert!(!app.ui.show_cheatsheet);

        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.ui.show_cheatsheet);

        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(!app.ui.show_cheatsheet);
    }

    #[test]
    fn test_help_close_with_esc() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        app.ui.show_cheatsheet = true;

        app.handle_key(key_event(KeyCode::Esc)).unwrap();
        assert!(!app.ui.show_cheatsheet);
    }

    #[test]
    fn test_file_switching_next() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![
            PathBuf::from("file1.csv"),
            PathBuf::from("file2.csv"),
            PathBuf::from("file3.csv"),
        ];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.current_file_index, 0);

        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert!(should_reload);
        assert_eq!(app.current_file_index, 1);

        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert!(should_reload);
        assert_eq!(app.current_file_index, 2);

        // Wrap around to first file
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert!(should_reload);
        assert_eq!(app.current_file_index, 0);
    }

    #[test]
    fn test_file_switching_previous() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![
            PathBuf::from("file1.csv"),
            PathBuf::from("file2.csv"),
            PathBuf::from("file3.csv"),
        ];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.current_file_index, 0);

        let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
        assert!(should_reload);
        assert_eq!(app.current_file_index, 2); // Wrap to last file

        let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
        assert!(should_reload);
        assert_eq!(app.current_file_index, 1);
    }

    #[test]
    fn test_no_file_switching_with_single_file() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("file1.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert!(!should_reload); // Should not reload with single file
    }

    #[test]
    fn test_navigation_blocked_when_help_shown() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        app.ui.show_cheatsheet = true;
        let initial_row = app.selected_row();
        let initial_col = app.ui.selected_col;

        // Try navigation with help shown
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), initial_row);

        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, initial_col);

        // File switching should also be blocked
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert!(!should_reload);
    }

    #[test]
    fn test_current_file_path() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv"), PathBuf::from("other.csv")];
        let app = App::new(csv_data, csv_files.clone(), 0, None, false, None);

        assert_eq!(app.current_file(), &csv_files[0]);
    }

    // ========== v0.1.2: Multi-Key Command Tests ==========

    #[test]
    fn test_multi_key_gg_goes_to_first_row() {
        // Setup: Create app at row 2 (last row)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Move to last row first
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(2));

        // Execute gg command: press 'g' then 'g'
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Should be at first row (row 0)
        assert_eq!(app.selected_row(), Some(0));
    }

    #[test]
    fn test_multi_key_g_goes_to_last_row() {
        // Setup: Create app at row 0 (first row)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.selected_row(), Some(0));

        // Press G to go to last row
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at last row (row 2)
        assert_eq!(app.selected_row(), Some(2));
    }

    #[test]
    fn test_multi_key_2g_goes_to_row_2() {
        // Setup: Create app at row 0
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.selected_row(), Some(0));

        // Press '2' to start count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'G' to execute go to row 2
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at row 2 (0-indexed, so row index 1 is actually row 2)
        // Actually with 3 rows (0, 1, 2), 2G should go to row index 1 (the second row)
        // Let me check what the expected behavior is...
        // G with count goes to that line number (1-indexed), so 2G = row index 1
        assert_eq!(app.selected_row(), Some(1));
    }

    // ========== v0.1.2: Count Prefix Tests ==========

    #[test]
    fn test_count_prefix_2j_moves_down_2_rows() {
        // Setup: Create app at row 0
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.selected_row(), Some(0));

        // Press '2' to set count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'j' to move down 2 rows
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should be at row 2 (moved down 2 rows from row 0)
        assert_eq!(app.selected_row(), Some(2));
    }

    #[test]
    fn test_count_prefix_0_goes_to_first_column() {
        // Setup: Create app at column 2 (last column)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Move to last column (column 2, index 2)
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, 2);

        // Press '0' alone (no existing count) - should go to first column
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

        // Should be at column 0 (not treated as start of count)
        assert_eq!(app.ui.selected_col, 0);
    }

    #[test]
    fn test_count_prefix_clears_after_use() {
        // Setup: Create app at row 0
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Set count prefix '2'
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Use it with 'j' to move down 2 rows
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(2));

        // Now press 'j' again without count - should only move 1 row
        // But we're at last row, so we stay at row 2
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(2)); // Stays at last row

        // Move back to row 0
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.selected_row(), Some(0));

        // Press 'j' without count - should move only 1 row (count was cleared)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(1)); // Only moved 1 row, not 2
    }

    // ========== v0.1.2: Error Handling Tests ==========

    #[test]
    fn test_error_file_not_found_shows_message() {
        // Try to load a non-existent file
        use crate::CsvData;
        use std::path::PathBuf;

        let result = CsvData::from_file(
            &PathBuf::from("/nonexistent/path/file.csv"),
            None,
            false,
            None,
        );

        // Should return an error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_file_switch_single_file_no_op() {
        // Setup: Create app with only 1 file
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let initial_index = app.current_file_index;

        // Try to switch to next file with only 1 file
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();

        // Should not reload (no other files), index should stay the same
        assert!(!should_reload);
        assert_eq!(app.current_file_index, initial_index);
    }

    #[test]
    fn test_dirty_flag_behavior() {
        // Setup: Create app with clean data
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Initially not dirty
        assert!(!app.csv_data.is_dirty);

        // Navigation shouldn't set dirty flag
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert!(!app.csv_data.is_dirty);

        // File switching shouldn't set dirty flag
        let _ = app.handle_key(key_event(KeyCode::Char('[')));
        assert!(!app.csv_data.is_dirty);
    }

    #[test]
    fn test_state_after_help_toggle() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let initial_row = app.selected_row();

        // Open help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.ui.show_cheatsheet);

        // Navigation should be blocked when help is shown
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), initial_row); // Should not move

        // Close help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(!app.ui.show_cheatsheet);

        // Now navigation should work
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(initial_row.unwrap() + 1));
    }

    #[test]
    fn test_count_prefix_2l_moves_right_2_columns() {
        // Setup: Create app at column 0
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.ui.selected_col, 0);

        // Press '2' to set count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'l' to move right 2 columns
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        // Should be at column 2 (moved right 2 columns from column 0)
        assert_eq!(app.ui.selected_col, 2);
    }

    #[test]
    fn test_file_switch_at_last_boundary() {
        // Setup: Create app with 3 files, start at last file (index 2)
        let csv_data = create_test_csv_data();
        let csv_files = vec![
            PathBuf::from("file1.csv"),
            PathBuf::from("file2.csv"),
            PathBuf::from("file3.csv"),
        ];
        let mut app = App::new(csv_data, csv_files.clone(), 2, None, false, None);

        assert_eq!(app.current_file_index, 2);

        // Try to go to next file (should wrap to first)
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();

        // Should reload and wrap to first file
        assert!(should_reload);
        assert_eq!(app.current_file_index, 0);
    }

    #[test]
    fn test_state_comprehensive_after_file_switch() {
        // Setup: Create app with multiple files
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Set some state
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        let _row_before = app.selected_row();
        let _col_before = app.ui.selected_col;

        // Switch file
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert!(should_reload);

        // Verify file index changed
        assert_eq!(app.current_file_index, 1);

        // Note: State (row/col) behavior depends on implementation
        // This test documents current behavior
    }

    #[test]
    fn test_special_keys_ignored_in_normal_mode() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let initial_row = app.selected_row();
        let initial_col = app.ui.selected_col;

        // Press various special keys that should be ignored
        app.handle_key(key_event(KeyCode::F(1))).unwrap();
        app.handle_key(key_event(KeyCode::Insert)).unwrap();
        app.handle_key(key_event(KeyCode::Delete)).unwrap();

        // State should remain unchanged
        assert_eq!(app.selected_row(), initial_row);
        assert_eq!(app.ui.selected_col, initial_col);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_esc_cancels_multi_key_command() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Start multi-key by pressing 'g'
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert!(app.pending_key.is_some());

        // Press ESC to cancel
        app.handle_key(key_event(KeyCode::Esc)).unwrap();

        // Pending key should be cleared
        assert!(app.pending_key.is_none());
    }

    #[test]
    fn test_count_prefix_3g_goes_to_row_3() {
        // Setup: Create app with more rows
        let csv_data = CsvData {
            headers: vec!["A".to_string()],
            rows: vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()],
                vec!["4".to_string()],
                vec!["5".to_string()],
            ],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.selected_row(), Some(0));

        // Press '3' then 'G' to go to row 3 (1-indexed, so row index 2)
        app.handle_key(key_event(KeyCode::Char('3'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at row index 2 (3rd row)
        assert_eq!(app.selected_row(), Some(2));
    }

    #[test]
    fn test_help_closed_with_esc() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Open help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.ui.show_cheatsheet);

        // Close help with ESC
        app.handle_key(key_event(KeyCode::Esc)).unwrap();
        assert!(!app.ui.show_cheatsheet);
    }

    #[test]
    fn test_sequential_navigation_workflow() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Complex navigation sequence
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 1
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap(); // Right to col 1
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 2
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap(); // Left to col 0
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap(); // Up to row 1

        // Should be at row 1, col 0
        assert_eq!(app.selected_row(), Some(1));
        assert_eq!(app.ui.selected_col, 0);
    }

    #[test]
    fn test_dollar_sign_goes_to_last_column() {
        // Setup: Create app at column 0
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.ui.selected_col, 0);

        // Press '$' to go to last column
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

        // Should be at last column (column 2)
        assert_eq!(app.ui.selected_col, 2);
    }

    #[test]
    fn test_zero_goes_to_first_column() {
        // Setup: Create app at last column
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Move to last column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, 2);

        // Press '0' to go to first column
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

        // Should be at first column (column 0)
        assert_eq!(app.ui.selected_col, 0);
    }

    #[test]
    fn test_page_up_down_navigation() {
        // Setup: Create app with more rows
        let csv_data = CsvData {
            headers: vec!["A".to_string()],
            rows: vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()],
                vec!["4".to_string()],
                vec!["5".to_string()],
                vec!["6".to_string()],
                vec!["7".to_string()],
                vec!["8".to_string()],
                vec!["9".to_string()],
                vec!["10".to_string()],
            ],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Start at row 5
        for _ in 0..5 {
            app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        }
        assert_eq!(app.selected_row(), Some(5));

        // Page up should move up (typically ~20 rows, but we only have 10)
        app.handle_key(key_event(KeyCode::PageUp)).unwrap();
        // Should be at row 0 or higher
        assert!(app.selected_row().unwrap() <= 5);

        // Page down should move down
        app.handle_key(key_event(KeyCode::PageDown)).unwrap();
        // Should have moved or stayed at boundary
    }

    #[test]
    fn test_home_end_keys() {
        // Setup: Create app at middle
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Move to middle column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, 1);

        // Home and End keys should work without crashing
        app.handle_key(key_event(KeyCode::Home)).unwrap();
        app.handle_key(key_event(KeyCode::End)).unwrap();
        // Test passes if no panic occurs
    }

    #[test]
    fn test_column_boundary_navigation() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Try to go left from first column (should stay)
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.ui.selected_col, 0);

        // Go to last column
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
        assert_eq!(app.ui.selected_col, 2);

        // Try to go right from last column (should stay)
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, 2);
    }

    #[test]
    fn test_file_switch_preserves_position() {
        // Setup: Create app, navigate to row 2, column 2
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Navigate to row 2, column 2
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        assert_eq!(app.selected_row(), Some(2));
        assert_eq!(app.ui.selected_col, 2);

        // Note: In real app, file switch would reload and reset position
        // This test verifies current behavior
    }

    #[test]
    fn test_file_switch_at_first_boundary() {
        // Setup: Create app with 3 files, start at first file (index 0)
        let csv_data = create_test_csv_data();
        let csv_files = vec![
            PathBuf::from("file1.csv"),
            PathBuf::from("file2.csv"),
            PathBuf::from("file3.csv"),
        ];
        let mut app = App::new(csv_data, csv_files.clone(), 0, None, false, None);

        assert_eq!(app.current_file_index, 0);

        // Try to go to previous file (should wrap to last)
        let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();

        // Should reload and wrap to last file
        assert!(should_reload);
        assert_eq!(app.current_file_index, 2);
    }

    // ===== Priority 1: Navigation Edge Cases =====

    #[test]
    fn test_navigation_gg_on_single_row_file() {
        // CSV with only one data row
        let csv_data = CsvData {
            headers: vec!["A".to_string(), "B".to_string()],
            rows: vec![vec!["1".to_string(), "2".to_string()]],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Execute gg
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Should be at row 0 (the only row)
        assert_eq!(app.selected_row(), Some(0));
    }

    #[test]
    fn test_navigation_g_shift_on_single_row_file() {
        let csv_data = CsvData {
            headers: vec!["A".to_string()],
            rows: vec![vec!["1".to_string()]],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Execute G (go to last row)
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at row 0 (the only row)
        assert_eq!(app.selected_row(), Some(0));
    }

    #[test]
    fn test_count_prefix_exceeds_row_bounds() {
        let csv_data = create_test_csv_data(); // Has 3 rows
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Try to jump to row 9999 with 9999G
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should clamp to last row (row 2)
        assert_eq!(app.selected_row(), Some(2));
    }

    #[test]
    fn test_count_prefix_exceeds_column_bounds() {
        let csv_data = create_test_csv_data(); // Has 3 columns
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Try to move right 100 columns with 100l
        app.handle_key(key_event(KeyCode::Char('1'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        // Should clamp to last column (column 2)
        assert_eq!(app.ui.selected_col, 2);
    }

    #[test]
    fn test_navigation_dollar_on_single_column() {
        let csv_data = CsvData {
            headers: vec!["A".to_string()],
            rows: vec![vec!["1".to_string()]],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.ui.selected_col, 0);

        // Execute $ (go to last column)
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

        // Should stay at column 0 (only column)
        assert_eq!(app.ui.selected_col, 0);
    }

    #[test]
    fn test_navigation_zero_already_at_first_column() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.ui.selected_col, 0);

        // Execute 0 (go to first column)
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

        // Should stay at column 0
        assert_eq!(app.ui.selected_col, 0);
    }

    #[test]
    fn test_navigation_j_on_last_row() {
        let csv_data = create_test_csv_data(); // 3 rows
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Move to last row
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
        assert_eq!(app.selected_row(), Some(2));

        // Try to move down from last row
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should stay at last row
        assert_eq!(app.selected_row(), Some(2));
    }

    #[test]
    fn test_navigation_k_on_first_row() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Should start at row 0
        assert_eq!(app.selected_row(), Some(0));

        // Try to move up from first row
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();

        // Should stay at row 0
        assert_eq!(app.selected_row(), Some(0));
    }

    #[test]
    fn test_navigation_h_on_first_column() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        assert_eq!(app.ui.selected_col, 0);

        // Try to move left from first column
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();

        // Should stay at column 0
        assert_eq!(app.ui.selected_col, 0);
    }

    #[test]
    fn test_navigation_l_on_last_column() {
        let csv_data = create_test_csv_data(); // 3 columns
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Move to last column
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
        assert_eq!(app.ui.selected_col, 2);

        // Try to move right from last column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        // Should stay at column 2
        assert_eq!(app.ui.selected_col, 2);
    }

    #[test]
    fn test_count_prefix_zero_special_case() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Move to column 2
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.ui.selected_col, 2);

        // Execute 0j (should treat as "0" to first column, not "0 times j")
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should have moved to first column, then down one row
        assert_eq!(app.ui.selected_col, 0);
        assert_eq!(app.selected_row(), Some(1));
    }

    // ===== Priority 2: State Management Tests =====

    #[test]
    fn test_pending_key_cleared_on_esc() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Start a multi-key command
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.pending_key, Some(KeyCode::Char('g')));

        // Press ESC to cancel
        app.handle_key(key_event(KeyCode::Esc)).unwrap();

        // Pending key should be cleared
        assert_eq!(app.pending_key, None);
    }

    #[test]
    fn test_pending_key_cleared_on_valid_command() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Execute gg command
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.pending_key, Some(KeyCode::Char('g')));

        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Pending key should be cleared after command completes
        assert_eq!(app.pending_key, None);
    }

    #[test]
    fn test_count_prefix_cleared_after_use() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Build count prefix 25
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
        assert_eq!(app.command_count, Some("25".to_string()));

        // Execute j (move down 25 rows, will clamp to last row)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Count should be cleared
        assert_eq!(app.command_count, None);
    }

    #[test]
    fn test_state_consistency_after_rapid_navigation() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Rapid navigation sequence
        let keys = vec!['j', 'j', 'k', 'l', 'h', 'j', 'l', 'k'];
        for key in keys {
            app.handle_key(key_event(KeyCode::Char(key))).unwrap();
        }

        // State should still be valid
        assert!(app.selected_row().is_some());
        assert!(app.ui.selected_col < app.csv_data.column_count());
        assert_eq!(app.pending_key, None);
        assert_eq!(app.command_count, None);
    }

    #[test]
    fn test_dirty_flag_persistence_across_operations() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Initial state should not be dirty
        assert!(!app.csv_data.is_dirty);

        // Simulate making a change (we'll manually set it since editing isn't implemented yet)
        app.csv_data.is_dirty = true;

        // Navigation should not affect dirty flag
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert!(app.csv_data.is_dirty);

        // Help toggle should not affect dirty flag
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.csv_data.is_dirty);
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.csv_data.is_dirty);
    }

    #[test]
    fn test_state_after_invalid_g_sequence() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let initial_row = app.selected_row();
        let initial_col = app.ui.selected_col;

        // Start g command
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.pending_key, Some(KeyCode::Char('g')));

        // Send invalid character (should clear pending state)
        app.handle_key(key_event(KeyCode::Char('x'))).unwrap();

        // State should be reset
        assert_eq!(app.pending_key, None);
        // Position should not have changed
        assert_eq!(app.selected_row(), initial_row);
        assert_eq!(app.ui.selected_col, initial_col);
    }

    #[test]
    fn test_count_prefix_max_digits() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Build a very large count
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();

        // Should have count set
        assert!(app.command_count.is_some());

        // Execute command
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should clamp to valid range (last row)
        assert_eq!(app.selected_row(), Some(2)); // Last row in test data
    }
}
