//! Command executor for ex-style commands (e.g., :q, :w, :sort)

use crate::app::App;
use crate::csv::Document;
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use crate::navigation;
use crate::ui::utils::excel_letter_to_column;
use anyhow::Result;

/// Execute command from command buffer
pub fn execute(app: &mut App) -> Result<InputResult> {
    let cmd = app.input_state.command_buffer.trim().to_string();

    if cmd.is_empty() {
        return Ok(InputResult::Continue);
    }

    // Pure number → go to line (vim :# syntax, e.g., :5, :100)
    if let Ok(line_num) = cmd.parse::<usize>() {
        navigation::commands::goto_line(app, line_num);
        return Ok(InputResult::Continue);
    }

    // Special handling for :c command (column jump)
    // Support both `:cA` (no space) and `:c A` (with space)
    // Exclude known commands that start with 'c' (count)
    let cmd_lower_full = cmd.to_lowercase();
    let is_known_c_command =
        cmd_lower_full.starts_with("count") || cmd_lower_full.starts_with("copy");
    if !is_known_c_command && (cmd.starts_with('c') || cmd.starts_with('C')) {
        let rest = &cmd[1..]; // Get everything after 'c'

        // Check if rest starts with a letter or digit (column specifier)
        // AND doesn't contain a comma (to avoid conflicting with column range commands like :C,Cd)
        if !rest.is_empty() && !rest.starts_with(' ') && !rest.contains(',') {
            // This is :cA, :cB, :c1, etc. (no space)
            let column_input = rest.trim();

            // Check if it's a numeric input (e.g., :c1, :c27)
            if let Ok(col_num) = column_input.parse::<usize>() {
                // Numeric column jump (1-indexed: 1=A, 2=B, 27=AA)
                navigation::commands::goto_column_by_number(app, col_num);
            } else {
                // Letter column jump (e.g., :cA, :cB, :cAA)
                // Validate it's only letters
                let column_letters = column_input.to_uppercase();
                if column_letters.chars().all(|c| c.is_ascii_alphabetic()) {
                    navigation::commands::goto_column(app, &column_letters);
                } else {
                    app.status_message = Some(StatusMessage::from(
                        "Invalid column name. Use letters (e.g., :cA, :cAA) or numbers (e.g., :c1, :c27)"
                    ));
                }
            }
            return Ok(InputResult::Continue);
        }
    }

    // Special handling for range operations: :5,10d, :%d, :.d, :$d, etc.
    if let Some(range_result) =
        crate::input::command_mode::range_commands::parse_and_execute(app, &cmd)
    {
        return range_result;
    }

    // Split command into parts for commands with arguments
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let cmd_name_original = parts[0]; // Keep original case
    let cmd_name_lower = cmd_name_original.to_lowercase();
    let _arg = parts.get(1).map(|s| s.trim());

    // Check case-sensitive commands first
    match cmd_name_original {
        "W" => {
            // Save all dirty files
            match app.save_all_files() {
                Ok(paths) => {
                    if paths.is_empty() {
                        app.status_message = Some(StatusMessage::from("No files to save"));
                    } else {
                        app.status_message = Some(StatusMessage::from(format!(
                            "{} file(s) written",
                            paths.len()
                        )));
                    }
                }
                Err(e) => {
                    app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
                }
            }
            return Ok(InputResult::Continue);
        }
        "Wq" => {
            // Save all dirty files and quit
            match app.save_all_files() {
                Ok(_) => {
                    app.should_quit = true;
                }
                Err(e) => {
                    app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
                }
            }
            return Ok(InputResult::Continue);
        }
        _ => {} // Fall through to case-insensitive commands
    }

    // Case-insensitive commands
    match cmd_name_lower.as_str() {
        "q" | "quit" => execute_quit(app),
        "q!" => execute_force_quit(app),
        "w" | "w!" | "write" => execute_write(app),
        "wq" | "wq!" | "x" => execute_write_quit(app),
        "h" | "help" => execute_help(app),
        "delim" => execute_delimiter_change(app, _arg),
        "new" => execute_new_document(app, _arg),
        "c" => execute_column_jump(app, _arg),
        "f" => execute_filename(app, _arg),
        "noh" | "nohlsearch" => execute_clear_search(app),
        "sort" | "sort!" => execute_sort(app, &cmd_name_lower, _arg),
        "stats" => super::stats::execute_stats(app, _arg),
        "sum" => super::stats::execute_sum(app, _arg),
        "avg" | "average" => super::stats::execute_avg(app, _arg),
        "count" => super::stats::execute_count(app, _arg),
        "distinct" => super::stats::execute_distinct(app, _arg),
        "width" | "resize" => execute_width(app, _arg),
        "copy" => execute_copy_to_clipboard(app),
        "paste" => execute_paste_from_clipboard(app),
        "upper" | "uppercase" => execute_cell_transform(app, crate::transforms::to_upper),
        "lower" | "lowercase" => execute_cell_transform(app, crate::transforms::to_lower),
        "title" | "titlecase" => execute_cell_transform(app, crate::transforms::to_title),
        "trim" => execute_cell_transform(app, crate::transforms::trim),
        _ => {
            // Unknown command
            app.status_message = Some(StatusMessage::from(format!("Unknown command: :{}", cmd)));
            Ok(InputResult::Continue)
        }
    }
}

/// Execute :q (quit) command
fn execute_quit(app: &mut App) -> Result<InputResult> {
    if app.document.is_dirty {
        app.status_message = Some(StatusMessage::from(
            "No write since last change (add ! to override)",
        ));
    } else {
        app.should_quit = true;
    }
    Ok(InputResult::Continue)
}

/// Execute :q! (force quit) command
fn execute_force_quit(app: &mut App) -> Result<InputResult> {
    // Force quit - clear cache and quit
    app.session.clear_cache();
    app.should_quit = true;
    Ok(InputResult::Continue)
}

/// Execute :w (write) command
fn execute_write(app: &mut App) -> Result<InputResult> {
    // Save current file only
    match app.save_current_file() {
        Ok(path) => {
            app.status_message = Some(StatusMessage::from(format!(
                "\"{}\" written",
                path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
            )));
        }
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
        }
    }
    Ok(InputResult::Continue)
}

/// Execute :wq/:x (write and quit) command
fn execute_write_quit(app: &mut App) -> Result<InputResult> {
    // Save current file and quit
    match app.save_current_file() {
        Ok(_) => {
            app.should_quit = true;
        }
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!("Error: {}", e)));
        }
    }
    Ok(InputResult::Continue)
}

/// Execute :h/:help command
fn execute_help(app: &mut App) -> Result<InputResult> {
    app.status_message = Some(StatusMessage::from("Press ? for help"));
    Ok(InputResult::Continue)
}

/// Execute :delim command to change CSV delimiter
fn execute_delimiter_change(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    // Change CSV delimiter for current file and reload
    if let Some(arg) = arg {
        if arg.len() == 1 {
            let new_delim = arg
                .chars()
                .next()
                .expect("arg length already validated as 1");

            // Track in session for current file
            let current_file = app.current_file().clone();
            app.session.set_delimiter(current_file.clone(), new_delim);

            // Reload file with new delimiter
            match app.reload_current_file_with_delimiter(new_delim) {
                Ok(_) => {
                    app.status_message = Some(StatusMessage::from(format!(
                        "Delimiter changed to '{}' and file reloaded",
                        new_delim
                    )));
                }
                Err(e) => {
                    app.status_message = Some(StatusMessage::from(format!("Reload failed: {}", e)));
                }
            }
        } else {
            app.status_message = Some(StatusMessage::from("Delimiter must be a single character"));
        }
    } else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :delim <char> (e.g., :delim ; or :delim |)",
        ));
    }
    Ok(InputResult::Continue)
}

/// Execute :new command to create a new CSV document
fn execute_new_document(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    // Create a new CSV document with optional headers
    let headers = if let Some(arg) = arg {
        // Parse comma-separated headers
        arg.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>()
    } else {
        // Default: single column named "Column 1"
        vec!["Column 1".to_string()]
    };

    // Create new document with headers only (0 data rows)
    let filename = app.document.filename.clone();
    let delimiter = app.document.delimiter;

    app.document = Document::new(headers.clone(), vec![], filename);
    app.document.delimiter = delimiter; // Preserve current delimiter
    app.document.is_dirty = true;

    // Mark current file as dirty in session
    let current_file = app.current_file().clone();
    app.session.mark_dirty(&current_file);

    // Reset view state and position cursor at row 0
    app.view_state = crate::ui::ViewState::default();
    app.view_state.table_state.select(Some(0));

    app.status_message = Some(StatusMessage::from(format!(
        "New CSV created with {} column(s)",
        headers.len()
    )));
    Ok(InputResult::Continue)
}

/// Execute :c command for column jumping
fn execute_column_jump(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    // Column jump: :cA, :cB, :cAA, :c1, etc.
    if let Some(arg) = arg {
        let column_input = arg.trim();

        // Check if it's a numeric input (e.g., :c1, :c27)
        if let Ok(col_num) = column_input.parse::<usize>() {
            // Numeric column jump (1-indexed: 1=A, 2=B, 27=AA)
            navigation::commands::goto_column_by_number(app, col_num);
        } else {
            // Letter column jump (e.g., :cA, :cB, :cAA)
            // Validate it's only letters
            let column_letters = column_input.to_uppercase();
            if column_letters.chars().all(|c| c.is_ascii_alphabetic()) {
                navigation::commands::goto_column(app, &column_letters);
            } else {
                app.status_message = Some(StatusMessage::from(
                    "Invalid column name. Use letters (e.g., :cA, :cAA) or numbers (e.g., :c1, :c27)"
                ));
            }
        }
    } else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :c<column> (e.g., :cA, :cB, :cAA, :c1, :c27)",
        ));
    }
    Ok(InputResult::Continue)
}

/// Execute :f command to show/rename filename
fn execute_filename(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    // :f (no arg) shows current filename, :f <name> renames
    if let Some(arg) = arg {
        let new_name = arg.to_string();
        app.document.filename = new_name.clone();
        let new_path = std::path::PathBuf::from(&new_name);
        app.session.rename_current_file(new_path.clone());
        app.document.is_dirty = true;
        app.session.mark_dirty(&new_path);
        app.status_message = Some(StatusMessage::from(format!("Renamed to \"{}\"", new_name)));
    } else {
        let current = app.document.filename.clone();
        app.status_message = Some(StatusMessage::from(format!("\"{}\"", current)));
    }
    Ok(InputResult::Continue)
}

/// Execute :noh/:nohlsearch command to clear search
fn execute_clear_search(app: &mut App) -> Result<InputResult> {
    app.search_state = None;
    app.status_message = Some(StatusMessage::from("Search cleared"));
    Ok(InputResult::Continue)
}

/// Execute :copy/:clipboard command — copy current document as CSV to system clipboard.
fn execute_copy_to_clipboard(app: &mut App) -> Result<InputResult> {
    use std::io::Write;

    // Build CSV content from current document
    let mut buf = Vec::new();
    crate::csv::write_csv_content(&mut buf, &app.document, app.document.delimiter)
        .map_err(|e| anyhow::anyhow!("Failed to build CSV: {}", e))?;

    // Spawn clipboard command
    #[cfg(target_os = "macos")]
    let child_result = {
        use std::process::{Command, Stdio};
        Command::new("pbcopy").stdin(Stdio::piped()).spawn()
    };

    #[cfg(target_os = "linux")]
    let child_result = {
        use std::process::{Command, Stdio};
        Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
            .or_else(|_| {
                Command::new("xsel")
                    .args(["--clipboard", "--input"])
                    .stdin(Stdio::piped())
                    .spawn()
            })
            .or_else(|_| Command::new("wl-copy").stdin(Stdio::piped()).spawn())
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let child_result: Result<std::process::Child, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Clipboard not supported",
    ));

    match child_result {
        Ok(mut child) => {
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(&buf);
            }
            let _ = child.wait();
            let rows = app.document.row_count().saturating_sub(1);
            app.status_message = Some(StatusMessage::from(format!(
                "Copied {} rows to clipboard",
                rows
            )));
        }
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!("Clipboard error: {}", e)));
        }
    }

    Ok(InputResult::Continue)
}

/// Execute :paste command — read CSV/TSV from system clipboard and replace current document.
fn execute_paste_from_clipboard(app: &mut App) -> Result<InputResult> {
    // Read from clipboard
    #[cfg(target_os = "macos")]
    let output_result = {
        use std::process::{Command, Stdio};
        Command::new("pbpaste").stdout(Stdio::piped()).output()
    };

    #[cfg(target_os = "linux")]
    let output_result = {
        use std::process::{Command, Stdio};
        Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .stdout(Stdio::piped())
            .output()
            .or_else(|_| {
                Command::new("xsel")
                    .args(["--clipboard", "--output"])
                    .stdout(Stdio::piped())
                    .output()
            })
            .or_else(|_| Command::new("wl-paste").stdout(Stdio::piped()).output())
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let output_result: Result<std::process::Output, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Clipboard not supported",
    ));

    let output = match output_result {
        Ok(o) if o.status.success() => o,
        Ok(_) => {
            app.status_message = Some(StatusMessage::from("Clipboard read failed"));
            return Ok(InputResult::Continue);
        }
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!("Clipboard error: {}", e)));
            return Ok(InputResult::Continue);
        }
    };

    let text = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => {
            app.status_message = Some(StatusMessage::from("Clipboard contains invalid UTF-8"));
            return Ok(InputResult::Continue);
        }
    };

    if text.trim().is_empty() {
        app.status_message = Some(StatusMessage::from("Clipboard is empty"));
        return Ok(InputResult::Continue);
    }

    // Auto-detect delimiter
    let delimiter = crate::csv::detect_delimiter(&text);

    // Parse the clipboard content as CSV with detected delimiter
    let reader = std::io::Cursor::new(text.as_bytes());
    match crate::csv::Document::from_reader(
        reader,
        Some(delimiter),
        false,
        "clipboard.csv".to_string(),
    ) {
        Ok(doc) => {
            let rows = doc.row_count().saturating_sub(1);
            let delim_name = match delimiter {
                b'\t' => "tab",
                b'|' => "pipe",
                b';' => "semicolon",
                b',' => "comma",
                _ => "auto",
            };
            // Replace current document
            app.document.storage = doc.storage;
            app.document.filename = "clipboard.csv".to_string();
            app.document.delimiter = ',';
            app.document.is_dirty = true;
            app.document.generation += 1;
            app.document.xlsx_formulas = vec![];
            // Reset view
            app.view_state.table_state.select(Some(1));
            app.view_state.column_scroll_offset = 0;
            app.view_state.selected_column = crate::domain::position::ColIndex::new(0);
            app.status_message = Some(StatusMessage::from(format!(
                "Pasted {} rows from clipboard ({}-delimited)",
                rows, delim_name
            )));
        }
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!(
                "Failed to parse clipboard: {}",
                e
            )));
        }
    }

    Ok(InputResult::Continue)
}

/// Execute :width/:resize command
///
/// Usage:
///   :width A 20   - Set column A width to 20 characters
///   :width B auto - Auto-size column B (clear manual width)
/// Apply a transform function to the current cell.
fn execute_cell_transform(app: &mut App, transform: fn(&str) -> String) -> Result<InputResult> {
    if let Some(row) = app.selected_row() {
        let col = app.view_state.selected_column;
        let old = app.document.cell(row, col);
        let new_value = transform(&old);
        if new_value != old {
            app.document.set_cell(row, col, new_value.clone());
            app.history.push(crate::history::EditCommand::SetCell {
                row,
                col,
                old_value: old,
                new_value,
            });
        }
    }
    Ok(InputResult::Continue)
}

///   :width * auto - Auto-size all columns (clear all manual widths)
///   :width * 15   - Set all columns to 15 characters
fn execute_width(app: &mut App, arg: Option<&str>) -> Result<InputResult> {
    let arg = match arg {
        Some(a) if !a.is_empty() => a,
        _ => {
            app.status_message = Some(StatusMessage::from("Usage: :width <column> <size|auto>"));
            return Ok(InputResult::Continue);
        }
    };

    let parts: Vec<&str> = arg.split_whitespace().collect();
    if parts.len() != 2 {
        app.status_message = Some(StatusMessage::from("Usage: :width <column> <size|auto>"));
        return Ok(InputResult::Continue);
    }

    let col_spec = parts[0];
    let width_spec = parts[1];

    if col_spec == "*" {
        // Apply to all columns
        if width_spec.eq_ignore_ascii_case("auto") {
            app.session.clear_all_column_widths();
            app.status_message = Some(StatusMessage::from("All columns set to auto width"));
        } else if let Ok(w) = width_spec.parse::<u16>() {
            let col_count = app.document.column_count();
            for i in 0..col_count {
                app.session.set_column_width(i, w);
            }
            app.status_message = Some(StatusMessage::from(format!(
                "All columns set to width {}",
                w
            )));
        } else {
            app.status_message = Some(StatusMessage::from("Width must be a number or 'auto'"));
        }
    } else {
        // Parse column letter(s) to index
        let col_index = match excel_letter_to_column(col_spec) {
            Ok(idx) => idx,
            Err(_) => {
                app.status_message =
                    Some(StatusMessage::from(format!("Invalid column: {}", col_spec)));
                return Ok(InputResult::Continue);
            }
        };

        if col_index >= app.document.column_count() {
            app.status_message = Some(StatusMessage::from(format!(
                "Column {} does not exist",
                col_spec.to_uppercase()
            )));
            return Ok(InputResult::Continue);
        }

        if width_spec.eq_ignore_ascii_case("auto") {
            app.session.clear_column_width(col_index);
            app.status_message = Some(StatusMessage::from(format!(
                "Column {} set to auto width",
                col_spec.to_uppercase()
            )));
        } else if let Ok(w) = width_spec.parse::<u16>() {
            app.session.set_column_width(col_index, w);
            app.status_message = Some(StatusMessage::from(format!(
                "Column {} width set to {}",
                col_spec.to_uppercase(),
                w
            )));
        } else {
            app.status_message = Some(StatusMessage::from("Width must be a number or 'auto'"));
        }
    }

    Ok(InputResult::Continue)
}

/// Execute :sort/:sort! command
fn execute_sort(app: &mut App, cmd_name: &str, arg: Option<&str>) -> Result<InputResult> {
    let ascending = cmd_name == "sort";
    if let Some(arg) = arg {
        let specs: Vec<&str> = arg.split(',').map(|s| s.trim()).collect();
        let mut col_indices = Vec::new();
        for spec in &specs {
            if let Ok(num) = spec.parse::<usize>() {
                if num == 0 || num > app.document.column_count() {
                    app.status_message = Some(StatusMessage::from(format!(
                        "Column {} out of range (1-{})",
                        num,
                        app.document.column_count()
                    )));
                    return Ok(InputResult::Continue);
                }
                col_indices.push(num - 1);
            } else {
                // Try header name first (case-insensitive), then Excel column letter
                let header_row = app.document.storage.header_row();
                let header_match = header_row
                    .iter()
                    .position(|name| name.eq_ignore_ascii_case(spec));
                if let Some(idx) = header_match {
                    col_indices.push(idx);
                } else if spec.chars().all(|c| c.is_ascii_alphabetic()) {
                    // Try as Excel-style column letter (A, B, AA, etc.)
                    match excel_letter_to_column(spec) {
                        Ok(idx) if idx < app.document.column_count() => {
                            col_indices.push(idx);
                        }
                        _ => {
                            app.status_message = Some(StatusMessage::from(format!(
                                "Column \"{}\" not found",
                                spec
                            )));
                            return Ok(InputResult::Continue);
                        }
                    }
                } else {
                    app.status_message = Some(StatusMessage::from(format!(
                        "Column \"{}\" not found",
                        spec
                    )));
                    return Ok(InputResult::Continue);
                }
            }
        }
        return Ok(InputResult::SortDocument {
            col_indices,
            ascending,
            description: arg.to_string(),
        });
    } else {
        app.status_message = Some(StatusMessage::from(
            "Usage: :sort <col,...> or :sort! <col,...> (e.g., :sort 1 or :sort! Name,Age)",
        ));
    }
    Ok(InputResult::Continue)
}
