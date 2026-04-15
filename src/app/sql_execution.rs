//! Helper functions for SQL query execution with CSV loading.
//!
//! Extracted from execute_sql_query_cancellable to keep the main function under 50 lines.

use crate::cancel;
use crate::csv::Document;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use super::{App, DuckDbCache, Mode};

use crate::ui::utils::format_number;

/// Remove stale tables (files no longer in the session).
///
/// Returns true if cancellation was detected.
pub fn cleanup_stale_tables(
    cache: &mut DuckDbCache,
    session_paths: &[PathBuf],
    cancelled: &AtomicBool,
) -> bool {
    if cancel::check_esc(cancelled) {
        return true;
    }

    let session_set: HashSet<&Path> = session_paths.iter().map(|p| p.as_path()).collect();
    let stale: Vec<PathBuf> = cache
        .loaded_generations()
        .keys()
        .filter(|p| !session_set.contains(p.as_path()))
        .cloned()
        .collect();

    for path in stale {
        let table_name = crate::query::table_name_from_path(&path);
        cache.remove_table(&path, &table_name);
    }

    false
}

/// Load the currently active document into DuckDB if needed.
///
/// Returns (success, was_cancelled).
pub fn load_current_document(
    cache: &mut DuckDbCache,
    file_path: &Path,
    document: &Document,
    cancelled: &AtomicBool,
) -> (bool, bool) {
    if cancel::check_esc(cancelled) {
        return (false, true);
    }

    let table_name = crate::query::table_name_from_path(file_path);

    if !cache.needs_reload(file_path, document.generation) {
        return (true, false);
    }

    if document.row_count() == 0 || document.column_count() == 0 {
        return (true, false);
    }

    match cache.reload_table(
        file_path,
        &table_name,
        document,
        document.generation,
        cancelled,
        false,
    ) {
        Ok(()) => (true, false),
        Err(e) => {
            if e.downcast_ref::<cancel::CancelledError>().is_some() {
                (false, true)
            } else {
                (false, false)
            }
        }
    }
}

/// Load a cached document from the session into DuckDB if needed.
///
/// Returns (success, was_cancelled).
pub fn load_cached_document(
    cache: &mut DuckDbCache,
    file_path: &Path,
    cached_doc: &Document,
    cancelled: &AtomicBool,
) -> (bool, bool) {
    if cancel::check_esc(cancelled) {
        return (false, true);
    }

    let table_name = crate::query::table_name_from_path(file_path);

    if !cache.needs_reload(file_path, cached_doc.generation) {
        return (true, false);
    }

    if cached_doc.row_count() == 0 || cached_doc.column_count() == 0 {
        return (true, false);
    }

    let gen = cached_doc.generation;
    match cache.reload_table(file_path, &table_name, cached_doc, gen, cancelled, false) {
        Ok(()) => (true, false),
        Err(e) => {
            if e.downcast_ref::<cancel::CancelledError>().is_some() {
                (false, true)
            } else {
                (false, false)
            }
        }
    }
}

/// Load a file from disk into DuckDB if needed.
///
/// Returns (success, was_cancelled).
pub fn load_file_from_disk(
    cache: &mut DuckDbCache,
    file_path: &Path,
    delimiter: Option<u8>,
    no_headers: bool,
    encoding: Option<String>,
    cancelled: &AtomicBool,
) -> (bool, bool) {
    if cancel::check_esc(cancelled) {
        return (false, true);
    }

    if !file_path.exists() {
        return (true, false);
    }

    let table_name = crate::query::table_name_from_path(file_path);

    // Files loaded from disk always start at generation 0
    if !cache.needs_reload(file_path, 0) {
        return (true, false);
    }

    match crate::csv::Document::from_file_cancellable(
        file_path, delimiter, no_headers, encoding, cancelled,
    ) {
        Ok(d) => {
            if d.row_count() == 0 || d.column_count() == 0 {
                return (true, false);
            }
            match cache.reload_table(file_path, &table_name, &d, d.generation, cancelled, false) {
                Ok(()) => (true, false),
                Err(e) => {
                    if e.downcast_ref::<cancel::CancelledError>().is_some() {
                        (false, true)
                    } else {
                        (false, false)
                    }
                }
            }
        }
        Err(e) => {
            if e.downcast_ref::<cancel::CancelledError>().is_some() {
                (false, true)
            } else {
                (false, false)
            }
        }
    }
}

/// Execute SQL query and return result as a Document.
///
/// Strategy:
/// 1. Use DuckDB's COPY to write results directly to /tmp/<result>.csv
/// 2. Load that file as a lazy mmap-backed Document (minimal RAM)
/// 3. Drop all DuckDB tables to free memory
/// 4. Fallback: materialize in-memory for DML readback (small results)
///
/// Returns (result, was_cancelled, error_message).
pub fn execute_and_convert_query(
    cache: &mut DuckDbCache,
    query: &str,
    output_name: &str,
    cancelled: &AtomicBool,
    on_progress: &mut dyn FnMut(&str),
) -> (Option<Document>, bool, Option<String>) {
    let query_upper = query.trim().to_ascii_uppercase();
    let is_select = query_upper.starts_with("SELECT");

    // Skip COPY path for DML readback (bare SELECT * FROM table after INSERT/UPDATE/DELETE).
    // These are always small and the table name matches the output name.
    let result_table_upper = output_name
        .strip_suffix(".csv")
        .unwrap_or(output_name)
        .to_ascii_uppercase();
    let is_bare_select_from_same_table = is_select
        && (query_upper.trim() == format!("SELECT * FROM \"{}\"", result_table_upper)
            || query_upper.trim() == format!("SELECT * FROM {}", result_table_upper));

    // For SELECT queries: COPY results to /tmp file, load as lazy mmap document
    if is_select && !is_bare_select_from_same_table {
        let result_stem = output_name.strip_suffix(".csv").unwrap_or(output_name);
        let temp_path = std::env::temp_dir().join(format!(
            "lazycsv_{}_{}.csv",
            result_stem,
            std::process::id()
        ));
        let temp_str = temp_path.display().to_string().replace('\'', "''");
        let copy_sql = format!("COPY ({}) TO '{}' (HEADER, DELIMITER ',')", query, temp_str);

        if let Ok(row_count) = cache.conn().execute(&copy_sql, []) {
            let mb = std::fs::metadata(&temp_path)
                .map(|m| m.len() as f64 / (1024.0 * 1024.0))
                .unwrap_or(0.0);
            on_progress(&format!(
                "Loading {} rows ({:.0} MB)...",
                format_number(row_count),
                mb
            ));
            match crate::csv::row_storage::RowStorage::lazy_from_file(&temp_path, None, false) {
                Ok(storage) => {
                    let doc =
                        crate::csv::Document::from_storage(storage, output_name.to_string(), ',');
                    on_progress("Cleaning up...");
                    cache.drop_all_tables();
                    return (Some(doc), false, None);
                }
                Err(_) => {
                    let _ = std::fs::remove_file(&temp_path);
                }
            }
        }
    }

    // Fallback: materialize in-memory (DML readback or COPY failure)
    match crate::query::execute_query_to_document_cancellable(
        cache.conn(),
        query,
        "result.csv".to_string(),
        cancelled,
    ) {
        Ok(mut doc) => {
            doc.filename = output_name.to_string();
            (Some(doc), false, None)
        }
        Err(e) => {
            if e.downcast_ref::<cancel::CancelledError>().is_some() {
                (None, true, None)
            } else {
                (None, false, Some(format!("SQL error: {}", e)))
            }
        }
    }
}

/// File configuration for loading CSVs into DuckDB.
pub struct FileLoadConfig {
    pub delimiter: Option<u8>,
    pub no_headers: bool,
    pub encoding: Option<String>,
}

/// Load a single file from the session into DuckDB.
///
/// Handles current document, cached document, or loading from disk.
/// Returns true if cancelled.
pub fn load_session_file<'a>(
    cache: &mut DuckDbCache,
    file_path: &Path,
    current_doc: &Document,
    session_get_cached: impl FnOnce() -> Option<&'a Document>,
    config: FileLoadConfig,
    cancelled: &AtomicBool,
) -> bool {
    let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Load current document
    if filename == current_doc.filename {
        let (_, cancelled_flag) = load_current_document(cache, file_path, current_doc, cancelled);
        return cancelled_flag;
    }

    // Load cached document
    if let Some(cached_doc) = session_get_cached() {
        let (_, cancelled_flag) = load_cached_document(cache, file_path, cached_doc, cancelled);
        return cancelled_flag;
    }

    // Load from disk
    let (_, cancelled_flag) = load_file_from_disk(
        cache,
        file_path,
        config.delimiter,
        config.no_headers,
        config.encoding,
        cancelled,
    );
    cancelled_flag
}

// ============================================================================
// impl App — SQL execution methods
// ============================================================================

impl App {
    /// Execute a SQL query with cancellation support.
    /// Returns (Some(doc), false) on success, (None, true) if cancelled,
    /// (None, false) on query error.
    pub fn execute_sql_query_cancellable(
        &mut self,
        query: &str,
        output_name: &str,
        cancelled: &AtomicBool,
        on_progress: &mut dyn FnMut(&str),
    ) -> (Option<Document>, bool) {
        let query = crate::query::strip_csv_extensions(query);
        let query = query.as_str();

        let mut cache = self.duckdb_cache.take().unwrap_or_else(DuckDbCache::new);

        on_progress("Preparing database...");
        if cleanup_stale_tables(&mut cache, self.session.files(), cancelled) {
            self.duckdb_cache = Some(cache);
            return (None, true);
        }

        let all_files = self.session.files().to_vec();
        let referenced = crate::query::files_referenced_by_query(query, &all_files);
        let files: Vec<PathBuf> = referenced.into_iter().cloned().collect();
        let total_files = files.len();
        for (i, file_path) in files.into_iter().enumerate() {
            let file_label = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            if total_files > 1 {
                on_progress(&format!(
                    "Loading {} into database ({}/{})...",
                    file_label,
                    i + 1,
                    total_files
                ));
            } else {
                on_progress(&format!("Loading {} into database...", file_label));
            }
            let config = self.session.config();
            let file_config = FileLoadConfig {
                delimiter: config.delimiter,
                no_headers: config.no_headers,
                encoding: config.encoding.clone(),
            };
            let cancelled_flag = load_session_file(
                &mut cache,
                &file_path,
                &self.document,
                || self.session.cached_document(&file_path),
                file_config,
                cancelled,
            );
            if cancelled_flag {
                self.duckdb_cache = Some(cache);
                return (None, true);
            }
        }

        on_progress("Querying...");
        let (result_doc, cancelled_flag, error_msg) =
            execute_and_convert_query(&mut cache, query, output_name, cancelled, on_progress);

        self.duckdb_cache = Some(cache);

        if let Some(err) = error_msg {
            self.sql_error = Some(err);
            return (None, false);
        }

        if cancelled_flag {
            return (None, true);
        }

        self.sql_error = None;
        self.mode = Mode::Normal;
        (result_doc, false)
    }

    /// Execute a SQL DML statement (INSERT, UPDATE, DELETE, ALTER) against the current document.
    pub fn execute_sql_dml_cancellable(
        &mut self,
        query: &str,
        cancelled: &AtomicBool,
        on_progress: &mut dyn FnMut(&str),
    ) -> (bool, bool) {
        let query = crate::query::strip_csv_extensions(query);
        let query = query.as_str();

        let mut cache = self.duckdb_cache.take().unwrap_or_else(DuckDbCache::new);

        let doc_table_name = self
            .document
            .filename
            .strip_suffix(".csv")
            .or_else(|| self.document.filename.strip_suffix(".tsv"))
            .or_else(|| self.document.filename.strip_suffix(".txt"))
            .unwrap_or(&self.document.filename)
            .to_string();
        let file_path = self.current_file().clone();

        on_progress("Syncing data to database...");
        cache.force_reload_generation(&file_path);
        if self.document.row_count() > 0 && self.document.column_count() > 0 {
            match cache.reload_table(
                &file_path,
                &doc_table_name,
                &self.document,
                self.document.generation,
                cancelled,
                true,
            ) {
                Ok(()) => {}
                Err(e) => {
                    if e.downcast_ref::<cancel::CancelledError>().is_some() {
                        self.duckdb_cache = Some(cache);
                        return (false, true);
                    }
                    self.sql_error = Some(format!("Failed to load data: {}", e));
                    self.duckdb_cache = Some(cache);
                    return (false, false);
                }
            }
        }

        on_progress("Executing DML...");
        let escaped_table = doc_table_name.replace('"', "\"\"");
        match cache.conn().execute(query, []) {
            Ok(_) => {}
            Err(e) => {
                self.sql_error = Some(format!("SQL error: {}", e));
                self.duckdb_cache = Some(cache);
                return (false, false);
            }
        }

        on_progress("Exporting modified data...");
        let dml_unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_path = std::env::temp_dir().join(format!(
            "lazycsv_dml_{}_{}_{}.csv",
            doc_table_name,
            std::process::id(),
            dml_unique
        ));
        let temp_str = temp_path.display().to_string().replace('\'', "''");
        let copy_sql = format!(
            "COPY (SELECT * FROM \"{}\") TO '{}' (HEADER, DELIMITER ',')",
            escaped_table, temp_str
        );

        if cache.conn().execute_batch(&copy_sql).is_err() {
            self.sql_error = Some("Failed to export modified data".to_string());
            self.duckdb_cache = Some(cache);
            return (false, false);
        }

        on_progress("Loading modified data...");
        let storage =
            match crate::csv::row_storage::RowStorage::lazy_from_file(&temp_path, None, false) {
                Ok(s) => s,
                Err(e) => {
                    self.sql_error = Some(format!("Failed to load modified data: {}", e));
                    self.duckdb_cache = Some(cache);
                    return (false, false);
                }
            };

        cache.drop_all_tables();
        self.duckdb_cache = Some(cache);

        {
            self.document.storage = storage;
            self.document.is_dirty = true;
            self.document.generation += 1;
            self.document.xlsx_formulas = vec![];

            self.view_state.table_state.select(Some(1));
            self.view_state.column_scroll_offset = 0;
            self.view_state.selected_column = crate::domain::position::ColIndex::new(0);

            self.sql_error = None;
            self.mode = Mode::Normal;
            (true, false)
        }
    }

    /// Generate a unique output filename for SQL query results
    pub fn generate_output_filename(&self) -> String {
        let existing: std::collections::HashSet<String> = self
            .session
            .files()
            .iter()
            .filter_map(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        let base = "result.csv".to_string();
        if !existing.contains(&base) {
            return base;
        }
        let mut i = 1;
        loop {
            let name = format!("result{}.csv", i);
            if !existing.contains(&name) {
                return name;
            }
            i += 1;
        }
    }
}
