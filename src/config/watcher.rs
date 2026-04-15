//! Config file watcher — detects changes by tracking modification times.

use std::path::PathBuf;
use std::time::SystemTime;

use super::global_config_path;

/// Watches config files for changes by tracking their modification times.
#[derive(Debug, Clone)]
pub struct ConfigWatcher {
    pub(super) global_path: Option<PathBuf>,
    pub(super) local_path: PathBuf,
    pub(super) global_mtime: Option<SystemTime>,
    pub(super) local_mtime: Option<SystemTime>,
}

impl Default for ConfigWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWatcher {
    /// Create a new watcher, recording the current mtimes of config files.
    pub fn new() -> Self {
        let global_path = global_config_path();
        let local_path = PathBuf::from(".lazycsv.toml");

        let global_mtime = global_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        let local_mtime = std::fs::metadata(&local_path)
            .ok()
            .and_then(|m| m.modified().ok());

        Self {
            global_path,
            local_path,
            global_mtime,
            local_mtime,
        }
    }

    /// Check if any config file has been modified since last check.
    /// Returns true if config should be reloaded.
    pub fn has_changed(&mut self) -> bool {
        let new_global = self
            .global_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        let new_local = std::fs::metadata(&self.local_path)
            .ok()
            .and_then(|m| m.modified().ok());

        let changed = new_global != self.global_mtime || new_local != self.local_mtime;

        if changed {
            self.global_mtime = new_global;
            self.local_mtime = new_local;
        }

        changed
    }
}
