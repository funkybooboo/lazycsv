//! CSV header schema cache with mtime-based invalidation.

/// Cached CSV header schema for a single file.
#[derive(Debug, Clone)]
struct CachedSchema {
    headers: Vec<String>,
    mtime: std::time::SystemTime,
}

/// Cache of CSV headers keyed by file path, invalidated by mtime changes.
/// Avoids re-reading headers from disk on every keystroke (validation) and
/// every Ctrl+N (completions).
#[derive(Debug, Default)]
pub struct SchemaCache {
    cache: std::collections::HashMap<std::path::PathBuf, CachedSchema>,
}

impl SchemaCache {
    /// Return cached headers if the file's mtime hasn't changed, otherwise
    /// re-read from disk and update the cache. Returns `None` on read failure
    /// (failures are not cached).
    pub fn get_headers(&mut self, path: &std::path::Path) -> Option<Vec<String>> {
        let meta = std::fs::metadata(path).ok()?;
        let mtime = meta.modified().ok()?;

        if let Some(cached) = self.cache.get(path) {
            if cached.mtime == mtime {
                return Some(cached.headers.clone());
            }
        }

        // Read headers from CSV
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .ok()?;
        let headers: Vec<String> = reader.headers().ok()?.iter().map(String::from).collect();

        self.cache.insert(
            path.to_path_buf(),
            CachedSchema {
                headers: headers.clone(),
                mtime,
            },
        );

        Some(headers)
    }

    /// Convenience alias for `get_headers` – returns a cloned `Vec<String>`.
    pub fn get_or_read(&mut self, path: &std::path::Path) -> Option<Vec<String>> {
        self.get_headers(path)
    }
}
