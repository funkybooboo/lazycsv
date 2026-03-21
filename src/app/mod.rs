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

pub mod messages;
mod sql_execution;

use crate::cancel;
use crate::clipboard::DualClipboard;
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::{InputResult, InputState, StatusMessage};
use crate::session::Session;
use crate::ui::ViewState;
use crate::Document;
use anyhow::{Context, Result};
use crossterm::event::KeyEvent;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Application modes (vim-style modal editing)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Default mode for navigation and commands
    Normal,
    /// Quick single-cell editing (entered via i, a, s)
    Insert,
    /// Full vim editor for cell content (entered via Enter)
    Magnifier,
    /// Visual Block mode - rectangular cell selection (entered via v)
    VisualBlock,
    /// Visual Line mode - whole row selection (entered via V)
    VisualLine,
    /// Visual Column mode - whole column selection (entered via ,v)
    VisualColumn,
    /// Execute commands (entered via :)
    Command,
    /// File list picker (entered via :files)
    FileList,
    /// SQL query editor (entered via q)
    SqlEditor,
    /// Search input (entered via /)
    Search,
    /// Prompt for file operations (rename, move, copy, create)
    FileOperationPrompt,
}

/// Maximum visible rows in the completion popup
pub const COMPLETION_MAX_VISIBLE: usize = 10;

/// Category of a completion item
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionKind {
    Keyword,
    Function,
    Column,
    Table,
}

impl CompletionKind {
    /// Short tag shown in the completion popup
    pub fn tag(self) -> &'static str {
        match self {
            CompletionKind::Keyword => "[K]",
            CompletionKind::Function => "[F]",
            CompletionKind::Column => "[C]",
            CompletionKind::Table => "[T]",
        }
    }

    pub fn color(self) -> ratatui::style::Color {
        match self {
            CompletionKind::Keyword => ratatui::style::Color::Cyan,
            CompletionKind::Function => ratatui::style::Color::Yellow,
            CompletionKind::Column => ratatui::style::Color::Green,
            CompletionKind::Table => ratatui::style::Color::Magenta,
        }
    }
}

/// A single item in the completion popup
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub text: String,
    pub kind: CompletionKind,
    /// If set, selecting this item replaces the entire editor content
    /// (used for query templates).
    pub template: Option<String>,
    /// If set, these steps are executed after the template is inserted.
    /// Each step is either literal text or a table-pick prompt.
    pub template_steps: Vec<TemplateStep>,
}

/// SQL completion popup state
#[derive(Debug, Clone)]
pub struct SqlCompletion {
    /// All available items (unfiltered)
    pub all_items: Vec<CompletionItem>,
    /// Current filter/search string
    pub filter: String,
    /// Number of characters of the partial word that were already typed before
    /// the popup was opened. Used to replace the prefix on accept.
    pub prefix_len: usize,
    /// Currently selected index (within filtered list)
    pub selected: usize,
    /// Scroll offset for the visible window
    pub scroll_offset: usize,
}

impl SqlCompletion {
    pub fn new(items: Vec<CompletionItem>, prefix: &str) -> Self {
        Self {
            all_items: items,
            filter: prefix.to_string(),
            prefix_len: prefix.chars().count(),
            selected: 0,
            scroll_offset: 0,
        }
    }

    /// Get the filtered list of items matching the current filter.
    /// Uses fuzzy matching: characters must appear in order but not contiguously.
    /// Results are sorted by match quality (prefix > substring > fuzzy).
    pub fn filtered_items(&self) -> Vec<&CompletionItem> {
        if self.filter.is_empty() {
            return self.all_items.iter().collect();
        }

        let filter_lower = self.filter.to_lowercase();
        let mut scored: Vec<(i32, &CompletionItem)> = self
            .all_items
            .iter()
            .filter_map(|item| {
                let name_lower = item.text.to_lowercase();
                fuzzy_match_score(&name_lower, &filter_lower).map(|score| (score, item))
            })
            .collect();

        // Sort by score descending (higher = better match)
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, item)| item).collect()
    }

    pub fn move_down(&mut self) {
        let count = self.filtered_items().len();
        if count > 0 {
            self.selected = (self.selected + 1).min(count - 1);
            if self.selected >= self.scroll_offset + COMPLETION_MAX_VISIBLE {
                self.scroll_offset = self.selected + 1 - COMPLETION_MAX_VISIBLE;
            }
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    /// Append a character to the filter and reset selection.
    pub fn push_filter(&mut self, ch: char) {
        self.filter.push(ch);
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Remove the last character from the filter.
    pub fn pop_filter(&mut self) {
        self.filter.pop();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn selected_item(&self) -> Option<&CompletionItem> {
        let filtered = self.filtered_items();
        filtered.get(self.selected).copied()
    }
}

/// Fuzzy match a name against a filter pattern.
///
/// Returns a score if the filter characters appear in order within the name.
/// Higher scores indicate better matches:
/// - 100: exact match
/// - 90: prefix match
/// - 80: substring (contiguous) match
/// - 50-79: fuzzy match (bonus for consecutive chars and early matches)
/// - None: no match
fn fuzzy_match_score(name: &str, filter: &str) -> Option<i32> {
    if filter.is_empty() {
        return Some(0);
    }
    if name == filter {
        return Some(100);
    }
    if name.starts_with(filter) {
        return Some(90);
    }
    if name.contains(filter) {
        return Some(80);
    }

    // Fuzzy: each filter char must appear in order
    let mut name_chars = name.chars().peekable();
    let mut score: i32 = 50;
    let mut last_match_pos = 0usize;

    for (fi, fc) in filter.chars().enumerate() {
        let mut found = false;
        let mut pos = last_match_pos;
        for nc in name_chars.by_ref() {
            if nc == fc {
                // Bonus for consecutive matches
                if fi > 0 && pos == last_match_pos {
                    score += 3;
                }
                // Bonus for matching near the start
                if pos < 3 {
                    score += 2;
                }
                last_match_pos = pos + 1;
                found = true;
                break;
            }
            pos += 1;
        }
        if !found {
            return None;
        }
    }

    Some(score)
}

/// Severity level for a SQL diagnostic
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// A pre-execution diagnostic marker in the SQL editor
#[derive(Debug, Clone)]
pub struct SqlDiagnostic {
    /// 0-based line number
    pub line: usize,
    /// 0-based start column (inclusive)
    pub col_start: usize,
    /// 0-based end column (exclusive)
    pub col_end: usize,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: DiagnosticSeverity,
}

/// Cached CSV header schema for a single file.
#[derive(Debug, Clone)]
struct CachedSchema {
    headers: Vec<String>,
    mtime: std::time::SystemTime,
}

/// Cache of CSV headers keyed by file path, invalidated by mtime changes.
/// Avoids re-reading headers from disk on every keystroke (validation) and
/// every Ctrl+N (completions).
#[derive(Debug, Default)]
pub struct SchemaCache {
    cache: std::collections::HashMap<std::path::PathBuf, CachedSchema>,
}

impl SchemaCache {
    /// Return cached headers if the file's mtime hasn't changed, otherwise
    /// re-read from disk and update the cache. Returns `None` on read failure
    /// (failures are not cached).
    pub fn get_headers(&mut self, path: &std::path::Path) -> Option<Vec<String>> {
        let meta = std::fs::metadata(path).ok()?;
        let mtime = meta.modified().ok()?;

        if let Some(cached) = self.cache.get(path) {
            if cached.mtime == mtime {
                return Some(cached.headers.clone());
            }
        }

        // Read headers from CSV
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .ok()?;
        let headers: Vec<String> = reader.headers().ok()?.iter().map(String::from).collect();

        self.cache.insert(
            path.to_path_buf(),
            CachedSchema {
                headers: headers.clone(),
                mtime,
            },
        );

        Some(headers)
    }

    /// Convenience alias for `get_headers` – returns a cloned `Vec<String>`.
    pub fn get_or_read(&mut self, path: &std::path::Path) -> Option<Vec<String>> {
        self.get_headers(path)
    }
}

/// A step in a multi-part query template.
#[derive(Debug, Clone)]
pub enum TemplateStep {
    /// Insert literal text at cursor.
    Text(String),
    /// Prompt the user to pick a table name via completion popup.
    PickTable,
    /// Prompt the user to pick a column from the table aliased as `alias`
    /// in the current query. The alias is resolved at execution time.
    /// Use `"*"` to pick from all referenced tables.
    PickColumn(String),
    /// Insert the same column name that was last picked via `PickColumn`.
    RepeatLastColumn,
    /// Replace the entire editor content with a format string.
    /// `{table}` is replaced with the last picked table name,
    /// `{column}` is replaced with the last picked column name.
    Assemble(String),
}

/// Visual mode selection anchor and type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualSelection {
    /// Starting position of the selection
    pub anchor: (RowIndex, ColIndex),
    /// Current cursor position (end of selection)
    pub cursor: (RowIndex, ColIndex),
    /// Type of visual selection
    pub mode: VisualMode,
}

/// Type of visual selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualMode {
    /// Rectangular block selection
    Block,
    /// Whole row selection
    Line,
    /// Whole column selection
    Column,
}

impl VisualSelection {
    /// Create a new visual selection starting at the given position
    pub fn new(row: RowIndex, col: ColIndex, mode: VisualMode) -> Self {
        Self {
            anchor: (row, col),
            cursor: (row, col),
            mode,
        }
    }

    /// Update the cursor position
    pub fn update_cursor(&mut self, row: RowIndex, col: ColIndex) {
        self.cursor = (row, col);
    }

    /// Get the selection bounds as (start_row, end_row, start_col, end_col)
    /// Returns normalized bounds (start <= end)
    pub fn bounds(&self) -> (RowIndex, RowIndex, ColIndex, ColIndex) {
        let (start_row, end_row) = if self.anchor.0 <= self.cursor.0 {
            (self.anchor.0, self.cursor.0)
        } else {
            (self.cursor.0, self.anchor.0)
        };

        let (start_col, end_col) = if self.anchor.1 <= self.cursor.1 {
            (self.anchor.1, self.cursor.1)
        } else {
            (self.cursor.1, self.anchor.1)
        };

        (start_row, end_row, start_col, end_col)
    }
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

/// Cached in-memory SQLite connection with generation tracking.
/// Keeps loaded tables across query executions so unchanged data isn't reloaded.
pub struct SqliteCache {
    conn: rusqlite::Connection,
    /// Map from file path -> Document generation loaded in that table.
    loaded_generations: HashMap<PathBuf, u64>,
}

impl std::fmt::Debug for SqliteCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteCache")
            .field("loaded_generations", &self.loaded_generations)
            .finish_non_exhaustive()
    }
}

/// Threshold in rows above which SQLite uses a temp file on disk instead of in-memory.
/// This prevents excessive RAM usage for large datasets.
const SQLITE_DISK_THRESHOLD: usize = 10_000_000;

impl SqliteCache {
    /// Create an in-memory SQLite connection (fast, but uses RAM for all data).
    fn new_in_memory() -> Self {
        let conn = rusqlite::Connection::open_in_memory().expect("Failed to open in-memory SQLite");
        conn.execute_batch(
            "PRAGMA journal_mode=OFF;
             PRAGMA synchronous=OFF;
             PRAGMA temp_store=MEMORY;
             PRAGMA cache_size=-64000;",
        )
        .expect("Failed to set SQLite pragmas");
        SqliteCache {
            conn,
            loaded_generations: HashMap::new(),
        }
    }

    /// Create a disk-backed SQLite connection in a temp file.
    /// Uses much less RAM for large datasets at the cost of some I/O.
    fn new_on_disk() -> Self {
        let temp_path =
            std::env::temp_dir().join(format!("lazycsv_sqlite_{}.db", std::process::id()));
        let conn =
            rusqlite::Connection::open(&temp_path).expect("Failed to open disk-backed SQLite");
        conn.execute_batch(
            "PRAGMA journal_mode=OFF;
             PRAGMA synchronous=OFF;
             PRAGMA temp_store=FILE;
             PRAGMA cache_size=-256000;
             PRAGMA page_size=8192;
             PRAGMA mmap_size=1073741824;",
        )
        .expect("Failed to set SQLite pragmas");
        SqliteCache {
            conn,
            loaded_generations: HashMap::new(),
        }
    }

    /// Create a connection appropriate for the dataset size.
    fn for_row_count(row_count: usize) -> Self {
        if row_count > SQLITE_DISK_THRESHOLD {
            Self::new_on_disk()
        } else {
            Self::new_in_memory()
        }
    }

    /// Get reference to loaded generations map.
    pub(crate) fn loaded_generations(&self) -> &HashMap<PathBuf, u64> {
        &self.loaded_generations
    }

    /// Check whether the table for `path` needs to be reloaded.
    pub(crate) fn needs_reload(&self, path: &Path, generation: u64) -> bool {
        match self.loaded_generations.get(path) {
            Some(&cached_gen) => cached_gen != generation,
            None => true,
        }
    }

    /// Drop and reload a single table. Tracks the new generation on success.
    pub(crate) fn reload_table(
        &mut self,
        path: &Path,
        table_name: &str,
        doc: &Document,
        generation: u64,
        cancelled: &AtomicBool,
    ) -> std::result::Result<(), anyhow::Error> {
        // Drop existing table (ignore error if it doesn't exist)
        let _ = self.conn.execute(
            &format!(
                "DROP TABLE IF EXISTS \"{}\"",
                table_name.replace('"', "\"\"")
            ),
            [],
        );
        self.loaded_generations.remove(path);

        crate::query::load_csv_into_sqlite_cancellable(&self.conn, doc, table_name, cancelled)?;
        self.loaded_generations
            .insert(path.to_path_buf(), generation);
        Ok(())
    }

    /// Remove a single table from the cache.
    pub(crate) fn remove_table(&mut self, path: &Path, table_name: &str) {
        let _ = self.conn.execute(
            &format!(
                "DROP TABLE IF EXISTS \"{}\"",
                table_name.replace('"', "\"\"")
            ),
            [],
        );
        self.loaded_generations.remove(path);
    }

    /// Force a table to be reloaded on next use by removing its generation entry.
    pub(crate) fn force_reload_generation(&mut self, path: &Path) {
        self.loaded_generations.remove(path);
    }

    /// Get a reference to the underlying connection.
    pub(crate) fn conn(&self) -> &rusqlite::Connection {
        &self.conn
    }
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

    /// Vim editor for SQL query editing (None when not in SQL editor mode)
    pub sql_vim_editor: Option<crate::vim_editor::VimEditor>,

    /// SQL table-name completion popup state (None when not showing)
    pub sql_completion: Option<SqlCompletion>,

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

    /// Cached SQLite connection for repeated SQL queries
    pub sqlite_cache: Option<SqliteCache>,

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
}

/// File operation being prompted for
#[derive(Debug, Clone, PartialEq)]
pub enum FileOperation {
    Rename(PathBuf), // Original path
    Delete(PathBuf), // Path to delete
    Move(PathBuf),   // Source path
    Copy(PathBuf),   // Source path
    Create,          // New file in current directory
}

impl App {
    /// Resolve file paths from CLI arguments (fast, no file loading).
    /// Returns (file_path, csv_files, current_file_index, file_config).
    pub fn resolve_files(
        cli_args: &crate::cli::CliArgs,
    ) -> Result<(PathBuf, Vec<PathBuf>, usize, crate::session::FileConfig)> {
        let path = cli_args.file_path().unwrap_or_else(|| PathBuf::from("."));

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

        Ok((file_path, csv_files, current_file_index, file_config))
    }

    /// Load a CSV file and create an App instance.
    /// Call after `resolve_files` to actually load the document.
    pub fn load_file(
        file_path: &Path,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        file_config: crate::session::FileConfig,
        cli_args: &crate::cli::CliArgs,
    ) -> Result<Self> {
        let csv_data = crate::csv::Document::from_file(
            file_path,
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        )
        .context(messages::failed_to_load_csv(file_path))?;

        let mut app = Self::new(csv_data, csv_files, current_file_index, file_config);
        app.session.record_file_mtime(file_path);
        Ok(app)
    }

    /// Create a new `App` instance from CLI arguments.
    /// This function handles file scanning, initial data loading, and App creation.
    pub fn from_cli(cli_args: crate::cli::CliArgs) -> Result<Self> {
        let (file_path, csv_files, current_file_index, file_config) =
            Self::resolve_files(&cli_args)?;
        Self::load_file(
            &file_path,
            csv_files,
            current_file_index,
            file_config,
            &cli_args,
        )
    }

    /// Create new App from loaded CSV data, file list, and file configuration
    pub fn new(
        csv_data: Document,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        file_config: crate::session::FileConfig,
    ) -> Self {
        // Create session
        let session = Session::new(csv_files, current_file_index, file_config);

        // Initialize view state - start at row 0 (displays as row 1)
        let mut view_state = ViewState::default();
        view_state.table_state.select(Some(0));

        // Create input state
        let input_state = InputState::new();

        let mut app = Self {
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
            sql_vim_editor: None,
            sql_completion: None,
            sql_diagnostics: Vec::new(),
            sql_template_steps: Vec::new(),
            sql_template_last_column: None,
            sql_template_last_table: None,
            magnifier_state: None,
            should_quit: false,
            sqlite_cache: None,
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

        app
    }

    /// Handle keyboard input events
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<InputResult> {
        crate::input::handle_key(self, key)
    }

    /// Get current selected row index (for status display)
    pub fn selected_row(&self) -> Option<RowIndex> {
        self.view_state.table_state.selected().map(RowIndex::new)
    }

    /// Get current file path
    pub fn current_file(&self) -> &PathBuf {
        self.session.current_file()
    }

    /// Reload CSV data from current file
    pub fn reload_current_file(&mut self) -> Result<()> {
        let file_path = self.current_file().clone();
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
        let file_path = self.current_file().clone();
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
        let file_path = self.current_file().clone();
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
        // Record new mtime so we don't treat our own save as external modification
        self.session.record_file_mtime(&file_path);

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
        let dirty_files: Vec<PathBuf> = self.session.dirty_files();
        for file_path in dirty_files {
            // Skip current file (already saved above)
            if &file_path == self.current_file() {
                continue;
            }

            // Get cached document
            if let Some(doc) = self.session.cached_document(&file_path) {
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

    // ============================================================================
    // Formula Methods
    // ============================================================================

    /// Commit a cell value, detecting and storing formulas.
    /// If `content` starts with '=', tries to parse it as a formula.
    /// The document always stores the computed value; the formula is kept separately.
    pub fn commit_cell_value(
        &mut self,
        row: crate::domain::position::RowIndex,
        col: crate::domain::position::ColIndex,
        content: String,
    ) {
        if let Some(formula) = crate::formula::parse_formula(&content) {
            // Evaluate using document cell values
            let computed = formula.evaluate(&|r, c| self.document.storage.get_cell(r, c));
            self.formula_store
                .set(row.get(), col.get(), content, formula);
            self.document.set_cell(row, col, computed);
        } else {
            // Not a formula — remove any existing formula and store raw value
            self.formula_store.remove(row.get(), col.get());
            self.document.set_cell(row, col, content);
        }

        self.document.is_dirty = true;
        let file_path = self.current_file().clone();
        self.session.mark_dirty(&file_path);
        self.last_edit_position = Some((row, col));

        // Re-evaluate any formulas that reference this cell
        self.re_evaluate_formulas_referencing(row.get(), col.get());
    }

    /// Re-evaluate all formulas that reference the given cell.
    fn re_evaluate_formulas_referencing(&mut self, changed_row: usize, changed_col: usize) {
        let dependents = self
            .formula_store
            .cells_referencing(changed_row, changed_col);
        for (r, c) in dependents {
            if let Some(formula) = self.formula_store.get_formula(r, c).cloned() {
                let computed =
                    formula.evaluate(&|row, col| self.document.storage.get_cell(row, col));
                self.document.storage.set_cell(r, c, computed);
            }
        }
    }

    /// Get the display value for a cell — the formula text if it has a formula, otherwise the raw value.
    /// Used when entering edit mode or showing in the formula bar.
    pub fn cell_formula_or_value(
        &self,
        row: crate::domain::position::RowIndex,
        col: crate::domain::position::ColIndex,
    ) -> String {
        if let Some(raw) = self.formula_store.get_raw(row.get(), col.get()) {
            raw.to_string()
        } else {
            self.document.cell(row, col).to_string()
        }
    }

    // ============================================================================
    // Magnifier Mode Methods (Phase 4)
    // ============================================================================

    /// Open magnifier for the current cell
    pub fn open_magnifier(&mut self) {
        let row = self
            .view_state
            .table_state
            .selected()
            .map(RowIndex::new)
            .unwrap_or(RowIndex::new(0));
        let col = self.view_state.selected_column;

        // Get cell content — show formula text if this cell has a formula
        let cell_content = self.cell_formula_or_value(row, col);

        // Create magnifier state
        self.magnifier_state = Some(crate::magnifier::MagnifierState::new(
            cell_content,
            (row, col),
        ));

        // Switch to magnifier mode
        self.mode = Mode::Magnifier;
    }

    /// Save magnifier content to cell (keep magnifier open)
    pub fn save_magnifier_content(&mut self) {
        if let Some(mag) = &self.magnifier_state {
            let content = mag.content();
            let (row, col) = mag.cell_position();

            // Use commit_cell_value to handle formula detection
            self.commit_cell_value(row, col, content.clone());

            // Update magnifier's original content so it's no longer dirty
            if let Some(mag) = &mut self.magnifier_state {
                mag.mark_clean_with_content(content);
            }
        }
    }

    /// Save magnifier content to cell and close magnifier
    pub fn save_and_close_magnifier(&mut self) {
        if let Some(mag) = self.magnifier_state.take() {
            let content = mag.content();
            let (row, col) = mag.cell_position();

            // Use commit_cell_value to handle formula detection
            self.commit_cell_value(row, col, content);

            // Return to normal mode
            self.mode = Mode::Normal;
        }
    }

    /// Close magnifier without saving changes
    pub fn close_magnifier_discard(&mut self) {
        self.magnifier_state = None;
        self.mode = Mode::Normal;
    }

    /// Check if magnifier has unsaved changes
    pub fn magnifier_is_dirty(&self) -> bool {
        self.magnifier_state
            .as_ref()
            .map(|m| m.is_dirty())
            .unwrap_or(false)
    }

    /// Get mutable reference to magnifier state (for input handling)
    pub fn magnifier_state_mut(&mut self) -> Option<&mut crate::magnifier::MagnifierState> {
        self.magnifier_state.as_mut()
    }

    /// Invalidate the SQLite cache entry for a specific file path.
    /// Called after file reloads from disk to force re-import on next query.
    pub fn invalidate_sqlite_cache_for(&mut self, path: &Path) {
        if let Some(cache) = &mut self.sqlite_cache {
            let table_name = crate::query::table_name_from_path(path);
            cache.remove_table(path, &table_name);
        }
    }

    /// Drop the entire SQLite cache (e.g. when session structure changes drastically).
    pub fn invalidate_sqlite_cache(&mut self) {
        self.sqlite_cache = None;
    }

    /// Check if the current file has been modified externally.
    /// Sets `external_modification_pending` and a status message if so.
    /// Returns true if a modification was detected (triggers redraw).
    pub fn check_current_file_modification(&mut self) -> bool {
        // Don't re-check if we're already prompting
        if self.external_modification_pending {
            return false;
        }
        let path = self.current_file().clone();
        // Skip query output files (they don't live on disk in the normal sense)
        if self.session.is_query_output(&path) {
            return false;
        }
        if self.session.check_file_modified(&path) {
            self.external_modification_pending = true;
            let msg = if self.document.is_dirty {
                "File modified externally (unsaved changes). Press 'r' to reload, Esc to ignore"
            } else {
                "File modified externally. Press 'r' to reload, Esc to ignore"
            };
            self.status_message = Some(StatusMessage::new_persistent(msg.to_string()));
            true
        } else {
            false
        }
    }

    /// Load a CSV or XLSX file with cancellation support.
    /// Same as `load_file` but uses `Document::from_file_cancellable`.
    pub fn load_file_cancellable(
        file_path: &Path,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        file_config: crate::session::FileConfig,
        cli_args: &crate::cli::CliArgs,
        cancelled: &AtomicBool,
        sheet_name: Option<&str>,
    ) -> Result<Self> {
        let csv_data = crate::csv::Document::from_file_cancellable_with_sheet(
            file_path,
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
            cancelled,
            sheet_name,
        )
        .context(messages::failed_to_load_csv(file_path))?;

        let mut app = Self::new(csv_data, csv_files, current_file_index, file_config);
        app.session.record_file_mtime(file_path);
        Ok(app)
    }

    /// Reload CSV data from current file with cancellation support.
    /// Returns Ok(true) if loaded successfully, Ok(false) if cancelled (keeps existing document).
    pub fn reload_current_file_cancellable(&mut self, cancelled: &AtomicBool) -> Result<bool> {
        let file_path = self.current_file().clone();

        // Check for cached document first (e.g. query output files that don't exist on disk)
        if let Some(cached) = self.session.cached_document(&file_path) {
            // Drop old rows on background thread to avoid blocking UI
            let old_storage = self.document.take_storage();
            std::thread::spawn(move || drop(old_storage));
            self.document = cached.clone();
            self.view_state = ViewState::default();
            // Start at row 0 (displays as row 1)
            self.view_state.table_state.select(Some(0));
            return Ok(true);
        }

        // Query output files don't exist on disk — if not cached, keep current document
        if self.session.is_query_output(&file_path) {
            return Ok(true);
        }

        // Not cached — reload from disk
        let config = self.session.config();
        match crate::csv::Document::from_file_cancellable(
            &file_path,
            config.delimiter,
            config.no_headers,
            config.encoding.clone(),
            cancelled,
        ) {
            Ok(doc) => {
                // Drop old rows on background thread
                let old_storage = self.document.take_storage();
                std::thread::spawn(move || drop(old_storage));
                self.document = doc;
                // Invalidate SQLite cache for this file (generation reset to 0)
                self.invalidate_sqlite_cache_for(&file_path);
                // Record new mtime so we don't re-prompt for this version
                self.session.record_file_mtime(&file_path);
                self.view_state = ViewState::default();
                // Start at row 0 (displays as row 1)
                self.view_state.table_state.select(Some(0));
                Ok(true)
            }
            Err(e) => {
                if e.downcast_ref::<cancel::CancelledError>().is_some() {
                    Ok(false)
                } else {
                    Err(e).context(messages::failed_to_reload_file(&file_path))
                }
            }
        }
    }

    /// Execute a SQL query with cancellation support.
    /// Returns (Some(doc), false) on success, (None, true) if cancelled,
    /// (None, false) on query error.
    ///
    /// Uses a cached SQLite connection so that unchanged documents are not
    /// re-imported on subsequent queries.
    ///
    /// `output_name` is the filename to assign to the result document.
    pub fn execute_sql_query_cancellable(
        &mut self,
        query: &str,
        output_name: &str,
        cancelled: &AtomicBool,
        on_progress: &mut dyn FnMut(&str),
    ) -> (Option<Document>, bool) {
        // Strip .csv/.tsv/.txt extensions from table references so users can
        // write "SELECT * FROM myfile.csv" and have it resolve correctly.
        let query = crate::query::strip_csv_extensions(query);
        let query = query.as_str();

        // Take the cache out of self for independent borrowing
        let mut cache = self
            .sqlite_cache
            .take()
            .unwrap_or_else(|| SqliteCache::for_row_count(self.document.row_count()));

        // Clean up stale tables
        on_progress("Preparing database...");
        if sql_execution::cleanup_stale_tables(&mut cache, self.session.files(), cancelled) {
            self.sqlite_cache = Some(cache);
            return (None, true);
        }

        // Load only session files referenced by the query
        let all_files = self.session.files().to_vec();
        let referenced = crate::query::files_referenced_by_query(query, &all_files);
        let files: Vec<std::path::PathBuf> = referenced.into_iter().cloned().collect();
        let total_files = files.len();
        for (i, file_path) in files.into_iter().enumerate() {
            let file_label = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            if total_files > 1 {
                on_progress(&format!(
                    "Loading {} into database ({}/{})...",
                    file_label,
                    i + 1,
                    total_files
                ));
            } else {
                on_progress(&format!("Loading {} into database...", file_label));
            }
            let config = self.session.config();
            let file_config = sql_execution::FileLoadConfig {
                delimiter: config.delimiter,
                no_headers: config.no_headers,
                encoding: config.encoding.clone(),
            };
            let cancelled_flag = sql_execution::load_session_file(
                &mut cache,
                &file_path,
                &self.document,
                || self.session.cached_document(&file_path),
                file_config,
                cancelled,
            );
            if cancelled_flag {
                self.sqlite_cache = Some(cache);
                return (None, true);
            }
        }

        // Execute query and convert result
        on_progress("Running query...");
        let (result_doc, cancelled_flag, error_msg) =
            sql_execution::execute_and_convert_query(&mut cache, query, output_name, cancelled);

        self.sqlite_cache = Some(cache);

        if let Some(err) = error_msg {
            self.sql_error = Some(err);
            return (None, false);
        }

        if cancelled_flag {
            return (None, true);
        }

        self.sql_error = None;
        self.mode = Mode::Normal;
        (result_doc, false)
    }

    /// Execute a SQL DML statement (INSERT, UPDATE, DELETE, ALTER) against the current document.
    /// Loads the current document into SQLite, executes the DML, then SELECT * to get updated data.
    /// Replaces the current document in-place and marks it dirty.
    /// Returns (success, was_cancelled).
    pub fn execute_sql_dml_cancellable(
        &mut self,
        query: &str,
        cancelled: &AtomicBool,
        on_progress: &mut dyn FnMut(&str),
    ) -> (bool, bool) {
        let query = crate::query::strip_csv_extensions(query);
        let query = query.as_str();

        let mut cache = self
            .sqlite_cache
            .take()
            .unwrap_or_else(|| SqliteCache::for_row_count(self.document.row_count()));

        // Derive table name from the document's filename (not the session file path,
        // which may point to a different file after switching).
        let doc_table_name = self
            .document
            .filename
            .strip_suffix(".csv")
            .or_else(|| self.document.filename.strip_suffix(".tsv"))
            .or_else(|| self.document.filename.strip_suffix(".txt"))
            .unwrap_or(&self.document.filename)
            .to_string();
        let file_path = self.current_file().clone();

        // Force reload current document into SQLite using the document's own table name
        on_progress("Syncing data to database...");
        cache.force_reload_generation(&file_path);
        if self.document.row_count() > 0 && self.document.column_count() > 0 {
            // Drop any existing table with this name and recreate from current data
            let _ = cache.conn().execute(
                &format!(
                    "DROP TABLE IF EXISTS \"{}\"",
                    doc_table_name.replace('"', "\"\"")
                ),
                [],
            );
            match crate::query::load_csv_into_sqlite_cancellable(
                cache.conn(),
                &self.document,
                &doc_table_name,
                cancelled,
            ) {
                Ok(()) => {}
                Err(e) => {
                    if e.downcast_ref::<cancel::CancelledError>().is_some() {
                        self.sqlite_cache = Some(cache);
                        return (false, true);
                    }
                    self.sql_error = Some(format!("Failed to load data: {}", e));
                    self.sqlite_cache = Some(cache);
                    return (false, false);
                }
            }
        }

        // Execute the DML statement
        on_progress("Executing DML...");
        match cache.conn().execute(query, []) {
            Ok(_) => {}
            Err(e) => {
                self.sql_error = Some(format!("SQL error: {}", e));
                self.sqlite_cache = Some(cache);
                return (false, false);
            }
        }

        // SELECT * from the table to get updated data
        on_progress("Reading updated data...");
        let select_query = format!("SELECT * FROM \"{}\"", doc_table_name.replace('"', "\"\""));
        let (result_doc, select_cancelled, error_msg) = sql_execution::execute_and_convert_query(
            &mut cache,
            &select_query,
            &self.document.filename,
            cancelled,
        );

        self.sqlite_cache = Some(cache);

        if let Some(err) = error_msg {
            self.sql_error = Some(err);
            return (false, false);
        }
        if select_cancelled {
            return (false, true);
        }

        if let Some(doc) = result_doc {
            // Replace current document data in-place
            self.document.storage = doc.storage;
            self.document.is_dirty = true;
            self.document.generation += 1;
            self.document.xlsx_formulas = vec![];

            // Reset view state
            self.view_state.table_state.select(Some(1));
            self.view_state.column_scroll_offset = 0;
            self.view_state.selected_column = crate::domain::position::ColIndex::new(0);

            self.sql_error = None;
            self.mode = Mode::Normal;
            (true, false)
        } else {
            self.sql_error = Some("DML succeeded but failed to read back results".to_string());
            (false, false)
        }
    }

    /// Generate a unique output filename for SQL query results
    pub fn generate_output_filename(&self) -> String {
        let existing: std::collections::HashSet<String> = self
            .session
            .files()
            .iter()
            .filter_map(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        let base = "result.csv".to_string();
        if !existing.contains(&base) {
            return base;
        }
        let mut i = 1;
        loop {
            let name = format!("result{}.csv", i);
            if !existing.contains(&name) {
                return name;
            }
            i += 1;
        }
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

        // Always starts at row 0 (displays as row 1)
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
        assert!(!app.should_quit);
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_navigation_down() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Starts at row 0, move down to row 1
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

        // Move down to row 2
        app.handle_key(key_event(KeyCode::Down)).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(2)));

        // Move down to row 3 (last row)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

        // Try to go beyond last row - should stay at last row
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_navigation_up() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.view_state.table_state.select(Some(2));

        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

        // Can navigate to row 0
        app.handle_key(key_event(KeyCode::Up)).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

        // Try to go before first row - should stay at row 0
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
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
        assert_eq!(app.selected_row(), Some(RowIndex::new(3))); // Last row

        // gg - Go to first row (row 0)
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(0))); // First row
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

        // Space+q opens SQL editor
        app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
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
    fn test_navigation_blocked_when_help_shown() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.view_state.help_overlay_visible = true;
        let initial_row = app.selected_row();
        let initial_col = app.view_state.selected_column;

        // Try navigation with help shown
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), initial_row);

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

        assert_eq!(app.current_file(), &csv_files[0]);
    }

    // ========== v0.1.2: Multi-Key Command Tests ==========

    #[test]
    fn test_multi_key_gg_goes_to_first_row() {
        // Setup: Create app (starts at row 0), move to last row
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to last row (row 3)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

        // Execute gg command: press 'g' then 'g'
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Should go to row 0 (first row)
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
    }

    #[test]
    fn test_multi_key_g_goes_to_last_row() {
        // Setup: Create app (starts at row 0)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

        // Press G to go to last row
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at last row (row 3)
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_multi_key_2g_goes_to_row_2() {
        // Setup: Create app (starts at row 0)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

        // Press '2' to start count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'G' to execute go to row 2
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // 2G should go to absolute row 2
        assert_eq!(app.selected_row(), Some(RowIndex::new(2)));
    }

    // ========== v0.1.2: Count Prefix Tests ==========

    #[test]
    fn test_count_prefix_2j_moves_down_2_rows() {
        // Setup: Create app (starts at row 0)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

        // Press '2' to set count prefix
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Press 'j' to move down 2 rows
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should be at row 2 (moved down 2 rows from row 0)
        assert_eq!(app.selected_row(), Some(RowIndex::new(2)));
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
        // Setup: Create app (starts at row 0)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Set count prefix '2'
        app.handle_key(key_event(KeyCode::Char('2'))).unwrap();
        // Use it with 'j' to move down 2 rows
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(2)));

        // Now press 'j' again without count - should only move 1 row
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

        // Move back to row 0 (gg goes to row 0)
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

        // Press 'j' without count - should move only 1 row (count was cleared)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(1))); // Only moved 1 row, not 2
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
    }

    #[test]
    fn test_state_after_help_toggle() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        let initial_row = app.selected_row();

        // Open help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(app.view_state.help_overlay_visible);

        // Navigation should be blocked when help is shown
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), initial_row); // Should not move

        // Close help
        app.handle_key(key_event(KeyCode::Char('?'))).unwrap();
        assert!(!app.view_state.help_overlay_visible);

        // Now navigation should work
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(
            app.selected_row(),
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
    fn test_special_keys_ignored_in_normal_mode() {
        // Setup: Create app
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        let initial_row = app.selected_row();
        let initial_col = app.view_state.selected_column;

        // Press various special keys that should be ignored
        app.handle_key(key_event(KeyCode::F(1))).unwrap();
        app.handle_key(key_event(KeyCode::Insert)).unwrap();
        app.handle_key(key_event(KeyCode::Delete)).unwrap();

        // State should remain unchanged
        assert_eq!(app.selected_row(), initial_row);
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

        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

        // Press '3' then 'G' to go to absolute row 3
        app.handle_key(key_event(KeyCode::Char('3'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Should be at row 3
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
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
        // Setup: Create app (starts at row 0)
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Complex navigation sequence
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 1
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap(); // Right to col 1
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap(); // Down to row 2
        app.handle_key(key_event(KeyCode::Char('h'))).unwrap(); // Left to col 0
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap(); // Up to row 1

        // Should be at row 1, col 0
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
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

        // Start at row 5 (move 5 times from row 0)
        for _ in 0..5 {
            app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        }
        assert_eq!(app.selected_row(), Some(RowIndex::new(5)));

        // Page up should move up (typically ~20 rows, but we only have 10)
        app.handle_key(key_event(KeyCode::PageUp)).unwrap();
        // Should be at row 5 or lower
        assert!(app.selected_row().unwrap().get() <= 5);

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
        // Setup: Create app, navigate to row 2, column 2
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("file1.csv"), PathBuf::from("file2.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Navigate to row 2, column 2 (move 2 rows down from row 0, 2 columns right)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('l'))).unwrap();

        assert_eq!(app.selected_row(), Some(RowIndex::new(2)));
        assert_eq!(app.view_state.selected_column, ColIndex::new(2));

        // Note: In real app, file switch would reload and reset position
        // This test verifies current behavior
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

        // Execute gg - should go to row 0 (first row)
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('g'))).unwrap();

        // Should be at row 0
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
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
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
    }

    #[test]
    fn test_count_prefix_exceeds_row_bounds() {
        let csv_data = create_test_csv_data(); // Has 3 rows
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());
        let initial_row = app.selected_row();

        // Try to jump to row 9999 with 9999G
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('9'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('G'))).unwrap();

        // Position should not change when out of bounds
        assert_eq!(app.selected_row(), initial_row);
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
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));

        // Try to move down from last row
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();

        // Should stay at last row
        assert_eq!(app.selected_row(), Some(RowIndex::new(3)));
    }

    #[test]
    fn test_navigation_k_on_first_row() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Should start at row 0 (first row)
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));

        // Try to move up from first row - should stay at row 0
        app.handle_key(key_event(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(0)));
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

        // Should have moved to first column, then down one row (from row 0 to row 1)
        assert_eq!(app.view_state.selected_column, ColIndex::new(0));
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));
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
        assert!(app.selected_row().is_some());
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

        let initial_row = app.selected_row();

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
        assert_eq!(app.selected_row(), initial_row);
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
        assert_eq!(app.selected_row(), Some(RowIndex::new(3))); // Last row in test data
    }

    // ===== Z-Command Integration Tests (Viewport Positioning) =====

    #[test]
    fn test_z_command_top_viewport() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to next row (row 1)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

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

        // Move to next row (row 1)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

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

        // Move to next row (row 1)
        app.handle_key(key_event(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.selected_row(), Some(RowIndex::new(1)));

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
        assert!(
            msg.contains("test.csv"),
            "Expected filename in status, got: {}",
            msg
        );
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
        assert_eq!(app.current_file(), &PathBuf::from("newname.csv"));
        assert!(app.document.is_dirty);

        let msg = app.status_message.as_ref().unwrap().as_str();
        assert!(
            msg.contains("newname.csv"),
            "Expected rename confirmation, got: {}",
            msg
        );
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
        let csv_files = vec![PathBuf::from("result.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Mark as query output (simulating what happens after SQL execution)
        let path = app.current_file().clone();
        app.session.mark_query_output(&path);

        // :f results.csv
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('f'))).unwrap();
        app.handle_key(key_event(KeyCode::Char(' '))).unwrap();
        for c in "renamed_result.csv".chars() {
            app.handle_key(key_event(KeyCode::Char(c))).unwrap();
        }
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        // Query output status should follow the renamed file
        let new_path = app.current_file().clone();
        assert_eq!(new_path, PathBuf::from("renamed_result.csv"));
        assert!(app.session.is_query_output(&new_path));
        assert!(!app.session.is_query_output(&PathBuf::from("result.csv")));
    }

    // ============================================================================
    // Phase 4: Magnifier Integration Tests
    // ============================================================================

    #[test]
    fn test_open_magnifier() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Position cursor at a cell with content
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(0);

        app.open_magnifier();

        assert!(app.magnifier_state.is_some());
        assert_eq!(app.mode, Mode::Magnifier);

        let mag = app.magnifier_state.as_ref().unwrap();
        assert_eq!(mag.cell_position(), (RowIndex::new(1), ColIndex::new(0)));
    }

    #[test]
    fn test_save_and_close_magnifier() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Open magnifier
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(0);
        app.open_magnifier();

        // Edit content
        if let Some(mag) = app.magnifier_state.as_mut() {
            mag.enter_insert_mode();
            mag.insert_char('X');
        }

        // Save and close
        app.save_and_close_magnifier();

        assert!(app.magnifier_state.is_none());
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.document.is_dirty);

        // Check that content was updated
        let cell = app.document.cell(RowIndex::new(1), ColIndex::new(0));
        assert!(cell.starts_with('X'));
    }

    #[test]
    fn test_close_magnifier_discard() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Open magnifier and edit
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(0);
        app.open_magnifier();

        let original_content = app
            .document
            .cell(RowIndex::new(1), ColIndex::new(0))
            .to_string();

        if let Some(mag) = app.magnifier_state.as_mut() {
            mag.enter_insert_mode();
            mag.insert_char('X');
        }

        // Discard changes
        app.close_magnifier_discard();

        assert!(app.magnifier_state.is_none());
        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.document.is_dirty);

        // Check that content was NOT updated
        let cell = app.document.cell(RowIndex::new(1), ColIndex::new(0));
        assert_eq!(cell, original_content);
    }

    #[test]
    fn test_magnifier_is_dirty() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Initially not dirty
        assert!(!app.magnifier_is_dirty());

        // Open magnifier
        app.open_magnifier();
        assert!(!app.magnifier_is_dirty());

        // Edit content
        if let Some(mag) = app.magnifier_state.as_mut() {
            mag.enter_insert_mode();
            mag.insert_char('X');
        }

        // Now it should be dirty
        assert!(app.magnifier_is_dirty());
    }

    #[test]
    fn test_magnifier_with_empty_cell() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Position at an empty cell (beyond data)
        app.view_state.table_state.select(Some(10));
        app.view_state.selected_column = ColIndex::new(0);

        app.open_magnifier();

        assert!(app.magnifier_state.is_some());
        let mag = app.magnifier_state.as_ref().unwrap();
        assert_eq!(mag.content(), "");
    }

    #[test]
    fn test_magnifier_multiline_content() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        app.open_magnifier();

        // Clear existing content and add multiline content
        if let Some(mag) = app.magnifier_state.as_mut() {
            // Delete existing content
            mag.delete_line();
            // Add new multiline content
            mag.enter_insert_mode();
            mag.insert_char('L');
            mag.insert_char('1');
            mag.newline();
            mag.insert_char('L');
            mag.insert_char('2');
        }

        app.save_and_close_magnifier();

        // Check that multiline content was saved (app starts at row 0)
        let cell = app.document.cell(RowIndex::new(0), ColIndex::new(0));
        assert_eq!(cell, "L1\nL2");
    }

    #[test]
    fn test_search_slash_enters_search_mode() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        assert_eq!(app.mode, Mode::Normal);
        app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
        assert_eq!(app.mode, Mode::Search);
    }

    #[test]
    fn test_search_esc_returns_to_normal() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Enter search mode
        app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
        assert_eq!(app.mode, Mode::Search);

        // Type something
        app.handle_key(key_event(KeyCode::Char('t'))).unwrap();
        assert_eq!(app.mode, Mode::Search);
        assert_eq!(app.search_buffer, "t");

        // Esc should return to Normal
        app.handle_key(key_event(KeyCode::Esc)).unwrap();
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_search_enter_executes_search() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Enter search, type "5", press Enter
        app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        assert_eq!(app.mode, Mode::Normal);
        assert!(app.search_state.is_some());
        let state = app.search_state.as_ref().unwrap();
        assert_eq!(state.pattern, "5");
        assert_eq!(state.match_count(), 1);
    }

    #[test]
    fn test_search_esc_in_normal_mode_clears_search() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Perform a search
        app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
        app.handle_key(key_event(KeyCode::Enter)).unwrap();
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.search_state.is_some());

        // Esc in Normal mode should clear search highlighting
        app.handle_key(key_event(KeyCode::Esc)).unwrap();
        assert!(app.search_state.is_none());
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_search_asterisk_searches_current_cell() {
        let csv_data = Document::new(
            vec!["Name".to_string(), "City".to_string()],
            vec![
                vec!["Alice".to_string(), "Portland".to_string()],
                vec!["Bob".to_string(), "Boston".to_string()],
                vec!["Charlie".to_string(), "Portland".to_string()],
            ],
            "test.csv".to_string(),
        );
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Move to row 1, col 1 ("Portland")
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(1);

        // Press * to search for current cell content
        app.handle_key(key_event(KeyCode::Char('*'))).unwrap();

        assert_eq!(app.mode, Mode::Normal);
        assert!(app.search_state.is_some());
        let state = app.search_state.as_ref().unwrap();
        assert_eq!(state.pattern, "Portland");
        assert_eq!(state.match_count(), 2);
    }

    #[test]
    fn test_search_noh_clears_search() {
        let csv_data = create_test_csv_data();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

        // Perform a search
        app.handle_key(key_event(KeyCode::Char('/'))).unwrap();
        app.handle_key(key_event(KeyCode::Char('5'))).unwrap();
        app.handle_key(key_event(KeyCode::Enter)).unwrap();
        assert!(app.search_state.is_some());

        // Execute :noh
        app.handle_key(key_event(KeyCode::Char(':'))).unwrap();
        for c in "noh".chars() {
            app.handle_key(key_event(KeyCode::Char(c))).unwrap();
        }
        app.handle_key(key_event(KeyCode::Enter)).unwrap();

        assert!(app.search_state.is_none());
    }
}
