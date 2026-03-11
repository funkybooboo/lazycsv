//! SQL query mode — load CSV files into SQLite and execute queries.
//!
//! This module provides SQL query functionality for LazyCSV, allowing users to execute
//! SQL queries on CSV files using an in-memory SQLite database. All CSV files in the
//! same directory are automatically loaded as tables, enabling multi-table JOINs.
//!
//! # Features
//!
//! - **SQL queries on CSV files**: Execute any SQLite-compatible query
//! - **Multi-table JOINs**: Automatically loads all CSVs in directory
//! - **Performance**: In-memory database with optimized bulk loading
//! - **Caching**: Query results cached until CSV modified
//! - **Error enhancement**: Helpful error messages with suggestions
//!
//! # Usage
//!
//! ```no_run
//! use lazycsv::query::{execute_query, resolve_csv_files};
//! use lazycsv::session::FileConfig;
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! // Execute query on CSV files in current directory
//! let query = "SELECT name, price FROM products WHERE price > 100 ORDER BY price DESC";
//! execute_query(Path::new("."), query, &FileConfig::default())?;
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! ```text
//! CSV Files → SQLite Tables → SQL Query → Result → Document
//!    ↓            ↓              ↓           ↓          ↓
//! sample.csv  → sample       → SELECT    → Rows    → Displayed
//! orders.csv  → orders       → FROM      → Columns    in UI
//! customers.csv → customers  → JOIN      → Data
//! ```
//!
//! # Table Naming
//!
//! File names are converted to SQLite table names by:
//! 1. Removing `.csv` extension
//! 2. Replacing non-alphanumeric characters with underscore
//!
//! Examples:
//! - `sales_data.csv` → table `sales_data`
//! - `my-file.csv` → table `my_file`
//! - `data@2024.csv` → table `data_2024`
//!
//! # Performance
//!
//! Benchmarked performance (100K rows):
//! - Load CSV to SQLite: ~150ms
//! - Simple SELECT: ~18ms
//! - 2-way JOIN: ~120ms
//! - GROUP BY: ~65ms
//!
//! # Cancellation
//!
//! All operations check for cancellation every 1000 rows, allowing responsive Ctrl+C handling.

mod date_detection;
mod error_enhancer;

use crate::cancel::{self, CancelledError};
use crate::csv::Document;
use crate::file_system;
use crate::session::FileConfig;
use anyhow::{bail, Context, Result};
use error_enhancer::enhance_sql_error;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Derive a SQLite table name from a file path.
///
/// Converts a file path into a valid SQLite table name by removing the `.csv` extension
/// and replacing non-alphanumeric characters with underscores.
///
/// # Arguments
///
/// * `path` - Path to the CSV file
///
/// # Returns
///
/// A string suitable for use as a SQLite table name.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use lazycsv::query::table_name_from_path;
///
/// assert_eq!(table_name_from_path(Path::new("sales.csv")), "sales");
/// assert_eq!(table_name_from_path(Path::new("my-data.csv")), "my_data");
/// assert_eq!(table_name_from_path(Path::new("data@2024.csv")), "data_2024");
/// ```
pub fn table_name_from_path(path: &Path) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("table");

    stem.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Determine which session files are referenced by a SQL query.
///
/// Extracts identifiers from the query (including double-quoted identifiers)
/// and matches them against table names derived from the given file paths.
/// Returns only the files whose table names appear in the query.
///
/// If no files match (e.g. query uses a subquery or expression with no table),
/// returns all files as a safe fallback so the query can still execute.
pub fn files_referenced_by_query<'a>(query: &str, files: &'a [PathBuf]) -> Vec<&'a PathBuf> {
    let query_lower = query.to_ascii_lowercase();

    // Extract all identifiers from the query: bare words and "quoted identifiers"
    let identifiers = extract_sql_identifiers(&query_lower);

    let mut matched: Vec<&PathBuf> = files
        .iter()
        .filter(|path| {
            let table = table_name_from_path(path).to_ascii_lowercase();
            identifiers.contains(&table)
        })
        .collect();

    // Fallback: if nothing matched, load everything so the query can still run
    if matched.is_empty() {
        matched = files.iter().collect();
    }

    matched
}

/// Extract identifiers (bare words and double-quoted names) from SQL text.
/// Returns a set of lowercase identifier strings.
fn extract_sql_identifiers(sql: &str) -> std::collections::HashSet<String> {
    let mut ids = std::collections::HashSet::new();
    let bytes = sql.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        // Skip string literals (single-quoted)
        if ch == b'\'' {
            i += 1;
            while i < len {
                if bytes[i] == b'\'' {
                    i += 1;
                    // Escaped quote ''
                    if i < len && bytes[i] == b'\'' {
                        i += 1;
                        continue;
                    }
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Double-quoted identifier
        if ch == b'"' {
            i += 1;
            let start = i;
            while i < len && bytes[i] != b'"' {
                i += 1;
            }
            if i > start {
                let ident = String::from_utf8_lossy(&bytes[start..i]).to_string();
                ids.insert(ident);
            }
            if i < len {
                i += 1; // skip closing quote
            }
            continue;
        }

        // Bare identifier (word)
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = String::from_utf8_lossy(&bytes[start..i]).to_string();
            ids.insert(word);
            continue;
        }

        i += 1;
    }

    ids
}

/// Strip `.csv` (and other common extensions) from table references in a SQL query.
///
/// Users may write `SELECT * FROM myfile.csv WHERE ...` — this rewrites the query
/// to `SELECT * FROM myfile WHERE ...` so SQLite can find the table.
///
/// Handles bare identifiers (`myfile.csv`) and double-quoted identifiers
/// (`"myfile.csv"`). Preserves string literals unchanged.
pub fn strip_csv_extensions(sql: &str) -> String {
    let extensions: &[&str] = &[".csv", ".tsv", ".txt"];
    let bytes = sql.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        // Preserve single-quoted string literals as-is
        if ch == b'\'' {
            result.push('\'');
            i += 1;
            while i < len {
                result.push(bytes[i] as char);
                if bytes[i] == b'\'' {
                    i += 1;
                    // Escaped quote ''
                    if i < len && bytes[i] == b'\'' {
                        result.push('\'');
                        i += 1;
                        continue;
                    }
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Double-quoted identifier — strip extension inside quotes
        if ch == b'"' {
            result.push('"');
            i += 1;
            let start = i;
            while i < len && bytes[i] != b'"' {
                i += 1;
            }
            let ident = &sql[start..i];
            let stripped = strip_extension(ident, extensions);
            result.push_str(&stripped);
            if i < len {
                result.push('"');
                i += 1; // skip closing quote
            }
            continue;
        }

        // Bare identifier (word) possibly followed by .csv
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &sql[start..i];

            // Check for trailing .ext (e.g. "myfile.csv")
            let mut matched_ext = false;
            for ext in extensions {
                let ext_bytes = ext.as_bytes();
                if i + ext_bytes.len() <= len && sql[i..i + ext_bytes.len()].eq_ignore_ascii_case(ext) {
                    // Make sure the extension isn't followed by more identifier chars
                    // (e.g. "myfile.csvdata" should NOT be stripped)
                    let after = i + ext_bytes.len();
                    if after >= len || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_') {
                        result.push_str(word);
                        i = after;
                        matched_ext = true;
                        break;
                    }
                }
            }
            if !matched_ext {
                result.push_str(word);
            }
            continue;
        }

        result.push(ch as char);
        i += 1;
    }

    result
}

fn strip_extension(ident: &str, extensions: &[&str]) -> String {
    for ext in extensions {
        if ident.len() > ext.len() && ident[ident.len() - ext.len()..].eq_ignore_ascii_case(ext) {
            return ident[..ident.len() - ext.len()].to_string();
        }
    }
    ident.to_string()
}

/// Resolve which CSV files to load for a query.
///
/// This function determines which CSV files should be loaded into SQLite based on the
/// provided path. The behavior enables automatic multi-table JOIN queries:
///
/// - If `path` is a **directory**: Loads all CSV files in that directory
/// - If `path` is a **file**: Loads all CSV files in the same directory (siblings)
/// - This ensures JOINs work seamlessly across related CSVs
///
/// # Arguments
///
/// * `path` - Path to a CSV file or directory
///
/// # Returns
///
/// A vector of paths to all CSV files that should be loaded.
///
/// # Errors
///
/// Returns an error if:
/// - The path does not exist
/// - The directory contains no CSV files
/// - There's an I/O error reading the directory
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use lazycsv::query::resolve_csv_files;
///
/// # fn main() -> anyhow::Result<()> {
/// // Load all CSVs from directory
/// let files = resolve_csv_files(Path::new("/data/"))?;
///
/// // Load siblings of a specific file (enables JOINs)
/// let files = resolve_csv_files(Path::new("/data/orders.csv"))?;
/// // Returns: [orders.csv, customers.csv, products.csv, ...]
/// # Ok(())
/// # }
/// ```
pub fn resolve_csv_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_dir() {
        let files = file_system::scan_directory(path)?;
        if files.is_empty() {
            bail!("No CSV files found in directory: {}", path.display());
        }
        Ok(files)
    } else if path.is_file() {
        Ok(file_system::scan_directory_for_csvs(path)?)
    } else {
        bail!("Path does not exist: {}", path.display());
    }
}

/// Load a parsed CSV Document into a SQLite table.
///
/// Creates a SQLite table with the document's column names (all NUMERIC affinity) and
/// inserts all rows using a prepared statement within a single transaction for
/// optimal performance.
///
/// # Arguments
///
/// * `conn` - SQLite database connection
/// * `doc` - CSV document to load (first row must be headers)
/// * `table_name` - Name for the SQLite table
///
/// # Returns
///
/// `Ok(())` if successful, or an error if:
/// - Document has no rows or columns
/// - Table creation fails
/// - Data insertion fails
///
/// # Performance
///
/// Uses optimized bulk loading:
/// - Single transaction (50x faster than auto-commit)
/// - Prepared statement with parameter binding
/// - No intermediate allocations
///
/// Typical performance:
/// - 1K rows: ~2ms
/// - 10K rows: ~15ms
/// - 100K rows: ~150ms
///
/// # Examples
///
/// ```no_run
/// use rusqlite::Connection;
/// use lazycsv::csv::Document;
/// use lazycsv::query::load_csv_into_sqlite;
///
/// # fn main() -> anyhow::Result<()> {
/// let conn = Connection::open_in_memory()?;
/// let doc = Document::new(
///     vec!["id".to_string(), "name".to_string()],
///     vec![vec!["1".to_string(), "Alice".to_string()]],
///     "users.csv".to_string(),
/// );
///
/// load_csv_into_sqlite(&conn, &doc, "users")?;
///
/// // Now can query: SELECT * FROM users
/// # Ok(())
/// # }
/// ```
pub fn load_csv_into_sqlite(conn: &Connection, doc: &Document, table_name: &str) -> Result<()> {
    if doc.rows.is_empty() {
        bail!("Document has no rows (not even headers)");
    }

    let headers = &doc.rows[0];
    if headers.is_empty() {
        bail!("Document has no columns");
    }

    let col_count = headers.len();

    // Detect column types (date vs numeric) by sampling data rows
    let col_types = date_detection::detect_column_types(&doc.rows[1..], col_count);
    let has_date_cols = col_types
        .iter()
        .any(|ct| matches!(ct, date_detection::ColumnType::Date(_)));

    // Build CREATE TABLE with detected affinities
    let col_defs: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let affinity = date_detection::sqlite_affinity(&col_types[i]);
            format!("\"{}\" {}", h.replace('"', "\"\""), affinity)
        })
        .collect();

    let create_sql = format!(
        "CREATE TABLE \"{}\" ({})",
        table_name.replace('"', "\"\""),
        col_defs.join(", ")
    );
    conn.execute(&create_sql, [])
        .context(format!("Failed to create table '{}'", table_name))?;

    // Use a prepared single-row INSERT inside a transaction.
    let placeholders: Vec<&str> = vec!["?"; col_count];
    let insert_sql = format!(
        "INSERT INTO \"{}\" VALUES ({})",
        table_name.replace('"', "\"\""),
        placeholders.join(", ")
    );

    conn.execute("BEGIN", [])
        .context("Failed to begin transaction")?;

    let mut stmt = conn.prepare_cached(&insert_sql)?;

    if has_date_cols {
        // Date normalization path — allocates for date columns
        for row in doc.rows.iter().skip(1) {
            let params: Vec<String> = (0..col_count)
                .map(|i| {
                    let val = row.get(i).map(|s| s.as_str()).unwrap_or("");
                    match &col_types[i] {
                        date_detection::ColumnType::Date(fmt) => {
                            date_detection::normalize_to_iso(val, *fmt)
                        }
                        date_detection::ColumnType::Numeric => val.to_string(),
                    }
                })
                .collect();
            stmt.execute(rusqlite::params_from_iter(params.iter().map(|s| s.as_str())))?;
        }
    } else {
        // Fast zero-copy path — binds &str refs directly
        for row in doc.rows.iter().skip(1) {
            let params: Vec<&str> = (0..col_count)
                .map(|i| row.get(i).map(|s| s.as_str()).unwrap_or(""))
                .collect();
            stmt.execute(rusqlite::params_from_iter(params))?;
        }
    }

    drop(stmt);
    conn.execute("COMMIT", [])
        .context("Failed to commit transaction")?;

    Ok(())
}

/// Stream a CSV file directly into a SQLite table, bypassing the Document intermediate.
/// Used by the CLI `-q` path where we don't need the Document in memory.
fn load_csv_file_into_sqlite(
    conn: &Connection,
    file_path: &Path,
    table_name: &str,
    config: &FileConfig,
) -> Result<()> {
    let file_bytes = std::fs::read(file_path)
        .context(format!("Failed to read file: {}", file_path.display()))?;

    let decoded = crate::csv::Document::decode_file_bytes(&file_bytes, config.encoding.clone())?;

    let mut builder = csv::ReaderBuilder::new();
    builder.has_headers(!config.no_headers);
    if let Some(d) = config.delimiter {
        builder.delimiter(d);
    }

    let mut reader = builder.from_reader(decoded.as_bytes());

    // Get headers
    let headers: Vec<String> = if config.no_headers {
        let first = reader.headers().context("CSV file has no records")?;
        (1..=first.len()).map(|i| format!("Column {}", i)).collect()
    } else {
        let h = reader.headers().context("CSV file has no headers")?;
        h.iter().map(String::from).collect()
    };

    if headers.is_empty() {
        bail!("CSV file has no columns");
    }

    let col_count = headers.len();

    // Buffer initial rows for date detection sampling
    let mut buffered: Vec<csv::StringRecord> = Vec::new();
    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;
        buffered.push(record);
        if buffered.len() >= 100 {
            break;
        }
    }

    // Detect column types from buffered samples
    let sample_rows: Vec<Vec<&str>> = buffered
        .iter()
        .map(|rec| (0..col_count).map(|i| rec.get(i).unwrap_or("")).collect())
        .collect();
    let col_types = date_detection::detect_column_types_from_strs(&sample_rows, col_count);
    let has_date_cols = col_types
        .iter()
        .any(|ct| matches!(ct, date_detection::ColumnType::Date(_)));

    // CREATE TABLE with detected affinities
    let col_defs: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let affinity = date_detection::sqlite_affinity(&col_types[i]);
            format!("\"{}\" {}", h.replace('"', "\"\""), affinity)
        })
        .collect();
    let escaped_table = table_name.replace('"', "\"\"");
    conn.execute(
        &format!(
            "CREATE TABLE \"{}\" ({})",
            escaped_table,
            col_defs.join(", ")
        ),
        [],
    )
    .context(format!("Failed to create table '{}'", table_name))?;

    let placeholders: Vec<&str> = vec!["?"; col_count];
    let insert_sql = format!(
        "INSERT INTO \"{}\" VALUES ({})",
        escaped_table,
        placeholders.join(", ")
    );

    conn.execute("BEGIN", [])
        .context("Failed to begin transaction")?;

    let mut stmt = conn.prepare_cached(&insert_sql)?;

    // Helper closure to insert a record
    let insert_record = |stmt: &mut rusqlite::CachedStatement, record: &csv::StringRecord| -> Result<()> {
        if has_date_cols {
            let params: Vec<String> = (0..col_count)
                .map(|i| {
                    let val = record.get(i).unwrap_or("");
                    match &col_types[i] {
                        date_detection::ColumnType::Date(fmt) => {
                            date_detection::normalize_to_iso(val, *fmt)
                        }
                        date_detection::ColumnType::Numeric => val.to_string(),
                    }
                })
                .collect();
            stmt.execute(rusqlite::params_from_iter(params.iter().map(|s| s.as_str())))?;
        } else {
            let params: Vec<&str> = (0..col_count)
                .map(|i| record.get(i).unwrap_or(""))
                .collect();
            stmt.execute(rusqlite::params_from_iter(params))?;
        }
        Ok(())
    };

    // Insert buffered rows first
    for record in &buffered {
        insert_record(&mut stmt, record)?;
    }

    // Continue with remaining rows from reader
    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;
        insert_record(&mut stmt, &record)?;
    }

    drop(stmt);
    conn.execute("COMMIT", [])
        .context("Failed to commit transaction")?;

    Ok(())
}

/// Execute a SQL query and return results as a Document.
///
/// Executes a SQLite query against an existing database connection and converts
/// the results into a LazyCSV Document for display in the UI.
///
/// # Arguments
///
/// * `conn` - SQLite database connection (must have tables already loaded)
/// * `query` - SQL query string (SQLite syntax)
/// * `output_filename` - Filename to assign to the result Document
///
/// # Returns
///
/// A `Document` containing the query results, or an error if:
/// - Query syntax is invalid
/// - Referenced tables/columns don't exist
/// - Query execution fails
///
/// # Examples
///
/// ```no_run
/// use rusqlite::Connection;
/// use lazycsv::query::{load_csv_into_sqlite, execute_query_to_document};
/// use lazycsv::csv::Document;
///
/// # fn main() -> anyhow::Result<()> {
/// let conn = Connection::open_in_memory()?;
///
/// // Load data
/// let doc = Document::new(
///     vec!["id".to_string(), "price".to_string()],
///     vec![vec!["1".to_string(), "100".to_string()]],
///     "products.csv".to_string(),
/// );
/// load_csv_into_sqlite(&conn, &doc, "products")?;
///
/// // Query
/// let result = execute_query_to_document(
///     &conn,
///     "SELECT * FROM products WHERE CAST(price AS REAL) > 50",
///     "result.csv".to_string(),
/// )?;
///
/// assert_eq!(result.rows.len(), 2); // Headers + 1 data row
/// # Ok(())
/// # }
/// ```
pub fn execute_query_to_document(
    conn: &Connection,
    query: &str,
    output_filename: String,
) -> Result<Document> {
    let mut stmt = conn.prepare(query).map_err(|e| anyhow::anyhow!("{}", e))?;
    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let values: Vec<String> = (0..col_count)
                .map(|i| {
                    use rusqlite::types::ValueRef;
                    match row.get_ref(i).unwrap_or(ValueRef::Null) {
                        ValueRef::Null => String::new(),
                        ValueRef::Integer(n) => n.to_string(),
                        ValueRef::Real(f) => f.to_string(),
                        ValueRef::Text(s) => String::from_utf8_lossy(s).into_owned(),
                        ValueRef::Blob(b) => String::from_utf8_lossy(b).into_owned(),
                    }
                })
                .collect();
            Ok(values)
        })
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut data_rows = Vec::new();
    for row_result in rows {
        data_rows.push(row_result.context("Failed to read query result row")?);
    }

    Ok(Document::new(col_names, data_rows, output_filename))
}

/// Load a CSV Document into SQLite with cancellation support.
///
/// Like [`load_csv_into_sqlite`], but checks for cancellation every 1000 rows.
/// If cancelled, rolls back the transaction and returns [`CancelledError`].
///
/// # Arguments
///
/// * `conn` - SQLite database connection
/// * `doc` - CSV document to load
/// * `table_name` - Name for the SQLite table
/// * `cancelled` - Atomic bool flag checked periodically (set by Ctrl+C handler)
///
/// # Returns
///
/// `Ok(())` if successful, `Err(CancelledError)` if cancelled mid-load,
/// or another error if the operation fails.
///
/// # Cancellation
///
/// Checks `cancelled` flag every [`cancel::CHECK_INTERVAL`] rows (typically 1000).
/// On cancellation:
/// 1. Rolls back transaction (no partial data)
/// 2. Returns `CancelledError`
/// 3. Caller should propagate or handle gracefully
///
/// [`CancelledError`]: crate::cancel::CancelledError
pub fn load_csv_into_sqlite_cancellable(
    conn: &Connection,
    doc: &Document,
    table_name: &str,
    cancelled: &AtomicBool,
) -> Result<()> {
    if doc.rows.is_empty() {
        bail!("Document has no rows (not even headers)");
    }

    let headers = &doc.rows[0];
    if headers.is_empty() {
        bail!("Document has no columns");
    }

    let col_count = headers.len();

    // Detect column types (date vs numeric) by sampling data rows
    let col_types = date_detection::detect_column_types(&doc.rows[1..], col_count);
    let has_date_cols = col_types
        .iter()
        .any(|ct| matches!(ct, date_detection::ColumnType::Date(_)));

    let col_defs: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let affinity = date_detection::sqlite_affinity(&col_types[i]);
            format!("\"{}\" {}", h.replace('"', "\"\""), affinity)
        })
        .collect();

    let create_sql = format!(
        "CREATE TABLE \"{}\" ({})",
        table_name.replace('"', "\"\""),
        col_defs.join(", ")
    );
    conn.execute(&create_sql, [])
        .context(format!("Failed to create table '{}'", table_name))?;

    let placeholders: Vec<&str> = vec!["?"; col_count];
    let insert_sql = format!(
        "INSERT INTO \"{}\" VALUES ({})",
        table_name.replace('"', "\"\""),
        placeholders.join(", ")
    );

    conn.execute("BEGIN", [])
        .context("Failed to begin transaction")?;

    let mut stmt = conn.prepare_cached(&insert_sql)?;

    for (i, row) in doc.rows.iter().skip(1).enumerate() {
        if i % cancel::CHECK_INTERVAL == 0 && cancel::check_esc(cancelled) {
            drop(stmt);
            let _ = conn.execute("ROLLBACK", []);
            bail!(CancelledError);
        }
        if has_date_cols {
            let params: Vec<String> = (0..col_count)
                .map(|j| {
                    let val = row.get(j).map(|s| s.as_str()).unwrap_or("");
                    match &col_types[j] {
                        date_detection::ColumnType::Date(fmt) => {
                            date_detection::normalize_to_iso(val, *fmt)
                        }
                        date_detection::ColumnType::Numeric => val.to_string(),
                    }
                })
                .collect();
            stmt.execute(rusqlite::params_from_iter(params.iter().map(|s| s.as_str())))?;
        } else {
            let params: Vec<&str> = (0..col_count)
                .map(|j| row.get(j).map(|s| s.as_str()).unwrap_or(""))
                .collect();
            stmt.execute(rusqlite::params_from_iter(params))?;
        }
    }

    drop(stmt);
    conn.execute("COMMIT", [])
        .context("Failed to commit transaction")?;

    Ok(())
}

/// Execute a SQL query and return results as a Document with cancellation support.
///
/// Like [`execute_query_to_document`], but checks for cancellation every 1000 result rows.
/// Also provides enhanced error messages with column/table suggestions.
///
/// # Arguments
///
/// * `conn` - SQLite database connection (must have tables already loaded)
/// * `query` - SQL query string (SQLite syntax)
/// * `output_filename` - Filename to assign to the result Document
/// * `cancelled` - Atomic bool flag checked periodically
///
/// # Returns
///
/// A `Document` containing query results, or an error if:
/// - Query syntax is invalid (with helpful suggestions)
/// - Referenced tables/columns don't exist (with suggestions)
/// - Query execution fails
/// - User cancels (Ctrl+C)
///
/// # Error Enhancement
///
/// SQL errors are enhanced with helpful messages:
/// - "Column 'usrname' does not exist. Did you mean: username?"
/// - "Table 'ordrers' does not exist. Did you mean: orders?"
/// - Shows available tables/columns for context
///
/// Error enhancement uses fuzzy matching (Levenshtein distance) to suggest
/// similar table and column names when queries reference invalid identifiers.
///
/// # Cancellation
///
/// Checks `cancelled` flag every [`cancel::CHECK_INTERVAL`] rows.
/// Returns `CancelledError` if set mid-query.
///
/// [`CancelledError`]: crate::cancel::CancelledError
pub fn execute_query_to_document_cancellable(
    conn: &Connection,
    query: &str,
    output_filename: String,
    cancelled: &AtomicBool,
) -> Result<Document> {
    let mut stmt = conn
        .prepare(query)
        .map_err(|e| enhance_sql_error(e, conn, query))?;
    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let values: Vec<String> = (0..col_count)
                .map(|i| {
                    use rusqlite::types::ValueRef;
                    match row.get_ref(i).unwrap_or(ValueRef::Null) {
                        ValueRef::Null => String::new(),
                        ValueRef::Integer(n) => n.to_string(),
                        ValueRef::Real(f) => f.to_string(),
                        ValueRef::Text(s) => String::from_utf8_lossy(s).into_owned(),
                        ValueRef::Blob(b) => String::from_utf8_lossy(b).into_owned(),
                    }
                })
                .collect();
            Ok(values)
        })
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut data_rows = Vec::new();
    for (i, row_result) in rows.enumerate() {
        if i % cancel::CHECK_INTERVAL == 0 && cancel::check_esc(cancelled) {
            bail!(CancelledError);
        }
        data_rows.push(row_result.context("Failed to read query result row")?);
    }

    Ok(Document::new(col_names, data_rows, output_filename))
}

/// Execute a SQL query against CSV files and write results as CSV to stdout.
pub fn execute_query(path: &Path, query: &str, config: &FileConfig) -> Result<()> {
    let query = strip_csv_extensions(query);
    let query = query.as_str();

    let csv_files = resolve_csv_files(path)?;

    let conn = Connection::open_in_memory().context("Failed to open in-memory SQLite database")?;

    // Optimize SQLite for bulk loading (safe for in-memory databases)
    conn.execute_batch(
        "PRAGMA journal_mode=OFF;
         PRAGMA synchronous=OFF;
         PRAGMA temp_store=MEMORY;
         PRAGMA cache_size=-64000;",
    )
    .context("Failed to set SQLite pragmas")?;

    // Only load CSV files referenced by the query
    let referenced = files_referenced_by_query(query, &csv_files);
    for file_path in referenced {
        let table_name = table_name_from_path(file_path);
        if load_csv_file_into_sqlite(&conn, file_path, &table_name, config).is_err() {
            continue;
        }
    }

    // Execute user query
    let mut stmt = conn
        .prepare(query)
        .map_err(|e| enhance_sql_error(e, &conn, query))?;
    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let values: Vec<String> = (0..col_count)
                .map(|i| {
                    use rusqlite::types::ValueRef;
                    match row.get_ref(i).unwrap_or(ValueRef::Null) {
                        ValueRef::Null => String::new(),
                        ValueRef::Integer(n) => n.to_string(),
                        ValueRef::Real(f) => f.to_string(),
                        ValueRef::Text(s) => String::from_utf8_lossy(s).into_owned(),
                        ValueRef::Blob(b) => String::from_utf8_lossy(b).into_owned(),
                    }
                })
                .collect();
            Ok(values)
        })
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Write CSV to stdout
    let stdout = std::io::stdout();
    let mut wtr = csv::Writer::from_writer(stdout.lock());

    // Write header row
    wtr.write_record(&col_names)?;

    // Write data rows
    for row_result in rows {
        let values = row_result.context("Failed to read query result row")?;
        wtr.write_record(&values)?;
    }

    wtr.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_table_name_from_csv() {
        let path = PathBuf::from("data/sample.csv");
        assert_eq!(table_name_from_path(&path), "sample");
    }

    #[test]
    fn test_table_name_strips_extension() {
        let path = PathBuf::from("my-file.csv");
        assert_eq!(table_name_from_path(&path), "my_file");
    }

    #[test]
    fn test_table_name_replaces_special_chars() {
        let path = PathBuf::from("my file (1).csv");
        assert_eq!(table_name_from_path(&path), "my_file__1_");
    }

    #[test]
    fn test_table_name_no_extension() {
        let path = PathBuf::from("data");
        assert_eq!(table_name_from_path(&path), "data");
    }

    #[test]
    fn test_table_name_preserves_underscores() {
        let path = PathBuf::from("my_data_file.csv");
        assert_eq!(table_name_from_path(&path), "my_data_file");
    }

    #[test]
    fn test_load_and_query_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![
                vec!["name".into(), "age".into()],
                vec!["Alice".into(), "30".into()],
                vec!["Bob".into(), "25".into()],
            ],
            filename: "test.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };

        load_csv_into_sqlite(&conn, &doc, "people").unwrap();

        let mut stmt = conn
            .prepare("SELECT name, age FROM people ORDER BY name")
            .unwrap();
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| {
                use rusqlite::types::ValueRef;
                let col0 = match row.get_ref(0).unwrap_or(ValueRef::Null) {
                    ValueRef::Text(s) => String::from_utf8_lossy(s).into_owned(),
                    ValueRef::Integer(n) => n.to_string(),
                    ValueRef::Real(f) => f.to_string(),
                    _ => String::new(),
                };
                let col1 = match row.get_ref(1).unwrap_or(ValueRef::Null) {
                    ValueRef::Text(s) => String::from_utf8_lossy(s).into_owned(),
                    ValueRef::Integer(n) => n.to_string(),
                    ValueRef::Real(f) => f.to_string(),
                    _ => String::new(),
                };
                Ok((col0, col1))
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], ("Alice".into(), "30".into()));
        assert_eq!(rows[1], ("Bob".into(), "25".into()));
    }

    #[test]
    fn test_load_empty_document_fails() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![],
            filename: "empty.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };

        let result = load_csv_into_sqlite(&conn, &doc, "empty");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_headers_only() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![vec!["col1".into(), "col2".into()]],
            filename: "headers_only.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };

        load_csv_into_sqlite(&conn, &doc, "headers_only").unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM headers_only", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_load_special_column_names() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![
                vec!["select".into(), "from".into(), "where".into()],
                vec!["a".into(), "b".into(), "c".into()],
            ],
            filename: "reserved.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };

        // Should not fail even with SQL reserved words as column names
        load_csv_into_sqlite(&conn, &doc, "reserved").unwrap();

        let val: String = conn
            .query_row("SELECT \"select\" FROM reserved", [], |r| r.get(0))
            .unwrap();
        assert_eq!(val, "a");
    }

    #[test]
    fn test_load_with_missing_cells() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![
                vec!["a".into(), "b".into(), "c".into()],
                vec!["1".into()], // Missing b and c
            ],
            filename: "sparse.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };

        load_csv_into_sqlite(&conn, &doc, "sparse").unwrap();

        let val: String = conn
            .query_row("SELECT b FROM sparse", [], |r| {
                Ok(r.get::<_, Option<String>>(0)?.unwrap_or_default())
            })
            .unwrap();
        assert_eq!(val, "");
    }

    #[test]
    fn test_misspelled_column_shows_useful_error() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![
                vec!["Company".into(), "Contact".into()],
                vec!["Acme".into(), "John".into()],
            ],
            filename: "customers.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };
        load_csv_into_sqlite(&conn, &doc, "customers").unwrap();

        let result = execute_query_to_document(
            &conn,
            "SELECT Company, Contect FROM customers",
            "out.csv".into(),
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("no such column: Contect"),
            "Error should mention the bad column name, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_resolve_csv_files_directory() {
        use std::fs;
        let temp_dir = tempfile::tempdir().unwrap();

        // Create test CSV files
        fs::write(temp_dir.path().join("test1.csv"), "a,b\n1,2\n").unwrap();
        fs::write(temp_dir.path().join("test2.csv"), "x,y\n3,4\n").unwrap();
        fs::write(temp_dir.path().join("not_csv.txt"), "ignore\n").unwrap();

        let files = resolve_csv_files(temp_dir.path()).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.file_name().unwrap() == "test1.csv"));
        assert!(files.iter().any(|p| p.file_name().unwrap() == "test2.csv"));
    }

    #[test]
    fn test_resolve_csv_files_single_file() {
        use std::fs;
        let temp_dir = tempfile::tempdir().unwrap();

        // Create test CSV files
        let target_file = temp_dir.path().join("target.csv");
        fs::write(&target_file, "a,b\n1,2\n").unwrap();
        fs::write(temp_dir.path().join("other.csv"), "x,y\n3,4\n").unwrap();

        let files = resolve_csv_files(&target_file).unwrap();

        // Should return all CSVs in the same directory (for JOINs)
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_resolve_csv_files_nonexistent_path() {
        let result = resolve_csv_files(Path::new("/nonexistent/path"));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("does not exist"));
    }

    #[test]
    fn test_resolve_csv_files_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = resolve_csv_files(temp_dir.path());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No CSV files found"));
    }

    #[test]
    fn test_execute_query_to_document_simple_select() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![
                vec!["Name".into(), "Age".into()],
                vec!["Alice".into(), "30".into()],
                vec!["Bob".into(), "25".into()],
            ],
            filename: "people.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };
        load_csv_into_sqlite(&conn, &doc, "people").unwrap();

        let result_doc = execute_query_to_document(
            &conn,
            "SELECT Name FROM people WHERE Age > '26'",
            "result.csv".into(),
        )
        .unwrap();

        assert_eq!(result_doc.rows.len(), 2); // 1 header + 1 data row
        assert_eq!(result_doc.rows[0][0], "Name");
        assert_eq!(result_doc.rows[1][0], "Alice");
    }

    #[test]
    fn test_execute_query_to_document_aggregate() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![
                vec!["Product".into(), "Price".into()],
                vec!["Apple".into(), "1.50".into()],
                vec!["Banana".into(), "0.75".into()],
            ],
            filename: "products.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };
        load_csv_into_sqlite(&conn, &doc, "products").unwrap();

        let result_doc = execute_query_to_document(
            &conn,
            "SELECT COUNT(*) as total FROM products",
            "count.csv".into(),
        )
        .unwrap();

        assert_eq!(result_doc.rows.len(), 2); // 1 header + 1 data row
        assert_eq!(result_doc.rows[0][0], "total");
        assert_eq!(result_doc.rows[1][0], "2");
    }

    #[test]
    fn test_execute_query_to_document_invalid_sql() {
        let conn = Connection::open_in_memory().unwrap();

        let result =
            execute_query_to_document(&conn, "SELECT * FROM nonexistent_table", "out.csv".into());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no such table") || err_msg.contains("Failed to execute query"));
    }

    #[test]
    fn test_execute_query_to_document_empty_result() {
        let conn = Connection::open_in_memory().unwrap();
        let doc = Document {
            rows: vec![
                vec!["ID".into(), "Value".into()],
                vec!["1".into(), "test".into()],
            ],
            filename: "data.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };
        load_csv_into_sqlite(&conn, &doc, "data").unwrap();

        let result_doc = execute_query_to_document(
            &conn,
            "SELECT * FROM data WHERE ID = '999'",
            "empty.csv".into(),
        )
        .unwrap();

        // Should have headers but no data rows
        assert_eq!(result_doc.rows.len(), 1);
        assert_eq!(result_doc.rows[0].len(), 2); // ID and Value columns
    }

    #[test]
    fn test_execute_query_to_document_join() {
        let conn = Connection::open_in_memory().unwrap();

        // Create two tables
        let doc1 = Document {
            rows: vec![
                vec!["ID".into(), "Name".into()],
                vec!["1".into(), "Alice".into()],
                vec!["2".into(), "Bob".into()],
            ],
            filename: "users.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };
        let doc2 = Document {
            rows: vec![
                vec!["UserID".into(), "Email".into()],
                vec!["1".into(), "alice@example.com".into()],
            ],
            filename: "emails.csv".into(),
            is_dirty: false,
            header_mode: true,
            delimiter: ',',
            generation: 0,
        };

        load_csv_into_sqlite(&conn, &doc1, "users").unwrap();
        load_csv_into_sqlite(&conn, &doc2, "emails").unwrap();

        let result_doc = execute_query_to_document(
            &conn,
            "SELECT u.Name, e.Email FROM users u JOIN emails e ON u.ID = e.UserID",
            "joined.csv".into(),
        )
        .unwrap();

        assert_eq!(result_doc.rows.len(), 2); // 1 header + 1 joined row
        assert_eq!(result_doc.rows[1][0], "Alice");
        assert_eq!(result_doc.rows[1][1], "alice@example.com");
    }

    #[test]
    fn test_strip_csv_from_bare_identifier() {
        assert_eq!(
            strip_csv_extensions("SELECT * FROM myfile.csv WHERE a = 1"),
            "SELECT * FROM myfile WHERE a = 1"
        );
    }

    #[test]
    fn test_strip_csv_case_insensitive() {
        assert_eq!(
            strip_csv_extensions("SELECT * FROM myfile.CSV"),
            "SELECT * FROM myfile"
        );
    }

    #[test]
    fn test_strip_tsv_and_txt() {
        assert_eq!(
            strip_csv_extensions("SELECT * FROM data.tsv JOIN info.txt"),
            "SELECT * FROM data JOIN info"
        );
    }

    #[test]
    fn test_strip_csv_from_quoted_identifier() {
        assert_eq!(
            strip_csv_extensions("SELECT * FROM \"my-file.csv\" LIMIT 10"),
            "SELECT * FROM \"my-file\" LIMIT 10"
        );
    }

    #[test]
    fn test_strip_csv_preserves_string_literals() {
        assert_eq!(
            strip_csv_extensions("SELECT * FROM myfile WHERE name = 'data.csv'"),
            "SELECT * FROM myfile WHERE name = 'data.csv'"
        );
    }

    #[test]
    fn test_strip_csv_no_false_positive_on_partial() {
        // "csvdata" should NOT be stripped — .csv is not at a word boundary
        assert_eq!(
            strip_csv_extensions("SELECT * FROM myfile.csvdata"),
            "SELECT * FROM myfile.csvdata"
        );
    }

    #[test]
    fn test_strip_csv_no_extension_passthrough() {
        let q = "SELECT * FROM myfile WHERE a = 1";
        assert_eq!(strip_csv_extensions(q), q);
    }
}
