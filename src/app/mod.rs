mod input;
mod navigation;

use crate::csv_data::CsvData;
use anyhow::{Context, Result};
use crossterm::event::KeyEvent;
use ratatui::widgets::TableState;
use std::path::PathBuf;

/// Application modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal, // Navigation mode
            // Edit,    // Phase 2: Editing a cell
            // Visual,  // Phase 4: Visual selection
            // Command, // Phase 4: Command input
}

/// Main application state
pub struct App {
    /// Loaded CSV data
    pub csv_data: CsvData,

    /// Current table selection state (tracks selected row)
    pub table_state: TableState,

    /// Currently selected column
    pub selected_col: usize,

    /// Horizontal scroll offset (for wide tables)
    pub horizontal_offset: usize,

    /// Whether to show help overlay
    pub show_cheatsheet: bool,

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
    // Phase 2: Cell editing
    // pub edit_buffer: String,
}

impl App {
    /// Create new App from loaded CSV data and file list
    pub fn new(csv_data: CsvData, csv_files: Vec<PathBuf>, current_file_index: usize) -> Self {
        let mut table_state = TableState::default();
        // Select first row by default
        table_state.select(Some(0));

        Self {
            csv_data,
            table_state,
            selected_col: 0,
            horizontal_offset: 0,
            show_cheatsheet: false,
            should_quit: false,
            mode: Mode::Normal,
            csv_files,
            current_file_index,
            status_message: None,
        }
    }

    /// Handle keyboard input events
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        input::handle_key(self, key)
    }

    /// Get current selected row index (for status display)
    pub fn selected_row(&self) -> Option<usize> {
        self.table_state.selected()
    }

    /// Get current file path
    pub fn current_file(&self) -> &PathBuf {
        &self.csv_files[self.current_file_index]
    }

    /// Reload CSV data from current file
    pub fn reload_current_file(&mut self) -> Result<()> {
        let file_path = self.current_file().clone();
        self.csv_data = CsvData::from_file(&file_path)
            .context(format!("Failed to reload file: {}", file_path.display()))?;

        // Reset cursor to first row
        self.table_state.select(Some(0));
        self.selected_col = 0;
        self.horizontal_offset = 0;

        Ok(())
    }
}
