//! Multi-file session management and CSV configuration.
//!
//! This module handles file switching between multiple CSV files and
//! maintains the configuration settings for parsing CSV files.

use crate::Document;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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

    /// Per-file header mode settings (true = header row styled/frozen)
    header_modes: HashMap<PathBuf, bool>,

    /// Per-file delimiter settings
    delimiters: HashMap<PathBuf, char>,

    /// Set of dirty (modified) files
    dirty_files: HashSet<PathBuf>,

    /// Cache of dirty documents (avoids reloading from disk when switching files)
    document_cache: HashMap<PathBuf, Document>,

    /// Files that are SQL query output sheets (should be reused on next query)
    query_output_files: HashSet<PathBuf>,
}

impl Session {
    /// Create a new session
    pub fn new(files: Vec<PathBuf>, active_file_index: usize, config: FileConfig) -> Self {
        Self {
            files,
            active_file_index,
            config,
            header_modes: HashMap::new(),
            delimiters: HashMap::new(),
            dirty_files: HashSet::new(),
            document_cache: HashMap::new(),
            query_output_files: HashSet::new(),
        }
    }

    /// Get the currently active file path
    pub fn get_current_file(&self) -> &PathBuf {
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

    /// Get header mode for the current file (default: true)
    pub fn get_header_mode(&self) -> bool {
        self.header_modes
            .get(&self.files[self.active_file_index])
            .copied()
            .unwrap_or(true) // Default: header mode ON
    }

    /// Set header mode for the current file
    pub fn set_header_mode(&mut self, mode: bool) {
        self.header_modes
            .insert(self.files[self.active_file_index].clone(), mode);
    }

    /// Get delimiter for a specific file (default: ',')
    pub fn get_delimiter(&self, file: &PathBuf) -> char {
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
    pub fn get_dirty_files(&self) -> Vec<PathBuf> {
        self.dirty_files.iter().cloned().collect()
    }

    /// Cache a document for a file (used when switching files with unsaved changes)
    pub fn cache_document(&mut self, path: PathBuf, doc: Document) {
        self.document_cache.insert(path, doc);
    }

    /// Get a cached document for a file
    pub fn get_cached_document(&self, path: &PathBuf) -> Option<&Document> {
        self.document_cache.get(path)
    }

    /// Remove a document from cache (called after saving)
    pub fn remove_from_cache(&mut self, path: &PathBuf) {
        self.document_cache.remove(path);
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
        // Migrate header mode and delimiter settings
        if let Some(mode) = self.header_modes.remove(&old_path) {
            self.header_modes.insert(new_path.clone(), mode);
        }
        if let Some(delim) = self.delimiters.remove(&old_path) {
            self.delimiters.insert(new_path.clone(), delim);
        }
        // Migrate query output tracking
        if self.query_output_files.remove(&old_path) {
            self.query_output_files.insert(new_path);
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

    /// Remove a file from the session by path.
    /// Cleans up dirty_files, document_cache, query_output_files, header_modes,
    /// delimiters, and adjusts active_file_index. Returns true if a file was removed.
    pub fn remove_file(&mut self, path: &Path) -> bool {
        let Some(idx) = self.files.iter().position(|p| p == path) else {
            return false;
        };

        let removed = self.files.remove(idx);
        self.dirty_files.remove(&removed);
        self.document_cache.remove(&removed);
        self.query_output_files.remove(&removed);
        self.header_modes.remove(&removed);
        self.delimiters.remove(&removed);

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

        assert_eq!(session.get_current_file(), &files[0]);
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
        assert_eq!(session.get_dirty_files().len(), 2);

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
        assert!(session.get_cached_document(&files[0]).is_some());

        // Remove from cache
        session.remove_from_cache(&files[0]);
        assert!(session.get_cached_document(&files[0]).is_none());
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

        assert!(session.get_cached_document(&files[0]).is_some());
        assert!(session.get_cached_document(&files[1]).is_some());

        // Clear all
        session.clear_cache();
        assert!(session.get_cached_document(&files[0]).is_none());
        assert!(session.get_cached_document(&files[1]).is_none());
    }

    #[test]
    fn test_rename_current_file_updates_path() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        let new_path = PathBuf::from("renamed.csv");
        session.rename_current_file(new_path.clone());

        assert_eq!(session.get_current_file(), &new_path);
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

        assert!(session.get_cached_document(&files[0]).is_none());
        assert!(session.get_cached_document(&new_path).is_some());
    }

    #[test]
    fn test_rename_migrates_header_mode_and_delimiter() {
        let files = test_files();
        let config = FileConfig::new();
        let mut session = Session::new(files.clone(), 0, config);

        session.set_header_mode(false);
        session.set_delimiter(files[0].clone(), ';');

        let new_path = PathBuf::from("renamed.csv");
        session.rename_current_file(new_path.clone());

        assert_eq!(session.get_header_mode(), false);
        assert_eq!(session.get_delimiter(&new_path), ';');
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
}
