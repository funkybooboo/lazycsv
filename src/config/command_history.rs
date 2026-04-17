//! Persistent ex-command (`:`) history.
//!
//! Mirrors `sql_history`: one command per line at `~/.config/lazycsv/command_history`.

use std::path::PathBuf;

use super::dirs_path;

/// Path to the command history file (~/.config/lazycsv/command_history).
pub fn command_history_path() -> Option<PathBuf> {
    dirs_path().map(|p| p.join("command_history"))
}

/// Load command history from disk. Returns an empty vec if the file doesn't exist or can't be read.
/// Order: most recent first (matches the in-memory representation).
pub fn load_command_history() -> Vec<String> {
    let path = match command_history_path() {
        Some(p) => p,
        None => return Vec::new(),
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// Save command history to disk, capped to `limit` entries.
/// Silently ignores write errors.
pub fn save_command_history(history: &[String], limit: usize) {
    let path = match command_history_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content: String = history
        .iter()
        .take(limit)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write(&path, content);
}
