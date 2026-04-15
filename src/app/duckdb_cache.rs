//! Cached in-memory DuckDB connection with generation tracking.

use crate::csv::Document;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Cached in-memory DuckDB connection with generation tracking.
/// Keeps loaded tables across query executions so unchanged data isn't reloaded.
pub struct DuckDbCache {
    conn: duckdb::Connection,
    /// Map from file path -> Document generation loaded in that table/view.
    loaded_generations: HashMap<PathBuf, u64>,
    /// Paths loaded as VIEWs (vs TABLEs). Views are zero-cost to create;
    /// DuckDB scans the CSV at query time with column/predicate pushdown.
    view_paths: HashSet<PathBuf>,
}

impl Drop for DuckDbCache {
    fn drop(&mut self) {
        // Clean up the spill directory created in DuckDbCache::new().
        let spill_dir = std::env::temp_dir().join(format!("lazycsv_spill_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&spill_dir);
    }
}

impl std::fmt::Debug for DuckDbCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DuckDbCache")
            .field("loaded_generations", &self.loaded_generations)
            .finish_non_exhaustive()
    }
}

impl DuckDbCache {
    /// Create a new in-memory DuckDB connection.
    ///
    /// A per-process spill directory is configured so DuckDB can overflow to disk
    /// instead of OOMing when large tables are materialized (e.g. for DML).
    pub(super) fn new() -> Self {
        let conn = duckdb::Connection::open_in_memory().expect("Failed to open DuckDB");

        // Set a spill directory so DuckDB can overflow large materialized tables
        // to disk rather than crashing with out-of-memory.
        let spill_dir = std::env::temp_dir().join(format!("lazycsv_spill_{}", std::process::id()));
        if std::fs::create_dir_all(&spill_dir).is_ok() {
            let dir_str = spill_dir.display().to_string().replace('\'', "''");
            let _ = conn.execute_batch(&format!("SET temp_directory = '{}'", dir_str));
        }

        DuckDbCache {
            conn,
            loaded_generations: HashMap::new(),
            view_paths: HashSet::new(),
        }
    }

    /// Get reference to loaded generations map.
    pub(crate) fn loaded_generations(&self) -> &HashMap<PathBuf, u64> {
        &self.loaded_generations
    }

    /// Check whether the table for `path` needs to be reloaded.
    /// Checks generation tracking AND verifies the table exists in DuckDB
    /// (tables may have been dropped to free memory).
    pub(crate) fn needs_reload(&self, path: &Path, generation: u64) -> bool {
        match self.loaded_generations.get(path) {
            Some(&cached_gen) if cached_gen == generation => {
                // Generation matches — verify table actually exists in DuckDB
                let table_name = crate::query::table_name_from_path(path);
                let exists = self
                    .conn
                    .prepare(&format!(
                        "SELECT 1 FROM information_schema.tables WHERE table_name = '{}'",
                        table_name.replace('\'', "''")
                    ))
                    .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
                    .is_ok();
                !exists
            }
            Some(_) => true,
            None => true,
        }
    }

    /// Load a table into DuckDB for querying.
    ///
    /// Strategy:
    /// 1. If doc is clean and file exists on disk → DuckDB reads it directly via read_csv
    /// 2. If doc is dirty (edited) → write TUI contents to /tmp, DuckDB reads that
    /// 3. Fallback → row-by-row INSERT (slowest, for small datasets)
    pub(crate) fn reload_table(
        &mut self,
        path: &Path,
        table_name: &str,
        doc: &Document,
        generation: u64,
        cancelled: &AtomicBool,
        // When true, always create a TABLE (required for DML — VIEWs are read-only).
        // When false, create a VIEW for clean on-disk files (zero-cost load).
        force_table: bool,
    ) -> std::result::Result<(), anyhow::Error> {
        let escaped = table_name.replace('"', "\"\"");

        if !self.needs_reload(path, generation) {
            return Ok(());
        }

        // Drop any existing table or view for this name.
        let is_view = self.view_paths.contains(path);
        if is_view {
            let _ = self
                .conn
                .execute(&format!("DROP VIEW IF EXISTS \"{}\"", escaped), []);
        } else {
            let _ = self
                .conn
                .execute(&format!("DROP TABLE IF EXISTS \"{}\"", escaped), []);
        }
        self.loaded_generations.remove(path);
        self.view_paths.remove(path);

        // Strategy 1: Clean file on disk.
        // SELECT queries → VIEW (zero-cost; DuckDB scans at query time with pushdown).
        // DML queries → TABLE (DML cannot target a VIEW).
        if !doc.is_dirty && path.is_file() {
            let ok = if force_table {
                self.load_via_table(&escaped, &path.display().to_string())
            } else {
                self.load_via_view(&escaped, &path.display().to_string())
            };
            if ok {
                self.loaded_generations
                    .insert(path.to_path_buf(), generation);
                if !force_table {
                    self.view_paths.insert(path.to_path_buf());
                }
                return Ok(());
            }
        }

        // Strategy 2: Existing temp file (from a previous query result).
        let temp_result =
            std::env::temp_dir().join(format!("lazycsv_{}_{}.csv", table_name, std::process::id()));
        if temp_result.is_file() {
            let ok = if force_table {
                self.load_via_table(&escaped, &temp_result.display().to_string())
            } else {
                self.load_via_view(&escaped, &temp_result.display().to_string())
            };
            if ok {
                self.loaded_generations
                    .insert(path.to_path_buf(), generation);
                if !force_table {
                    self.view_paths.insert(path.to_path_buf());
                }
                return Ok(());
            }
        }

        // Strategy 3: Write doc to temp file, then DuckDB reads it
        let temp_path = std::env::temp_dir().join(format!(
            "lazycsv_dirty_{}_{}.csv",
            std::process::id(),
            table_name
        ));
        if crate::csv::write_csv_atomic(doc, &temp_path, doc.delimiter).is_ok() {
            // Must use TABLE here: the temp file is deleted immediately after loading.
            let ok = self.load_via_table(&escaped, &temp_path.display().to_string());
            let _ = std::fs::remove_file(&temp_path);
            if ok {
                self.loaded_generations
                    .insert(path.to_path_buf(), generation);
                return Ok(());
            }
        }

        // Strategy 4: Fallback row-by-row INSERT (small datasets)

        crate::query::load_csv_into_duckdb_cancellable(&self.conn, doc, table_name, cancelled)?;
        self.loaded_generations
            .insert(path.to_path_buf(), generation);
        Ok(())
    }

    /// Helper: load a CSV file into a DuckDB TABLE via read_csv. Returns true on success.
    /// Materializes all data into memory — use for dirty docs whose temp file will be deleted.
    fn load_via_table(&self, escaped_table: &str, file_path: &str) -> bool {
        let path_str = file_path.replace('\'', "''");
        let sql = format!(
            "CREATE TABLE \"{}\" AS SELECT * FROM read_csv('{}')",
            escaped_table, path_str
        );
        self.conn.execute_batch(&sql).is_ok()
    }

    /// Helper: create a DuckDB VIEW over a CSV file. Returns true on success.
    /// Zero-cost — no data is read upfront; DuckDB scans at query time with
    /// column and predicate pushdown.
    fn load_via_view(&self, escaped_table: &str, file_path: &str) -> bool {
        let path_str = file_path.replace('\'', "''");
        let sql = format!(
            "CREATE VIEW \"{}\" AS SELECT * FROM read_csv('{}')",
            escaped_table, path_str
        );
        self.conn.execute_batch(&sql).is_ok()
    }

    /// Remove a single table or view from the cache.
    pub(crate) fn remove_table(&mut self, path: &Path, table_name: &str) {
        let escaped = table_name.replace('"', "\"\"");
        if self.view_paths.contains(path) {
            let _ = self
                .conn
                .execute(&format!("DROP VIEW IF EXISTS \"{}\"", escaped), []);
        } else {
            let _ = self
                .conn
                .execute(&format!("DROP TABLE IF EXISTS \"{}\"", escaped), []);
        }
        self.loaded_generations.remove(path);
        self.view_paths.remove(path);
    }

    /// Drop all tables and views from DuckDB to free memory.
    /// All generation tracking is cleared so objects will be recreated on next use.
    pub(crate) fn drop_all_tables(&mut self) {
        if let Ok(mut stmt) = self.conn.prepare(
            "SELECT table_name, table_type FROM information_schema.tables WHERE table_schema = 'main'"
        ) {
            let objects: Vec<(String, String)> = stmt
                .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default();
            for (name, obj_type) in objects {
                let escaped = name.replace('"', "\"\"");
                if obj_type == "VIEW" {
                    let _ = self.conn.execute(&format!("DROP VIEW IF EXISTS \"{}\"", escaped), []);
                } else {
                    let _ = self.conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", escaped), []);
                }
            }
        }
        self.loaded_generations.clear();
        self.view_paths.clear();
    }

    /// Force a table/view to be reloaded on next use by removing its generation entry.
    pub(crate) fn force_reload_generation(&mut self, path: &Path) {
        self.loaded_generations.remove(path);
        self.view_paths.remove(path);
    }

    /// Get a reference to the underlying connection.
    pub(crate) fn conn(&self) -> &duckdb::Connection {
        &self.conn
    }
}
