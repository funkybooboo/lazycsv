mod input;
mod navigation;

use crate::csv_data::CsvData;
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
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
    pub status_message: Option<String>,

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
