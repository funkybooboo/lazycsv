//! SQL Editor mode input handling
//!
//! This module handles keyboard input when the user is editing SQL queries
//! (after pressing 'q' in Normal mode). Delegates to VimEditor for text editing.

use crate::app::{App, CompletionItem, CompletionKind, Mode, SqlCompletion};
use crate::input::{InputResult, StatusMessage};
use crate::vim_editor::VimMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};

const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "JOIN", "LEFT JOIN", "RIGHT JOIN",
    "INNER JOIN", "CROSS JOIN", "ON", "GROUP BY", "ORDER BY",
    "HAVING", "LIMIT", "OFFSET", "UNION", "UNION ALL", "DISTINCT",
    "AS", "AND", "OR", "NOT", "IN", "BETWEEN", "LIKE", "IS NULL",
    "IS NOT NULL", "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END",
    "ASC", "DESC", "INSERT", "UPDATE", "DELETE", "SET", "VALUES",
    "CREATE", "DROP", "ALTER", "TABLE",
];

const SQL_FUNCTIONS: &[&str] = &[
    "COUNT", "SUM", "AVG", "MIN", "MAX",
    "COALESCE", "IFNULL", "NULLIF",
    "UPPER", "LOWER", "LENGTH", "TRIM", "SUBSTR", "REPLACE",
    "CAST", "TYPEOF", "ABS", "ROUND",
    "DATE", "TIME", "DATETIME", "STRFTIME",
    "GROUP_CONCAT", "TOTAL",
];

/// The SQL clause context at the cursor position
enum CompletionContext {
    /// After SELECT → columns, *, functions
    Select,
    /// After FROM/JOIN → table names
    From,
    /// After WHERE/HAVING → columns, functions
    Where,
    /// After GROUP BY → columns
    GroupBy,
    /// After ORDER BY → columns, ASC/DESC
    OrderBy,
    /// After "alias." → columns from that table
    AliasPrefix(String),
    /// Fallback → keywords
    General,
}

/// Handle keyboard input in SQL editor mode
pub fn handle(app: &mut App, key: KeyEvent) -> Result<InputResult> {
    let editor = match app.sql_vim_editor.as_mut() {
        Some(e) => e,
        None => {
            // No vim editor — fall back to closing the SQL editor
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
    };

    let vim_mode = editor.mode();

    // --- Completion popup active ---
    if app.sql_completion.is_some() {
        match key.code {
            KeyCode::Down => {
                if let Some(ref mut comp) = app.sql_completion {
                    comp.move_down();
                }
                return Ok(InputResult::Continue);
            }
            KeyCode::Up => {
                if let Some(ref mut comp) = app.sql_completion {
                    comp.move_up();
                }
                return Ok(InputResult::Continue);
            }
            KeyCode::Enter | KeyCode::Tab => {
                // Insert selected item at cursor, replacing the already-typed prefix
                let selected = app
                    .sql_completion
                    .as_ref()
                    .and_then(|c| {
                        let item = c.selected_item()?;
                        Some((item.text.clone(), c.prefix_len))
                    });
                app.sql_completion = None;

                if let Some((text, prefix_len)) = selected {
                    if let Some(ref mut ed) = app.sql_vim_editor {
                        if ed.mode() != VimMode::Insert {
                            ed.enter_insert_mode();
                        }
                        // Delete the already-typed prefix characters
                        for _ in 0..prefix_len {
                            ed.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
                        }
                        // Insert the full completion text
                        for ch in text.chars() {
                            ed.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
                        }
                    }
                }
                return Ok(InputResult::Continue);
            }
            KeyCode::Backspace => {
                if let Some(ref mut comp) = app.sql_completion {
                    if comp.filter.is_empty() {
                        app.sql_completion = None;
                    } else {
                        comp.pop_filter();
                        // Dismiss if nothing matches
                        if comp.filtered_items().is_empty() {
                            app.sql_completion = None;
                        }
                    }
                }
                return Ok(InputResult::Continue);
            }
            KeyCode::Char(ch) if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT => {
                if let Some(ref mut comp) = app.sql_completion {
                    comp.push_filter(ch);
                    // Dismiss if nothing matches
                    if comp.filtered_items().is_empty() {
                        app.sql_completion = None;
                    }
                }
                return Ok(InputResult::Continue);
            }
            KeyCode::Esc => {
                app.sql_completion = None;
                return Ok(InputResult::Continue);
            }
            _ => {
                // Any other key dismisses the completion and is processed normally
                app.sql_completion = None;
            }
        }
    }

    // --- Ctrl+N: context-aware completion ---
    if key.code == KeyCode::Char('n') && key.modifiers.contains(KeyModifiers::CONTROL) {
        let sql_text = editor.content();
        let (cursor_line, cursor_col) = editor.cursor();

        let text_before_cursor = text_up_to_cursor(&sql_text, cursor_line, cursor_col);
        let prefix = word_before_cursor(&text_before_cursor).to_string();
        let files = app.session.files().to_vec();
        let context = detect_completion_context(&text_before_cursor);

        let items: Vec<CompletionItem> = match context {
            CompletionContext::From => {
                files
                    .iter()
                    .map(|p| CompletionItem {
                        text: crate::query::table_name_from_path(p),
                        kind: CompletionKind::Table,
                    })
                    .collect()
            }
            CompletionContext::AliasPrefix(alias) => {
                let aliases = parse_table_aliases(&sql_text, &files);
                let headers = if let Some(path) = resolve_alias(&alias, &aliases, &files) {
                    read_csv_headers(&path).unwrap_or_default()
                } else {
                    Vec::new()
                };
                headers.into_iter().map(|h| CompletionItem {
                    text: h,
                    kind: CompletionKind::Column,
                }).collect()
            }
            CompletionContext::Select => {
                let mut items = vec![CompletionItem { text: "*".to_string(), kind: CompletionKind::Keyword }];
                items.extend(column_items_from_query(&sql_text, &files));
                items.extend(function_items());
                items.extend(keyword_items(&["DISTINCT", "CASE"]));
                items
            }
            CompletionContext::Where => {
                let mut items = column_items_from_query(&sql_text, &files);
                items.extend(function_items());
                items.extend(keyword_items(&["AND", "OR", "NOT", "IN", "BETWEEN", "LIKE", "IS NULL", "IS NOT NULL", "EXISTS"]));
                items
            }
            CompletionContext::GroupBy => {
                column_items_from_query(&sql_text, &files)
            }
            CompletionContext::OrderBy => {
                let mut items = column_items_from_query(&sql_text, &files);
                items.extend(keyword_items(&["ASC", "DESC"]));
                items
            }
            CompletionContext::General => {
                let mut items: Vec<CompletionItem> = SQL_KEYWORDS.iter().map(|kw| CompletionItem {
                    text: kw.to_string(),
                    kind: CompletionKind::Keyword,
                }).collect();
                items.extend(function_items());
                items.extend(column_items_from_query(&sql_text, &files));
                items
            }
        };

        if !items.is_empty() {
            let mut comp = SqlCompletion::new(items, &prefix);
            // If pre-filtered to nothing, don't show
            if comp.filtered_items().is_empty() {
                comp.filter.clear();
                comp.prefix_len = 0;
            }
            if !comp.filtered_items().is_empty() {
                app.sql_completion = Some(comp);
            }
        }
        return Ok(InputResult::Continue);
    }

    // In vim Normal mode, Esc exits the SQL editor entirely
    if vim_mode == VimMode::Normal && key.code == KeyCode::Esc {
        app.sql_buffer = editor.content();
        app.mode = Mode::Normal;
        app.sql_vim_editor = None;
        return Ok(InputResult::Continue);
    }

    // Ctrl+Enter executes the query from any vim mode.
    if key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL) {
        if vim_mode != VimMode::Normal {
            editor.exit_insert_mode();
        }
        let query = editor.content().trim().to_string();
        app.sql_buffer = editor.content();
        if query.is_empty() {
            app.status_message = Some(StatusMessage::new_owned("Empty query".to_string()));
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
        return Ok(InputResult::ExecuteQuery { query });
    }

    // In vim Normal mode, plain Enter also executes the query
    if vim_mode == VimMode::Normal
        && key.code == KeyCode::Enter
        && key.modifiers == KeyModifiers::NONE
    {
        let query = editor.content().trim().to_string();
        app.sql_buffer = editor.content();
        if query.is_empty() {
            app.status_message = Some(StatusMessage::new_owned("Empty query".to_string()));
            app.mode = Mode::Normal;
            return Ok(InputResult::Continue);
        }
        return Ok(InputResult::ExecuteQuery { query });
    }

    // Delegate all other keys to the vim editor
    editor.handle_key(key);

    // Check for ex commands (:q exits, :w could execute, etc.)
    if let Some(cmd) = editor.check_ex_command() {
        match cmd.as_str() {
            "q" | "q!" => {
                app.sql_buffer = editor.content();
                app.mode = Mode::Normal;
            }
            "w" | "wq" => {
                let query = editor.content().trim().to_string();
                app.sql_buffer = editor.content();
                if query.is_empty() {
                    app.status_message =
                        Some(StatusMessage::new_owned("Empty query".to_string()));
                    app.mode = Mode::Normal;
                    return Ok(InputResult::Continue);
                }
                return Ok(InputResult::ExecuteQuery { query });
            }
            _ => {}
        }
    }

    Ok(InputResult::Continue)
}

/// Get the SQL text up to the cursor position as a single string.
fn text_up_to_cursor(sql: &str, cursor_line: usize, cursor_col: usize) -> String {
    let mut result = String::new();
    for (i, line) in sql.lines().enumerate() {
        if i == cursor_line {
            let end = cursor_col.min(line.len());
            result.push_str(&line[..end]);
            break;
        }
        result.push_str(line);
        result.push(' '); // normalize newlines to spaces
    }
    result
}

/// Extract the partial word immediately before the cursor.
/// Used to pre-fill the completion filter and to know how many chars to replace on accept.
fn word_before_cursor(text_before_cursor: &str) -> &str {
    let bytes = text_before_cursor.as_bytes();
    let mut end = bytes.len();
    // Walk backwards over word characters
    while end > 0 && (bytes[end - 1].is_ascii_alphanumeric() || bytes[end - 1] == b'_') {
        end -= 1;
    }
    &text_before_cursor[end..]
}

/// Determine the SQL clause context at cursor position.
fn detect_completion_context(text_before_cursor: &str) -> CompletionContext {
    // Check for alias.prefix first
    if let Some(alias) = alias_prefix_at_cursor(text_before_cursor) {
        return CompletionContext::AliasPrefix(alias);
    }

    let upper = text_before_cursor.to_ascii_uppercase();

    // Find the last occurrence of each clause keyword
    let clauses: &[(&str, fn() -> CompletionContext)] = &[
        ("SELECT", || CompletionContext::Select),
        ("FROM", || CompletionContext::From),
        ("JOIN", || CompletionContext::From),
        ("INNER JOIN", || CompletionContext::From),
        ("LEFT JOIN", || CompletionContext::From),
        ("RIGHT JOIN", || CompletionContext::From),
        ("CROSS JOIN", || CompletionContext::From),
        ("WHERE", || CompletionContext::Where),
        ("HAVING", || CompletionContext::Where),
        ("GROUP BY", || CompletionContext::GroupBy),
        ("ORDER BY", || CompletionContext::OrderBy),
    ];

    let mut best_pos: Option<usize> = None;
    let mut best_ctx: Option<CompletionContext> = None;

    for (kw, make_ctx) in clauses {
        if let Some(pos) = upper.rfind(kw) {
            let after = pos + kw.len();
            // Verify it's a whole keyword (followed by whitespace or end)
            if after >= upper.len() || upper.as_bytes()[after].is_ascii_whitespace() {
                if best_pos.map_or(true, |prev| pos > prev) {
                    best_pos = Some(pos);
                    best_ctx = Some(make_ctx());
                }
            }
        }
    }

    best_ctx.unwrap_or(CompletionContext::General)
}

/// Build CompletionItems for SQL functions
fn function_items() -> Vec<CompletionItem> {
    SQL_FUNCTIONS.iter().map(|f| CompletionItem {
        text: f.to_string(),
        kind: CompletionKind::Function,
    }).collect()
}

/// Build CompletionItems for specific keywords
fn keyword_items(keywords: &[&str]) -> Vec<CompletionItem> {
    keywords.iter().map(|kw| CompletionItem {
        text: kw.to_string(),
        kind: CompletionKind::Keyword,
    }).collect()
}

/// Get column CompletionItems from all tables referenced in the query
fn column_items_from_query(sql: &str, files: &[PathBuf]) -> Vec<CompletionItem> {
    let referenced = crate::query::files_referenced_by_query(sql, files);
    let mut columns = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for path in referenced {
        if let Ok(headers) = read_csv_headers(path) {
            for h in headers {
                if seen.insert(h.clone()) {
                    columns.push(CompletionItem {
                        text: h,
                        kind: CompletionKind::Column,
                    });
                }
            }
        }
    }
    columns
}

/// Read just the header row from a CSV file (first line).
fn read_csv_headers(path: &Path) -> Result<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    let headers = reader.headers()?.clone();
    Ok(headers.iter().map(String::from).collect())
}

/// Check if cursor is right after "alias." and return the alias name.
/// e.g. "select t." → Some("t"), "select t.col" → None (already typing)
fn alias_prefix_at_cursor(text_before_cursor: &str) -> Option<String> {
    let trimmed = text_before_cursor.trim_end();
    if !trimmed.ends_with('.') {
        return None;
    }

    // Get the word before the dot
    let before_dot = &trimmed[..trimmed.len() - 1];
    let word: String = before_dot
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect();

    if word.is_empty() {
        None
    } else {
        Some(word)
    }
}

/// Alias entry: table name (as derived from file) and optional alias.
struct TableAlias {
    table_name: String,
    alias: Option<String>,
}

/// Parse table names and aliases from FROM/JOIN clauses in the SQL.
/// Handles patterns like: FROM table1 t, FROM table1 AS t, JOIN table2 t2
fn parse_table_aliases(sql: &str, files: &[PathBuf]) -> Vec<TableAlias> {
    let tokens = tokenize_sql(sql);
    let upper_tokens: Vec<String> = tokens.iter().map(|t| t.to_ascii_uppercase()).collect();

    let table_keywords = ["FROM", "JOIN"];
    let non_alias_keywords = [
        "WHERE", "GROUP", "ORDER", "HAVING", "SET", "SELECT", "ON", "LIMIT", "LEFT", "RIGHT",
        "INNER", "CROSS", "OUTER", "NATURAL", "JOIN", "FROM", "AS", "AND", "OR", "NOT", "IN",
        "BETWEEN", "LIKE", "IS", "NULL", "TRUE", "FALSE", "CASE", "WHEN", "THEN", "ELSE", "END",
        "UNION", "EXCEPT", "INTERSECT",
    ];

    // Build set of known table names for matching
    let known_tables: Vec<String> = files
        .iter()
        .map(|p| crate::query::table_name_from_path(p).to_ascii_lowercase())
        .collect();

    let mut aliases = Vec::new();
    let mut i = 0;

    while i < upper_tokens.len() {
        // Look for FROM or JOIN keywords
        let is_table_kw = table_keywords.contains(&upper_tokens[i].as_str());
        if !is_table_kw {
            i += 1;
            continue;
        }

        i += 1; // skip the keyword

        // Now parse comma-separated table references: table1 t, table2 t2
        while i < upper_tokens.len() {
            // Skip commas
            if tokens[i] == "," {
                i += 1;
                continue;
            }

            // If we hit a keyword, stop
            if non_alias_keywords.contains(&upper_tokens[i].as_str()) {
                break;
            }

            // This should be a table name
            let table_name = tokens[i].to_string();
            let table_lower = table_name.to_ascii_lowercase();
            i += 1;

            // Check if it's a known table
            if !known_tables.contains(&table_lower) {
                continue;
            }

            // Check for optional alias: skip "AS" if present
            let mut alias = None;
            if i < upper_tokens.len() && upper_tokens[i] == "AS" {
                i += 1; // skip AS
            }

            // Next non-keyword token is the alias
            if i < upper_tokens.len()
                && !non_alias_keywords.contains(&upper_tokens[i].as_str())
                && tokens[i] != ","
            {
                alias = Some(tokens[i].to_string());
                i += 1;
            }

            aliases.push(TableAlias { table_name, alias });
        }
    }

    aliases
}

/// Resolve an alias (or table name) to a file path.
fn resolve_alias(alias: &str, aliases: &[TableAlias], files: &[PathBuf]) -> Option<PathBuf> {
    let alias_lower = alias.to_ascii_lowercase();

    // First check if it matches any alias
    for entry in aliases {
        if let Some(ref a) = entry.alias {
            if a.to_ascii_lowercase() == alias_lower {
                return find_file_for_table(&entry.table_name, files);
            }
        }
    }

    // Then check if it's a direct table name
    for entry in aliases {
        if entry.table_name.to_ascii_lowercase() == alias_lower {
            return find_file_for_table(&entry.table_name, files);
        }
    }

    // Fallback: try matching directly against file-derived table names
    find_file_for_table(alias, files)
}

/// Find the file path for a given table name.
fn find_file_for_table(table_name: &str, files: &[PathBuf]) -> Option<PathBuf> {
    let table_lower = table_name.to_ascii_lowercase();
    files
        .iter()
        .find(|p| crate::query::table_name_from_path(p).to_ascii_lowercase() == table_lower)
        .cloned()
}

/// Simple SQL tokenizer: splits on whitespace and punctuation, preserving tokens.
fn tokenize_sql(sql: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in sql.chars() {
        if ch.is_ascii_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else if ch == ',' || ch == '(' || ch == ')' || ch == ';' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push(ch.to_string());
        } else if ch == '.' {
            // Keep dot separate so "t.col" becomes ["t", ".", "col"]
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push(".".to_string());
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
