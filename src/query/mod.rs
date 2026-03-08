//! SQL query mode — load CSV files into SQLite and execute queries.

use crate::cancel::{self, CancelledError};
use crate::csv::Document;
use crate::file_system;
use crate::session::FileConfig;
use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Derive a SQLite table name from a file path.
/// Strips the `.csv` extension and replaces non-alphanumeric characters with `_`.
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

/// Resolve which CSV files to load.
/// If the path is a directory, loads all CSVs in it.
/// If the path is a file, loads all sibling CSVs in the same directory (enables JOINs).
/// If no path is given, scans the current directory.
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
pub fn load_csv_into_sqlite(conn: &Connection, doc: &Document, table_name: &str) -> Result<()> {
    if doc.rows.is_empty() {
        bail!("Document has no rows (not even headers)");
    }

    let headers = &doc.rows[0];
    if headers.is_empty() {
        bail!("Document has no columns");
    }

    // Build CREATE TABLE with all TEXT columns, quoting column names
    let col_defs: Vec<String> = headers
        .iter()
        .map(|h| format!("\"{}\" TEXT", h.replace('"', "\"\"")))
        .collect();

    let create_sql = format!(
        "CREATE TABLE \"{}\" ({})",
        table_name.replace('"', "\"\""),
        col_defs.join(", ")
    );
    conn.execute(&create_sql, [])
        .context(format!("Failed to create table '{}'", table_name))?;

    // Use a prepared single-row INSERT inside a transaction.
    // Binds &str refs directly — no heap allocation per cell.
    let col_count = headers.len();
    let placeholders: Vec<&str> = vec!["?"; col_count];
    let insert_sql = format!(
        "INSERT INTO \"{}\" VALUES ({})",
        table_name.replace('"', "\"\""),
        placeholders.join(", ")
    );

    conn.execute("BEGIN", [])
        .context("Failed to begin transaction")?;

    let mut stmt = conn.prepare_cached(&insert_sql)?;

    for row in doc.rows.iter().skip(1) {
        let params: Vec<&str> = (0..col_count)
            .map(|i| row.get(i).map(|s| s.as_str()).unwrap_or(""))
            .collect();
        stmt.execute(rusqlite::params_from_iter(params))?;
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

    // CREATE TABLE
    let col_defs: Vec<String> = headers
        .iter()
        .map(|h| format!("\"{}\" TEXT", h.replace('"', "\"\"")))
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

    // Use a prepared single-row INSERT inside a transaction.
    // This avoids all heap allocation for params — we bind &str refs directly
    // from the CSV StringRecord into SQLite's prepared statement.
    let col_count = headers.len();
    let placeholders: Vec<&str> = vec!["?"; col_count];
    let insert_sql = format!(
        "INSERT INTO \"{}\" VALUES ({})",
        escaped_table,
        placeholders.join(", ")
    );

    conn.execute("BEGIN", [])
        .context("Failed to begin transaction")?;

    let mut stmt = conn.prepare_cached(&insert_sql)?;

    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;
        let params: Vec<&str> = (0..col_count)
            .map(|i| record.get(i).unwrap_or(""))
            .collect();
        stmt.execute(rusqlite::params_from_iter(params))?;
    }

    drop(stmt);
    conn.execute("COMMIT", [])
        .context("Failed to commit transaction")?;

    Ok(())
}

/// Execute a SQL query on an existing connection and return results as a Document.
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

/// Load a parsed CSV Document into a SQLite table with cancellation support.
/// Checks the `cancelled` flag every CHECK_INTERVAL rows.
/// On cancellation, rolls back the transaction and returns CancelledError.
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

    let col_defs: Vec<String> = headers
        .iter()
        .map(|h| format!("\"{}\" TEXT", h.replace('"', "\"\"")))
        .collect();

    let create_sql = format!(
        "CREATE TABLE \"{}\" ({})",
        table_name.replace('"', "\"\""),
        col_defs.join(", ")
    );
    conn.execute(&create_sql, [])
        .context(format!("Failed to create table '{}'", table_name))?;

    let col_count = headers.len();
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
        let params: Vec<&str> = (0..col_count)
            .map(|j| row.get(j).map(|s| s.as_str()).unwrap_or(""))
            .collect();
        stmt.execute(rusqlite::params_from_iter(params))?;
    }

    drop(stmt);
    conn.execute("COMMIT", [])
        .context("Failed to commit transaction")?;

    Ok(())
}

/// Execute a SQL query and return results as a Document, with cancellation support.
/// Checks the `cancelled` flag every CHECK_INTERVAL result rows.
pub fn execute_query_to_document_cancellable(
    conn: &Connection,
    query: &str,
    output_filename: String,
    cancelled: &AtomicBool,
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

    // Stream each CSV file directly into SQLite, skipping files that can't be parsed
    for file_path in &csv_files {
        let table_name = table_name_from_path(file_path);
        if load_csv_file_into_sqlite(&conn, file_path, &table_name, config).is_err() {
            continue;
        }
    }

    // Execute user query
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
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
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
}
