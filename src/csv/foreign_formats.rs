//! Support for non-CSV file formats (Parquet, JSON/NDJSON, SQLite)
//! using DuckDB as the universal reader.

use anyhow::{Context, Result};
use std::path::Path;

/// Supported foreign (non-CSV, non-spreadsheet) file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignFormat {
    Parquet,
    Json,
    Ndjson,
    Sqlite,
}

/// Detect foreign format from file extension.
pub fn foreign_format_type(path: &Path) -> Option<ForeignFormat> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "parquet" => Some(ForeignFormat::Parquet),
        "json" => Some(ForeignFormat::Json),
        "ndjson" | "jsonl" => Some(ForeignFormat::Ndjson),
        "db" | "sqlite" | "sqlite3" => Some(ForeignFormat::Sqlite),
        _ => None,
    }
}

/// Returns true if the file is a supported foreign format.
pub fn is_foreign_format(path: &Path) -> bool {
    foreign_format_type(path).is_some()
}

/// Returns true if the file is a SQLite database.
pub fn is_sqlite(path: &Path) -> bool {
    foreign_format_type(path) == Some(ForeignFormat::Sqlite)
}

/// List user tables in a SQLite database file.
pub fn get_sqlite_tables(path: &Path) -> Result<Vec<String>> {
    let conn = duckdb::Connection::open_in_memory().context("Failed to open DuckDB")?;
    install_sqlite_extension(&conn)?;

    let path_str = path.display().to_string().replace('\'', "''");
    let sql = format!(
        "SELECT name FROM sqlite_scan('{}', 'sqlite_master') WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        path_str
    );

    let mut stmt = conn.prepare(&sql).context("Failed to list SQLite tables")?;
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .context("Failed to query SQLite tables")?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tables)
}

/// Load a foreign format file into rows (header row + data rows) using DuckDB.
/// For SQLite, `table_name` selects which table to load (defaults to first table).
pub fn load_foreign_format(path: &Path, table_name: Option<&str>) -> Result<Vec<Vec<String>>> {
    let format = foreign_format_type(path).context(format!(
        "Not a supported foreign format: {}",
        path.display()
    ))?;

    let conn = duckdb::Connection::open_in_memory().context("Failed to open DuckDB")?;
    let path_str = path.display().to_string().replace('\'', "''");

    let select_sql = match format {
        ForeignFormat::Parquet => {
            format!("SELECT * FROM read_parquet('{}')", path_str)
        }
        ForeignFormat::Json => {
            format!("SELECT * FROM read_json('{}')", path_str)
        }
        ForeignFormat::Ndjson => {
            format!("SELECT * FROM read_ndjson('{}')", path_str)
        }
        ForeignFormat::Sqlite => {
            install_sqlite_extension(&conn)?;
            let table = match table_name {
                Some(t) => t.to_string(),
                None => {
                    let tables = get_sqlite_tables(path)?;
                    if tables.is_empty() {
                        anyhow::bail!("SQLite database has no tables");
                    }
                    tables[0].clone()
                }
            };
            let escaped_table = table.replace('\'', "''");
            format!(
                "SELECT * FROM sqlite_scan('{}', '{}')",
                path_str, escaped_table
            )
        }
    };

    // Execute query to get column metadata, then re-execute to collect rows.
    // DuckDB requires execution before column_count/column_names are available.
    let mut stmt = conn
        .prepare(&select_sql)
        .context(format!("Failed to read '{}'", path.display()))?;
    let _ = stmt.execute([]).ok();
    let col_count = stmt.column_count();
    let col_names: Vec<String> = stmt.column_names();

    // Re-prepare to get fresh cursor for data
    let mut stmt = conn.prepare(&select_sql)?;
    let data_rows = stmt
        .query_map([], |row| {
            let values: Vec<String> = (0..col_count)
                .map(|i| crate::query::duckdb_get_string(row, i))
                .collect();
            Ok(values)
        })?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    // Build result: header row + data rows
    let mut rows = Vec::with_capacity(data_rows.len() + 1);
    rows.push(col_names);
    rows.extend(data_rows);

    Ok(rows)
}

/// Install and load the DuckDB sqlite extension.
fn install_sqlite_extension(conn: &duckdb::Connection) -> Result<()> {
    conn.execute_batch("INSTALL sqlite; LOAD sqlite;")
        .context("Failed to load DuckDB sqlite extension")?;
    Ok(())
}

/// Build the DuckDB reader SQL for a foreign format file (for query-mode VIEWs).
/// Returns None if not a foreign format.
pub fn duckdb_reader_sql(
    path: &Path,
    escaped_table: &str,
    table_hint: Option<&str>,
) -> Option<String> {
    let format = foreign_format_type(path)?;
    let path_str = path.display().to_string().replace('\'', "''");

    let sql = match format {
        ForeignFormat::Parquet => {
            format!(
                "CREATE VIEW \"{}\" AS SELECT * FROM read_parquet('{}')",
                escaped_table, path_str
            )
        }
        ForeignFormat::Json => {
            format!(
                "CREATE VIEW \"{}\" AS SELECT * FROM read_json('{}')",
                escaped_table, path_str
            )
        }
        ForeignFormat::Ndjson => {
            format!(
                "CREATE VIEW \"{}\" AS SELECT * FROM read_ndjson('{}')",
                escaped_table, path_str
            )
        }
        ForeignFormat::Sqlite => {
            let table = table_hint.unwrap_or(escaped_table);
            let escaped_sqlite_table = table.replace('\'', "''");
            format!(
                "CREATE VIEW \"{}\" AS SELECT * FROM sqlite_scan('{}', '{}')",
                escaped_table, path_str, escaped_sqlite_table
            )
        }
    };

    Some(sql)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_detection_parquet() {
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.parquet")),
            Some(ForeignFormat::Parquet)
        );
    }

    #[test]
    fn test_format_detection_json() {
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.json")),
            Some(ForeignFormat::Json)
        );
    }

    #[test]
    fn test_format_detection_ndjson() {
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.ndjson")),
            Some(ForeignFormat::Ndjson)
        );
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.jsonl")),
            Some(ForeignFormat::Ndjson)
        );
    }

    #[test]
    fn test_format_detection_sqlite() {
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.db")),
            Some(ForeignFormat::Sqlite)
        );
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.sqlite")),
            Some(ForeignFormat::Sqlite)
        );
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.sqlite3")),
            Some(ForeignFormat::Sqlite)
        );
    }

    #[test]
    fn test_format_detection_csv_not_foreign() {
        assert_eq!(foreign_format_type(&PathBuf::from("data.csv")), None);
    }

    #[test]
    fn test_format_detection_case_insensitive() {
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.PARQUET")),
            Some(ForeignFormat::Parquet)
        );
        assert_eq!(
            foreign_format_type(&PathBuf::from("data.JSON")),
            Some(ForeignFormat::Json)
        );
    }

    #[test]
    fn test_is_foreign_format() {
        assert!(is_foreign_format(&PathBuf::from("data.parquet")));
        assert!(is_foreign_format(&PathBuf::from("data.json")));
        assert!(!is_foreign_format(&PathBuf::from("data.csv")));
        assert!(!is_foreign_format(&PathBuf::from("data.xlsx")));
    }

    #[test]
    fn test_duckdb_reader_sql_parquet() {
        let sql = duckdb_reader_sql(&PathBuf::from("data.parquet"), "data", None);
        assert!(sql.is_some());
        assert!(sql.unwrap().contains("read_parquet"));
    }

    #[test]
    fn test_duckdb_reader_sql_csv_returns_none() {
        let sql = duckdb_reader_sql(&PathBuf::from("data.csv"), "data", None);
        assert!(sql.is_none());
    }
}
