//! Directory browser for yazi-style file navigation

use std::path::{Path, PathBuf};

/// Entry in the file browser (either a directory or CSV file)
#[derive(Debug, Clone, PartialEq)]
pub enum BrowserEntry {
    Directory(PathBuf),
    CsvFile(PathBuf),
}

impl BrowserEntry {
    pub fn path(&self) -> &Path {
        match self {
            BrowserEntry::Directory(p) | BrowserEntry::CsvFile(p) => p,
        }
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, BrowserEntry::Directory(_))
    }

    pub fn filename(&self) -> Option<&str> {
        self.path().file_name()?.to_str()
    }
}

/// Scan directory for CSV files and subdirectories
pub fn scan_directory(dir: &Path) -> Result<Vec<BrowserEntry>, std::io::Error> {
    scan_directory_filtered(dir, false)
}

/// Scan directory with optional hidden file display
pub fn scan_directory_filtered(
    dir: &Path,
    show_hidden: bool,
) -> Result<Vec<BrowserEntry>, std::io::Error> {
    let mut entries = Vec::new();

    // Always add parent directory (..) if not at root
    if dir.parent().is_some() {
        entries.push(BrowserEntry::Directory(dir.join("..")));
    }

    // Read directory contents
    let read_dir = std::fs::read_dir(dir)?;

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        // Skip hidden files/directories unless show_hidden is true
        if !show_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
        }

        if metadata.is_dir() {
            // Add directories
            entries.push(BrowserEntry::Directory(path));
        } else if metadata.is_file() {
            // Add CSV files only
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("csv")
                    || ext.eq_ignore_ascii_case("xlsx")
                    || ext.eq_ignore_ascii_case("xls")
                    || ext.eq_ignore_ascii_case("ods")
                {
                    entries.push(BrowserEntry::CsvFile(path));
                }
            }
        }
    }

    // Sort: directories first (except ..), then CSV files, both alphabetically
    entries.sort_by(|a, b| {
        use std::cmp::Ordering;

        // .. always first
        if a.filename() == Some("..") {
            return Ordering::Less;
        }
        if b.filename() == Some("..") {
            return Ordering::Greater;
        }

        // Then directories
        match (a.is_directory(), b.is_directory()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => {
                // Same type: alphabetical by filename
                let a_name = a.filename().unwrap_or("");
                let b_name = b.filename().unwrap_or("");
                a_name.to_lowercase().cmp(&b_name.to_lowercase())
            }
        }
    });

    Ok(entries)
}
