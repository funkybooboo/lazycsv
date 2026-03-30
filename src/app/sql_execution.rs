//! Helper functions for SQL query execution with CSV loading.
//!
//! Extracted from execute_sql_query_cancellable to keep the main function under 50 lines.

use crate::cancel;
use crate::csv::Document;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use super::DuckDbCache;

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
