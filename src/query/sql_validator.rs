//! Pre-execution SQL validation for inline diagnostics.
//!
//! Analyzes SQL text against loaded CSV schemas to detect errors before execution:
//! - Unknown table references
//! - Unknown column references (with typo suggestions)
//! - Missing JOIN conditions
//! - Ambiguous column references (column exists in multiple tables)

use crate::app::{DiagnosticSeverity, SqlDiagnostic};
use crate::query::error_enhancer::levenshtein_distance;
use crate::query::table_name_from_path;
use std::collections::HashMap;
use std::path::PathBuf;

/// SQL keywords that should not be treated as identifiers
const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "JOIN", "LEFT", "RIGHT", "INNER", "CROSS",
    "OUTER", "NATURAL", "ON", "GROUP", "ORDER", "BY", "HAVING", "LIMIT",
    "OFFSET", "UNION", "ALL", "DISTINCT", "AS", "AND", "OR", "NOT", "IN",
    "BETWEEN", "LIKE", "IS", "NULL", "EXISTS", "CASE", "WHEN", "THEN",
    "ELSE", "END", "ASC", "DESC", "INSERT", "UPDATE", "DELETE", "SET",
    "VALUES", "CREATE", "DROP", "ALTER", "TABLE", "TRUE", "FALSE",
    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "IFNULL", "NULLIF",
    "UPPER", "LOWER", "LENGTH", "TRIM", "SUBSTR", "REPLACE", "CAST",
    "TYPEOF", "ABS", "ROUND", "DATE", "TIME", "DATETIME", "STRFTIME",
    "GROUP_CONCAT", "TOTAL", "EXCEPT", "INTERSECT",
];

/// A token with its position in the source text
#[derive(Debug, Clone)]
struct LocatedToken {
    text: String,
    line: usize,
    col: usize,
}

/// Tokenize SQL text, tracking line and column positions for each token.
fn tokenize_with_positions(sql: &str) -> Vec<LocatedToken> {
    let mut tokens = Vec::new();
    let mut line = 0usize;
    let mut col = 0usize;
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if ch == '\n' {
            line += 1;
            col = 0;
            i += 1;
            continue;
        }

        if ch.is_ascii_whitespace() {
            col += 1;
            i += 1;
            continue;
        }

        // Skip string literals (single-quoted)
        if ch == '\'' {
            let start_col = col;
            i += 1;
            col += 1;
            while i < len {
                if chars[i] == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
                if chars[i] == '\'' {
                    i += 1;
                    col += 1;
                    // Escaped quote ''
                    if i < len && chars[i] == '\'' {
                        i += 1;
                        col += 1;
                        continue;
                    }
                    break;
                }
                i += 1;
            }
            // Don't emit string literals as tokens
            let _ = start_col;
            continue;
        }

        // Punctuation tokens
        if ch == ',' || ch == '(' || ch == ')' || ch == ';' || ch == '.' || ch == '*' {
            tokens.push(LocatedToken {
                text: ch.to_string(),
                line,
                col,
            });
            i += 1;
            col += 1;
            continue;
        }

        // Operators (skip)
        if ch == '=' || ch == '<' || ch == '>' || ch == '!' || ch == '+' || ch == '-' || ch == '/' || ch == '%' {
            i += 1;
            col += 1;
            // Skip multi-char operators
            if i < len && (chars[i] == '=' || chars[i] == '>') {
                i += 1;
                col += 1;
            }
            continue;
        }

        // Numbers (skip)
        if ch.is_ascii_digit() {
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
                col += 1;
            }
            continue;
        }

        // Double-quoted identifier
        if ch == '"' {
            let start_col = col;
            let start_line = line;
            i += 1;
            col += 1;
            let content_start = i;
            while i < len && chars[i] != '"' {
                if chars[i] == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
                i += 1;
            }
            let text: String = chars[content_start..i].iter().collect();
            if i < len {
                i += 1; // skip closing quote
                col += 1;
            }
            tokens.push(LocatedToken {
                text,
                line: start_line,
                col: start_col,
            });
            continue;
        }

        // Identifier or keyword
        if ch.is_alphabetic() || ch == '_' {
            let start_col = col;
            let start_line = line;
            let start_i = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
                col += 1;
            }
            let text: String = chars[start_i..i].iter().collect();
            tokens.push(LocatedToken {
                text,
                line: start_line,
                col: start_col,
            });
            continue;
        }

        // Skip anything else
        i += 1;
        col += 1;
    }

    tokens
}

/// Build a map from table name (lowercase) to its file path and column headers,
/// using a pre-built header map (from SchemaCache) instead of reading from disk.
fn build_schema(
    files: &[PathBuf],
    header_map: &HashMap<PathBuf, Vec<String>>,
) -> HashMap<String, (PathBuf, Vec<String>)> {
    let mut schema = HashMap::new();
    for path in files {
        let table = table_name_from_path(path).to_lowercase();
        if let Some(headers) = header_map.get(path) {
            schema.insert(table, (path.clone(), headers.clone()));
        }
    }
    schema
}

/// Run all validation checks on the SQL text.
///
/// `schema_map` provides pre-cached CSV headers keyed by file path,
/// so the validator does not need to read from disk.
pub fn validate(
    sql: &str,
    files: &[PathBuf],
    schema_map: &HashMap<PathBuf, Vec<String>>,
) -> Vec<SqlDiagnostic> {
    if sql.trim().is_empty() {
        return Vec::new();
    }

    let tokens = tokenize_with_positions(sql);
    if tokens.is_empty() {
        return Vec::new();
    }

    let schema = build_schema(files, schema_map);
    let known_tables: Vec<String> = schema.keys().cloned().collect();

    let mut diagnostics = Vec::new();

    // Parse table references (FROM/JOIN) with their aliases
    let table_refs = parse_table_references(&tokens, &known_tables);

    // Check 1: Unknown table references
    check_unknown_tables(&tokens, &known_tables, &mut diagnostics);

    // Check 2: Missing JOIN conditions
    check_missing_join_conditions(&tokens, &mut diagnostics);

    // Build column set from referenced tables for column validation
    let referenced_tables = resolve_referenced_tables(&table_refs, &schema);
    let all_columns = collect_all_columns(&referenced_tables);
    let column_to_tables = build_column_table_map(&referenced_tables);

    // Check 3: Unknown column references
    check_unknown_columns(&tokens, &table_refs, &all_columns, &schema, &mut diagnostics);

    // Check 4: Ambiguous column references
    check_ambiguous_columns(&tokens, &table_refs, &column_to_tables, &mut diagnostics);

    diagnostics
}

/// A parsed table reference: table name, optional alias, position
#[derive(Debug, Clone)]
struct TableRef {
    table_name: String,  // lowercase
    alias: Option<String>, // lowercase
}

/// Parse FROM/JOIN table references and their aliases.
fn parse_table_references(tokens: &[LocatedToken], known_tables: &[String]) -> Vec<TableRef> {
    let upper: Vec<String> = tokens.iter().map(|t| t.text.to_ascii_uppercase()).collect();
    let mut refs = Vec::new();
    let mut i = 0;

    while i < upper.len() {
        let is_from = upper[i] == "FROM";
        let is_join = upper[i] == "JOIN"
            || (i + 1 < upper.len() && upper[i + 1] == "JOIN"
                && matches!(upper[i].as_str(), "LEFT" | "RIGHT" | "INNER" | "CROSS" | "OUTER" | "NATURAL"));

        if !is_from && !is_join {
            i += 1;
            continue;
        }

        // Skip to the token after FROM/JOIN keyword(s)
        if is_from {
            i += 1;
        } else {
            // Skip compound: LEFT JOIN, etc.
            while i < upper.len() && upper[i] != "JOIN" {
                i += 1;
            }
            if i < upper.len() {
                i += 1; // skip JOIN
            }
        }

        // Parse comma-separated table list (for FROM)
        while i < upper.len() {
            // Skip commas
            if tokens[i].text == "," {
                i += 1;
                continue;
            }

            // Stop at clause keywords
            if is_sql_keyword_non_alias(&upper[i]) {
                break;
            }

            let table_lower = tokens[i].text.to_lowercase();
            i += 1;

            if !known_tables.contains(&table_lower) {
                // Not a known table, still record for reference
                refs.push(TableRef { table_name: table_lower, alias: None });
                continue;
            }

            // Check for optional alias
            let mut alias = None;
            if i < upper.len() && upper[i] == "AS" {
                i += 1; // skip AS
            }
            if i < upper.len()
                && !is_sql_keyword_non_alias(&upper[i])
                && tokens[i].text != ","
                && tokens[i].text != "("
            {
                alias = Some(tokens[i].text.to_lowercase());
                i += 1;
            }

            refs.push(TableRef { table_name: table_lower, alias });

            // For JOIN, only one table before ON
            if !is_from {
                break;
            }
        }
    }

    refs
}

fn is_sql_keyword_non_alias(word: &str) -> bool {
    matches!(word,
        "WHERE" | "GROUP" | "ORDER" | "HAVING" | "SET" | "SELECT" | "ON" | "LIMIT"
        | "LEFT" | "RIGHT" | "INNER" | "CROSS" | "OUTER" | "NATURAL" | "JOIN"
        | "FROM" | "AND" | "OR" | "NOT" | "IN" | "BETWEEN" | "LIKE" | "IS"
        | "NULL" | "UNION" | "EXCEPT" | "INTERSECT"
    )
}

/// Check for unknown table names after FROM/JOIN.
fn check_unknown_tables(
    tokens: &[LocatedToken],
    known_tables: &[String],
    diagnostics: &mut Vec<SqlDiagnostic>,
) {
    let upper: Vec<String> = tokens.iter().map(|t| t.text.to_ascii_uppercase()).collect();
    let mut i = 0;

    while i < upper.len() {
        let is_from = upper[i] == "FROM";
        let is_join = upper[i] == "JOIN";

        if !is_from && !is_join {
            i += 1;
            continue;
        }

        i += 1; // skip FROM/JOIN

        // Parse table references
        while i < upper.len() {
            if tokens[i].text == "," {
                i += 1;
                continue;
            }
            if is_sql_keyword_non_alias(&upper[i]) {
                break;
            }

            let table_lower = tokens[i].text.to_lowercase();
            let tok = &tokens[i];
            i += 1;

            if known_tables.contains(&table_lower) {
                // Skip optional alias
                if i < upper.len() && upper[i] == "AS" {
                    i += 1;
                }
                if i < upper.len() && !is_sql_keyword_non_alias(&upper[i]) && tokens[i].text != "," {
                    i += 1;
                }
            } else {
                // Unknown table - find suggestion
                let mut msg = format!("Unknown table '{}'", tok.text);
                let suggestions = find_similar(
                    &table_lower,
                    known_tables,
                    2,
                );
                if !suggestions.is_empty() {
                    msg.push_str(&format!(". Did you mean: {}?", suggestions.join(", ")));
                }
                diagnostics.push(SqlDiagnostic {
                    line: tok.line,
                    col_start: tok.col,
                    col_end: tok.col + tok.text.chars().count(),
                    message: msg,
                    severity: DiagnosticSeverity::Error,
                });
            }

            if !is_from {
                break;
            }
        }
    }
}

/// Check for JOIN without ON condition.
fn check_missing_join_conditions(
    tokens: &[LocatedToken],
    diagnostics: &mut Vec<SqlDiagnostic>,
) {
    let upper: Vec<String> = tokens.iter().map(|t| t.text.to_ascii_uppercase()).collect();

    for i in 0..upper.len() {
        if upper[i] != "JOIN" {
            continue;
        }

        // Find the JOIN keyword token (could be preceded by LEFT/RIGHT/etc.)
        let join_tok = &tokens[i];

        // Look ahead for ON before the next major clause keyword or another JOIN
        let mut found_on = false;
        let mut j = i + 1;
        // Skip table name and optional alias
        while j < upper.len() && !is_sql_keyword_non_alias(&upper[j]) && tokens[j].text != "," {
            j += 1;
        }
        // Check if ON follows
        if j < upper.len() && upper[j] == "ON" {
            found_on = true;
        }

        if !found_on {
            // Check if it's a CROSS JOIN (no ON needed)
            let is_cross = i > 0 && upper[i - 1] == "CROSS";
            if !is_cross {
                diagnostics.push(SqlDiagnostic {
                    line: join_tok.line,
                    col_start: join_tok.col,
                    col_end: join_tok.col + join_tok.text.chars().count(),
                    message: "JOIN without ON condition".to_string(),
                    severity: DiagnosticSeverity::Warning,
                });
            }
        }
    }
}

/// Resolved table with its column headers.
struct ResolvedTable {
    name: String,
    headers: Vec<String>,
}

/// Resolve table references to their schemas.
fn resolve_referenced_tables(
    table_refs: &[TableRef],
    schema: &HashMap<String, (PathBuf, Vec<String>)>,
) -> Vec<ResolvedTable> {
    let mut result = Vec::new();
    for tr in table_refs {
        if let Some((_, headers)) = schema.get(&tr.table_name) {
            result.push(ResolvedTable {
                name: tr.table_name.clone(),
                headers: headers.clone(),
            });
        }
    }
    result
}

/// Collect all column names from all referenced tables (lowercase).
fn collect_all_columns(tables: &[ResolvedTable]) -> Vec<String> {
    let mut columns = Vec::new();
    for table in tables {
        for h in &table.headers {
            let lower = h.to_lowercase();
            if !columns.contains(&lower) {
                columns.push(lower);
            }
        }
    }
    columns
}

/// Map each column name to the list of tables it appears in.
fn build_column_table_map(tables: &[ResolvedTable]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for table in tables {
        for h in &table.headers {
            map.entry(h.to_lowercase())
                .or_default()
                .push(table.name.clone());
        }
    }
    map
}

/// Determine positions where column identifiers appear (not table positions, not aliases, not keywords).
/// Returns indices into the token array that represent column references.
fn find_column_positions(tokens: &[LocatedToken], table_refs: &[TableRef]) -> Vec<usize> {
    let upper: Vec<String> = tokens.iter().map(|t| t.text.to_ascii_uppercase()).collect();
    let known_aliases: Vec<String> = table_refs.iter().filter_map(|tr| tr.alias.clone()).collect();
    let known_table_names: Vec<String> = table_refs.iter().map(|tr| tr.table_name.clone()).collect();

    let mut positions = Vec::new();

    // Track which indices are table references (FROM/JOIN targets)
    let table_ref_indices = find_table_ref_indices(tokens);

    for i in 0..tokens.len() {
        // Skip punctuation
        if matches!(tokens[i].text.as_str(), "," | "(" | ")" | ";" | "." | "*") {
            continue;
        }

        // Skip SQL keywords
        if SQL_KEYWORDS.contains(&upper[i].as_str()) {
            continue;
        }

        // Skip function arguments (identifier preceded by '(')
        if i > 0 && tokens[i - 1].text == "(" {
            continue;
        }

        // Skip table reference positions
        if table_ref_indices.contains(&i) {
            continue;
        }

        // Skip known aliases
        if known_aliases.contains(&tokens[i].text.to_lowercase()) {
            continue;
        }

        // Skip table names used as qualifiers (followed by ".")
        if i + 1 < tokens.len() && tokens[i + 1].text == "." {
            continue;
        }

        // Skip if preceded by "." (qualified column - handled separately)
        if i > 0 && tokens[i - 1].text == "." {
            // This is a qualified column reference (table.column) - still validate it
            positions.push(i);
            continue;
        }

        // Skip known table names used standalone (might be alias)
        if known_table_names.contains(&tokens[i].text.to_lowercase()) {
            continue;
        }

        // This looks like a column reference
        positions.push(i);
    }

    positions
}

/// Find token indices that are table references (directly after FROM/JOIN).
fn find_table_ref_indices(tokens: &[LocatedToken]) -> Vec<usize> {
    let upper: Vec<String> = tokens.iter().map(|t| t.text.to_ascii_uppercase()).collect();
    let mut indices = Vec::new();
    let mut i = 0;

    while i < upper.len() {
        let is_from = upper[i] == "FROM";
        let is_join = upper[i] == "JOIN";

        if !is_from && !is_join {
            i += 1;
            continue;
        }

        i += 1; // skip FROM/JOIN

        while i < upper.len() {
            if tokens[i].text == "," {
                i += 1;
                continue;
            }
            if is_sql_keyword_non_alias(&upper[i]) {
                break;
            }

            // This is a table name
            indices.push(i);
            i += 1;

            // Skip optional AS + alias
            if i < upper.len() && upper[i] == "AS" {
                indices.push(i);
                i += 1;
            }
            if i < upper.len() && !is_sql_keyword_non_alias(&upper[i]) && tokens[i].text != "," {
                indices.push(i); // alias
                i += 1;
            }

            if !is_from {
                break;
            }
        }
    }

    indices
}

/// Check for unknown column references.
fn check_unknown_columns(
    tokens: &[LocatedToken],
    table_refs: &[TableRef],
    all_columns: &[String],
    schema: &HashMap<String, (PathBuf, Vec<String>)>,
    diagnostics: &mut Vec<SqlDiagnostic>,
) {
    // If no tables are referenced (or no schema info), skip column validation
    if all_columns.is_empty() {
        return;
    }

    let col_positions = find_column_positions(tokens, table_refs);

    for &idx in &col_positions {
        let tok = &tokens[idx];
        let col_lower = tok.text.to_lowercase();

        // Check if preceded by "." (qualified: table.column)
        let is_qualified = idx >= 2 && tokens[idx - 1].text == ".";

        if is_qualified {
            // Get the qualifier (table name or alias)
            let qualifier = tokens[idx - 2].text.to_lowercase();

            // Resolve qualifier to table name
            let table_name = table_refs
                .iter()
                .find(|tr| tr.alias.as_deref() == Some(&qualifier) || tr.table_name == qualifier)
                .map(|tr| &tr.table_name);

            if let Some(table) = table_name {
                if let Some((_, headers)) = schema.get(table.as_str()) {
                    let headers_lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();
                    if !headers_lower.contains(&col_lower) {
                        let mut msg = format!("Unknown column '{}'", tok.text);
                        let suggestions = find_similar(&col_lower, &headers_lower, 2);
                        if !suggestions.is_empty() {
                            msg.push_str(&format!(". Did you mean: {}?", suggestions.join(", ")));
                        }
                        diagnostics.push(SqlDiagnostic {
                            line: tok.line,
                            col_start: tok.col,
                            col_end: tok.col + tok.text.chars().count(),
                            message: msg,
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                }
            }
        } else {
            // Unqualified column: check against all columns from all referenced tables
            if !all_columns.contains(&col_lower) {
                let mut msg = format!("Unknown column '{}'", tok.text);
                let suggestions = find_similar(&col_lower, all_columns, 2);
                if !suggestions.is_empty() {
                    msg.push_str(&format!(". Did you mean: {}?", suggestions.join(", ")));
                }
                diagnostics.push(SqlDiagnostic {
                    line: tok.line,
                    col_start: tok.col,
                    col_end: tok.col + tok.text.chars().count(),
                    message: msg,
                    severity: DiagnosticSeverity::Error,
                });
            }
        }
    }
}

/// Check for ambiguous column references (column exists in multiple joined tables).
fn check_ambiguous_columns(
    tokens: &[LocatedToken],
    table_refs: &[TableRef],
    column_to_tables: &HashMap<String, Vec<String>>,
    diagnostics: &mut Vec<SqlDiagnostic>,
) {
    // Only relevant when multiple tables are referenced
    if table_refs.len() < 2 {
        return;
    }

    let col_positions = find_column_positions(tokens, table_refs);

    for &idx in &col_positions {
        // Skip qualified references (table.column)
        if idx >= 2 && tokens[idx - 1].text == "." {
            continue;
        }

        let tok = &tokens[idx];
        let col_lower = tok.text.to_lowercase();

        if let Some(tables) = column_to_tables.get(&col_lower) {
            if tables.len() > 1 {
                diagnostics.push(SqlDiagnostic {
                    line: tok.line,
                    col_start: tok.col,
                    col_end: tok.col + tok.text.chars().count(),
                    message: format!(
                        "Ambiguous column '{}' exists in tables: {}",
                        tok.text,
                        tables.join(", ")
                    ),
                    severity: DiagnosticSeverity::Warning,
                });
            }
        }
    }
}

/// Find similar strings using Levenshtein distance.
fn find_similar(target: &str, candidates: &[String], max_results: usize) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = candidates
        .iter()
        .map(|c| (levenshtein_distance(target, c), c.clone()))
        .filter(|(dist, _)| *dist <= 3 && *dist > 0)
        .collect();

    scored.sort_by_key(|(dist, _)| *dist);
    scored.truncate(max_results);
    scored.into_iter().map(|(_, name)| name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_csv(name: &str, headers: &str) -> (NamedTempFile, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "{}", headers).unwrap();
        writeln!(f, "dummy,data,row").unwrap();
        // Keep dir alive by leaking it (test only)
        std::mem::forget(dir);
        let tmp = NamedTempFile::new().unwrap();
        (tmp, path)
    }

    /// Build a schema map by reading CSV headers from the given file paths.
    fn build_test_schema(files: &[PathBuf]) -> HashMap<PathBuf, Vec<String>> {
        files
            .iter()
            .filter_map(|p| {
                let mut rdr = csv::ReaderBuilder::new()
                    .has_headers(true)
                    .from_path(p)
                    .ok()?;
                let hdrs = rdr.headers().ok()?.iter().map(String::from).collect();
                Some((p.clone(), hdrs))
            })
            .collect()
    }

    #[test]
    fn test_unknown_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.csv");
        std::fs::write(&path, "id,name\n1,Alice\n").unwrap();

        let files = vec![path];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT * FROM nonexistent", &files, &schema);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Unknown table"));
        assert_eq!(diags[0].severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_valid_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.csv");
        std::fs::write(&path, "id,name\n1,Alice\n").unwrap();

        let files = vec![path];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT * FROM users", &files, &schema);
        // Should have no table errors (may have column diagnostics for *)
        assert!(diags.iter().all(|d| !d.message.contains("Unknown table")));
    }

    #[test]
    fn test_unknown_column() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.csv");
        std::fs::write(&path, "id,name\n1,Alice\n").unwrap();

        let files = vec![path];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT nonexistent FROM users", &files, &schema);
        assert!(diags.iter().any(|d| d.message.contains("Unknown column")));
    }

    #[test]
    fn test_valid_column() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.csv");
        std::fs::write(&path, "id,name\n1,Alice\n").unwrap();

        let files = vec![path];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT name FROM users", &files, &schema);
        assert!(diags.iter().all(|d| !d.message.contains("Unknown column")));
    }

    #[test]
    fn test_missing_join_condition() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("users.csv");
        let p2 = dir.path().join("orders.csv");
        std::fs::write(&p1, "id,name\n1,Alice\n").unwrap();
        std::fs::write(&p2, "id,user_id\n1,1\n").unwrap();

        let files = vec![p1, p2];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT * FROM users JOIN orders", &files, &schema);
        assert!(diags.iter().any(|d| d.message.contains("JOIN without ON")));
    }

    #[test]
    fn test_join_with_on() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("users.csv");
        let p2 = dir.path().join("orders.csv");
        std::fs::write(&p1, "id,name\n1,Alice\n").unwrap();
        std::fs::write(&p2, "id,user_id\n1,1\n").unwrap();

        let files = vec![p1, p2];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT * FROM users JOIN orders ON users.id = orders.user_id", &files, &schema);
        assert!(diags.iter().all(|d| !d.message.contains("JOIN without ON")));
    }

    #[test]
    fn test_cross_join_no_warning() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("users.csv");
        let p2 = dir.path().join("orders.csv");
        std::fs::write(&p1, "id,name\n1,Alice\n").unwrap();
        std::fs::write(&p2, "id,user_id\n1,1\n").unwrap();

        let files = vec![p1, p2];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT * FROM users CROSS JOIN orders", &files, &schema);
        assert!(diags.iter().all(|d| !d.message.contains("JOIN without ON")));
    }

    #[test]
    fn test_ambiguous_column() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("users.csv");
        let p2 = dir.path().join("orders.csv");
        std::fs::write(&p1, "id,name\n1,Alice\n").unwrap();
        std::fs::write(&p2, "id,amount\n1,100\n").unwrap();

        let files = vec![p1, p2];
        let schema = build_test_schema(&files);
        let diags = validate(
            "SELECT id FROM users JOIN orders ON users.id = orders.id",
            &files,
            &schema,
        );
        assert!(diags.iter().any(|d| d.message.contains("Ambiguous column")));
    }

    #[test]
    fn test_typo_suggestion() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("users.csv");
        std::fs::write(&path, "id,name,email\n1,Alice,a@b.c\n").unwrap();

        let files = vec![path];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT nmae FROM users", &files, &schema);
        assert!(diags.iter().any(|d| d.message.contains("Did you mean")));
    }

    #[test]
    fn test_function_arguments_not_flagged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("orders.csv");
        std::fs::write(&path, "id,amount\n1,100\n").unwrap();

        let files = vec![path];
        let schema = build_test_schema(&files);
        let diags = validate("SELECT SUM(amount), COUNT(*) FROM orders", &files, &schema);
        assert!(diags.iter().all(|d| !d.message.contains("Unknown column")));
    }

    #[test]
    fn test_empty_query() {
        let diags = validate("", &[], &HashMap::new());
        assert!(diags.is_empty());
    }
}
