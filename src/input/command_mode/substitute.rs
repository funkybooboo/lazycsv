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
use crate::domain::position::{ColIndex, RowIndex};
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use anyhow::Result;
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
                // Count replacements
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
