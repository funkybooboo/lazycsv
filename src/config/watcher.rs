//! Config file watcher — detects changes by tracking modification times.

use std::path::PathBuf;
use std::time::SystemTime;

use super::global_config_path;

/// Watches config files for changes by tracking their modification times.
///
/// Tracks both the theme/defaults file (`config.toml`) and the keymap file
/// (`keys.toml`). Use [`ConfigWatcher::has_changed`] to learn when *any*
/// watched file has been touched, and [`ConfigWatcher::keymap_changed`] for
/// the keymap-specific case (lets the caller rebuild only the keymap
/// without re-parsing every section of `config.toml`).
#[derive(Debug, Clone)]
pub struct ConfigWatcher {
    pub(super) global_path: Option<PathBuf>,
    pub(super) local_path: PathBuf,
    pub(super) keys_path: Option<PathBuf>,
    pub(super) global_mtime: Option<SystemTime>,
    pub(super) local_mtime: Option<SystemTime>,
    pub(super) keys_mtime: Option<SystemTime>,
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
        let keys_path = super::dirs_path().map(|p| p.join("keys.toml"));

        let global_mtime = global_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        let local_mtime = std::fs::metadata(&local_path)
            .ok()
            .and_then(|m| m.modified().ok());
        let keys_mtime = keys_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());

        Self {
            global_path,
            local_path,
            keys_path,
            global_mtime,
            local_mtime,
            keys_mtime,
        }
    }

    /// Check if `config.toml` (global or local) has changed. Updates the
    /// recorded mtime so the next call only fires on the *next* edit.
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

    /// Check if `keys.toml` has changed since last check. Updates the
    /// recorded mtime so the next call only fires on the *next* edit.
    pub fn keymap_changed(&mut self) -> bool {
        let new = self
            .keys_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        let changed = new != self.keys_mtime;
        if changed {
            self.keys_mtime = new;
        }
        changed
    }

    /// Path the watcher is monitoring for keymap changes (or `None` if
    /// the OS provided no config dir).
    pub fn keys_path(&self) -> Option<&PathBuf> {
        self.keys_path.as_ref()
    }
}
