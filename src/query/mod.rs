//! SQL query mode — load CSV files into SQLite and execute queries.

use crate::csv::Document;
use crate::file_system;
use crate::session::FileConfig;
use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Derive a SQLite table name from a file path.
/// Strips the `.csv` extension and replaces non-alphanumeric characters with `_`.
pub fn table_name_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("table");

    stem.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
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

    // Prepare INSERT statement
    let placeholders: Vec<&str> = vec!["?"; headers.len()];
    let insert_sql = format!(
        "INSERT INTO \"{}\" VALUES ({})",
        table_name.replace('"', "\"\""),
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&insert_sql)?;

    // Insert data rows (skip row 0 which is headers)
    for row in doc.rows.iter().skip(1) {
        let params: Vec<&dyn rusqlite::types::ToSql> = (0..headers.len())
            .map(|i| {
                row.get(i)
                    .map(|s| s as &dyn rusqlite::types::ToSql)
                    .unwrap_or(&"" as &dyn rusqlite::types::ToSql)
            })
            .collect();
        stmt.execute(params.as_slice())?;
    }

    Ok(())
}

/// Execute a SQL query on an existing connection and return results as a Document.
pub fn execute_query_to_document(
    conn: &Connection,
    query: &str,
    output_filename: String,
) -> Result<Document> {
    let mut stmt = conn
        .prepare(query)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
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

/// Execute a SQL query against CSV files and write results as CSV to stdout.
pub fn execute_query(path: &Path, query: &str, config: &FileConfig) -> Result<()> {
    let csv_files = resolve_csv_files(path)?;

    let conn = Connection::open_in_memory().context("Failed to open in-memory SQLite database")?;

    // Load each CSV file as a table, skipping files that can't be parsed
    for file_path in &csv_files {
        let table_name = table_name_from_path(file_path);
        let doc = match Document::from_file(
            file_path,
            config.delimiter,
            config.no_headers,
            config.encoding.clone(),
        ) {
            Ok(doc) => doc,
            Err(_) => continue, // Skip unparseable files (e.g., empty files)
        };

        if doc.rows.is_empty() || doc.rows[0].is_empty() {
            continue; // Skip documents with no columns
        }

        load_csv_into_sqlite(&conn, &doc, &table_name)?;
    }

    // Execute user query
    let mut stmt = conn
        .prepare(query)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
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
        };

        load_csv_into_sqlite(&conn, &doc, "people").unwrap();

        let mut stmt = conn.prepare("SELECT name, age FROM people ORDER BY name").unwrap();
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
        };
        load_csv_into_sqlite(&conn, &doc, "customers").unwrap();

        let result =
            execute_query_to_document(&conn, "SELECT Company, Contect FROM customers", "out.csv".into());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("no such column: Contect"),
            "Error should mention the bad column name, got: {}",
            err_msg
        );
    }
}
