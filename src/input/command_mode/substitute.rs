//! Substitute command for find/replace across CSV cells.
//!
//! Supports:
//! - `s/old/new/`     — replace first in current cell
//! - `s/old/new/g`    — replace all in current cell
//! - `%s/old/new/g`   — replace all in all data cells
//! - `5,10s/old/new/g` — replace in row range
//! - `B,Ds/old/new/g`  — replace in column range
//! - `i` flag for case-insensitive matching
//! - Regex patterns supported

use crate::app::App;
use crate::csv::row_storage::{get_row_bytes, parse_single_row};
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;

/// Parsed substitute command
struct SubCommand {
    pattern: String,
    replacement: String,
    global: bool,
    case_insensitive: bool,
}

/// Range for the substitute operation
enum SubRange {
    CurrentCell,
    AllCells,
    RowRange(usize, usize),
    ColRange(usize, usize),
}

/// Try to parse and execute a substitute command.
/// Returns Some if the command was a substitute, None otherwise.
pub fn try_execute(app: &mut App, cmd: &str) -> Option<Result<InputResult>> {
    // Parse range prefix and find the 's' that starts the substitute
    let (range, sub_str) = parse_range(cmd)?;
    let sub = parse_substitute(sub_str)?;
    Some(execute_substitute(app, range, sub))
}

/// Parse the range prefix from a command, returning (range, remaining_str).
/// Returns None if not a substitute command.
fn parse_range(cmd: &str) -> Option<(SubRange, &str)> {
    // %s/... — all cells
    if let Some(rest) = cmd.strip_prefix('%') {
        if rest.starts_with('s') {
            return Some((SubRange::AllCells, rest));
        }
    }

    // .s/... — current cell
    if let Some(rest) = cmd.strip_prefix('.') {
        if rest.starts_with('s') {
            return Some((SubRange::CurrentCell, rest));
        }
    }

    // Plain s/... — current cell
    if cmd.starts_with('s') && cmd.chars().nth(1).is_some_and(|c| !c.is_alphanumeric()) {
        return Some((SubRange::CurrentCell, cmd));
    }

    // 5,10s/... — row range
    // B,Ds/... — column range
    if let Some(comma_pos) = cmd.find(',') {
        let start_str = &cmd[..comma_pos];
        let after_comma = &cmd[comma_pos + 1..];

        // Find where 's' starts in the after-comma part
        let s_pos = after_comma.find('s')?;
        let end_str = &after_comma[..s_pos];
        let rest = &after_comma[s_pos..];

        if !rest.starts_with('s') {
            return None;
        }

        // Numeric row range
        if let (Ok(start), Ok(end)) = (start_str.parse::<usize>(), end_str.parse::<usize>()) {
            return Some((SubRange::RowRange(start, end), rest));
        }

        // Column range (letters)
        if start_str.chars().all(|c| c.is_ascii_alphabetic())
            && end_str.chars().all(|c| c.is_ascii_alphabetic())
        {
            let start_col =
                crate::ui::utils::excel_letter_to_column(&start_str.to_uppercase()).ok()?;
            let end_col = crate::ui::utils::excel_letter_to_column(&end_str.to_uppercase()).ok()?;
            return Some((SubRange::ColRange(start_col, end_col), rest));
        }
    }

    None
}

/// Parse `s/pattern/replacement/flags` from a string starting with 's'.
fn parse_substitute(s: &str) -> Option<SubCommand> {
    // Must start with 's' followed by a delimiter
    let s = s.strip_prefix('s')?;
    if s.is_empty() {
        return None;
    }

    let delim = s.chars().next()?;
    let rest = &s[delim.len_utf8()..];

    // Split by delimiter, handling escaped delimiters
    let parts = split_by_delimiter(rest, delim);
    if parts.len() < 2 {
        return None;
    }

    let pattern = parts[0].replace(&format!("\\{}", delim), &delim.to_string());
    let replacement = parts[1].replace(&format!("\\{}", delim), &delim.to_string());
    let flags = if parts.len() > 2 { parts[2] } else { "" };

    let global = flags.contains('g');
    let case_insensitive = flags.contains('i');

    Some(SubCommand {
        pattern,
        replacement,
        global,
        case_insensitive,
    })
}

/// Split a string by a delimiter, respecting backslash escapes.
fn split_by_delimiter(s: &str, delim: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut prev_escape = false;

    for (i, c) in s.char_indices() {
        if c == delim && !prev_escape {
            parts.push(&s[start..i]);
            start = i + c.len_utf8();
            if parts.len() == 2 {
                // Everything after second delimiter is flags
                parts.push(&s[start..]);
                return parts;
            }
        }
        prev_escape = c == '\\' && !prev_escape;
    }

    // If no trailing delimiter, rest is the last part
    if start <= s.len() {
        parts.push(&s[start..]);
    }
    parts
}

/// Execute the substitute across the specified range.
fn execute_substitute(app: &mut App, range: SubRange, sub: SubCommand) -> Result<InputResult> {
    // Build regex
    let regex = if sub.case_insensitive {
        Regex::new(&format!("(?i){}", &sub.pattern))
    } else {
        Regex::new(&sub.pattern)
    };

    let regex = match regex {
        Ok(r) => r,
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!("Invalid pattern: {}", e)));
            return Ok(InputResult::Continue);
        }
    };

    let (row_start, row_end, col_start, col_end) = match range {
        SubRange::CurrentCell => {
            let row = app.selected_row().map(|r| r.get()).unwrap_or(1);
            let col = app.view_state.selected_column.get();
            (row, row, col, col)
        }
        SubRange::AllCells => {
            let row_count = app.document.row_count();
            let col_count = app.document.column_count();
            (
                1,
                row_count.saturating_sub(1),
                0,
                col_count.saturating_sub(1),
            )
        }
        SubRange::RowRange(start, end) => {
            let col_count = app.document.column_count();
            (start, end, 0, col_count.saturating_sub(1))
        }
        SubRange::ColRange(start, end) => {
            let row_count = app.document.row_count();
            (1, row_count.saturating_sub(1), start, end)
        }
    };

    // Use parallel fast path for large lazy documents with full-width ranges
    let is_full_width = col_start == 0 && col_end >= app.document.column_count().saturating_sub(1);
    if is_full_width && app.document.is_lazy() {
        return execute_substitute_parallel(
            app, row_start, row_end, col_start, col_end, &regex, &sub,
        );
    }

    execute_substitute_sequential(app, row_start, row_end, col_start, col_end, &regex, &sub)
}

/// Sequential substitute — used for small ranges and in-memory documents.
fn execute_substitute_sequential(
    app: &mut App,
    row_start: usize,
    row_end: usize,
    col_start: usize,
    col_end: usize,
    regex: &Regex,
    sub: &SubCommand,
) -> Result<InputResult> {
    let mut total_replacements = 0;
    let mut cells_changed = 0;
    let mut undo_commands = Vec::new();

    for row_idx in row_start..=row_end {
        if row_idx >= app.document.row_count() {
            break;
        }
        for col_idx in col_start..=col_end {
            if col_idx >= app.document.column_count() {
                break;
            }

            let r = RowIndex::new(row_idx);
            let c = ColIndex::new(col_idx);
            let old_value = app.document.cell(r, c);

            let new_value = if sub.global {
                regex
                    .replace_all(&old_value, sub.replacement.as_str())
                    .to_string()
            } else {
                regex
                    .replace(&old_value, sub.replacement.as_str())
                    .to_string()
            };

            if new_value != old_value {
                let matches: usize = regex.find_iter(&old_value).count();
                total_replacements += if sub.global { matches } else { 1.min(matches) };
                cells_changed += 1;

                app.document.set_cell(r, c, new_value.clone());
                undo_commands.push(crate::history::EditCommand::SetCell {
                    row: r,
                    col: c,
                    old_value,
                    new_value,
                });
            }
        }
    }

    finish_substitute(app, undo_commands, total_replacements, cells_changed)
}

/// Parallel substitute for lazy (mmap-backed) documents.
///
/// Strategy:
/// 1. Scan raw mmap bytes in parallel to find candidate rows containing the pattern
/// 2. Parse candidate rows and apply regex replacements in parallel
/// 3. Apply changed rows to the document edit overlay in bulk
fn execute_substitute_parallel(
    app: &mut App,
    row_start: usize,
    row_end: usize,
    col_start: usize,
    col_end: usize,
    regex: &Regex,
    sub: &SubCommand,
) -> Result<InputResult> {
    let lazy = app.document.storage.lazy_storage().unwrap();
    let raw = lazy.raw_bytes();
    let offsets = lazy.row_offsets();
    let delimiter = lazy.delimiter();
    let sort_order = lazy.sort_order();
    let edits = lazy.edits();

    // Step 1: Find candidate rows by scanning raw bytes in parallel.
    let byte_re = regex::bytes::RegexBuilder::new(regex.as_str())
        .case_insensitive(sub.case_insensitive)
        .build();

    let num_chunks = rayon::current_num_threads().max(1);
    let total_rows = offsets.len();
    let rows_per_chunk = (total_rows / num_chunks).max(1);

    let mut chunk_ranges: Vec<(usize, usize)> = Vec::new();
    let mut sr = 0;
    while sr < total_rows {
        let er = (sr + rows_per_chunk).min(total_rows);
        let byte_start = offsets[sr] as usize;
        let byte_end = if er < total_rows {
            offsets[er] as usize
        } else {
            raw.len()
        };
        if byte_start < byte_end {
            chunk_ranges.push((byte_start, byte_end));
        }
        sr = er;
    }

    let mut candidate_rows: Vec<usize> = if let Ok(ref byte_re) = byte_re {
        chunk_ranges
            .par_iter()
            .flat_map(|&(byte_start, byte_end)| {
                let chunk = &raw[byte_start..byte_end];
                let mut rows = Vec::new();
                let mut last_row_idx = usize::MAX;
                for m in byte_re.find_iter(chunk) {
                    let abs_pos = (byte_start + m.start()) as u64;
                    let row_idx = match offsets.binary_search(&abs_pos) {
                        Ok(i) => i,
                        Err(i) => i.saturating_sub(1),
                    };
                    if row_idx != last_row_idx {
                        rows.push(row_idx);
                        last_row_idx = row_idx;
                    }
                }
                rows
            })
            .collect()
    } else {
        // Literal fallback: scan for pattern bytes in parallel
        let pat_lower = regex.as_str().to_lowercase().into_bytes();
        let pat_upper = regex.as_str().to_uppercase().into_bytes();
        let search_upper = pat_lower != pat_upper;

        chunk_ranges
            .par_iter()
            .flat_map(|&(byte_start, byte_end)| {
                let chunk = &raw[byte_start..byte_end];
                let mut rows = std::collections::BTreeSet::new();
                for pos in memchr::memmem::find_iter(chunk, &pat_lower) {
                    let abs_pos = (byte_start + pos) as u64;
                    let row_idx = match offsets.binary_search(&abs_pos) {
                        Ok(i) => i,
                        Err(i) => i.saturating_sub(1),
                    };
                    rows.insert(row_idx);
                }
                if search_upper {
                    for pos in memchr::memmem::find_iter(chunk, &pat_upper) {
                        let abs_pos = (byte_start + pos) as u64;
                        let row_idx = match offsets.binary_search(&abs_pos) {
                            Ok(i) => i,
                            Err(i) => i.saturating_sub(1),
                        };
                        rows.insert(row_idx);
                    }
                }
                rows.into_iter().collect::<Vec<_>>()
            })
            .collect()
    };

    // Also include edited rows as candidates
    for &row_idx in edits.keys() {
        candidate_rows.push(row_idx);
    }
    candidate_rows.sort_unstable();
    candidate_rows.dedup();

    // Filter to the requested range
    candidate_rows.retain(|&r| r >= row_start && r <= row_end);

    // Step 2: Parse candidate rows in parallel, apply replacements, collect results.
    type SubResult = (usize, Vec<String>, Vec<(usize, String, String)>);
    let results: Vec<SubResult> = candidate_rows
        .par_iter()
        .filter_map(|&row_idx| {
            // Parse the row from mmap or edits
            let mut row = if let Some(edited) = edits.get(&row_idx) {
                edited.clone()
            } else {
                let phys = match sort_order {
                    Some(order) => {
                        let data_idx = row_idx - 1;
                        if data_idx < order.len() {
                            order[data_idx]
                        } else {
                            row_idx
                        }
                    }
                    None => row_idx,
                };
                if phys < offsets.len() {
                    let bytes = get_row_bytes(raw, offsets, phys);
                    parse_single_row(bytes, delimiter)
                } else {
                    return None;
                }
            };

            // Apply replacements to cells in the requested column range
            let mut cell_changes = Vec::new();
            let c_end = col_end.min(row.len().saturating_sub(1));
            for col_idx in col_start..=c_end {
                if col_idx >= row.len() {
                    break;
                }
                let old_value = &row[col_idx];
                let new_value = if sub.global {
                    regex
                        .replace_all(old_value, sub.replacement.as_str())
                        .to_string()
                } else {
                    regex
                        .replace(old_value, sub.replacement.as_str())
                        .to_string()
                };
                if new_value != *old_value {
                    cell_changes.push((col_idx, old_value.clone(), new_value.clone()));
                    row[col_idx] = new_value;
                }
            }

            if cell_changes.is_empty() {
                None
            } else {
                Some((row_idx, row, cell_changes))
            }
        })
        .collect();

    // Step 3: Apply results to document on the main thread.
    let mut total_replacements = 0usize;
    let mut cells_changed = 0usize;
    let mut undo_commands = Vec::new();
    let mut bulk_edits = Vec::new();

    for (row_idx, new_row, cell_changes) in results {
        for (col_idx, old_value, new_value) in cell_changes {
            let match_count = regex.find_iter(&old_value).count();
            total_replacements += if sub.global {
                match_count
            } else {
                1.min(match_count)
            };
            cells_changed += 1;
            undo_commands.push(crate::history::EditCommand::SetCell {
                row: RowIndex::new(row_idx),
                col: ColIndex::new(col_idx),
                old_value,
                new_value,
            });
        }
        bulk_edits.push((row_idx, new_row));
    }

    // Bulk-apply edits to the lazy storage overlay
    if let Some(lazy_mut) = app.document.storage.lazy_storage_mut() {
        lazy_mut.bulk_set_edits(bulk_edits);
    }
    if cells_changed > 0 {
        app.document.is_dirty = true;
        app.document.generation += 1;
    }

    finish_substitute(app, undo_commands, total_replacements, cells_changed)
}

/// Finalize substitute: push undo commands, set status message.
fn finish_substitute(
    app: &mut App,
    mut undo_commands: Vec<crate::history::EditCommand>,
    total_replacements: usize,
    cells_changed: usize,
) -> Result<InputResult> {
    if !undo_commands.is_empty() {
        if undo_commands.len() == 1 {
            app.history.push(undo_commands.remove(0));
        } else {
            app.history
                .push(crate::history::EditCommand::Compound(undo_commands));
        }
    }

    if total_replacements == 0 {
        app.status_message = Some(StatusMessage::from("Pattern not found"));
    } else {
        app.status_message = Some(StatusMessage::from(format!(
            "{} replacement(s) in {} cell(s)",
            total_replacements, cells_changed
        )));
    }

    Ok(InputResult::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_substitute_basic() {
        let sub = parse_substitute("s/foo/bar/").unwrap();
        assert_eq!(sub.pattern, "foo");
        assert_eq!(sub.replacement, "bar");
        assert!(!sub.global);
        assert!(!sub.case_insensitive);
    }

    #[test]
    fn test_parse_substitute_global() {
        let sub = parse_substitute("s/foo/bar/g").unwrap();
        assert!(sub.global);
    }

    #[test]
    fn test_parse_substitute_case_insensitive() {
        let sub = parse_substitute("s/foo/bar/gi").unwrap();
        assert!(sub.global);
        assert!(sub.case_insensitive);
    }

    #[test]
    fn test_parse_substitute_no_trailing_delim() {
        let sub = parse_substitute("s/foo/bar").unwrap();
        assert_eq!(sub.pattern, "foo");
        assert_eq!(sub.replacement, "bar");
    }

    #[test]
    fn test_parse_substitute_empty_replacement() {
        let sub = parse_substitute("s/foo//g").unwrap();
        assert_eq!(sub.pattern, "foo");
        assert_eq!(sub.replacement, "");
        assert!(sub.global);
    }

    #[test]
    fn test_parse_substitute_escaped_delimiter() {
        let sub = parse_substitute("s/a\\/b/c/").unwrap();
        assert_eq!(sub.pattern, "a/b");
        assert_eq!(sub.replacement, "c");
    }

    #[test]
    fn test_parse_substitute_alternate_delimiter() {
        let sub = parse_substitute("s|foo|bar|g").unwrap();
        assert_eq!(sub.pattern, "foo");
        assert_eq!(sub.replacement, "bar");
        assert!(sub.global);
    }

    #[test]
    fn test_parse_range_percent() {
        let (range, rest) = parse_range("%s/a/b/").unwrap();
        assert!(matches!(range, SubRange::AllCells));
        assert_eq!(rest, "s/a/b/");
    }

    #[test]
    fn test_parse_range_current() {
        let (range, rest) = parse_range("s/a/b/").unwrap();
        assert!(matches!(range, SubRange::CurrentCell));
        assert_eq!(rest, "s/a/b/");
    }

    #[test]
    fn test_parse_range_dot() {
        let (range, rest) = parse_range(".s/a/b/").unwrap();
        assert!(matches!(range, SubRange::CurrentCell));
        assert_eq!(rest, "s/a/b/");
    }

    #[test]
    fn test_parse_range_row() {
        let (range, rest) = parse_range("5,10s/a/b/g").unwrap();
        assert!(matches!(range, SubRange::RowRange(5, 10)));
        assert_eq!(rest, "s/a/b/g");
    }

    #[test]
    fn test_parse_range_column() {
        let (range, rest) = parse_range("B,Ds/a/b/g").unwrap();
        assert!(matches!(range, SubRange::ColRange(1, 3)));
        assert_eq!(rest, "s/a/b/g");
    }

    #[test]
    fn test_parse_range_not_substitute() {
        assert!(parse_range("sort").is_none());
        assert!(parse_range("5,10d").is_none());
        assert!(parse_range("help").is_none());
    }

    #[test]
    fn test_split_by_delimiter() {
        assert_eq!(
            split_by_delimiter("foo/bar/g", '/'),
            vec!["foo", "bar", "g"]
        );
        assert_eq!(split_by_delimiter("foo/bar", '/'), vec!["foo", "bar"]);
        assert_eq!(split_by_delimiter("a\\/b/c/", '/'), vec!["a\\/b", "c", ""]);
    }
}
