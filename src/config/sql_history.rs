//! SQL query history persistence.

use std::path::PathBuf;

use super::dirs_path;

/// Path to the SQL history file (~/.config/lazycsv/sql_history).
pub fn sql_history_path() -> Option<PathBuf> {
    dirs_path().map(|p| p.join("sql_history"))
}

/// Load SQL history from disk. Returns an empty vec if the file doesn't exist or can't be read.
///
/// Format: one query per line; embedded newlines are stored as the two-character sequence `\n`.
pub fn load_sql_history() -> Vec<String> {
    let path = match sql_history_path() {
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
        .map(|l| l.replace("\\n", "\n"))
        .collect()
}

/// Save SQL history to disk, capped to `limit` entries.
///
/// Silently ignores write errors (non-critical).
pub fn save_sql_history(history: &[String], limit: usize) {
    let path = match sql_history_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content: String = history
        .iter()
        .take(limit)
        .map(|q| q.replace('\n', "\\n"))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write(&path, content);
}
