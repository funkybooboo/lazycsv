//! Integration tests for foreign format support (Parquet, JSON, NDJSON, SQLite).
//!
//! Uses DuckDB to generate test files in each format, then verifies that
//! lazycsv can load, count, and query them correctly.

use lazycsv::csv::document::Document;
use lazycsv::csv::foreign_formats;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper: create a test Parquet file using DuckDB.
fn create_test_parquet(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("test.parquet");
    let conn = duckdb::Connection::open_in_memory().unwrap();
    conn.execute_batch(&format!(
        "COPY (SELECT 1 AS id, 'Alice' AS name, 30 AS age \
         UNION ALL SELECT 2, 'Bob', 25 \
         UNION ALL SELECT 3, 'Carol', 35) \
         TO '{}' (FORMAT PARQUET)",
        path.display()
    ))
    .unwrap();
    path
}

/// Helper: create a test JSON file (array of objects).
fn create_test_json(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("test.json");
    std::fs::write(
        &path,
        r#"[
            {"id": 1, "name": "Alice", "age": 30},
            {"id": 2, "name": "Bob", "age": 25},
            {"id": 3, "name": "Carol", "age": 35}
        ]"#,
    )
    .unwrap();
    path
}

/// Helper: create a test NDJSON file.
fn create_test_ndjson(dir: &std::path::Path, ext: &str) -> PathBuf {
    let path = dir.join(format!("test.{}", ext));
    std::fs::write(
        &path,
        r#"{"id": 1, "name": "Alice", "age": 30}
{"id": 2, "name": "Bob", "age": 25}
{"id": 3, "name": "Carol", "age": 35}
"#,
    )
    .unwrap();
    path
}

/// Helper: create a test SQLite database using DuckDB.
fn create_test_sqlite(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("test.db");
    let conn = duckdb::Connection::open_in_memory().unwrap();
    conn.execute_batch("INSTALL sqlite; LOAD sqlite;").unwrap();
    conn.execute_batch(&format!(
        "ATTACH '{}' AS testdb (TYPE sqlite); \
         CREATE TABLE testdb.users (id INTEGER, name VARCHAR, age INTEGER); \
         INSERT INTO testdb.users VALUES (1, 'Alice', 30), (2, 'Bob', 25), (3, 'Carol', 35); \
         CREATE TABLE testdb.products (id INTEGER, item VARCHAR); \
         INSERT INTO testdb.products VALUES (1, 'Widget'), (2, 'Gadget'); \
         DETACH testdb;",
        path.display()
    ))
    .unwrap();
    path
}

/// Helper: create a test gzip-compressed CSV file using DuckDB.
fn create_test_csv_gz(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("test.csv.gz");
    let conn = duckdb::Connection::open_in_memory().unwrap();
    conn.execute_batch(&format!(
        "COPY (SELECT 1 AS id, 'Alice' AS name, 30 AS age \
         UNION ALL SELECT 2, 'Bob', 25 \
         UNION ALL SELECT 3, 'Carol', 35) \
         TO '{}' (FORMAT CSV, HEADER, COMPRESSION GZIP)",
        path.display()
    ))
    .unwrap();
    path
}

/// Verify loaded rows have expected structure: header + 3 data rows, 3 columns.
fn assert_standard_3_rows(rows: &[Vec<String>]) {
    assert_eq!(
        rows.len(),
        4,
        "Expected header + 3 data rows, got {}",
        rows.len()
    );
    assert_eq!(
        rows[0].len(),
        3,
        "Expected 3 columns, got {}",
        rows[0].len()
    );
    // Check column names exist (case may vary by format)
    let header_lower: Vec<String> = rows[0].iter().map(|s| s.to_lowercase()).collect();
    assert!(header_lower.contains(&"id".to_string()));
    assert!(header_lower.contains(&"name".to_string()));
    assert!(header_lower.contains(&"age".to_string()));
}

// ── Format Detection ───────────────────────────────────────────

#[test]
fn test_parquet_detected_as_foreign() {
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.parquet"
    )));
    assert!(!foreign_formats::is_foreign_format(&PathBuf::from(
        "data.csv"
    )));
    assert!(!foreign_formats::is_foreign_format(&PathBuf::from(
        "data.xlsx"
    )));
}

#[test]
fn test_json_detected_as_foreign() {
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.json"
    )));
}

#[test]
fn test_ndjson_detected_as_foreign() {
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.ndjson"
    )));
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.jsonl"
    )));
}

#[test]
fn test_sqlite_detected_as_foreign() {
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.db"
    )));
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.sqlite"
    )));
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.sqlite3"
    )));
    assert!(foreign_formats::is_sqlite(&PathBuf::from("data.db")));
    assert!(!foreign_formats::is_sqlite(&PathBuf::from("data.parquet")));
}

#[test]
fn test_format_detection_case_insensitive() {
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.PARQUET"
    )));
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.Json"
    )));
    assert!(foreign_formats::is_foreign_format(&PathBuf::from(
        "data.DB"
    )));
}

// ── Parquet Loading ────────────────────────────────────────────

#[test]
fn test_load_parquet_via_foreign_formats() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());
    let rows = foreign_formats::load_foreign_format(&path, None).unwrap();
    assert_standard_3_rows(&rows);
}

#[test]
fn test_load_parquet_via_document() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());
    let doc = Document::from_file(&path, None, false, None).unwrap();
    assert_eq!(doc.row_count(), 4); // header + 3 data rows
    assert_eq!(doc.column_count(), 3);
}

#[test]
fn test_parquet_count_rows() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());
    let count = Document::count_rows(&path, None, false, None).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_parquet_count_columns() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());
    let count = Document::count_columns(&path, None, None).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_parquet_read_headers() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());
    let headers = Document::read_headers(&path, None, false, None).unwrap();
    assert_eq!(headers.len(), 3);
    let lower: Vec<String> = headers.iter().map(|s| s.to_lowercase()).collect();
    assert!(lower.contains(&"id".to_string()));
    assert!(lower.contains(&"name".to_string()));
    assert!(lower.contains(&"age".to_string()));
}

// ── JSON Loading ───────────────────────────────────────────────

#[test]
fn test_load_json_via_foreign_formats() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());
    let rows = foreign_formats::load_foreign_format(&path, None).unwrap();
    assert_standard_3_rows(&rows);
}

#[test]
fn test_load_json_via_document() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());
    let doc = Document::from_file(&path, None, false, None).unwrap();
    assert_eq!(doc.row_count(), 4);
    assert_eq!(doc.column_count(), 3);
}

#[test]
fn test_json_count_rows() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());
    let count = Document::count_rows(&path, None, false, None).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_json_read_headers() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());
    let headers = Document::read_headers(&path, None, false, None).unwrap();
    assert_eq!(headers.len(), 3);
}

// ── NDJSON Loading ─────────────────────────────────────────────

#[test]
fn test_load_ndjson_via_foreign_formats() {
    let dir = TempDir::new().unwrap();
    let path = create_test_ndjson(dir.path(), "ndjson");
    let rows = foreign_formats::load_foreign_format(&path, None).unwrap();
    assert_standard_3_rows(&rows);
}

#[test]
fn test_load_jsonl_via_document() {
    let dir = TempDir::new().unwrap();
    let path = create_test_ndjson(dir.path(), "jsonl");
    let doc = Document::from_file(&path, None, false, None).unwrap();
    assert_eq!(doc.row_count(), 4);
    assert_eq!(doc.column_count(), 3);
}

#[test]
fn test_ndjson_count_rows() {
    let dir = TempDir::new().unwrap();
    let path = create_test_ndjson(dir.path(), "ndjson");
    let count = Document::count_rows(&path, None, false, None).unwrap();
    assert_eq!(count, 3);
}

// ── SQLite Loading ─────────────────────────────────────────────

#[test]
fn test_get_sqlite_tables() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());
    let tables = foreign_formats::get_sqlite_tables(&path).unwrap();
    assert!(tables.len() >= 2);
    let lower: Vec<String> = tables.iter().map(|s| s.to_lowercase()).collect();
    assert!(lower.contains(&"users".to_string()));
    assert!(lower.contains(&"products".to_string()));
}

#[test]
fn test_load_sqlite_default_table() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());
    // Should load first table alphabetically
    let rows = foreign_formats::load_foreign_format(&path, None).unwrap();
    assert!(rows.len() > 1, "Should have header + data rows");
}

#[test]
fn test_load_sqlite_specific_table() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());
    let rows = foreign_formats::load_foreign_format(&path, Some("users")).unwrap();
    assert_standard_3_rows(&rows);
}

#[test]
fn test_load_sqlite_products_table() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());
    let rows = foreign_formats::load_foreign_format(&path, Some("products")).unwrap();
    assert_eq!(rows.len(), 3); // header + 2 data rows
    assert_eq!(rows[0].len(), 2); // id, item
}

#[test]
fn test_load_sqlite_via_document() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());
    // sheet_name parameter doubles as table name
    let doc = Document::from_file_with_sheet(&path, None, false, None, Some("users")).unwrap();
    assert_eq!(doc.row_count(), 4);
    assert_eq!(doc.column_count(), 3);
}

#[test]
fn test_sqlite_nonexistent_table_errors() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());
    let result = foreign_formats::load_foreign_format(&path, Some("nonexistent"));
    assert!(result.is_err());
}

// ── DuckDB Query Mode ──────────────────────────────────────────

#[test]
fn test_query_parquet_via_duckdb() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());

    let conn = duckdb::Connection::open_in_memory().unwrap();
    let config = lazycsv::session::FileConfig::default();
    lazycsv::query::load_csv_file_into_duckdb(&conn, &path, "test", &config).unwrap();

    let mut stmt = conn.prepare("SELECT count(*) FROM test").unwrap();
    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_query_json_via_duckdb() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());

    let conn = duckdb::Connection::open_in_memory().unwrap();
    let config = lazycsv::session::FileConfig::default();
    lazycsv::query::load_csv_file_into_duckdb(&conn, &path, "test", &config).unwrap();

    let mut stmt = conn.prepare("SELECT count(*) FROM test").unwrap();
    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_query_ndjson_via_duckdb() {
    let dir = TempDir::new().unwrap();
    let path = create_test_ndjson(dir.path(), "ndjson");

    let conn = duckdb::Connection::open_in_memory().unwrap();
    let config = lazycsv::session::FileConfig::default();
    lazycsv::query::load_csv_file_into_duckdb(&conn, &path, "test", &config).unwrap();

    let mut stmt = conn.prepare("SELECT count(*) FROM test").unwrap();
    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_query_sqlite_via_duckdb() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());

    let conn = duckdb::Connection::open_in_memory().unwrap();
    let config = lazycsv::session::FileConfig::default();
    // For sqlite, the table_name param is used as both view name and sqlite table name
    lazycsv::query::load_csv_file_into_duckdb(&conn, &path, "users", &config).unwrap();

    let mut stmt = conn.prepare("SELECT count(*) FROM users").unwrap();
    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
    assert_eq!(count, 3);
}

// ── DuckDB Reader SQL Generation ───────────────────────────────

#[test]
fn test_duckdb_reader_sql_parquet() {
    let sql = foreign_formats::duckdb_reader_sql(&PathBuf::from("data.parquet"), "data", None);
    assert!(sql.is_some());
    let sql = sql.unwrap();
    assert!(sql.contains("read_parquet"));
    assert!(sql.contains("CREATE VIEW"));
}

#[test]
fn test_duckdb_reader_sql_json() {
    let sql = foreign_formats::duckdb_reader_sql(&PathBuf::from("data.json"), "data", None);
    assert!(sql.unwrap().contains("read_json"));
}

#[test]
fn test_duckdb_reader_sql_ndjson() {
    let sql = foreign_formats::duckdb_reader_sql(&PathBuf::from("data.ndjson"), "data", None);
    assert!(sql.unwrap().contains("read_ndjson"));
}

#[test]
fn test_duckdb_reader_sql_sqlite() {
    let sql = foreign_formats::duckdb_reader_sql(&PathBuf::from("data.db"), "data", Some("users"));
    let sql = sql.unwrap();
    assert!(sql.contains("sqlite_scan"));
    assert!(sql.contains("users"));
}

#[test]
fn test_duckdb_reader_sql_csv_returns_none() {
    let sql = foreign_formats::duckdb_reader_sql(&PathBuf::from("data.csv"), "data", None);
    assert!(sql.is_none());
}

// ── File Discovery ─────────────────────────────────────────────

#[test]
fn test_discovery_finds_parquet_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("data.csv"), "a\n1").unwrap();
    create_test_parquet(dir.path());

    let files = lazycsv::file_system::scan_directory(dir.path()).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|p| p.extension().unwrap() == "csv"));
    assert!(files.iter().any(|p| p.extension().unwrap() == "parquet"));
}

#[test]
fn test_discovery_finds_json_files() {
    let dir = TempDir::new().unwrap();
    create_test_json(dir.path());

    let files = lazycsv::file_system::scan_directory(dir.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].extension().unwrap() == "json");
}

#[test]
fn test_discovery_finds_sqlite_files() {
    let dir = TempDir::new().unwrap();
    create_test_sqlite(dir.path());

    let files = lazycsv::file_system::scan_directory(dir.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].extension().unwrap() == "db");
}

#[test]
fn test_discovery_finds_ndjson_files() {
    let dir = TempDir::new().unwrap();
    create_test_ndjson(dir.path(), "ndjson");
    create_test_ndjson(dir.path(), "jsonl");

    let files = lazycsv::file_system::scan_directory(dir.path()).unwrap();
    assert_eq!(files.len(), 2);
}

// ── Extension Stripping in SQL ─────────────────────────────────

#[test]
fn test_strip_parquet_extension_in_query() {
    let result = lazycsv::query::strip_csv_extensions("SELECT * FROM data.parquet");
    assert_eq!(result, "SELECT * FROM data");
}

#[test]
fn test_strip_json_extension_in_query() {
    let result = lazycsv::query::strip_csv_extensions("SELECT * FROM data.json");
    assert_eq!(result, "SELECT * FROM data");
}

#[test]
fn test_strip_ndjson_extension_in_query() {
    let result = lazycsv::query::strip_csv_extensions("SELECT * FROM data.ndjson");
    assert_eq!(result, "SELECT * FROM data");
}

#[test]
fn test_strip_sqlite_extension_in_query() {
    let result = lazycsv::query::strip_csv_extensions("SELECT * FROM data.db");
    assert_eq!(result, "SELECT * FROM data");

    let result = lazycsv::query::strip_csv_extensions("SELECT * FROM data.sqlite");
    assert_eq!(result, "SELECT * FROM data");
}

// ── Error Paths ────────────────────────────────────────────────

#[test]
fn test_load_nonexistent_parquet_errors() {
    let result =
        foreign_formats::load_foreign_format(&PathBuf::from("/nonexistent/file.parquet"), None);
    assert!(result.is_err());
}

#[test]
fn test_load_nonexistent_json_errors() {
    let result =
        foreign_formats::load_foreign_format(&PathBuf::from("/nonexistent/file.json"), None);
    assert!(result.is_err());
}

#[test]
fn test_load_nonexistent_sqlite_errors() {
    let result = foreign_formats::load_foreign_format(&PathBuf::from("/nonexistent/file.db"), None);
    assert!(result.is_err());
}

#[test]
fn test_load_csv_not_detected_as_foreign() {
    // Ensure CSV files don't get routed through foreign format loader
    let result = foreign_formats::load_foreign_format(&PathBuf::from("data.csv"), None);
    assert!(result.is_err()); // "Not a supported foreign format"
}

// ── Data Integrity ─────────────────────────────────────────────
// Verify actual cell values, not just row/column counts.

#[test]
fn test_parquet_data_values() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());
    let rows = foreign_formats::load_foreign_format(&path, None).unwrap();

    // Find the "name" column index
    let name_idx = rows[0]
        .iter()
        .position(|h| h.to_lowercase() == "name")
        .unwrap();
    let names: Vec<&str> = rows[1..].iter().map(|r| r[name_idx].as_str()).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));
    assert!(names.contains(&"Carol"));
}

#[test]
fn test_json_data_values() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());
    let rows = foreign_formats::load_foreign_format(&path, None).unwrap();

    let name_idx = rows[0]
        .iter()
        .position(|h| h.to_lowercase() == "name")
        .unwrap();
    let age_idx = rows[0]
        .iter()
        .position(|h| h.to_lowercase() == "age")
        .unwrap();

    // Find Alice's row and check her age
    let alice_row = rows[1..].iter().find(|r| r[name_idx] == "Alice").unwrap();
    assert_eq!(alice_row[age_idx], "30");
}

#[test]
fn test_ndjson_data_values() {
    let dir = TempDir::new().unwrap();
    let path = create_test_ndjson(dir.path(), "ndjson");
    let rows = foreign_formats::load_foreign_format(&path, None).unwrap();

    let id_idx = rows[0]
        .iter()
        .position(|h| h.to_lowercase() == "id")
        .unwrap();
    let ids: Vec<&str> = rows[1..].iter().map(|r| r[id_idx].as_str()).collect();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"1"));
    assert!(ids.contains(&"2"));
    assert!(ids.contains(&"3"));
}

#[test]
fn test_sqlite_data_values() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());
    let rows = foreign_formats::load_foreign_format(&path, Some("users")).unwrap();

    let name_idx = rows[0]
        .iter()
        .position(|h| h.to_lowercase() == "name")
        .unwrap();
    let names: Vec<&str> = rows[1..].iter().map(|r| r[name_idx].as_str()).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));
    assert!(names.contains(&"Carol"));
}

// ── JSON Content Detection (clipboard paste logic) ─────────────
// These test the content-sniffing logic used by -P to distinguish JSON from CSV.

#[test]
fn test_json_array_written_to_temp_loads_correctly() {
    // Simulates the -P clipboard path: write JSON to a .json temp file, load via DuckDB
    let dir = TempDir::new().unwrap();
    let temp_file = dir.path().join("clipboard.json");
    std::fs::write(&temp_file, r#"[{"x":1,"y":"hello"},{"x":2,"y":"world"}]"#).unwrap();

    let rows = foreign_formats::load_foreign_format(&temp_file, None).unwrap();
    assert_eq!(rows.len(), 3); // header + 2 data rows
    assert_eq!(rows[0].len(), 2);
}

#[test]
fn test_ndjson_written_to_temp_loads_correctly() {
    let dir = TempDir::new().unwrap();
    let temp_file = dir.path().join("clipboard.json");
    std::fs::write(&temp_file, "{\"a\":1,\"b\":2}\n{\"a\":3,\"b\":4}\n").unwrap();

    // DuckDB's read_json auto-detects NDJSON even with .json extension
    let rows = foreign_formats::load_foreign_format(&temp_file, None).unwrap();
    assert_eq!(rows.len(), 3); // header + 2 data rows
}

#[test]
fn test_nested_json_fails_gracefully() {
    // Non-tabular JSON (nested objects) should error, triggering CSV fallback in -P
    let dir = TempDir::new().unwrap();
    let temp_file = dir.path().join("clipboard.json");
    std::fs::write(
        &temp_file,
        r#"{"schema":{"tables":[{"name":"t1","columns":["a","b"]}]}}"#,
    )
    .unwrap();

    let result = foreign_formats::load_foreign_format(&temp_file, None);
    // Either errors or produces only 1 row (header only) — both trigger CSV fallback
    match result {
        Err(_) => {} // expected for deeply nested JSON
        Ok(rows) => assert!(
            rows.len() <= 2,
            "Nested JSON should not produce many flat rows"
        ),
    }
}

#[test]
fn test_json_content_sniffing_heuristic() {
    // Verify the starts_with heuristic matches what the -P code does
    let json_array = r#"[{"id":1},{"id":2}]"#;
    let json_object = r#"{"id":1,"name":"test"}"#;
    let csv_content = "id,name\n1,Alice\n2,Bob\n";
    let tsv_content = "id\tname\n1\tAlice\n";

    assert!(json_array.trim_start().starts_with('['));
    assert!(json_object.trim_start().starts_with('{'));
    assert!(!csv_content.trim_start().starts_with('['));
    assert!(!csv_content.trim_start().starts_with('{'));
    assert!(!tsv_content.trim_start().starts_with('['));

    // Whitespace-prefixed JSON should still be detected
    let padded = "  \n  [{\"id\":1}]";
    assert!(padded.trim_start().starts_with('['));
}

// ── CLI Subprocess Tests ───────────────────────────────────────
// Test the actual binary for piped-stdin detection and format-specific CLI flags.

fn lazycsv_bin() -> PathBuf {
    // Use the test binary built by cargo
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove "deps"
    path.push("lazycsv");
    path
}

#[test]
fn test_cli_piped_stdin_with_file_arg_errors() {
    let dir = TempDir::new().unwrap();
    let csv_path = dir.path().join("test.csv");
    std::fs::write(&csv_path, "a,b\n1,2\n").unwrap();

    // Pipe stdin to lazycsv with a file argument — should exit with error, not hang
    let output = Command::new(lazycsv_bin())
        .arg(csv_path.to_str().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "Should exit with error when stdin is piped to TUI"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("stdin is piped") || stderr.contains("keyboard input"),
        "Should mention stdin/keyboard issue, got: {}",
        stderr
    );
}

#[test]
fn test_cli_parquet_row_count() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([path.to_str().unwrap(), "-r"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "lazycsv -r should succeed for parquet"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("3 rows"),
        "Expected '3 rows', got: {}",
        stdout
    );
}

#[test]
fn test_cli_json_row_count() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([path.to_str().unwrap(), "-r"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "lazycsv -r should succeed for json"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("3 rows"),
        "Expected '3 rows', got: {}",
        stdout
    );
}

#[test]
fn test_cli_sqlite_row_count() {
    let dir = TempDir::new().unwrap();
    let path = create_test_sqlite(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([path.to_str().unwrap(), "-r"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "lazycsv -r should succeed for sqlite"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // First table alphabetically is "products" (2 rows) or "users" (3 rows)
    assert!(
        stdout.contains("rows"),
        "Expected row count, got: {}",
        stdout
    );
}

#[test]
fn test_cli_parquet_headers() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([path.to_str().unwrap(), "-h"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    assert!(
        stdout.contains("id"),
        "Expected 'id' header, got: {}",
        stdout
    );
    assert!(
        stdout.contains("name"),
        "Expected 'name' header, got: {}",
        stdout
    );
}

#[test]
fn test_cli_parquet_column_count() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([path.to_str().unwrap(), "-c"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("3 columns"),
        "Expected '3 columns', got: {}",
        stdout
    );
}

#[test]
fn test_cli_parquet_query() {
    let dir = TempDir::new().unwrap();
    let path = create_test_parquet(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([
            path.to_str().unwrap(),
            "-q",
            "SELECT name FROM test WHERE age > 28 ORDER BY name",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "Query should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Alice"),
        "Expected Alice (age 30), got: {}",
        stdout
    );
    assert!(
        stdout.contains("Carol"),
        "Expected Carol (age 35), got: {}",
        stdout
    );
}

#[test]
fn test_cli_json_query() {
    let dir = TempDir::new().unwrap();
    let path = create_test_json(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([
            path.to_str().unwrap(),
            "-q",
            "SELECT count(*) as cnt FROM test",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "Expected count of 3, got: {}", stdout);
}

#[test]
fn test_cli_ndjson_row_count() {
    let dir = TempDir::new().unwrap();
    let path = create_test_ndjson(dir.path(), "ndjson");

    let output = Command::new(lazycsv_bin())
        .args([path.to_str().unwrap(), "-r"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("3 rows"),
        "Expected '3 rows', got: {}",
        stdout
    );
}

#[test]
fn test_cli_directory_scan_finds_mixed_formats() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("data.csv"), "a\n1\n2\n").unwrap();
    create_test_parquet(dir.path());
    create_test_json(dir.path());

    // Use -r on the directory to count rows in all files
    let output = Command::new(lazycsv_bin())
        .args([dir.path().to_str().unwrap(), "-r"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should list multiple files
    assert!(stdout.contains("data.csv"), "Should list CSV file");
    assert!(stdout.contains("test.parquet"), "Should list parquet file");
    assert!(stdout.contains("test.json"), "Should list JSON file");
}

// ── Gzip CSV Support (.csv.gz) ─────────────────────────────────

#[test]
fn test_is_gzip_detection() {
    assert!(lazycsv::csv::document::is_gzip(&PathBuf::from(
        "data.csv.gz"
    )));
    assert!(lazycsv::csv::document::is_gzip(&PathBuf::from(
        "data.tsv.gz"
    )));
    assert!(lazycsv::csv::document::is_gzip(&PathBuf::from(
        "data.CSV.GZ"
    )));
    assert!(!lazycsv::csv::document::is_gzip(&PathBuf::from("data.csv")));
    assert!(!lazycsv::csv::document::is_gzip(&PathBuf::from(
        "data.parquet"
    )));
}

#[test]
fn test_table_name_from_path_csv_gz() {
    assert_eq!(
        lazycsv::query::table_name_from_path(&PathBuf::from("largedata.csv.gz")),
        "largedata"
    );
    assert_eq!(
        lazycsv::query::table_name_from_path(&PathBuf::from("data.tsv.gz")),
        "data"
    );
    // Regular files still work
    assert_eq!(
        lazycsv::query::table_name_from_path(&PathBuf::from("data.csv")),
        "data"
    );
    assert_eq!(
        lazycsv::query::table_name_from_path(&PathBuf::from("data.parquet")),
        "data"
    );
}

#[test]
fn test_strip_csv_gz_extension_in_query() {
    let result = lazycsv::query::strip_csv_extensions("SELECT * FROM data.csv.gz");
    assert_eq!(result, "SELECT * FROM data");

    let result = lazycsv::query::strip_csv_extensions("SELECT * FROM data.tsv.gz");
    assert_eq!(result, "SELECT * FROM data");
}

#[test]
fn test_gzip_tui_load_returns_error() {
    let dir = TempDir::new().unwrap();
    let path = create_test_csv_gz(dir.path());

    // TUI load should fail with a helpful error, not OOM
    let result = Document::from_file(&path, None, false, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("query mode") || err.contains("-q"),
        "Error should suggest query mode, got: {}",
        err
    );
}

#[test]
fn test_cli_csv_gz_query() {
    let dir = TempDir::new().unwrap();
    let path = create_test_csv_gz(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([
            path.to_str().unwrap(),
            "-q",
            "SELECT count(*) as cnt FROM test",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Query should succeed for .csv.gz, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('3'), "Expected count of 3, got: {}", stdout);
}

#[test]
fn test_cli_csv_gz_query_with_filter() {
    let dir = TempDir::new().unwrap();
    let path = create_test_csv_gz(dir.path());

    let output = Command::new(lazycsv_bin())
        .args([
            path.to_str().unwrap(),
            "-q",
            "SELECT name FROM test WHERE age > 28 ORDER BY name",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Carol"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn test_cli_csv_gz_tui_shows_error() {
    let dir = TempDir::new().unwrap();
    let path = create_test_csv_gz(dir.path());

    // Opening .csv.gz without -q should fail with helpful message
    let output = Command::new(lazycsv_bin())
        .arg(path.to_str().unwrap())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("-q") || stderr.contains("query mode"),
        "Should suggest query mode, got: {}",
        stderr
    );
}

#[test]
fn test_discovery_finds_csv_gz_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("data.csv"), "a\n1").unwrap();
    create_test_csv_gz(dir.path());

    let files = lazycsv::file_system::scan_directory(dir.path()).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|p| p.extension().unwrap() == "csv"));
    assert!(files
        .iter()
        .any(|p| p.file_name().unwrap().to_str().unwrap() == "test.csv.gz"));
}
