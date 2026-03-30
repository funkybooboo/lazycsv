//! Multi-file session management and CSV configuration.
//!
//! This module handles file switching between multiple CSV files and
//! maintains the configuration settings for parsing CSV files.

use crate::column::metadata::ColumnType;
use crate::Document;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Configuration for CSV file parsing
#[derive(Debug, Clone)]
pub struct FileConfig {
    /// Custom delimiter (None = auto-detect, usually comma)
    pub delimiter: Option<u8>,

    /// Whether to treat first row as data (not headers)
    pub no_headers: bool,

    /// Character encoding for file loading
    pub encoding: Option<String>,
}

impl FileConfig {
    /// Create a new FileConfig with default settings
    pub fn new() -> Self {
        Self {
            delimiter: None,
            no_headers: false,
            encoding: None,
        }
    }

    /// Create a FileConfig with custom settings
    pub fn with_options(delimiter: Option<u8>, no_headers: bool, encoding: Option<String>) -> Self {
        Self {
            delimiter,
            no_headers,
            encoding,
        }
    }
}

impl Default for FileConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages multi-file session state
#[derive(Debug)]
pub struct Session {
    /// List of CSV files available in the session
    files: Vec<PathBuf>,

    /// Index of the currently active file
    active_file_index: usize,

    /// Configuration for CSV parsing
    config: FileConfig,

    /// Per-file delimiter settings
    delimiters: HashMap<PathBuf, char>,

    /// Set of dirty (modified) files
    dirty_files: HashSet<PathBuf>,

    /// Cache of dirty documents (avoids reloading from disk when switching files)
    document_cache: HashMap<PathBuf, Document>,

    /// Files that are SQL query output sheets (should be reused on next query)
    query_output_files: HashSet<PathBuf>,

    /// Last-known modification times for files (used to detect external changes)
    file_mtimes: HashMap<PathBuf, SystemTime>,

    /// Per-file manual column widths (sparse: only stores explicitly set widths).
    /// Key is column index, value is width in characters.
    column_widths: HashMap<PathBuf, HashMap<usize, u16>>,

    /// Per-file undo/redo history (preserved across file switches)
    history_cache: HashMap<PathBuf, crate::history::History>,

    /// Per-file frozen (pinned) column indices, kept sorted.
    frozen_columns: HashMap<PathBuf, Vec<usize>>,

    /// Per-file frozen (pinned) row indices, kept sorted.
    frozen_rows: HashMap<PathBuf, Vec<usize>>,

    /// Per-file column type annotations.
    column_types: HashMap<PathBuf, HashMap<usize, ColumnType>>,
}

impl Session {
    /// Create a new session
    pub fn new(files: Vec<PathBuf>, active_file_index: usize, config: FileConfig) -> Self {
        Self {
            files,
            active_file_index,
            config,
            delimiters: HashMap::new(),
            dirty_files: HashSet::new(),
            document_cache: HashMap::new(),
            query_output_files: HashSet::new(),
            file_mtimes: HashMap::new(),
            column_widths: HashMap::new(),
            history_cache: HashMap::new(),
            frozen_columns: HashMap::new(),
            frozen_rows: HashMap::new(),
            column_types: HashMap::new(),
        }
    }

    /// Get the currently active file path
    pub fn current_file(&self) -> &PathBuf {
        &self.files[self.active_file_index]
    }

    /// Get the current file index
    pub fn active_file_index(&self) -> usize {
        self.active_file_index
    }

    /// Get the total number of files in the session
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Get a reference to all files
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Get the file configuration
    pub fn config(&self) -> &FileConfig {
        &self.config
    }

    /// Switch to the next file in the list (wraps around)
    /// Returns true if the file changed, false otherwise
    pub fn next_file(&mut self) -> bool {
        if self.files.len() <= 1 {
            return false;
        }

        self.active_file_index = (self.active_file_index + 1) % self.files.len();
        true
    }

    /// Switch to the previous file in the list (wraps around)
    /// Returns true if the file changed, false otherwise
    pub fn prev_file(&mut self) -> bool {
        if self.files.len() <= 1 {
            return false;
        }

        if self.active_file_index == 0 {
            self.active_file_index = self.files.len() - 1;
        } else {
            self.active_file_index -= 1;
        }
        true
    }

    /// Check if there are multiple files in the session
    pub fn has_multiple_files(&self) -> bool {
        self.files.len() > 1
    }

    /// Get manual column width for a specific column of the current file.
    /// Returns None if no manual width is set (use auto-sizing).
    pub fn column_width(&self, col_index: usize) -> Option<u16> {
        self.column_widths
            .get(&self.files[self.active_file_index])
            .and_then(|widths| widths.get(&col_index))
            .copied()
    }

    /// Set manual column width for a specific column of the current file.
    pub fn set_column_width(&mut self, col_index: usize, width: u16) {
        let file = self.files[self.active_file_index].clone();
        self.column_widths
            .entry(file)
            .or_default()
            .insert(col_index, width);
    }

    /// Clear manual column width for a specific column (revert to auto-sizing).
    pub fn clear_column_width(&mut self, col_index: usize) {
        let file = &self.files[self.active_file_index];
        if let Some(widths) = self.column_widths.get_mut(file) {
            widths.remove(&col_index);
        }
    }

    /// Clear all manual column widths for the current file.
    pub fn clear_all_column_widths(&mut self) {
        let file = &self.files[self.active_file_index];
        self.column_widths.remove(file);
    }

    // ── Frozen columns ─────────────────────────────────────────

    /// Get frozen column indices for the current file (sorted).
    pub fn frozen_columns(&self) -> &[usize] {
        self.frozen_columns
            .get(&self.files[self.active_file_index])
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Freeze (pin) the given columns for the current file.
    pub fn freeze_columns(&mut self, mut col_indices: Vec<usize>) {
        col_indices.sort_unstable();
        col_indices.dedup();
        let file = self.files[self.active_file_index].clone();
        self.frozen_columns.insert(file, col_indices);
    }

    /// Unfreeze all columns for the current file.
    pub fn unfreeze_columns(&mut self) {
        let file = &self.files[self.active_file_index];
        self.frozen_columns.remove(file);
    }

    // ── Frozen rows ───────────────────────────────────────────

    /// Get frozen row indices for the current file (sorted).
    pub fn frozen_rows(&self) -> &[usize] {
        self.frozen_rows
            .get(&self.files[self.active_file_index])
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Freeze (pin) the given rows for the current file.
    pub fn freeze_rows(&mut self, mut row_indices: Vec<usize>) {
        row_indices.sort_unstable();
        row_indices.dedup();
        let file = self.files[self.active_file_index].clone();
        self.frozen_rows.insert(file, row_indices);
    }

    /// Unfreeze all rows for the current file.
    pub fn unfreeze_rows(&mut self) {
        let file = &self.files[self.active_file_index];
        self.frozen_rows.remove(file);
    }

    /// Unfreeze all columns and rows for the current file.
    pub fn unfreeze_all(&mut self) {
        self.unfreeze_columns();
        self.unfreeze_rows();
    }

    // ── Column types ──────────────────────────────────────────

    /// Get the column type for a column in the current file.
    pub fn column_type(&self, col_index: usize) -> Option<ColumnType> {
        self.column_types
            .get(&self.files[self.active_file_index])
            .and_then(|types| types.get(&col_index))
            .copied()
    }

    /// Get all column types for the current file.
    pub fn column_types(&self) -> Option<&HashMap<usize, ColumnType>> {
        self.column_types.get(&self.files[self.active_file_index])
    }

    /// Set the column type for a column in the current file.
    pub fn set_column_type(&mut self, col_index: usize, col_type: ColumnType) {
        let file = self.files[self.active_file_index].clone();
        self.column_types
            .entry(file)
            .or_default()
            .insert(col_index, col_type);
    }

    /// Clear the column type for a column in the current file.
    pub fn clear_column_type(&mut self, col_index: usize) {
        let file = &self.files[self.active_file_index];
        if let Some(types) = self.column_types.get_mut(file) {
            types.remove(&col_index);
        }
    }

    /// Get delimiter for a specific file (default: ',')
    pub fn delimiter(&self, file: &PathBuf) -> char {
        self.delimiters.get(file).copied().unwrap_or(',')
    }

    /// Set delimiter for a specific file
    pub fn set_delimiter(&mut self, file: PathBuf, delimiter: char) {
        self.delimiters.insert(file, delimiter);
    }

    /// Mark a file as dirty (modified)
    pub fn mark_dirty(&mut self, path: &Path) {
        self.dirty_files.insert(path.to_path_buf());
    }

    /// Mark a file as clean (saved)
    pub fn mark_clean(&mut self, path: &Path) {
        self.dirty_files.remove(path);
    }

    /// Check if a file is dirty
    pub fn is_dirty(&self, path: &Path) -> bool {
        self.dirty_files.contains(path)
    }

    /// Check if the current file is dirty
    pub fn is_current_file_dirty(&self) -> bool {
        self.is_dirty(&self.files[self.active_file_index])
    }

    /// Check if any file in the session is dirty
    pub fn has_any_dirty_files(&self) -> bool {
        !self.dirty_files.is_empty()
    }

    /// Get list of all dirty files
    pub fn dirty_files(&self) -> Vec<PathBuf> {
        self.dirty_files.iter().cloned().collect()
    }

    /// Cache a document for a file (used when switching files with unsaved changes)
    pub fn cache_document(&mut self, path: PathBuf, doc: Document) {
        self.document_cache.insert(path, doc);
    }

    /// Get a cached document for a file
    pub fn cached_document(&self, path: &PathBuf) -> Option<&Document> {
        self.document_cache.get(path)
    }

    /// Remove a document from cache (called after saving)
    pub fn remove_from_cache(&mut self, path: &PathBuf) {
        self.document_cache.remove(path);
    }

    /// Save undo/redo history for a file (called when switching away)
    pub fn cache_history(&mut self, path: PathBuf, history: crate::history::History) {
        self.history_cache.insert(path, history);
    }

    /// Restore undo/redo history for a file (called when switching to)
    pub fn take_history(&mut self, path: &PathBuf) -> Option<crate::history::History> {
        self.history_cache.remove(path)
    }

    /// Clear all cached documents
    pub fn clear_cache(&mut self) {
        self.document_cache.clear();
    }

    /// Add a file to the session and return its index
    pub fn add_file(&mut self, path: PathBuf) -> usize {
        self.files.push(path);
        self.files.len() - 1
    }

    /// Set the active file index
    pub fn set_active_file_index(&mut self, index: usize) {
        self.active_file_index = index;
    }

    /// Rename the current file (updates path in session and migrates all associated state)
    pub fn rename_current_file(&mut self, new_path: PathBuf) {
        let old_path = self.files[self.active_file_index].clone();
        self.files[self.active_file_index] = new_path.clone();
        // Migrate dirty tracking
        if self.dirty_files.remove(&old_path) {
            self.dirty_files.insert(new_path.clone());
        }
        // Migrate cached document
        if let Some(doc) = self.document_cache.remove(&old_path) {
            self.document_cache.insert(new_path.clone(), doc);
        }
        // Migrate delimiter settings
        if let Some(delim) = self.delimiters.remove(&old_path) {
            self.delimiters.insert(new_path.clone(), delim);
        }
        // Migrate query output tracking
        if self.query_output_files.remove(&old_path) {
            self.query_output_files.insert(new_path.clone());
        }
        // Migrate file mtime tracking
        if let Some(mtime) = self.file_mtimes.remove(&old_path) {
            self.file_mtimes.insert(new_path.clone(), mtime);
        }
        // Migrate frozen columns
        if let Some(frozen) = self.frozen_columns.remove(&old_path) {
            self.frozen_columns.insert(new_path.clone(), frozen);
        }
        // Migrate frozen rows
        if let Some(frozen) = self.frozen_rows.remove(&old_path) {
            self.frozen_rows.insert(new_path.clone(), frozen);
        }
        // Migrate column types
        if let Some(types) = self.column_types.remove(&old_path) {
            self.column_types.insert(new_path, types);
        }
    }

    /// Mark a file as a SQL query output sheet
    pub fn mark_query_output(&mut self, path: &Path) {
        self.query_output_files.insert(path.to_path_buf());
    }

    /// Unmark a file as a SQL query output sheet (e.g., after saving)
    pub fn unmark_query_output(&mut self, path: &Path) {
        self.query_output_files.remove(path);
    }

    /// Check if a file is a SQL query output sheet
    pub fn is_query_output(&self, path: &Path) -> bool {
        self.query_output_files.contains(path)
    }

    /// Record the current disk modification time for a file.
    /// Silently ignores errors (file might not exist for query outputs).
    pub fn record_file_mtime(&mut self, path: &Path) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(mtime) = metadata.modified() {
                self.file_mtimes.insert(path.to_path_buf(), mtime);
            }
        }
    }

    /// Check if a file was modified externally (disk mtime differs from stored).
    /// Returns false if no stored mtime, file doesn't exist, or metadata fails.
    pub fn check_file_modified(&self, path: &Path) -> bool {
        let Some(stored) = self.file_mtimes.get(path) else {
            return false;
        };
        match std::fs::metadata(path).and_then(|m| m.modified()) {
            Ok(current) => current != *stored,
            Err(_) => false,
        }
    }

    /// Remove the stored mtime entry for a file.
    pub fn clear_file_mtime(&mut self, path: &Path) {
        self.file_mtimes.remove(path);
    }

    /// Remove a file from the session by path.
    /// Cleans up dirty_files, document_cache, query_output_files,
    /// delimiters, and adjusts active_file_index. Returns true if a file was removed.
    pub fn remove_file(&mut self, path: &Path) -> bool {
        let Some(idx) = self.files.iter().position(|p| p == path) else {
            return false;
        };

        let removed = self.files.remove(idx);
        self.dirty_files.remove(&removed);
        self.document_cache.remove(&removed);
        self.query_output_files.remove(&removed);
        self.delimiters.remove(&removed);
        self.file_mtimes.remove(&removed);
        self.frozen_columns.remove(&removed);
        self.frozen_rows.remove(&removed);
        self.column_types.remove(&removed);

        // Adjust active_file_index
        if self.files.is_empty() {
            self.active_file_index = 0;
        } else if self.active_file_index >= self.files.len() {
            self.active_file_index = self.files.len() - 1;
        } else if self.active_file_index > idx {
            self.active_file_index -= 1;
        }

        true
    }

    /// Find the first unsaved query output file in the session
    pub fn find_query_output_file(&self) -> Option<&PathBuf> {
        self.files
            .iter()
            .find(|p| self.query_output_files.contains(*p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_files() -> Vec<PathBuf> {
        vec![
            PathBuf::from("file1.csv"),
            PathBuf::from("file2.csv"),
            PathBuf::from("file3.csv"),
        ]
    }

    #[test]
    fn test_file_config_default() {
        let config = FileConfig::new();
        assert_eq!(config.delimiter, None);
        assert!(!config.no_headers);
        assert_eq!(config.encoding, None);
    }

    #[test]
    fn test_file_config_with_options() {
        let config = FileConfig::with_options(Some(b';'), true, Some("utf-8".to_string()));
        assert_eq!(config.delimiter, Some(b';'));
        assert!(config.no_headers);
        assert_eq!(config.encoding, Some("utf-8".to_string()));
    }

    #[test]
    fn test_session_creation() {
        let files = test_files();
        let config = FileConfig::new();
        let session = Session::new(files.clone(), 0, config);

        assert_eq!(session.current_file(), &files[0]);
        assert_eq!(session.active_file_index(), 0);
        assert_eq!(session.file_count(), 3);
    }

    #[test]
    fn test_next_file() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        assert!(session.next_file());
        assert_eq!(session.active_file_index(), 1);

        assert!(session.next_file());
        assert_eq!(session.active_file_index(), 2);

        // Wrap around to first file
        assert!(session.next_file());
        assert_eq!(session.active_file_index(), 0);
    }

    #[test]
    fn test_prev_file() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        // Wrap to last file
        assert!(session.prev_file());
        assert_eq!(session.active_file_index(), 2);

        assert!(session.prev_file());
        assert_eq!(session.active_file_index(), 1);

        assert!(session.prev_file());
        assert_eq!(session.active_file_index(), 0);
    }

    #[test]
    fn test_single_file_no_switching() {
        let files = vec![PathBuf::from("single.csv")];
        let config = FileConfig::new();
        let mut session = Session::new(files, 0, config);

        assert!(!session.next_file());
        assert_eq!(session.active_file_index(), 0);

        assert!(!session.prev_file());
        assert_eq!(session.active_file_index(), 0);
    }

    #[test]
    fn test_has_multiple_files() {
        let config = FileConfig::new();

        let single = Session::new(vec![PathBuf::from("file.csv")], 0, config.clone());
        assert!(!single.has_multiple_files());

        let multiple = Session::new(test_files(), 0, config);
        assert!(multiple.has_multiple_files());
    }

    #[test]
    fn test_dirty_file_tracking() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        // Initially no dirty files
        assert!(!session.is_current_file_dirty());
        assert!(!session.has_any_dirty_files());

        // Mark current file as dirty
        session.mark_dirty(&files[0]);
        assert!(session.is_current_file_dirty());
        assert!(session.has_any_dirty_files());

        // Mark another file as dirty
        session.mark_dirty(&files[1]);
        assert_eq!(session.dirty_files().len(), 2);

        // Mark file as clean
        session.mark_clean(&files[0]);
        assert!(!session.is_current_file_dirty());
        assert!(session.has_any_dirty_files()); // Still have file[1] dirty
    }

    #[test]
    fn test_document_caching() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        // Create a test document
        let doc = crate::Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
            "test.csv".to_string(),
        );

        // Cache it
        session.cache_document(files[0].clone(), doc);
        assert!(session.cached_document(&files[0]).is_some());

        // Remove from cache
        session.remove_from_cache(&files[0]);
        assert!(session.cached_document(&files[0]).is_none());
    }

    #[test]
    fn test_clear_cache() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        // Cache multiple documents
        for file in &files {
            let doc = crate::Document::new(
                vec!["A".to_string()],
                vec![vec!["1".to_string()]],
                "test.csv".to_string(),
            );
            session.cache_document(file.clone(), doc);
        }

        assert!(session.cached_document(&files[0]).is_some());
        assert!(session.cached_document(&files[1]).is_some());

        // Clear all
        session.clear_cache();
        assert!(session.cached_document(&files[0]).is_none());
        assert!(session.cached_document(&files[1]).is_none());
    }

    #[test]
    fn test_rename_current_file_updates_path() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        let new_path = PathBuf::from("renamed.csv");
        session.rename_current_file(new_path.clone());

        assert_eq!(session.current_file(), &new_path);
        assert_eq!(session.files()[0], new_path);
        // Other files unchanged
        assert_eq!(session.files()[1], files[1]);
    }

    #[test]
    fn test_rename_migrates_dirty_tracking() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        session.mark_dirty(&files[0]);
        assert!(session.is_dirty(&files[0]));

        let new_path = PathBuf::from("renamed.csv");
        session.rename_current_file(new_path.clone());

        assert!(!session.is_dirty(&files[0]));
        assert!(session.is_dirty(&new_path));
    }

    #[test]
    fn test_rename_migrates_cached_document() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        let doc = crate::Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );
        session.cache_document(files[0].clone(), doc);

        let new_path = PathBuf::from("renamed.csv");
        session.rename_current_file(new_path.clone());

        assert!(session.cached_document(&files[0]).is_none());
        assert!(session.cached_document(&new_path).is_some());
    }

    #[test]
    fn test_rename_migrates_delimiter() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        session.set_delimiter(files[0].clone(), ';');

        let new_path = PathBuf::from("renamed.csv");
        session.rename_current_file(new_path.clone());

        assert_eq!(session.delimiter(&new_path), ';');
    }

    #[test]
    fn test_rename_migrates_query_output_tracking() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        session.mark_query_output(&files[0]);
        assert!(session.is_query_output(&files[0]));

        let new_path = PathBuf::from("results.csv");
        session.rename_current_file(new_path.clone());

        assert!(!session.is_query_output(&files[0]));
        assert!(session.is_query_output(&new_path));
        assert_eq!(session.find_query_output_file(), Some(&new_path));
    }

    #[test]
    fn test_query_output_tracking() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        // Initially no query outputs
        assert!(!session.is_query_output(&files[0]));
        assert!(session.find_query_output_file().is_none());

        // Mark as query output
        session.mark_query_output(&files[0]);
        assert!(session.is_query_output(&files[0]));
        assert_eq!(session.find_query_output_file(), Some(&files[0]));

        // Unmark
        session.unmark_query_output(&files[0]);
        assert!(!session.is_query_output(&files[0]));
        assert!(session.find_query_output_file().is_none());
    }

    // ── Frozen columns tests ─────────────────────────────

    #[test]
    fn test_freeze_columns() {
        let files = test_files();
        let mut session = Session::new(files, 0, FileConfig::new());

        assert!(session.frozen_columns().is_empty());

        session.freeze_columns(vec![2, 0, 2]); // duplicates and unsorted
        assert_eq!(session.frozen_columns(), &[0, 2]); // sorted and deduped
    }

    #[test]
    fn test_unfreeze_columns() {
        let files = test_files();
        let mut session = Session::new(files, 0, FileConfig::new());

        session.freeze_columns(vec![0, 1]);
        assert_eq!(session.frozen_columns().len(), 2);

        session.unfreeze_columns();
        assert!(session.frozen_columns().is_empty());
    }

    // ── Frozen rows tests ────────────────────────────────

    #[test]
    fn test_freeze_rows() {
        let files = test_files();
        let mut session = Session::new(files, 0, FileConfig::new());

        assert!(session.frozen_rows().is_empty());

        session.freeze_rows(vec![3, 1, 3]); // duplicates and unsorted
        assert_eq!(session.frozen_rows(), &[1, 3]); // sorted and deduped
    }

    #[test]
    fn test_unfreeze_rows() {
        let files = test_files();
        let mut session = Session::new(files, 0, FileConfig::new());

        session.freeze_rows(vec![0, 5]);
        assert_eq!(session.frozen_rows().len(), 2);

        session.unfreeze_rows();
        assert!(session.frozen_rows().is_empty());
    }

    #[test]
    fn test_unfreeze_all_clears_both() {
        let files = test_files();
        let mut session = Session::new(files, 0, FileConfig::new());

        session.freeze_columns(vec![0, 1]);
        session.freeze_rows(vec![0, 2]);

        session.unfreeze_all();
        assert!(session.frozen_columns().is_empty());
        assert!(session.frozen_rows().is_empty());
    }

    #[test]
    fn test_frozen_per_file() {
        let files = test_files();
        let mut session = Session::new(files.clone(), 0, FileConfig::new());

        // Freeze columns on file 0
        session.freeze_columns(vec![0]);
        assert_eq!(session.frozen_columns(), &[0]);

        // Switch to file 1 — no frozen columns
        session.next_file();
        assert!(session.frozen_columns().is_empty());

        // Switch back — still frozen
        session.prev_file();
        assert_eq!(session.frozen_columns(), &[0]);
    }

    // ── Column types tests ───────────────────────────────

    #[test]
    fn test_set_and_get_column_type() {
        let files = test_files();
        let mut session = Session::new(files, 0, FileConfig::new());

        assert!(session.column_type(0).is_none());

        session.set_column_type(0, ColumnType::Number);
        assert_eq!(session.column_type(0), Some(ColumnType::Number));

        session.set_column_type(1, ColumnType::Date);
        assert_eq!(session.column_type(1), Some(ColumnType::Date));
    }

    #[test]
    fn test_clear_column_type() {
        let files = test_files();
        let mut session = Session::new(files, 0, FileConfig::new());

        session.set_column_type(0, ColumnType::Number);
        assert!(session.column_type(0).is_some());

        session.clear_column_type(0);
        assert!(session.column_type(0).is_none());
    }

    #[test]
    fn test_column_types_per_file() {
        let files = test_files();
        let mut session = Session::new(files.clone(), 0, FileConfig::new());

        session.set_column_type(0, ColumnType::Number);
        assert_eq!(session.column_type(0), Some(ColumnType::Number));

        session.next_file();
        assert!(session.column_type(0).is_none());

        session.prev_file();
        assert_eq!(session.column_type(0), Some(ColumnType::Number));
    }

    #[test]
    fn test_rename_migrates_frozen_and_types() {
        let files = test_files();
        let mut session = Session::new(files.clone(), 0, FileConfig::new());

        session.freeze_columns(vec![0]);
        session.freeze_rows(vec![1]);
        session.set_column_type(0, ColumnType::Date);

        let new_path = PathBuf::from("renamed.csv");
        session.rename_current_file(new_path.clone());

        assert_eq!(session.frozen_columns(), &[0]);
        assert_eq!(session.frozen_rows(), &[1]);
        assert_eq!(session.column_type(0), Some(ColumnType::Date));
    }

    #[test]
    fn test_remove_file_cleans_frozen_and_types() {
        let files = test_files();
        let mut session = Session::new(files.clone(), 0, FileConfig::new());

        session.freeze_columns(vec![0]);
        session.set_column_type(0, ColumnType::Number);

        session.remove_file(&files[0]);
        // After removal, active file changed — the old file's state is gone
        assert!(session.frozen_columns().is_empty());
        assert!(session.column_type(0).is_none());
    }
}
