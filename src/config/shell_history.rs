//! Persistent shell-command (`:` in file menu) history.
//!
//! Mirrors `command_history`: one command per line at
//! `~/.config/lazycsv/shell_history`. Order is most-recent-first so the
//! in-memory representation maps directly to file order.

use std::path::PathBuf;

use super::dirs_path;

/// Path to the shell-command history file (~/.config/lazycsv/shell_history).
pub fn shell_history_path() -> Option<PathBuf> {
    dirs_path().map(|p| p.join("shell_history"))
}

/// Load shell history from disk. Returns an empty vec on missing/unreadable file.
pub fn load_shell_history() -> Vec<String> {
    let path = match shell_history_path() {
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

/// Save shell history to disk, capped to `limit` entries. Write errors are ignored.
pub fn save_shell_history(history: &[String], limit: usize) {
    let path = match shell_history_path() {
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
