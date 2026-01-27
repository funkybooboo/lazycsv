use crate::csv_data::CsvData;
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
use std::path::PathBuf;

/// Application modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,  // Navigation mode
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
        // Returns true if we need to reload a different file
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            // Phase 2: Mode::Edit => self.handle_edit_mode(key),
        }
    }

    /// Handle keyboard input in Normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Quit - vim-style (warns if unsaved in Phase 2)
            KeyCode::Char('q') if !self.show_cheatsheet => {
                if self.csv_data.is_dirty {
                    self.status_message = Some("Unsaved changes! Use :q! to force quit".to_string());
                } else {
                    self.should_quit = true;
                }
            }

            // Toggle help/cheatsheet
            KeyCode::Char('?') => {
                self.show_cheatsheet = !self.show_cheatsheet;
            }

            // Close help overlay
            KeyCode::Esc if self.show_cheatsheet => {
                self.show_cheatsheet = false;
            }

            // File switching - Previous file
            KeyCode::Char('[') if !self.show_cheatsheet && self.csv_files.len() > 1 => {
                self.current_file_index = if self.current_file_index == 0 {
                    self.csv_files.len() - 1
                } else {
                    self.current_file_index - 1
                };
                return Ok(true); // Signal to reload file
            }

            // File switching - Next file
            KeyCode::Char(']') if !self.show_cheatsheet && self.csv_files.len() > 1 => {
                self.current_file_index = (self.current_file_index + 1) % self.csv_files.len();
                return Ok(true); // Signal to reload file
            }

            // Navigation (only when help is closed)
            _ if !self.show_cheatsheet => self.handle_navigation(key.code),

            _ => {}
        }

        Ok(false)
    }

    /// Handle navigation keys
    fn handle_navigation(&mut self, code: KeyCode) {
        match code {
            // Move up
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous_row();
            }

            // Move down
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next_row();
            }

            // Move left (previous column)
            KeyCode::Left | KeyCode::Char('h') => {
                self.select_previous_col();
            }

            // Move right (next column)
            KeyCode::Right | KeyCode::Char('l') => {
                self.select_next_col();
            }

            // Word forward (next column, same as 'l')
            KeyCode::Char('w') => {
                self.select_next_col();
            }

            // Word backward (previous column, same as 'h')
            KeyCode::Char('b') => {
                self.select_previous_col();
            }

            // First column
            KeyCode::Char('0') => {
                self.selected_col = 0;
                self.horizontal_offset = 0;
            }

            // Last column
            KeyCode::Char('$') => {
                self.selected_col = self.csv_data.column_count().saturating_sub(1);
                // Adjust horizontal offset to show last column
                let max_visible_cols = 10;
                if self.csv_data.column_count() > max_visible_cols {
                    self.horizontal_offset = self.csv_data.column_count() - max_visible_cols;
                }
            }

            // Page down
            KeyCode::PageDown | KeyCode::Char('d') if code == KeyCode::Char('d') => {
                // Note: Ctrl+d is tricky to detect, using PageDown for now
                self.select_next_page();
            }

            // Page up
            KeyCode::PageUp | KeyCode::Char('u') if code == KeyCode::Char('u') => {
                // Note: Ctrl+u is tricky to detect, using PageUp for now
                self.select_previous_page();
            }

            // Home (first row) - gg in vim
            KeyCode::Home | KeyCode::Char('g') => {
                self.table_state.select(Some(0));
            }

            // End (last row) - G in vim
            KeyCode::End | KeyCode::Char('G') => {
                if self.csv_data.row_count() > 0 {
                    self.table_state.select(Some(self.csv_data.row_count() - 1));
                }
            }

            _ => {}
        }
    }

    fn select_next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i < self.csv_data.row_count().saturating_sub(1) {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_next_col(&mut self) {
        if self.selected_col < self.csv_data.column_count().saturating_sub(1) {
            self.selected_col += 1;

            // Auto-scroll horizontally if needed
            let max_visible_cols = 10;
            if self.selected_col >= self.horizontal_offset + max_visible_cols {
                self.horizontal_offset = self.selected_col - max_visible_cols + 1;
            }
        }
    }

    fn select_previous_col(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;

            // Auto-scroll horizontally if needed
            if self.selected_col < self.horizontal_offset {
                self.horizontal_offset = self.selected_col;
            }
        }
    }

    fn select_next_page(&mut self) {
        const PAGE_SIZE: usize = 20;
        let i = match self.table_state.selected() {
            Some(i) => (i + PAGE_SIZE).min(self.csv_data.row_count().saturating_sub(1)),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_previous_page(&mut self) {
        const PAGE_SIZE: usize = 20;
        let i = match self.table_state.selected() {
            Some(i) => i.saturating_sub(PAGE_SIZE),
            None => 0,
        };
        self.table_state.select(Some(i));
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

        self.status_message = Some(format!("Loaded: {}", self.csv_data.filename));
        Ok(())
    }
}
