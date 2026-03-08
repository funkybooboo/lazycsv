//! SQL error message enhancement
//!
//! Parses SQLite errors and provides user-friendly messages with helpful suggestions.

use anyhow::anyhow;
use rusqlite::Connection;

/// Enhance a SQL error with context-aware help
pub fn enhance_sql_error(error: rusqlite::Error, conn: &Connection, query: &str) -> anyhow::Error {
    let err_str = error.to_string();

    // Handle "no such column" errors with suggestions
    if err_str.contains("no such column:") {
        if let Some(column_name) = extract_column_name(&err_str) {
            return enhance_column_error(column_name, conn, query);
        }
    }

    // Handle "no such table" errors with table listing
    if err_str.contains("no such table:") {
        if let Some(table_name) = extract_table_name(&err_str) {
            return enhance_table_error(table_name, conn);
        }
    }

    // Handle syntax errors
    if err_str.contains("syntax error") || err_str.contains("near") {
        return enhance_syntax_error(&err_str, query);
    }

    // Fall back to original error
    anyhow!("SQL Error: {}", err_str)
}

/// Extract column name from "no such column: columnname" error
fn extract_column_name(err_str: &str) -> Option<String> {
    err_str
        .split("no such column:")
        .nth(1)
        .map(|s| s.trim().to_string())
}

/// Extract table name from "no such table: tablename" error
fn extract_table_name(err_str: &str) -> Option<String> {
    err_str
        .split("no such table:")
        .nth(1)
        .map(|s| s.trim().to_string())
}

/// Enhance "no such column" error with similar column suggestions
fn enhance_column_error(column_name: String, conn: &Connection, query: &str) -> anyhow::Error {
    let available_columns = get_available_columns(conn, query);

    if available_columns.is_empty() {
        return anyhow!(
            "Column '{}' does not exist. No tables found in query context.",
            column_name
        );
    }

    // Find similar column names using fuzzy matching
    let suggestions = find_similar_columns(&column_name, &available_columns, 3);

    if suggestions.is_empty() {
        anyhow!(
            "Column '{}' does not exist.\n\nAvailable columns: {}",
            column_name,
            available_columns.join(", ")
        )
    } else {
        anyhow!(
            "Column '{}' does not exist. Did you mean: {}?\n\nAvailable columns: {}",
            column_name,
            suggestions.join(", "),
            available_columns.join(", ")
        )
    }
}

/// Enhance "no such table" error with available table listing
fn enhance_table_error(table_name: String, conn: &Connection) -> anyhow::Error {
    let available_tables = get_available_tables(conn);

    if available_tables.is_empty() {
        return anyhow!(
            "Table '{}' does not exist. No CSV files have been loaded.",
            table_name
        );
    }

    // Find similar table names
    let suggestions = find_similar_columns(&table_name, &available_tables, 3);

    if suggestions.is_empty() {
        anyhow!(
            "Table '{}' does not exist.\n\nAvailable tables: {}",
            table_name,
            available_tables.join(", ")
        )
    } else {
        anyhow!(
            "Table '{}' does not exist. Did you mean: {}?\n\nAvailable tables: {}",
            table_name,
            suggestions.join(", "),
            available_tables.join(", ")
        )
    }
}

/// Enhance syntax errors with helpful context
fn enhance_syntax_error(err_str: &str, query: &str) -> anyhow::Error {
    // Try to extract the problematic part from "near X" errors
    if let Some(near_text) = err_str.split("near \"").nth(1) {
        if let Some(token) = near_text.split('"').next() {
            // Find position in query
            if let Some(pos) = query.find(token) {
                let line_start = query[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = query[pos..]
                    .find('\n')
                    .map(|i| pos + i)
                    .unwrap_or(query.len());
                let line = &query[line_start..line_end];
                let col = pos - line_start;

                return anyhow!(
                    "Syntax error near '{}' at column {}:\n  {}\n  {}^",
                    token,
                    col + 1,
                    line,
                    " ".repeat(col)
                );
            }
        }
    }

    anyhow!("Syntax error in SQL query:\n{}", err_str)
}

/// Get all available columns from tables in the database
fn get_available_columns(conn: &Connection, query: &str) -> Vec<String> {
    // Extract table names mentioned in the query (simple heuristic)
    let tables = extract_tables_from_query(query, conn);

    let mut columns = Vec::new();
    for table in tables {
        if let Ok(stmt) = conn.prepare(&format!("SELECT * FROM \"{}\" LIMIT 0", table)) {
            for i in 0..stmt.column_count() {
                if let Ok(name) = stmt.column_name(i) {
                    columns.push(format!("{}.{}", table, name));
                }
            }
        }
    }

    columns
}

/// Get list of all tables in the database
fn get_available_tables(conn: &Connection) -> Vec<String> {
    let mut tables = Vec::new();

    if let Ok(mut stmt) = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
    {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row_result in rows.flatten() {
                tables.push(row_result);
            }
        }
    }

    tables
}

/// Extract table names from query (simple heuristic: after FROM and JOIN)
fn extract_tables_from_query(query: &str, conn: &Connection) -> Vec<String> {
    let mut tables = Vec::new();
    let query_upper = query.to_uppercase();
    let available_tables = get_available_tables(conn);

    // Find all table references in FROM and JOIN clauses
    for table_name in &available_tables {
        if query_upper.contains(&table_name.to_uppercase()) {
            tables.push(table_name.clone());
        }
    }

    // If no tables found, return all available tables
    if tables.is_empty() {
        return available_tables;
    }

    tables
}

/// Find similar strings using Levenshtein distance (simple implementation)
fn find_similar_columns(target: &str, candidates: &[String], max_results: usize) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = candidates
        .iter()
        .map(|c| {
            let distance =
                levenshtein_distance(target.to_lowercase().as_str(), c.to_lowercase().as_str());
            (distance, c.clone())
        })
        .filter(|(dist, _)| *dist <= 3) // Only suggest if edit distance <= 3
        .collect();

    scored.sort_by_key(|(dist, _)| *dist);
    scored.truncate(max_results);
    scored.into_iter().map(|(_, name)| name).collect()
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; len2 + 1]; len1 + 1];

    #[allow(clippy::needless_range_loop)]
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    #[allow(clippy::needless_range_loop)]
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "abd"), 1);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
    }

    #[test]
    fn test_find_similar_columns() {
        let columns = vec![
            "name".to_string(),
            "email".to_string(),
            "phone".to_string(),
            "address".to_string(),
        ];

        let suggestions = find_similar_columns("nam", &columns, 3);
        assert!(suggestions.contains(&"name".to_string()));

        let suggestions = find_similar_columns("emai", &columns, 3);
        assert!(suggestions.contains(&"email".to_string()));

        let suggestions = find_similar_columns("xyz", &columns, 3);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_extract_column_name() {
        assert_eq!(
            extract_column_name("no such column: username"),
            Some("username".to_string())
        );
        assert_eq!(
            extract_column_name("no such column: user_id"),
            Some("user_id".to_string())
        );
        assert_eq!(extract_column_name("other error"), None);
    }

    #[test]
    fn test_extract_table_name() {
        assert_eq!(
            extract_table_name("no such table: users"),
            Some("users".to_string())
        );
        assert_eq!(
            extract_table_name("no such table: customers"),
            Some("customers".to_string())
        );
        assert_eq!(extract_table_name("other error"), None);
    }
}
