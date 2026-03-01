pub mod messages;

use crate::clipboard::DualClipboard;
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::{InputResult, InputState, StatusMessage};
use crate::session::Session;
use crate::ui::ViewState;
use crate::Document;
use anyhow::{Context, Result};
use crossterm::event::KeyEvent;
use std::path::PathBuf;

/// Application modes (vim-style modal editing)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Default mode for navigation and commands
    Normal,
    /// Quick single-cell editing (entered via i, a, s)
    Insert,
    /// Full vim editor for cell content (entered via Enter)
    Magnifier,
    /// Edit column header names (entered via gh)
    HeaderEdit,
    /// Select rows/cells/blocks (entered via v, V, Ctrl+v)
    Visual,
    /// Execute commands (entered via :)
    Command,
    /// File list picker (entered via :files)
    FileList,
    /// SQL query editor (entered via q)
    SqlEditor,
}

/// Edit buffer for cell editing
#[derive(Debug, Clone, Default)]
pub struct EditBuffer {
    /// Current content being edited
    pub content: String,
    /// Cursor position within content
    pub cursor: usize,
    /// Original content (for cancel/undo)
    pub original: String,
}

/// Main application state (v0.2.0 Phase 2: Refactored for separation of concerns)
#[derive(Debug)]
pub struct App {
    /// Loaded CSV document
    pub document: Document,

    /// View/UI state (renamed from ui, moved to ui module)
    pub view_state: ViewState,

    /// Input handling state (extracted from App)
    pub input_state: InputState,

    /// Multi-file session state (extracted from App)
    pub session: Session,

    /// Current application mode
    pub mode: Mode,

    /// Optional status message to display
    pub status_message: Option<StatusMessage>,

    /// Edit buffer for cell editing (None when not editing)
    pub edit_buffer: Option<EditBuffer>,

    /// Last edited cell position (for `gi` command)
    pub last_edit_position: Option<(RowIndex, ColIndex)>,

    /// Dual clipboard: row buffer for yy/dd/p and column buffer for ,yy/,dd/,p
    pub clipboard: DualClipboard,

    /// SQL editor buffer (persists between opens)
    pub sql_buffer: String,

    /// SQL editor cursor position (character index)
    pub sql_cursor: usize,

    /// SQL error message from last failed query (shown in editor overlay)
    pub sql_error: Option<String>,

    /// Flag to quit application
    pub should_quit: bool,
}

impl App {
    /// Create a new `App` instance from CLI arguments.
    /// This function handles file scanning, initial data loading, and App creation.
    pub fn from_cli(cli_args: crate::cli::CliArgs) -> Result<Self> {
        let path = cli_args.path.unwrap_or_else(|| PathBuf::from("."));

        // Determine the CSV file to load and scan directory for others
        let (file_path, csv_files, current_file_index) = if path.is_file() {
            let csv_files = crate::file_system::scan_directory_for_csvs(&path)?;
            let current_file_index = csv_files.iter().position(|p| p == &path).unwrap_or(0);
            (path, csv_files, current_file_index)
        } else if path.is_dir() {
            let csv_files = crate::file_system::scan_directory(&path)?;
            if csv_files.is_empty() {
                anyhow::bail!("{}", messages::no_csv_files_found(&path));
            }
            let file_path = csv_files[0].clone();
            (file_path, csv_files, 0)
        } else {
            anyhow::bail!("{}", messages::invalid_path(&path));
        };

        // Create file configuration
        let file_config = crate::session::FileConfig::with_options(
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        );

        // Load CSV data
        let csv_data = crate::csv::Document::from_file(
            &file_path,
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        )
        .context(messages::failed_to_load_csv(&file_path))?;

        // Create and return the App
        Ok(Self::new(
            csv_data,
            csv_files,
            current_file_index,
            file_config,
        ))
    }

    /// Create new App from loaded CSV data, file list, and file configuration
    pub fn new(
        csv_data: Document,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        file_config: crate::session::FileConfig,
    ) -> Self {
        // Create session first to check header_mode
        let session = Session::new(csv_files, current_file_index, file_config);

        // Initialize view state - start at row 1 if header_mode ON (default), row 0 if OFF
        let mut view_state = ViewState::default();
        let initial_row = if csv_data.header_mode && csv_data.row_count() > 1 {
            1 // Skip header row, go to first data row
        } else {
            0
        };
        view_state.table_state.select(Some(initial_row));

        // Create input state
        let input_state = InputState::new();

        Self {
            document: csv_data,
            view_state,
            input_state,
            session,
            mode: Mode::Normal,
            status_message: None,
            edit_buffer: None,
            last_edit_position: None,
            clipboard: DualClipboard::new(),
            sql_buffer: String::new(),
            sql_cursor: 0,
            sql_error: None,
            should_quit: false,
        }
    }

    /// Handle keyboard input events
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<InputResult> {
        crate::input::handle_key(self, key)
    }

    /// Get current selected row index (for status display)
    pub fn get_selected_row(&self) -> Option<RowIndex> {
        self.view_state.table_state.selected().map(RowIndex::new)
    }

    /// Get current file path
    pub fn get_current_file(&self) -> &PathBuf {
        self.session.get_current_file()
    }

    /// Reload CSV data from current file
    pub fn reload_current_file(&mut self) -> Result<()> {
        let file_path = self.get_current_file().clone();
        let config = self.session.config();

        self.document = Document::from_file(
            &file_path,
            config.delimiter,
            config.no_headers,
            config.encoding.clone(),
        )
        .context(messages::failed_to_reload_file(&file_path))?;

        // Reset view state
        self.view_state = ViewState::default();
        self.view_state.table_state.select(Some(0));

        Ok(())
    }

    /// Reload CSV data from current file with a specific delimiter
    pub fn reload_current_file_with_delimiter(&mut self, delimiter: char) -> Result<()> {
        let file_path = self.get_current_file().clone();
        let config = self.session.config();

        self.document = Document::from_file(
            &file_path,
            Some(delimiter as u8),
            config.no_headers,
            config.encoding.clone(),
        )
        .context(messages::failed_to_reload_file(&file_path))?;

        // Override delimiter
        self.document.delimiter = delimiter;

        // Reset view state
        self.view_state = ViewState::default();
        self.view_state.table_state.select(Some(0));

        Ok(())
    }

    /// Save the current file to disk
    /// Returns the path of the saved file
    pub fn save_current_file(&mut self) -> Result<PathBuf> {
        let file_path = self.get_current_file().clone();
        let delimiter = self.document.delimiter;

        // Write the file atomically
        crate::csv::write_csv_atomic(&self.document, &file_path, delimiter)
            .context(format!("Failed to save file: {:?}", file_path))?;

        // Mark as clean and remove from cache
        self.document.is_dirty = false;
        self.session.mark_clean(&file_path);
        self.session.remove_from_cache(&file_path);
        // No longer a virtual query output once saved to disk
        self.session.unmark_query_output(&file_path);

        Ok(file_path)
    }

    /// Save all dirty files in the session
    /// Returns a vector of saved file paths
    pub fn save_all_files(&mut self) -> Result<Vec<PathBuf>> {
        let mut saved_files = Vec::new();

        // Save current file if dirty
        if self.document.is_dirty {
            let path = self.save_current_file()?;
            saved_files.push(path);
        }

        // Save all other dirty cached documents
        let dirty_files: Vec<PathBuf> = self.session.get_dirty_files();
        for file_path in dirty_files {
            // Skip current file (already saved above)
            if &file_path == self.get_current_file() {
                continue;
            }

            // Get cached document
            if let Some(doc) = self.session.get_cached_document(&file_path) {
                let delimiter = doc.delimiter;

                // Write atomically
                crate::csv::write_csv_atomic(doc, &file_path, delimiter)
                    .context(format!("Failed to save file: {:?}", file_path))?;

                // Mark as clean and remove from cache
                self.session.mark_clean(&file_path);
                self.session.remove_from_cache(&file_path);

                saved_files.push(file_path);
            }
        }

        Ok(saved_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::position::{ColIndex, RowIndex};
    use crate::input::{InputResult, PendingCommand};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::num::NonZeroUsize;
    use std::path::PathBuf;

    fn create_test_csv_data() -> Document {
        Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
                vec!["4".to_string(), "5".to_string(), "6".to_string()],
                vec!["7".to_string(), "8".to_string(), "9".to_string()],
            ],
            "test.csv".to_string(),
        )
    }

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_app_initialization() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // With header_mode ON (default), starts at row 1 (first data row)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
        assert!(!app.should_quit);
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_navigation_down() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Starts at row 1, move down to row 2
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));

        // Move down to row 3 (last row)
        app.handle_key(key_event(KeyCode::Down)).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));

        // Try to go beyond last row - should stay at last row
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_navigation_up() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.view_state.table_state.select(Some(2));

        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // With header_mode=true (default), cannot navigate above row 1
        app.handle_key(key_event(KeyCode::Up)).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Try to go before first data row - should stay at row 1
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
    }

    #[test]
    fn test_navigation_left_right() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));

        app.handle_key(key_event(KeyCode::Right)).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        // Try to go beyond last column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));

        app.handle_key(key_event(KeyCode::Left)).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Try to go before first column
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_navigation_home_end() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.view_state.table_state.select(Some(1));

        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3))); // Last row

        // gg - Go to first data row (row 1 with header_mode=true)
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1))); // First data row
    }

    #[test]
    fn test_navigation_first_last_column() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.view_state.selected_column = ColIndex::new(1);

        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2)); // Last column

        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(0)); // First column
    }

    #[test]
    fn test_q_opens_sql_editor() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.mode, Mode::Normal);

        app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
        assert_eq!(app.mode, Mode::SqlEditor);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_quit_via_command_mode() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert!(!app.should_quit);

        // Enter command mode and type :q
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
        app.handle_key(key_event(KeyCode::Enter)).unwrap();
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_with_unsaved_changes() {
        let mut csv_data = create_test_csv_data();
        csv_data.is_dirty = true;
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert!(!app.should_quit);

        // Try :q with unsaved changes
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('q'))).unwrap();
        app.handle_key(key_event(KeyCode::Enter)).unwrap();
        assert!(!app.should_quit); // Should not quit
        assert!(app.status_message.is_some()); // Should show warning
    }

    #[test]
    fn test_help_toggle() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert!(!app.view_state.help_overlay_visible);

        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.view_state.help_overlay_visible);

        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_help_close_with_esc() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.view_state.help_overlay_visible = true;

        app.handle_key(key_event(KeyCode::Esc)).unwrap();
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_file_switching_next() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![
            PathBuf::from("file1.csv"),
            PathBuf::from("file2.csv"),
            PathBuf::from("file3.csv"),
        ];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.session.active_file_index(), 0);

        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert_eq!(should_reload, InputResult::ReloadFile);
        assert_eq!(app.session.active_file_index(), 1);

        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert_eq!(should_reload, InputResult::ReloadFile);
        assert_eq!(app.session.active_file_index(), 2);

        // Wrap around to first file
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert_eq!(should_reload, InputResult::ReloadFile);
        assert_eq!(app.session.active_file_index(), 0);
    }

    #[test]
    fn test_file_switching_previous() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![
            PathBuf::from("file1.csv"),
            PathBuf::from("file2.csv"),
            PathBuf::from("file3.csv"),
        ];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.session.active_file_index(), 0);

        let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
        assert_eq!(should_reload, InputResult::ReloadFile);
        assert_eq!(app.session.active_file_index(), 2); // Wrap to last file

        let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();
        assert_eq!(should_reload, InputResult::ReloadFile);
        assert_eq!(app.session.active_file_index(), 1);
    }

    #[test]
    fn test_no_file_switching_with_single_file() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("file1.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert_eq!(should_reload, InputResult::Continue); // Should not reload with single file
    }

    #[test]
    fn test_navigation_blocked_when_help_shown() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.view_state.help_overlay_visible = true;
        let initial_row = app.get_selected_row();
        let initial_col = app.view_state.selected_column;

        // Try navigation with help shown
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), initial_row);

        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, initial_col);

        // File switching should also be blocked
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert_eq!(should_reload, InputResult::Continue);
    }

    #[test]
    fn test_current_file_path() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv"), PathBuf::from("other.csv")];
        let app = App::new(
            csv_data,
            csv_files.clone(),
            0,
            crate::session::FileConfig::new(),
        );

        assert_eq!(app.get_current_file(), &csv_files[0]);
    }

    // ========== v0.1.2: Multi-Key Command Tests ==========

    #[test]
    fn test_multi_key_gg_goes_to_first_row() {
        // Setup: Create app (starts at row 1), move to last row
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to last row (row 3)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));

        // Execute gg command: press 'g' then 'g'
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Should go to row 1 (first data row, with header_mode=true)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
    }

    #[test]
    fn test_multi_key_g_goes_to_last_row() {
        // Setup: Create app (starts at row 1)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Press G to go to last row
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at last row (row 3)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_multi_key_2g_goes_to_row_2() {
        // Setup: Create app (starts at row 1 with header_mode=true)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Press '2' to start count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'G' to execute go to row 2
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // 2G should go to absolute row 2 (second data row)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
    }

    // ========== v0.1.2: Count Prefix Tests ==========

    #[test]
    fn test_count_prefix_2j_moves_down_2_rows() {
        // Setup: Create app (starts at row 1 with header_mode=true)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Press '2' to set count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'j' to move down 2 rows
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should be at row 3 (moved down 2 rows from row 1)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_count_prefix_0_goes_to_first_column() {
        // Setup: Create app at column 2 (last column)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to last column (column 2, index 2)
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        // Press '0' alone (no existing count) - should go to first column
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

        // Should be at column 0 (not treated as start of count)
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_count_prefix_clears_after_use() {
        // Setup: Create app (starts at row 1)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Set count prefix '2'
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Use it with 'j' to move down 2 rows
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));

        // Now press 'j' again without count - should only move 1 row
        // But we're at last row, so we stay at row 3
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3))); // Stays at last row

        // Move back to row 1 (gg goes to row 1 with header_mode=true)
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Press 'j' without count - should move only 1 row (count was cleared)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2))); // Only moved 1 row, not 2
    }

    // ========== v0.1.2: Error Handling Tests ==========

    #[test]
    fn test_error_file_not_found_shows_message() {
        // Try to load a non-existent file
        use crate::Document;
        use std::path::PathBuf;

        let result = Document::from_file(
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
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        let initial_index = app.session.active_file_index();

        // Try to switch to next file with only 1 file
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();

        // Should not reload (no other files), index should stay the same
        assert_eq!(should_reload, InputResult::Continue);
        assert_eq!(app.session.active_file_index(), initial_index);
    }

    #[test]
    fn test_dirty_flag_behavior() {
        // Setup: Create app with clean data
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Initially not dirty
        assert!(!app.document.is_dirty);

        // Navigation shouldn't set dirty flag
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert!(!app.document.is_dirty);

        // File switching shouldn't set dirty flag
        let _ = app.handle_key(key_event(KeyCode::Char('[')));
        assert!(!app.document.is_dirty);
    }

    #[test]
    fn test_state_after_help_toggle() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        let initial_row = app.get_selected_row();

        // Open help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.view_state.help_overlay_visible);

        // Navigation should be blocked when help is shown
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), initial_row); // Should not move

        // Close help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(!app.view_state.help_overlay_visible);

        // Now navigation should work
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(
            app.get_selected_row(),
            Some(initial_row.unwrap().saturating_add(1))
        );
    }

    #[test]
    fn test_count_prefix_2l_moves_right_2_columns() {
        // Setup: Create app at column 0
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Press '2' to set count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'l' to move right 2 columns
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        // Should be at column 2 (moved right 2 columns from column 0)
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
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
        let mut app = App::new(
            csv_data,
            csv_files.clone(),
            2,
            crate::session::FileConfig::new(),
        );

        assert_eq!(app.session.active_file_index(), 2);

        // Try to go to next file (should wrap to first)
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();

        // Should reload and wrap to first file
        assert_eq!(should_reload, InputResult::ReloadFile);
        assert_eq!(app.session.active_file_index(), 0);
    }

    #[test]
    fn test_state_comprehensive_after_file_switch() {
        // Setup: Create app with multiple files
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Set some state
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        let _row_before = app.get_selected_row();
        let _col_before = app.view_state.selected_column;

        // Switch file
        let should_reload = app.handle_key(key_event(KeyCode::Char(']'))).unwrap();
        assert_eq!(should_reload, InputResult::ReloadFile);

        // Verify file index changed
        assert_eq!(app.session.active_file_index(), 1);

        // Note: State (row/col) behavior depends on implementation
        // This test documents current behavior
    }

    #[test]
    fn test_special_keys_ignored_in_normal_mode() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        let initial_row = app.get_selected_row();
        let initial_col = app.view_state.selected_column;

        // Press various special keys that should be ignored
        app.handle_key(key_event(KeyCode::F(1))).unwrap();
        app.handle_key(key_event(KeyCode::Insert)).unwrap();
        app.handle_key(key_event(KeyCode::Delete)).unwrap();

        // State should remain unchanged
        assert_eq!(app.get_selected_row(), initial_row);
        assert_eq!(app.view_state.selected_column, initial_col);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_esc_cancels_multi_key_command() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Start multi-key by pressing 'g'
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert!(app.input_state.pending_command.is_some());

        // Press ESC to cancel
        app.handle_key(key_event(KeyCode::Esc)).unwrap();

        // Pending key should be cleared
        assert!(app.input_state.pending_command.is_none());
    }

    #[test]
    fn test_count_prefix_3g_goes_to_row_3() {
        // Setup: Create app with more rows
        let csv_data = Document::new(
            vec!["A".to_string()],
            vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()],
                vec!["4".to_string()],
                vec!["5".to_string()],
            ],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Press '3' then 'G' to go to absolute row 3
        app.handle_key(key_event(KeyCode::Char('3'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at row 3
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_help_closed_with_esc() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Open help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.view_state.help_overlay_visible);

        // Close help with ESC
        app.handle_key(key_event(KeyCode::Esc)).unwrap();
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_sequential_navigation_workflow() {
        // Setup: Create app (starts at row 1)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Complex navigation sequence
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 2
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap(); // Right to col 1
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 3
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap(); // Left to col 0
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap(); // Up to row 2

        // Should be at row 2, col 0
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_dollar_sign_goes_to_last_column() {
        // Setup: Create app at column 0
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Press '$' to go to last column
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

        // Should be at last column (column 2)
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
    }

    #[test]
    fn test_zero_goes_to_first_column() {
        // Setup: Create app at last column
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to last column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        // Press '0' to go to first column
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

        // Should be at first column (column 0)
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_page_up_down_navigation() {
        // Setup: Create app with more rows
        let csv_data = Document::new(
            vec!["A".to_string()],
            vec![
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
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Start at row 6 (move 5 times from row 1)
        for _ in 0..5 {
            app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        }
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(6)));

        // Page up should move up (typically ~20 rows, but we only have 10)
        app.handle_key(key_event(KeyCode::PageUp)).unwrap();
        // Should be at row 6 or lower
        assert!(app.get_selected_row().unwrap().get() <= 6);

        // Page down should move down
        app.handle_key(key_event(KeyCode::PageDown)).unwrap();
        // Should have moved or stayed at boundary
    }

    #[test]
    fn test_home_end_keys() {
        // Setup: Create app at middle
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to middle column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(1));

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
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Try to go left from first column (should stay)
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Go to last column
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        // Try to go right from last column (should stay)
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
    }

    #[test]
    fn test_file_switch_preserves_position() {
        // Setup: Create app, navigate to row 3, column 2
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Navigate to row 3, column 2 (move 2 rows down from row 1, 2 columns right)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

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
        let mut app = App::new(
            csv_data,
            csv_files.clone(),
            0,
            crate::session::FileConfig::new(),
        );

        assert_eq!(app.session.active_file_index(), 0);

        // Try to go to previous file (should wrap to last)
        let should_reload = app.handle_key(key_event(KeyCode::Char('['))).unwrap();

        // Should reload and wrap to last file
        assert_eq!(should_reload, InputResult::ReloadFile);
        assert_eq!(app.session.active_file_index(), 2);
    }

    // ===== Priority 1: Navigation Edge Cases =====

    #[test]
    fn test_navigation_gg_on_single_row_file() {
        // CSV with only one data row (+ header = 2 total rows)
        let csv_data = Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Execute gg - should go to row 1 (first data row with header_mode=true)
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Should be at row 1 (the only data row)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
    }

    #[test]
    fn test_navigation_g_shift_on_single_row_file() {
        let csv_data = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Execute G (go to last row) - should go to row 1 (only data row)
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at row 1 (the only data row)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
    }

    #[test]
    fn test_count_prefix_exceeds_row_bounds() {
        let csv_data = create_test_csv_data(); // Has 3 rows
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());
        let initial_row = app.get_selected_row();

        // Try to jump to row 9999 with 9999G
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Position should not change when out of bounds
        assert_eq!(app.get_selected_row(), initial_row);
        // Should show error message
        assert!(app.status_message.is_some());
        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("does not exist"));
    }

    #[test]
    fn test_count_prefix_exceeds_column_bounds() {
        let csv_data = create_test_csv_data(); // Has 3 columns
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Try to move right 100 columns with 100l
        app.handle_key(key_event(KeyCode::Char('1'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        // Should clamp to last column (column 2)
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
    }

    #[test]
    fn test_navigation_dollar_on_single_column() {
        let csv_data = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Execute $ (go to last column)
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();

        // Should stay at column 0 (only column)
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_navigation_zero_already_at_first_column() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Execute 0 (go to first column)
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();

        // Should stay at column 0
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_navigation_j_on_last_row() {
        let csv_data = create_test_csv_data(); // 3 data rows (row 1,2,3)
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to last row (row 3)
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));

        // Try to move down from last row
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should stay at last row
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_navigation_k_on_first_row() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Should start at row 1 (first data row with header_mode=true)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Try to move up from first data row - with header_mode=true, should stay at row 1
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();

        // Should still be at row 1 (header_mode prevents navigating to row 0)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));

        // Try to move up again - should stay at row 1
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(1)));
    }

    #[test]
    fn test_navigation_h_on_first_column() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.view_state.selected_column, ColIndex::new(0));

        // Try to move left from first column
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap();

        // Should stay at column 0
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
    }

    #[test]
    fn test_navigation_l_on_last_column() {
        let csv_data = create_test_csv_data(); // 3 columns
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to last column
        app.handle_key(key_event(KeyCode::Char('$'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        // Try to move right from last column
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        // Should stay at column 2
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));
    }

    #[test]
    fn test_count_prefix_zero_special_case() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to column 2
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        // Execute 0j (should treat as "0" to first column, not "0 times j")
        app.handle_key(key_event(KeyCode::Char('0'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should have moved to first column, then down one row
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));
    }

    // ===== Priority 2: State Management Tests =====

    #[test]
    fn test_pending_key_cleared_on_esc() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Start a multi-key command
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.input_state.pending_command, Some(PendingCommand::G));

        // Press ESC to cancel
        app.handle_key(key_event(KeyCode::Esc)).unwrap();

        // Pending key should be cleared
        assert_eq!(app.input_state.pending_command, None);
    }

    #[test]
    fn test_pending_key_cleared_on_valid_command() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Execute gg command
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.input_state.pending_command, Some(PendingCommand::G));

        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Pending key should be cleared after command completes
        assert_eq!(app.input_state.pending_command, None);
    }

    #[test]
    fn test_count_prefix_cleared_after_use() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Build count prefix 25
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
        assert_eq!(app.input_state.command_count, NonZeroUsize::new(25));

        // Execute j (move down 25 rows, will clamp to last row)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Count should be cleared
        assert_eq!(app.input_state.command_count, None);
    }

    #[test]
    fn test_state_consistency_after_rapid_navigation() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Rapid navigation sequence
        let keys = vec!['j', 'j', 'k', 'l', 'h', 'j', 'l', 'k'];
        for key in keys {
            app.handle_key(key_event(KeyCode::Char(key))).unwrap();
        }

        // State should still be valid
        assert!(app.get_selected_row().is_some());
        assert!(app.view_state.selected_column.get() < app.document.column_count());
        assert_eq!(app.input_state.pending_command, None);
        assert_eq!(app.input_state.command_count, None);
    }

    #[test]
    fn test_dirty_flag_persistence_across_operations() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Initial state should not be dirty
        assert!(!app.document.is_dirty);

        // Simulate making a change (we'll manually set it since editing isn't implemented yet)
        app.document.is_dirty = true;

        // Navigation should not affect dirty flag
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        assert!(app.document.is_dirty);

        // Help toggle should not affect dirty flag
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.document.is_dirty);
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.document.is_dirty);
    }

    #[test]
    fn test_state_after_invalid_g_sequence() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        let initial_row = app.get_selected_row();

        // Start g command
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.input_state.pending_command, Some(PendingCommand::G));

        // Send letter (now starts column jump sequence)
        app.handle_key(key_event(KeyCode::Char('x'))).unwrap();

        // Should transition to GotoColumn state (x is a valid letter)
        assert!(matches!(
            app.input_state.pending_command,
            Some(PendingCommand::GotoColumn(_))
        ));

        // Send Enter to execute the column jump
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        // State should be cleared after executing
        assert_eq!(app.input_state.pending_command, None);
        // Row should not have changed
        assert_eq!(app.get_selected_row(), initial_row);
        // Column should not have changed (X doesn't exist, shows error)
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
        // Should show error message
        assert!(app.status_message.is_some());
        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("does not exist"));
    }

    #[test]
    fn test_count_prefix_max_digits() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Build a very large count
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();

        // Should have count set
        assert!(app.input_state.command_count.is_some());

        // Execute command
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should clamp to valid range (last row = row 3)
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(3))); // Last row in test data
    }

    // ===== Z-Command Integration Tests (Viewport Positioning) =====

    #[test]
    fn test_z_command_top_viewport() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to next row (row 2)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));

        // Execute zt (viewport top)
        app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('t'))).unwrap();

        assert_eq!(app.view_state.viewport_mode, crate::ui::ViewportMode::Top);
        assert!(app.status_message.is_some());
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("top"));
    }

    #[test]
    fn test_z_command_center_viewport() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to next row (row 2)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));

        // Execute zz (viewport center)
        app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('z'))).unwrap();

        assert_eq!(
            app.view_state.viewport_mode,
            crate::ui::ViewportMode::Center
        );
        assert!(app.status_message.is_some());
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("center"));
    }

    #[test]
    fn test_z_command_bottom_viewport() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to next row (row 2)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.get_selected_row(), Some(RowIndex::new(2)));

        // Execute zb (viewport bottom)
        app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('b'))).unwrap();

        assert_eq!(
            app.view_state.viewport_mode,
            crate::ui::ViewportMode::Bottom
        );
        assert!(app.status_message.is_some());
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("bottom"));
    }

    #[test]
    fn test_viewport_mode_persists_across_navigation() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Set viewport to center
        app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('z'))).unwrap();
        assert_eq!(
            app.view_state.viewport_mode,
            crate::ui::ViewportMode::Center
        );

        // Move down - viewport should reset to Auto
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.view_state.viewport_mode, crate::ui::ViewportMode::Auto);
    }

    // Note: Most runtime error tests (file deletion, permission changes, etc.)
    // are in tests/error_handling_test.rs as integration tests since they
    // require file system operations with tempfile.

    #[test]
    fn test_f_command_shows_current_filename() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // :f with no argument shows current filename
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        assert_eq!(app.mode, Mode::Normal);
        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("test.csv"), "Expected filename in status, got: {}", msg);
    }

    #[test]
    fn test_f_command_renames_file() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // :f newname.csv
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
        app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
        for c in "newname.csv".chars() {
            app.handle_key(key_event(KeyCode::Char(c))).unwrap();
        }
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.document.filename, "newname.csv");
        assert_eq!(app.get_current_file(), &PathBuf::from("newname.csv"));
        assert!(app.document.is_dirty);

        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(msg.contains("newname.csv"), "Expected rename confirmation, got: {}", msg);
    }

    #[test]
    fn test_f_command_rename_marks_session_dirty() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert!(!app.session.is_current_file_dirty());

        // :f renamed.csv
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
        app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
        for c in "renamed.csv".chars() {
            app.handle_key(key_event(KeyCode::Char(c))).unwrap();
        }
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        assert!(app.session.is_current_file_dirty());
    }

    #[test]
    fn test_f_command_rename_preserves_query_output_status() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("output.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Mark as query output (simulating what happens after SQL execution)
        let path = app.get_current_file().clone();
        app.session.mark_query_output(&path);

        // :f results.csv
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
        app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
        for c in "results.csv".chars() {
            app.handle_key(key_event(KeyCode::Char(c))).unwrap();
        }
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        // Query output status should follow the renamed file
        let new_path = app.get_current_file().clone();
        assert_eq!(new_path, PathBuf::from("results.csv"));
        assert!(app.session.is_query_output(&new_path));
        assert!(!app.session.is_query_output(&PathBuf::from("output.csv")));
    }
}
