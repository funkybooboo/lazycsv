//! Multi-file session management and CSV configuration.
//!
//! This module handles file switching between multiple CSV files and
//! maintains the configuration settings for parsing CSV files.

use std::path::PathBuf;

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
}

impl Session {
    /// Create a new session
    pub fn new(files: Vec<PathBuf>, active_file_index: usize, config: FileConfig) -> Self {
        Self {
            files,
            active_file_index,
            config,
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
}
