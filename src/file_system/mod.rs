//! File system operations for CSV file discovery
//!
//! Scans directories to find CSV files, used for multi-file navigation.

pub mod discovery;

pub use discovery::{scan_directory, scan_directory_for_csvs};
