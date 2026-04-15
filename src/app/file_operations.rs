//! File reload, save, and monitoring methods on App.

use super::App;
use crate::input::StatusMessage;
use crate::ui::ViewState;
use crate::Document;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use super::messages;

impl App {
    /// Reload CSV data from current file
    pub fn reload_current_file(&mut self) -> Result<()> {
        let file_path = self.current_file().clone();
        let config = self.session.config();

        self.document = Document::from_file(
            &file_path,
            config.delimiter,
            config.no_headers,
            config.encoding.clone(),
        )
        .context(messages::failed_to_reload_file(&file_path))?;

        // Reset view state
        self.view_state = ViewState::default();
        self.view_state.table_state.select(Some(0));

        Ok(())
    }

    /// Reload CSV data from current file with a specific delimiter
    pub fn reload_current_file_with_delimiter(&mut self, delimiter: char) -> Result<()> {
        let file_path = self.current_file().clone();
        let config = self.session.config();

        self.document = Document::from_file(
            &file_path,
            Some(delimiter as u8),
            config.no_headers,
            config.encoding.clone(),
        )
        .context(messages::failed_to_reload_file(&file_path))?;

        // Override delimiter
        self.document.delimiter = delimiter;

        // Reset view state
        self.view_state = ViewState::default();
        self.view_state.table_state.select(Some(0));

        Ok(())
    }

    /// Save the current file to disk
    /// Returns the path of the saved file
    pub fn save_current_file(&mut self) -> Result<PathBuf> {
        let file_path = self.current_file().clone();
        let delimiter = self.document.delimiter;

        // Write the file atomically
        crate::csv::write_csv_atomic(&self.document, &file_path, delimiter)
            .context(format!("Failed to save file: {:?}", file_path))?;

        // Mark as clean and remove from cache
        self.document.is_dirty = false;
        self.session.mark_clean(&file_path);
        self.session.remove_from_cache(&file_path);
        // No longer a virtual query output once saved to disk
        self.session.unmark_query_output(&file_path);
        // Record new mtime so we don't treat our own save as external modification
        self.session.record_file_mtime(&file_path);

        Ok(file_path)
    }

    /// Save all dirty files in the session
    /// Returns a vector of saved file paths
    pub fn save_all_files(&mut self) -> Result<Vec<PathBuf>> {
        let mut saved_files = Vec::new();

        // Save current file if dirty
        if self.document.is_dirty {
            let path = self.save_current_file()?;
            saved_files.push(path);
        }

        // Save all other dirty cached documents
        let dirty_files: Vec<PathBuf> = self.session.dirty_files();
        for file_path in dirty_files {
            // Skip current file (already saved above)
            if &file_path == self.current_file() {
                continue;
            }

            // Get cached document
            if let Some(doc) = self.session.cached_document(&file_path) {
                let delimiter = doc.delimiter;

                // Write atomically
                crate::csv::write_csv_atomic(doc, &file_path, delimiter)
                    .context(format!("Failed to save file: {:?}", file_path))?;

                // Mark as clean and remove from cache
                self.session.mark_clean(&file_path);
                self.session.remove_from_cache(&file_path);

                saved_files.push(file_path);
            }
        }

        Ok(saved_files)
    }

    /// Invalidate the DuckDB cache entry for a specific file path.
    /// Called after file reloads from disk to force re-import on next query.
    pub fn invalidate_duckdb_cache_for(&mut self, path: &Path) {
        if let Some(cache) = &mut self.duckdb_cache {
            let table_name = crate::query::table_name_from_path(path);
            cache.remove_table(path, &table_name);
        }
    }

    /// Drop the entire DuckDB cache (e.g. when session structure changes drastically).
    pub fn invalidate_duckdb_cache(&mut self) {
        self.duckdb_cache = None;
    }

    /// Add a query to SQL history, deduplicating and capping to the configured limit.
    /// Does nothing if `sql_history_limit` is 0.
    pub fn push_sql_history(&mut self, query: String) {
        let limit = self.config.sql.sql_history_limit;
        if limit == 0 {
            return;
        }
        // Remove duplicate if already present
        self.sql_history.retain(|q| q != &query);
        self.sql_history.insert(0, query);
        self.sql_history.truncate(limit);
    }

    /// Check if the current file has been modified externally.
    /// Sets `external_modification_pending` and a status message if so.
    /// Returns true if a modification was detected (triggers redraw).
    pub fn check_current_file_modification(&mut self) -> bool {
        // Don't re-check if we're already prompting
        if self.external_modification_pending {
            return false;
        }
        let path = self.current_file().clone();
        // Skip query output files (they don't live on disk in the normal sense)
        if self.session.is_query_output(&path) {
            return false;
        }
        if self.session.check_file_modified(&path) {
            self.external_modification_pending = true;
            let msg = if self.document.is_dirty {
                "File modified externally (unsaved changes). Press 'r' to reload, Esc to ignore"
            } else {
                "File modified externally. Press 'r' to reload, Esc to ignore"
            };
            self.status_message = Some(StatusMessage::new_persistent(msg.to_string()));
            true
        } else {
            false
        }
    }

    /// Check if config files have changed and reload if so.
    /// Returns true if config was reloaded (triggers redraw).
    pub fn check_config_reload(&mut self) -> bool {
        if self.config_watcher.has_changed() {
            let result = crate::config::load_config_with_warnings();
            self.config = result.config;
            if result.warnings.is_empty() {
                self.status_message = Some(StatusMessage::from("Config reloaded".to_string()));
            } else {
                self.status_message = Some(StatusMessage::from(format!(
                    "Config reloaded: {}",
                    result.warnings.join("; ")
                )));
            }
            true
        } else {
            false
        }
    }

    /// Load a CSV or XLSX file with cancellation support.
    /// Same as `load_file` but uses `Document::from_file_cancellable`.
    pub fn load_file_cancellable(
        file_path: &Path,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        file_config: crate::session::FileConfig,
        cli_args: &crate::cli::CliArgs,
        cancelled: &AtomicBool,
        sheet_name: Option<&str>,
    ) -> Result<Self> {
        let csv_data = crate::csv::Document::from_file_cancellable_with_sheet(
            file_path,
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
            cancelled,
            sheet_name,
        )
        .context(messages::failed_to_load_csv(file_path))?;

        let mut app = Self::new(csv_data, csv_files, current_file_index, file_config);
        app.session.record_file_mtime(file_path);
        Ok(app)
    }

    /// Reload CSV data from current file with cancellation support.
    /// Returns Ok(true) if loaded successfully, Ok(false) if cancelled (keeps existing document).
    pub fn reload_current_file_cancellable(&mut self, cancelled: &AtomicBool) -> Result<bool> {
        let file_path = self.current_file().clone();

        // Check for cached document first (e.g. query output files that don't exist on disk)
        if let Some(cached) = self.session.cached_document(&file_path) {
            // Drop old rows on background thread to avoid blocking UI
            let old_storage = self.document.take_storage();
            std::thread::spawn(move || drop(old_storage));
            self.document = cached.clone();
            self.view_state = ViewState::default();
            // Start at row 0 (displays as row 1)
            self.view_state.table_state.select(Some(0));
            return Ok(true);
        }

        // Query output files don't exist on disk — if not cached, keep current document
        if self.session.is_query_output(&file_path) {
            return Ok(true);
        }

        // Not cached — reload from disk
        let config = self.session.config();
        match crate::csv::Document::from_file_cancellable(
            &file_path,
            config.delimiter,
            config.no_headers,
            config.encoding.clone(),
            cancelled,
        ) {
            Ok(doc) => {
                // Drop old rows on background thread
                let old_storage = self.document.take_storage();
                std::thread::spawn(move || drop(old_storage));
                self.document = doc;
                // Invalidate SQLite cache for this file (generation reset to 0)
                self.invalidate_duckdb_cache_for(&file_path);
                // Record new mtime so we don't re-prompt for this version
                self.session.record_file_mtime(&file_path);
                self.view_state = ViewState::default();
                // Start at row 0 (displays as row 1)
                self.view_state.table_state.select(Some(0));
                Ok(true)
            }
            Err(e) => {
                if e.downcast_ref::<crate::cancel::CancelledError>().is_some() {
                    Ok(false)
                } else {
                    Err(e).context(messages::failed_to_reload_file(&file_path))
                }
            }
        }
    }
}
