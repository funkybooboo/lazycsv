//! SQL Editor mode input handling
//!
//! This module handles keyboard input when the user is editing SQL queries
//! (after pressing 'q' in Normal mode). Delegates to VimEditor for text editing.

use crate::app::{App, CompletionItem, CompletionKind, Mode, SqlCompletion, TemplateStep};
use crate::input::{InputResult, StatusMessage};
use crate::vim_editor::VimMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

const SQL_KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "JOIN",
    "LEFT JOIN",
    "RIGHT JOIN",
    "INNER JOIN",
    "CROSS JOIN",
    "ON",
    "GROUP BY",
    "ORDER BY",
    "HAVING",
    "LIMIT",
    "OFFSET",
    "UNION",
    "UNION ALL",
    "DISTINCT",
    "AS",
    "AND",
    "OR",
    "NOT",
    "IN",
    "BETWEEN",
    "LIKE",
    "IS NULL",
    "IS NOT NULL",
    "EXISTS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "ASC",
    "DESC",
    "INSERT",
    "UPDATE",
    "DELETE",
    "SET",
    "VALUES",
    "CREATE",
    "DROP",
    "ALTER",
    "TABLE",
];

const SQL_FUNCTIONS: &[&str] = &[
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "COALESCE",
    "IFNULL",
    "NULLIF",
    "UPPER",
    "LOWER",
    "LENGTH",
    "TRIM",
    "SUBSTR",
    "REPLACE",
    "CAST",
    "TYPEOF",
    "ABS",
    "ROUND",
    "DATE",
    "TIME",
    "DATETIME",
    "STRFTIME",
    "GROUP_CONCAT",
    "TOTAL",
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
                // Get selected item info before clearing completion
                let selected = app.sql_completion.as_ref().and_then(|c| {
                    let item = c.selected_item()?;
                    Some((
                        item.text.clone(),
                        item.kind,
                        item.template.clone(),
                        item.template_steps.clone(),
                        c.prefix_len,
                    ))
                });
                app.sql_completion = None;

                if let Some((text, kind, template, steps, prefix_len)) = selected {
                    if let Some(ref template_sql) = template {
                        // Template: replace entire editor content and position cursor at end
                        let mut new_editor =
                            crate::vim_editor::VimEditor::new(template_sql.clone());
                        // Enter insert mode first so clamp_cursor allows cursor after last char
                        new_editor.enter_insert_mode();
                        let line_count = new_editor.line_count();
                        if line_count > 0 {
                            let last_line = line_count - 1;
                            let last_col = new_editor.lines()[last_line].chars().count();
                            new_editor.set_cursor_for_test(last_line, last_col);
                        }
                        app.sql_vim_editor = Some(new_editor);

                        // Store remaining template steps
                        app.sql_template_steps = steps;

                        // Execute pending steps (will show table picker if next step is PickTable)
                        execute_template_steps(app);
                    } else {
                        // Normal completion: insert at cursor
                        if let Some(ref mut ed) = app.sql_vim_editor {
                            if ed.mode() != VimMode::Insert {
                                ed.enter_insert_mode();
                            }
                            // Delete the already-typed prefix characters
                            for _ in 0..prefix_len {
                                ed.handle_key(KeyEvent::new(
                                    KeyCode::Backspace,
                                    KeyModifiers::NONE,
                                ));
                            }
                            // Insert the full completion text, quoting if needed
                            let insert_text = quote_if_needed(&text);
                            for ch in insert_text.chars() {
                                ed.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
                            }
                        }
                        // Record for RepeatLastColumn / Assemble
                        if !app.sql_template_steps.is_empty() {
                            match kind {
                                CompletionKind::Table => {
                                    app.sql_template_last_table = Some(text.clone());
                                }
                                CompletionKind::Column => {
                                    app.sql_template_last_column = Some(text.clone());
                                }
                                _ => {}
                            }
                        }
                        execute_template_steps(app);
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
            KeyCode::Char(ch)
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
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
                app.sql_template_steps.clear();
                app.sql_template_last_column = None;
                app.sql_template_last_table = None;
                return Ok(InputResult::Continue);
            }
            _ => {
                // Any other key dismisses the completion and is processed normally
                app.sql_completion = None;
                app.sql_template_steps.clear();
                app.sql_template_last_column = None;
                app.sql_template_last_table = None;
            }
        }
    }

    // --- Ctrl+N: context-aware completion (or templates if empty) ---
    if key.code == KeyCode::Char('n') && key.modifiers.contains(KeyModifiers::CONTROL) {
        let sql_text = editor.content();

        // If editor is empty, show query templates instead of completions
        if sql_text.trim().is_empty() {
            let items = build_template_items();
            if !items.is_empty() {
                app.sql_completion = Some(SqlCompletion::new(items, ""));
            }
            return Ok(InputResult::Continue);
        }

        let (cursor_line, cursor_col) = editor.cursor();

        let text_before_cursor = text_up_to_cursor(&sql_text, cursor_line, cursor_col);
        let prefix = word_before_cursor(&text_before_cursor).to_string();
        let files = app.session.files().to_vec();
        let context = detect_completion_context(&text_before_cursor);

        let items: Vec<CompletionItem> = match context {
            CompletionContext::From => files
                .iter()
                .map(|p| CompletionItem {
                    text: crate::query::table_name_from_path(p),
                    kind: CompletionKind::Table,
                    template: None,
                    template_steps: vec![],
                })
                .collect(),
            CompletionContext::AliasPrefix(alias) => {
                let aliases = parse_table_aliases(&sql_text, &files);
                let headers = if let Some(path) = resolve_alias(&alias, &aliases, &files) {
                    app.schema_cache.get_or_read(&path).unwrap_or_default()
                } else {
                    Vec::new()
                };
                headers
                    .into_iter()
                    .map(|h| CompletionItem {
                        text: h,
                        kind: CompletionKind::Column,
                        template: None,
                        template_steps: vec![],
                    })
                    .collect()
            }
            CompletionContext::Select => {
                let mut items = vec![CompletionItem {
                    text: "*".to_string(),
                    kind: CompletionKind::Keyword,
                    template: None,
                    template_steps: vec![],
                }];
                items.extend(column_items_from_query(&sql_text, &files, &mut app.schema_cache));
                items.extend(function_items());
                items.extend(keyword_items(&["DISTINCT", "CASE"]));
                items
            }
            CompletionContext::Where => {
                let mut items = column_items_from_query(&sql_text, &files, &mut app.schema_cache);
                items.extend(function_items());
                items.extend(keyword_items(&[
                    "AND",
                    "OR",
                    "NOT",
                    "IN",
                    "BETWEEN",
                    "LIKE",
                    "IS NULL",
                    "IS NOT NULL",
                    "EXISTS",
                ]));
                items
            }
            CompletionContext::GroupBy => column_items_from_query(&sql_text, &files, &mut app.schema_cache),
            CompletionContext::OrderBy => {
                let mut items = column_items_from_query(&sql_text, &files, &mut app.schema_cache);
                items.extend(keyword_items(&["ASC", "DESC"]));
                items
            }
            CompletionContext::General => {
                let mut items: Vec<CompletionItem> = SQL_KEYWORDS
                    .iter()
                    .map(|kw| CompletionItem {
                        text: kw.to_string(),
                        kind: CompletionKind::Keyword,
                        template: None,
                        template_steps: vec![],
                    })
                    .collect();
                items.extend(function_items());
                items.extend(column_items_from_query(&sql_text, &files, &mut app.schema_cache));
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
                    app.status_message = Some(StatusMessage::new_owned("Empty query".to_string()));
                    app.mode = Mode::Normal;
                    return Ok(InputResult::Continue);
                }
                return Ok(InputResult::ExecuteQuery { query });
            }
            _ => {}
        }
    }

    // Run inline validation after each keystroke
    if let Some(ref ed) = app.sql_vim_editor {
        let sql_text = ed.content();
        let files = app.session.files().to_vec();
        // Build schema from cache for validator
        let schema: std::collections::HashMap<std::path::PathBuf, Vec<String>> = files
            .iter()
            .filter_map(|p| app.schema_cache.get_or_read(p).map(|h| (p.clone(), h)))
            .collect();
        app.sql_diagnostics =
            crate::query::sql_validator::validate(&sql_text, &files, &schema);
    }

    Ok(InputResult::Continue)
}

/// Insert text into a VimEditor, handling newlines as Enter key presses.
fn insert_text_into_editor(ed: &mut crate::vim_editor::VimEditor, text: &str) {
    for ch in text.chars() {
        if ch == '\n' {
            ed.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        } else {
            ed.handle_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
    }
}

/// Drain pending template steps, inserting literal text and stopping at the
/// first `PickTable` or `PickColumn` step to show a completion popup.
fn execute_template_steps(app: &mut App) {
    while let Some(step) = app.sql_template_steps.first().cloned() {
        match step {
            TemplateStep::Text(text) => {
                app.sql_template_steps.remove(0);
                if let Some(ref mut ed) = app.sql_vim_editor {
                    if ed.mode() != VimMode::Insert {
                        ed.enter_insert_mode();
                    }
                    insert_text_into_editor(ed, &text);
                }
            }
            TemplateStep::PickTable => {
                app.sql_template_steps.remove(0);
                let files = app.session.files().to_vec();
                let items: Vec<CompletionItem> = files
                    .iter()
                    .map(|p| CompletionItem {
                        text: crate::query::table_name_from_path(p),
                        kind: CompletionKind::Table,
                        template: None,
                        template_steps: vec![],
                    })
                    .collect();
                if !items.is_empty() {
                    app.sql_completion = Some(SqlCompletion::new(items, ""));
                }
                break;
            }
            TemplateStep::PickColumn(alias) => {
                app.sql_template_steps.remove(0);
                let sql_text = app
                    .sql_vim_editor
                    .as_ref()
                    .map(|ed| ed.content())
                    .unwrap_or_default();
                let files = app.session.files().to_vec();

                let headers = if alias == "*" {
                    // Collect columns from all tables referenced in the query
                    let referenced = crate::query::files_referenced_by_query(&sql_text, &files);
                    let mut cols = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    for path in referenced {
                        if let Some(hdrs) = app.schema_cache.get_or_read(path) {
                            for h in hdrs {
                                if seen.insert(h.clone()) {
                                    cols.push(h);
                                }
                            }
                        }
                    }
                    cols
                } else {
                    // Resolve a specific alias to its table's columns
                    let aliases = parse_table_aliases(&sql_text, &files);
                    resolve_alias(&alias, &aliases, &files)
                        .and_then(|path| app.schema_cache.get_or_read(&path))
                        .unwrap_or_default()
                };

                let items: Vec<CompletionItem> = headers
                    .into_iter()
                    .map(|h| CompletionItem {
                        text: h,
                        kind: CompletionKind::Column,
                        template: None,
                        template_steps: vec![],
                    })
                    .collect();
                if !items.is_empty() {
                    app.sql_completion = Some(SqlCompletion::new(items, ""));
                }
                break;
            }
            TemplateStep::RepeatLastColumn => {
                app.sql_template_steps.remove(0);
                if let Some(ref col) = app.sql_template_last_column {
                    let quoted = quote_if_needed(col);
                    if let Some(ref mut ed) = app.sql_vim_editor {
                        if ed.mode() != VimMode::Insert {
                            ed.enter_insert_mode();
                        }
                        insert_text_into_editor(ed, &quoted);
                    }
                }
            }
            TemplateStep::Assemble(fmt) => {
                app.sql_template_steps.remove(0);
                let table = app.sql_template_last_table.as_deref().unwrap_or("table");
                let column = app.sql_template_last_column.as_deref().unwrap_or("column");
                let table_q = quote_if_needed(table);
                let column_q = quote_if_needed(column);
                let sql = fmt.replace("{table}", &table_q).replace("{column}", &column_q);
                let mut new_editor = crate::vim_editor::VimEditor::new(sql);
                new_editor.enter_insert_mode();
                let line_count = new_editor.line_count();
                if line_count > 0 {
                    let last_line = line_count - 1;
                    let last_col = new_editor.lines()[last_line].chars().count();
                    new_editor.set_cursor_for_test(last_line, last_col);
                }
                app.sql_vim_editor = Some(new_editor);
            }
        }
    }
}

/// Check if an identifier needs double-quoting for SQL.
fn needs_quoting(identifier: &str) -> bool {
    identifier.chars().any(|c| !c.is_alphanumeric() && c != '_')
}

/// Wrap an identifier in double quotes if it contains special characters.
fn quote_if_needed(identifier: &str) -> String {
    if needs_quoting(identifier) {
        format!("\"{}\"", identifier)
    } else {
        identifier.to_string()
    }
}

/// Get the SQL text up to the cursor position as a single string.
fn text_up_to_cursor(sql: &str, cursor_line: usize, cursor_col: usize) -> String {
    let mut result = String::new();
    for (i, line) in sql.lines().enumerate() {
        if i == cursor_line {
            let prefix: String = line.chars().take(cursor_col).collect();
            result.push_str(&prefix);
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
    // Walk backwards over chars that are alphanumeric (including Unicode) or underscore,
    // tracking the byte offset so we can return a valid &str slice.
    let mut byte_offset = text_before_cursor.len();
    for ch in text_before_cursor.chars().rev() {
        if ch.is_alphanumeric() || ch == '_' {
            byte_offset -= ch.len_utf8();
        } else {
            break;
        }
    }
    &text_before_cursor[byte_offset..]
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
    SQL_FUNCTIONS
        .iter()
        .map(|f| CompletionItem {
            text: f.to_string(),
            kind: CompletionKind::Function,
            template: None,
            template_steps: vec![],
        })
        .collect()
}

/// Build CompletionItems for specific keywords
fn keyword_items(keywords: &[&str]) -> Vec<CompletionItem> {
    keywords
        .iter()
        .map(|kw| CompletionItem {
            text: kw.to_string(),
            kind: CompletionKind::Keyword,
            template: None,
            template_steps: vec![],
        })
        .collect()
}

/// Get column CompletionItems from all tables referenced in the query
fn column_items_from_query(
    sql: &str,
    files: &[PathBuf],
    schema_cache: &mut crate::app::SchemaCache,
) -> Vec<CompletionItem> {
    let referenced = crate::query::files_referenced_by_query(sql, files);
    let mut columns = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for path in referenced {
        if let Some(headers) = schema_cache.get_or_read(path) {
            for h in headers {
                if seen.insert(h.clone()) {
                    columns.push(CompletionItem {
                        text: h,
                        kind: CompletionKind::Column,
                        template: None,
                        template_steps: vec![],
                    });
                }
            }
        }
    }
    columns
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
        .take_while(|c| c.is_alphanumeric() || *c == '_')
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
        "WHERE",
        "GROUP",
        "ORDER",
        "HAVING",
        "SET",
        "SELECT",
        "ON",
        "LIMIT",
        "LEFT",
        "RIGHT",
        "INNER",
        "CROSS",
        "OUTER",
        "NATURAL",
        "JOIN",
        "FROM",
        "AS",
        "AND",
        "OR",
        "NOT",
        "IN",
        "BETWEEN",
        "LIKE",
        "IS",
        "NULL",
        "TRUE",
        "FALSE",
        "CASE",
        "WHEN",
        "THEN",
        "ELSE",
        "END",
        "UNION",
        "EXCEPT",
        "INTERSECT",
    ];

    // Build set of known table names for matching
    let known_tables: Vec<String> = files
        .iter()
        .map(|p| crate::query::table_name_from_path(p).to_lowercase())
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
            let table_lower = table_name.to_lowercase();
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
    let alias_lower = alias.to_lowercase();

    // First check if it matches any alias
    for entry in aliases {
        if let Some(ref a) = entry.alias {
            if a.to_lowercase() == alias_lower {
                return find_file_for_table(&entry.table_name, files);
            }
        }
    }

    // Then check if it's a direct table name
    for entry in aliases {
        if entry.table_name.to_lowercase() == alias_lower {
            return find_file_for_table(&entry.table_name, files);
        }
    }

    // Fallback: try matching directly against file-derived table names
    find_file_for_table(alias, files)
}

/// Find the file path for a given table name.
fn find_file_for_table(table_name: &str, files: &[PathBuf]) -> Option<PathBuf> {
    let table_lower = table_name.to_lowercase();
    files
        .iter()
        .find(|p| crate::query::table_name_from_path(p).to_lowercase() == table_lower)
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
        } else if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            // Skip operators and other non-identifier characters
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Build query template items.
///
/// Returns 4 templates: select-all, join-two, group-count, order-limit.
/// Templates that reference a table name leave the table position blank and
/// Templates that reference a table name use `TemplateStep::PickTable` steps
/// to trigger a FROM-context completion popup, letting the user pick interactively.
fn build_template_items() -> Vec<CompletionItem> {
    use crate::app::TemplateStep;
    vec![
        CompletionItem {
            text: "Select All".to_string(),
            kind: CompletionKind::Keyword,
            template: Some("SELECT *\nFROM ".to_string()),
            template_steps: vec![TemplateStep::PickTable],
        },
        CompletionItem {
            text: "Join Two Tables".to_string(),
            kind: CompletionKind::Keyword,
            template: Some("SELECT *\nFROM ".to_string()),
            template_steps: vec![
                TemplateStep::PickTable,
                TemplateStep::Text(" a\nJOIN ".to_string()),
                TemplateStep::PickTable,
                TemplateStep::Text(" b ON a.".to_string()),
                TemplateStep::PickColumn("a".to_string()),
                TemplateStep::Text(" = b.".to_string()),
                TemplateStep::PickColumn("b".to_string()),
            ],
        },
        CompletionItem {
            text: "Group & Count".to_string(),
            kind: CompletionKind::Keyword,
            template: Some("SELECT *\nFROM ".to_string()),
            template_steps: vec![
                TemplateStep::PickTable,
                TemplateStep::Text("\nGROUP BY ".to_string()),
                TemplateStep::PickColumn("*".to_string()),
                TemplateStep::Assemble(
                    "SELECT {column}, COUNT(*)\nFROM {table}\nGROUP BY {column}".to_string(),
                ),
            ],
        },
        CompletionItem {
            text: "Order & Limit".to_string(),
            kind: CompletionKind::Keyword,
            template: Some("SELECT *\nFROM ".to_string()),
            template_steps: vec![
                TemplateStep::PickTable,
                TemplateStep::Text("\nORDER BY ".to_string()),
                TemplateStep::PickColumn("*".to_string()),
                TemplateStep::Text("\nLIMIT 10".to_string()),
            ],
        },
    ]
}
