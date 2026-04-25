//! Shell-command prompt for the file menu (`:` key).
//!
//! Variable substitution and command preparation live here. Actual execution
//! happens in `main.rs` after the TUI is suspended (see `InputResult::RunShell`).
//!
//! Substitutions performed before the command leaves this module:
//!
//! - `$CWD`  — file menu's current directory (shell-quoted absolute path)
//! - `$FILE` — currently highlighted entry's full path (shell-quoted)
//! - `$NAME` — highlighted entry's basename (shell-quoted)
//! - `$EXT`  — highlighted entry's extension (no leading dot, shell-quoted)
//!
//! Literal `$` can be escaped as `\$`. Unknown `$<name>` placeholders are
//! left untouched so the user's shell can resolve them as env vars.

use super::{scan_directory_filtered, BrowserEntry};
use crate::app::App;
use std::path::Path;

/// Quote a path/string for safe interpolation into a POSIX shell command.
/// Wraps in single quotes and escapes any embedded single quote.
fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

/// Look up the file entry currently highlighted in the file menu.
fn highlighted_entry(app: &App) -> Option<BrowserEntry> {
    let entries = scan_directory_filtered(
        &app.view_state.current_directory,
        app.view_state.show_hidden_files,
    )
    .ok()?;
    let filter = app.input_state.file_filter_buffer.to_lowercase();
    let filtered: Vec<&BrowserEntry> = entries
        .iter()
        .filter(|e| {
            if filter.is_empty() {
                true
            } else if let Some(name) = e.filename() {
                name.to_lowercase().contains(&filter)
            } else {
                false
            }
        })
        .collect();
    let idx = app
        .view_state
        .file_list_selected
        .min(filtered.len().saturating_sub(1));
    filtered.get(idx).map(|e| (*e).clone())
}

fn entry_path(entry: &BrowserEntry) -> &Path {
    match entry {
        BrowserEntry::CsvFile(p) | BrowserEntry::Directory(p) => p,
    }
}

/// Replace `$CWD` / `$FILE` / `$NAME` / `$EXT` in `template` with shell-safe
/// values pulled from `app`. Literal `\$` is preserved as `$`. Unknown
/// `$<name>` tokens pass through unchanged.
pub fn substitute(template: &str, app: &App) -> String {
    let cwd = app
        .view_state
        .current_directory
        .to_string_lossy()
        .into_owned();
    let entry = highlighted_entry(app);
    let entry_p = entry.as_ref().map(entry_path);

    let file = entry_p
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = entry_p
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = entry_p
        .and_then(|p| p.extension())
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'$') {
            out.push('$');
            chars.next();
            continue;
        }
        if c != '$' {
            out.push(c);
            continue;
        }
        // Read identifier ([A-Za-z_][A-Za-z0-9_]*)
        let mut name_buf = String::new();
        while let Some(&nc) = chars.peek() {
            if nc.is_ascii_alphanumeric() || nc == '_' {
                name_buf.push(nc);
                chars.next();
            } else {
                break;
            }
        }
        match name_buf.as_str() {
            "CWD" => out.push_str(&shell_quote(&cwd)),
            "FILE" => out.push_str(&shell_quote(&file)),
            "NAME" => out.push_str(&shell_quote(&name)),
            "EXT" => out.push_str(&shell_quote(&ext)),
            "" => out.push('$'),
            other => {
                // Unknown — leave for the user's shell to resolve.
                out.push('$');
                out.push_str(other);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::shell_quote;

    #[test]
    fn quotes_simple() {
        assert_eq!(shell_quote("hello"), "'hello'");
    }

    #[test]
    fn escapes_single_quote() {
        // foo'bar -> 'foo'\''bar'
        assert_eq!(shell_quote("foo'bar"), "'foo'\\''bar'");
    }

    #[test]
    fn quotes_path_with_spaces() {
        assert_eq!(shell_quote("/tmp/my file.csv"), "'/tmp/my file.csv'");
    }
}
