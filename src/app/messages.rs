//! Centralized user-facing message strings.
//!
//! All status messages, error messages, and user feedback strings
//! are defined here for consistency and easy localization.

// Command-related messages
pub const CMD_TIMEOUT: &str = "Command timeout";
pub const CMD_CANCELLED: &str = "Command cancelled";

// Quit-related messages
pub const UNSAVED_CHANGES: &str = "Unsaved changes! Use :q! to force quit";

// Navigation messages
pub const JUMPED_TO_FIRST_ROW: &str = "Jumped to first row";

/// Format a "jumped to line" message
pub fn jumped_to_line(line: usize) -> String {
    format!("Jumped to line {}", line)
}

// Viewport positioning messages
pub const VIEW_TOP: &str = "View: top";
pub const VIEW_CENTER: &str = "View: center";
pub const VIEW_BOTTOM: &str = "View: bottom";

/// Format an "unknown command" message
pub fn unknown_command(cmd1: &str, cmd2: &str) -> String {
    format!("Unknown command: {} {}", cmd1, cmd2)
}

// Error messages
pub const NO_PATH_PROVIDED: &str = "No path provided";

/// Format a "no CSV files found" error
pub fn no_csv_files_found(path: &std::path::Path) -> String {
    format!("No CSV files found in directory: {}", path.display())
}

/// Format an "invalid path" error
pub fn invalid_path(path: &std::path::Path) -> String {
    format!("Invalid path: {}", path.display())
}

/// Format a "failed to load CSV" error
pub fn failed_to_load_csv(path: &std::path::Path) -> String {
    format!("Failed to load CSV file: {}", path.display())
}

/// Format a "failed to reload file" error
pub fn failed_to_reload_file(path: &std::path::Path) -> String {
    format!("Failed to reload file: {}", path.display())
}
