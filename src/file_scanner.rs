use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Scan a specific directory for CSV files
pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut csv_files = Vec::new();

    // Read directory entries
    for entry in std::fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Check if it's a CSV file
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_str() == Some("csv") {
                    csv_files.push(path);
                }
            }
        }
    }

    // Sort alphabetically
    csv_files.sort();

    Ok(csv_files)
}

/// Scan directory for CSV files (given a file path, scans its parent directory)
pub fn scan_directory_for_csvs(file_path: &Path) -> Result<Vec<PathBuf>> {
    // Get the directory containing the file
    // If parent is None or empty, use current directory
    let dir = match file_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };

    let mut csv_files = scan_directory(dir)?;

    // If no CSV files found (shouldn't happen), at least include the current file
    if csv_files.is_empty() {
        csv_files.push(file_path.to_path_buf());
    }

    Ok(csv_files)
}
