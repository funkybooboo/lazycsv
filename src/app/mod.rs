//! Core application state and logic
//!
//! This module contains the main `App` struct which holds all application state
//! including the current document, session, input state, clipboard, and UI state.
//! It orchestrates CSV operations, vim-style modal editing, and coordinates between
//! different subsystems.
//!
//! # Application Structure
//!
//! The `App` serves as the central hub connecting:
//!
//! - **Document**: Current CSV file data and operations
//! - **Session**: Multi-file management, caching, and configuration
//! - **Input State**: Keyboard input handling, command buffers, pending operations
//! - **Clipboard**: Dual-buffer system for row/column operations
//! - **View State**: UI state (scrolling, help overlay, file list)
//! - **Modes**: Vim-style modal editing (Normal, Insert, Visual, Command, etc.)
//!
//! # Modal Editing
//!
//! LazyCSV uses vim-style modes for different operations:
//!
//! - **Normal**: Navigation and commands (default)
//! - **Insert**: Quick inline cell editing
//! - **Magnifier**: Full vim editor for complex multi-line content
//! - **Visual**: Block/Line/Column selection for bulk operations
//! - **Command**: Ex commands (`:w`, `:q`, `:sort`, etc.)
//! - **Search**: Pattern search across all cells
//! - **SQL Editor**: SQL query execution on CSV data
//! - **File List**: Fuzzy file picker for multi-file sessions
//!
//! # Key Responsibilities
//!
//! - CSV file loading, editing, and saving
//! - Undo/redo for document mutations
//! - Multi-file session management
//! - SQL query execution via DuckDB
//! - External file modification detection
//! - Dirty state tracking across operations

mod constructors;
mod context_menu;
mod duckdb_cache;
mod file_operations;
mod formula_methods;
mod magnifier_methods;
pub mod messages;
mod schema_cache;
mod sql_completion;
mod sql_diagnostics;
mod sql_execution;
mod types;
mod visual_selection;

pub use context_menu::{ContextMenu, ContextMenuItem};
pub use duckdb_cache::DuckDbCache;
pub use schema_cache::SchemaCache;
pub use sql_completion::{
    CompletionItem, CompletionKind, SqlCompletion, SqlHistoryPopup, TemplateStep,
    COMPLETION_MAX_VISIBLE,
};
pub use sql_diagnostics::{DiagnosticSeverity, SqlDiagnostic};
pub use types::{EditBuffer, FileOperation, Mode};
pub use visual_selection::{VisualMode, VisualSelection};

use crate::clipboard::DualClipboard;
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::{InputResult, InputState, StatusMessage};
use crate::session::Session;
use crate::ui::ViewState;
use crate::Document;
use anyhow::Result;
use crossterm::event::KeyEvent;
use std::path::PathBuf;

/// Main application state (v0.2.0 Phase 2: Refactored for separation of concerns)
#[derive(Debug)]
pub struct App {
    /// Loaded CSV document
    pub document: Document,

    /// User configuration (theme, defaults, etc.)
    pub config: crate::config::Config,

    /// Watches config files for live reload
    config_watcher: crate::config::ConfigWatcher,

    /// CSV-level undo/redo history
    pub history: crate::history::History,

    /// Last edit command for dot-repeat (.)
    pub last_edit: Option<crate::history::EditCommand>,

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

    /// Vim editor for SQL query editing (None when not in SQL editor mode)
    pub sql_vim_editor: Option<crate::vim_editor::VimEditor>,

    /// SQL table-name completion popup state (None when not showing)
    pub sql_completion: Option<SqlCompletion>,

    /// SQL query history (most recent first)
    pub sql_history: Vec<String>,

    /// SQL history popup state (None when not showing)
    pub sql_history_popup: Option<SqlHistoryPopup>,

    /// Pre-execution SQL diagnostics (inline error/warning markers)
    pub sql_diagnostics: Vec<SqlDiagnostic>,

    /// Pending template steps to execute after each table/column selection.
    pub sql_template_steps: Vec<TemplateStep>,

    /// Last column name picked via a template `PickColumn` step.
    pub sql_template_last_column: Option<String>,

    /// Last table name picked via a template `PickTable` step.
    pub sql_template_last_table: Option<String>,

    /// Magnifier state for complex cell editing (None when not in magnifier mode)
    pub magnifier_state: Option<crate::magnifier::MagnifierState>,

    /// Flag to quit application
    pub should_quit: bool,

    /// Cached DuckDB connection for repeated SQL queries
    pub duckdb_cache: Option<DuckDbCache>,

    /// Cached CSV header schemas (avoids re-reading from disk on every keystroke)
    pub schema_cache: SchemaCache,

    /// True when an external file modification has been detected and we're waiting for user response
    pub external_modification_pending: bool,

    /// Active search state (persists after search for n/N navigation)
    pub search_state: Option<crate::search::SearchState>,

    /// Search input buffer (typed text during / search prompt)
    pub search_buffer: String,

    /// Help search input buffer (typed text during help / search)
    pub help_search_buffer: String,

    /// Visual mode selection state (None when not in visual mode)
    pub visual_selection: Option<VisualSelection>,

    /// Last visual selection (for gv command to reselect)
    pub last_visual_selection: Option<VisualSelection>,

    /// File operation prompt state
    pub file_operation: Option<FileOperation>,

    /// File operation prompt buffer
    pub file_operation_buffer: String,

    /// Formula store for cell formulas (TUI-only, not persisted to CSV)
    pub formula_store: crate::formula::FormulaStore,

    /// Formula completion popup state (shown during insert mode when typing '=')
    pub formula_completion: Option<SqlCompletion>,

    /// Right-click context menu state
    pub context_menu: Option<ContextMenu>,

    /// Vim-style macro recording/playback state
    pub macros: crate::macros::MacroState,

    /// Compiled keymap (vim default + user overrides). Loaded at startup
    /// and reloaded by the file watcher on `keys.toml` changes.
    pub keymap: crate::config::keys::Keymap,

    /// Persistent ex-command (`:`) history (most recent first)
    pub command_history: Vec<String>,

    /// Cursor into `command_history` while navigating with Up/Down (None = at fresh prompt)
    pub command_history_index: Option<usize>,

    /// Snapshot of `command_buffer` before history navigation began (restored on Down past newest)
    pub command_history_pending: Option<String>,

    /// Persistent file-menu shell-command history (most recent first)
    pub shell_history: Vec<String>,

    /// Cursor into `shell_history` while Up/Down is walking through it
    pub shell_history_index: Option<usize>,

    /// Snapshot of `shell_buffer` before history navigation began
    pub shell_history_pending: Option<String>,

    /// Captured stderr from the last shell command (for the scrollable popup)
    pub shell_error_popup: Option<ShellErrorPopup>,
}

/// Scrollable popup state for multi-line shell-command stderr.
#[derive(Debug, Clone)]
pub struct ShellErrorPopup {
    pub title: String,
    pub body: String,
    pub scroll: u16,
}

/// Key handler for the shell-stderr popup overlay. j/k scroll; anything else
/// dismisses the popup and falls through to normal dispatch on the *next* key.
fn handle_shell_error_popup_key(app: &mut App, key: KeyEvent) -> InputResult {
    use crossterm::event::KeyCode;
    let lines = match app.shell_error_popup.as_ref() {
        Some(p) => p.body.lines().count() as u16,
        None => return InputResult::Continue,
    };
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(p) = app.shell_error_popup.as_mut() {
                p.scroll = p.scroll.saturating_add(1).min(lines.saturating_sub(1));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(p) = app.shell_error_popup.as_mut() {
                p.scroll = p.scroll.saturating_sub(1);
            }
        }
        KeyCode::PageDown | KeyCode::Char('d') => {
            if let Some(p) = app.shell_error_popup.as_mut() {
                p.scroll = p.scroll.saturating_add(10).min(lines.saturating_sub(1));
            }
        }
        KeyCode::PageUp | KeyCode::Char('u') => {
            if let Some(p) = app.shell_error_popup.as_mut() {
                p.scroll = p.scroll.saturating_sub(10);
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            if let Some(p) = app.shell_error_popup.as_mut() {
                p.scroll = 0;
            }
        }
        KeyCode::Char('G') | KeyCode::End => {
            if let Some(p) = app.shell_error_popup.as_mut() {
                p.scroll = lines.saturating_sub(1);
            }
        }
        _ => {
            // Any other key dismisses.
            app.shell_error_popup = None;
        }
    }
    InputResult::Continue
}

impl App {
    /// Create new App from loaded CSV data, file list, and file configuration
    pub fn new(
        csv_data: Document,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        file_config: crate::session::FileConfig,
    ) -> Self {
        let config_result = crate::config::load_config_with_warnings();
        let config_warnings = config_result.warnings.clone();
        let config = config_result.config;

        // Apply config defaults where CLI didn't specify values
        let mut file_config = file_config;
        if file_config.delimiter.is_none() {
            if let Some(d) = config.defaults.delimiter {
                file_config.delimiter = Some(d as u8);
            }
        }
        if file_config.encoding.is_none() {
            file_config.encoding = config.defaults.encoding.clone();
        }

        // Create session
        let session = Session::new(csv_files, current_file_index, file_config);

        // Initialize view state - start at row 0 (displays as row 1)
        let mut view_state = ViewState::default();
        view_state.table_state.select(Some(0));
        view_state.show_footer_row = config.defaults.show_footer;

        // Create input state
        let input_state = InputState::new();

        let config_watcher = crate::config::ConfigWatcher::new();
        let history = crate::history::History::new(config.defaults.undo_limit);

        let mut app = Self {
            document: csv_data,
            config,
            config_watcher,
            history,
            last_edit: None,
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
            sql_vim_editor: None,
            sql_completion: None,
            sql_history: Vec::new(),
            sql_history_popup: None,
            sql_diagnostics: Vec::new(),
            sql_template_steps: Vec::new(),
            sql_template_last_column: None,
            sql_template_last_table: None,
            magnifier_state: None,
            should_quit: false,
            duckdb_cache: None,
            schema_cache: SchemaCache::default(),
            external_modification_pending: false,
            search_state: None,
            search_buffer: String::new(),
            help_search_buffer: String::new(),
            visual_selection: None,
            last_visual_selection: None,
            file_operation: None,
            file_operation_buffer: String::new(),
            formula_store: crate::formula::FormulaStore::new(),
            formula_completion: None,
            context_menu: None,
            macros: crate::macros::MacroState::new(),
            keymap: crate::config::keys::Keymap::vim_default(),
            command_history: Vec::new(),
            command_history_index: None,
            command_history_pending: None,
            shell_history: Vec::new(),
            shell_history_index: None,
            shell_history_pending: None,
            shell_error_popup: None,
        };
        let xlsx_formulas = std::mem::take(&mut app.document.xlsx_formulas);
        for ((row, col), raw) in xlsx_formulas {
            if let Some(formula) = crate::formula::parse_formula(&raw) {
                app.formula_store.set(row, col, raw, formula);
            } else {
                // Store unsupported formulas with a dummy formula for display in the formula bar
                app.formula_store
                    .set(row, col, raw, crate::formula::Formula::unsupported());
            }
        }

        // Show config warnings in status bar
        if !config_warnings.is_empty() {
            app.status_message = Some(crate::input::StatusMessage::from(format!(
                "Config: {}",
                config_warnings.join("; ")
            )));
        }

        app
    }

    /// Replay the macro stored in `register`, feeding each event through `handle_key`.
    /// Returns `Ok(())` even if the register is empty (no-op).
    /// Aborts silently if max replay depth is exceeded (prevents `@a` calling itself).
    pub fn replay_macro(&mut self, register: char) -> Result<()> {
        let keys: Vec<KeyEvent> = match self.macros.get(register) {
            Some(slice) => slice.to_vec(),
            None => return Ok(()),
        };
        if !self.macros.begin_replay() {
            self.status_message = Some(StatusMessage::from(
                "Macro replay depth exceeded".to_string(),
            ));
            return Ok(());
        }
        let result = (|| -> Result<()> {
            for k in keys {
                self.handle_key(k)?;
            }
            Ok(())
        })();
        self.macros.end_replay();
        self.macros.set_last_played(register);
        result
    }

    /// Handle keyboard input events
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<InputResult> {
        // Shell stderr popup intercepts all keys until dismissed.
        if self.shell_error_popup.is_some() {
            return Ok(handle_shell_error_popup_key(self, key));
        }
        // Record raw input into the active macro register before dispatch.
        // The handlers themselves decide which keys start/stop recording — those
        // keys are filtered out here by checking that recording was already in
        // progress *before* we look at the key's effect.
        if self.macros.is_recording() && !self.macros.is_replaying() {
            self.macros.record_key(key);
        }
        crate::input::handle_key(self, key)
    }

    /// Handle mouse input events. Returns (InputResult, needs_redraw).
    pub fn handle_mouse(&mut self, event: crossterm::event::MouseEvent) -> (InputResult, bool) {
        crate::input::mouse_handler::handle_mouse(self, event)
    }

    /// Get current selected row index (for status display)
    pub fn selected_row(&self) -> Option<RowIndex> {
        self.view_state.table_state.selected().map(RowIndex::new)
    }

    /// Get current file path
    pub fn current_file(&self) -> &PathBuf {
        self.session.current_file()
    }
}

#[cfg(test)]
mod tests;
