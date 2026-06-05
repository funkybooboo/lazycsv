use anyhow::{Context, Result};
use clap::CommandFactory;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use lazycsv::config::views;
use lazycsv::{cli, ui, App, FileConfig, InputResult};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

type Term = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

fn main() {
    match run_main() {
        Ok(()) => {}
        Err(e) => {
            // Ensure terminal is in a sane state before printing errors
            let _ = crossterm::terminal::disable_raw_mode();
            // Silently exit on broken pipe (e.g., `lazycsv -q ... | head`)
            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                    std::process::exit(0);
                }
            }
            eprintln!("Error: {:#}", e);
            std::process::exit(1);
        }
    }
}

fn run_main() -> Result<()> {
    let cli_args = cli::parse_args();

    // Handle --help: show command-specific help if a command flag is present
    if cli_args.help {
        if cli_args.add_header.is_some() {
            cli::print_command_help("A");
        } else if cli_args.dedup.is_some() {
            cli::print_command_help("D");
        } else if cli_args.query.is_some() {
            cli::print_command_help("q");
        } else if cli_args.xlsx {
            cli::print_command_help("x");
        } else if cli_args.split.is_some() {
            cli::print_command_help("S");
        } else if cli_args.generate {
            cli::print_command_help("g");
        } else if cli_args.is_stats_flag() {
            cli::print_command_help("t");
        } else if cli_args.sort.is_some() {
            cli::print_command_help("s");
        } else if cli_args.headers {
            cli::print_command_help("h");
        } else if cli_args.is_rows_flag() || cli_args.is_columns_flag() {
            cli::print_command_help("rc");
        } else {
            cli::CliArgs::command().print_help()?;
            println!();
        }
        return Ok(());
    }

    // Non-interactive generate mode: create synthetic CSV data
    if cli_args.generate {
        let rows = cli_args
            .gen_rows()
            .context("Usage: lazycsv -g -r <ROWS> -c <COLUMNS> [-t <TYPE>]\n  -r (rows) is required and must be a positive number")?;
        let cols = cli_args
            .gen_cols()
            .context("Usage: lazycsv -g -r <ROWS> -c <COLUMNS> [-t <TYPE>]\n  -c (columns) is required and must be a positive number")?;
        if rows == 0 {
            anyhow::bail!("Row count must be greater than 0");
        }
        if cols == 0 {
            anyhow::bail!("Column count must be greater than 0");
        }
        let gen_type = cli_args.gen_type();
        lazycsv::generate::validate_type(gen_type).map_err(|e| anyhow::anyhow!(e))?;
        return execute_generate(rows, cols, gen_type, &cli_args);
    }

    // Catch likely typos: -r <N> or -c <N> without -g
    if !cli_args.generate {
        let has_row_value = cli_args.rows.as_deref().is_some_and(|v| !v.is_empty());
        let has_col_value = cli_args.columns.as_deref().is_some_and(|v| !v.is_empty());
        if has_row_value || has_col_value {
            anyhow::bail!(
                "Did you mean: lazycsv -g -r <ROWS> -c <COLUMNS> [-t <TYPE>] -o <FILE>\n  \
                 The -g flag is required to generate CSV data"
            );
        }
    }

    // Non-interactive xlsx-to-csv extraction mode
    if cli_args.xlsx {
        let path = cli_args
            .file_path()
            .context("Usage: lazycsv <file.xlsx> -x")?;
        return execute_xlsx_convert(&path, &cli_args);
    }

    // Non-interactive query mode: execute SQL and exit
    if let Some(ref query) = cli_args.query {
        return execute_query_mode(query, &cli_args);
    }

    // Copy file contents to clipboard (non-interactive, streaming)
    if cli_args.clipboard && cli_args.query.is_none() {
        let path = cli_args.file_path().context("Usage: lazycsv <file> -C")?;
        let row_count = stream_file_to_clipboard(&path, &cli_args)?;
        eprintln!("Copied {} rows to clipboard", row_count);
        return Ok(());
    }

    // Paste clipboard contents to a CSV file (non-interactive)
    if cli_args.paste {
        let content = read_from_clipboard()?;
        if content.trim().is_empty() {
            anyhow::bail!("Clipboard is empty");
        }
        let out_path = match &cli_args.output {
            Some(val) if val != "-" => PathBuf::from(val),
            _ => PathBuf::from("clipboard.csv"),
        };
        if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        // Detect format: JSON (array or NDJSON) vs CSV/delimited
        let trimmed = content.trim_start();
        let is_json = trimmed.starts_with('[') || trimmed.starts_with('{');

        // Try JSON conversion; fall back to CSV if it fails or produces no data rows
        let json_rows = if is_json {
            let temp_file = std::env::temp_dir().join("lazycsv_clipboard.json");
            std::fs::write(&temp_file, &content)?;
            let result = lazycsv::csv::foreign_formats::load_foreign_format(&temp_file, None);
            let _ = std::fs::remove_file(&temp_file);
            match result {
                Ok(rows) if rows.len() > 1 => Some(rows),
                _ => None,
            }
        } else {
            None
        };

        if let Some(rows) = json_rows {
            let data_count = rows.len().saturating_sub(1);
            let mut wtr = csv::Writer::from_path(&out_path)?;
            for row in &rows {
                wtr.write_record(row)?;
            }
            wtr.flush()?;
            eprintln!(
                "Pasted {} rows to {} (JSON input → CSV)",
                data_count,
                out_path.display()
            );
        } else {
            // Auto-detect delimiter and convert to CSV if needed
            let detected = lazycsv::csv::detect_delimiter(&content);
            let output = if detected != b',' {
                // Parse with detected delimiter and re-write as CSV
                let reader = std::io::Cursor::new(content.as_bytes());
                let doc = lazycsv::csv::Document::from_reader(
                    reader,
                    Some(detected),
                    false,
                    "clipboard.csv".to_string(),
                )?;
                let mut buf = Vec::new();
                lazycsv::csv::write_csv_content(&mut buf, &doc, ',')?;
                String::from_utf8(buf)?
            } else {
                content.clone()
            };
            std::fs::write(&out_path, &output)?;
            let lines = output.lines().count().saturating_sub(1);
            let delim_name = match detected {
                b'\t' => "tab",
                b'|' => "pipe",
                b';' => "semicolon",
                _ => "comma",
            };
            eprintln!(
                "Pasted {} rows to {} ({}-delimited input → CSV)",
                lines,
                out_path.display(),
                delim_name
            );
        }
        return Ok(());
    }

    // Non-interactive split mode
    if let Some(rows_per_file) = cli_args.split {
        let path = cli_args
            .file_path()
            .context("Usage: lazycsv <file> -S <rows>")?;
        return execute_split(&path, rows_per_file, &cli_args);
    }

    // Non-interactive add-header mode: prepend a header row to a CSV file
    if let Some(ref header_spec) = cli_args.add_header {
        return execute_add_header(header_spec, &cli_args);
    }

    // Non-interactive dedup mode: remove duplicate rows and output CSV to stdout
    if let Some(ref dedup_spec) = cli_args.dedup {
        return execute_dedup(dedup_spec, &cli_args);
    }

    // Non-interactive sort mode: load, sort, output CSV to stdout, and exit
    if let Some(ref sort_spec) = cli_args.sort {
        return execute_sort_and_output(sort_spec, &cli_args);
    }

    // Non-interactive stats mode: print column statistics and exit
    if cli_args.is_stats_flag() {
        return execute_stats_mode(&cli_args);
    }

    // Non-interactive row/column count mode: print counts and exit
    if cli_args.headers || cli_args.is_rows_flag() || cli_args.is_columns_flag() {
        return execute_count_mode(&cli_args);
    }

    // Standalone format conversion: lazycsv data.xlsx -o data.parquet
    // Input is loaded via Document::from_file (CSV/TSV/XLSX/ODS/Parquet/JSON/SQLite),
    // output format is chosen by the output file's extension.
    if let Some(out) = cli_args.output.as_deref().filter(|o| *o != "-") {
        let out_path = std::path::Path::new(out);
        let Some(format) = lazycsv::export::ExportFormat::from_extension(out_path) else {
            anyhow::bail!(
                "Unsupported output format for '{}'.\n\
                 Supported output extensions: .csv, .tsv, .json, .md, .xlsx, .ods, .parquet",
                out
            );
        };
        let (input_path, _stdin_cleanup) = if let Some(path) = cli_args.file_path() {
            (path, None)
        } else if stdin_is_piped() {
            let temp_path = save_stdin_to_tempfile()?;
            (temp_path.clone(), Some(temp_path))
        } else {
            anyhow::bail!("No input file specified. Provide a file path or pipe data via stdin.");
        };
        let config = FileConfig::with_options(
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        );
        let sheet_name = if lazycsv::csv::xlsx::is_spreadsheet(&input_path) {
            match cli_args.sheet_from_path() {
                Some(spec) => {
                    let sheets = lazycsv::csv::xlsx::get_sheet_names(&input_path)?;
                    Some(resolve_sheet_spec(spec, &sheets)?)
                }
                None => None,
            }
        } else {
            None
        };
        let doc = lazycsv::Document::from_file_with_sheet(
            &input_path,
            config.delimiter,
            config.no_headers,
            config.encoding,
            sheet_name.as_deref(),
        )?;
        let row_count = if format == lazycsv::export::ExportFormat::Csv {
            if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            lazycsv::csv::write_csv_atomic(&doc, out_path, ',')?;
            doc.row_count().saturating_sub(1)
        } else {
            let (headers, rows) = lazycsv::export::collect_document_data(&doc);
            lazycsv::export::export_to_file(format, out_path, &headers, &rows)?;
            rows.len()
        };
        eprintln!(
            "Converted {} → {} ({} rows)",
            input_path.display(),
            out_path.display(),
            row_count
        );
        return Ok(());
    }

    // Piped stdin can't be used with the interactive TUI (stdin is needed for keyboard input)
    if stdin_is_piped() {
        if cli_args.file_path().is_none() {
            let _ = crossterm::terminal::disable_raw_mode();
            eprintln!("Piped stdin is not supported in interactive TUI mode.");
            eprintln!("Use a non-interactive flag: -q <query>, --sort <col>, --headers, --rows, or --columns.");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  cat data.csv | lazycsv -q \"SELECT * FROM stdin\"");
            eprintln!("  cat data.csv | lazycsv --sort Salary");
            eprintln!("  cat data.csv | lazycsv --rows");
            std::process::exit(1);
        } else {
            let _ = crossterm::terminal::disable_raw_mode();
            eprintln!(
                "Cannot open interactive TUI when stdin is piped (keyboard input unavailable)."
            );
            eprintln!("Use '&&' instead of '|' to chain commands, or use a non-interactive flag.");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  lazycsv -P && lazycsv clipboard.csv");
            eprintln!("  lazycsv data.csv -q \"SELECT * FROM data\"");
            std::process::exit(1);
        }
    }

    // Interactive TUI mode: resolve files first, then show loading screen
    let (file_path, csv_files, index, config) = App::resolve_files(&cli_args)?;

    // Initialize terminal before loading so we can show feedback
    let mut terminal = ratatui::init();

    // Enable keyboard enhancement so Ctrl+Enter is distinguishable from Enter.
    // Gracefully ignored if the terminal doesn't support it.
    let supports_enhancement =
        crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
    if supports_enhancement {
        crossterm::execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )
        .ok();
    }

    // Enable mouse capture for click and scroll support
    crossterm::execute!(std::io::stdout(), EnableMouseCapture).ok();

    // For xlsx files, resolve sheet from CLI arg or prompt user
    // For sqlite files, resolve table name from CLI arg or use first table
    let sheet_name = if lazycsv::csv::foreign_formats::is_sqlite(&file_path) {
        let cli_table = cli_args.sheet_from_path();
        match lazycsv::csv::foreign_formats::get_sqlite_tables(&file_path) {
            Ok(tables) => {
                if let Some(spec) = cli_table {
                    Some(spec.to_string())
                } else {
                    tables.into_iter().next()
                }
            }
            Err(e) => {
                restore_terminal(supports_enhancement);
                return Err(e);
            }
        }
    } else if lazycsv::csv::xlsx::is_spreadsheet(&file_path) {
        let cli_sheet = cli_args.sheet_from_path();
        match lazycsv::csv::xlsx::get_sheet_names(&file_path) {
            Ok(sheets) => {
                if let Some(spec) = cli_sheet {
                    Some(resolve_sheet_spec(spec, &sheets)?)
                } else {
                    sheets.into_iter().next()
                }
            }
            Err(e) => {
                restore_terminal(supports_enhancement);
                return Err(e);
            }
        }
    } else {
        None
    };

    // Show loading message while file loads (with Esc hint)
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    terminal.draw(|frame| {
        ui::render_loading(frame, &format!("Loading {}... (Esc to cancel)", filename))
    })?;

    // Load file with cancellation support — background thread watches for Esc
    let cancelled = Arc::new(AtomicBool::new(false));
    let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
    let app_result = App::load_file_cancellable(
        &file_path,
        csv_files,
        index,
        config,
        &cli_args,
        &cancelled,
        sheet_name.as_deref(),
    );
    watcher.stop();

    let mut app = match app_result {
        Ok(mut app) => {
            app.sql_history = lazycsv::config::load_sql_history();
            app.command_history = lazycsv::config::load_command_history();
            app.shell_history = lazycsv::config::load_shell_history();
            // Layer the user's keys.toml (if present) on top of the baked
            // vim default. Warnings on bad bindings surface in the status
            // bar at startup.
            if let Some(keymap_path) = lazycsv::config::dirs_path().map(|p| p.join("keys.toml")) {
                match lazycsv::config::keys::load_toml_file(&keymap_path) {
                    Ok(Some(toml)) => {
                        let mut warnings = Vec::new();
                        app.keymap = lazycsv::config::keys::Keymap::from_toml(&toml, &mut warnings);
                        if !warnings.is_empty() {
                            app.status_message = Some(lazycsv::input::StatusMessage::from(
                                format!("keys.toml: {}", warnings.join("; ")),
                            ));
                        }
                    }
                    Ok(None) => {} // user has no keys.toml — keep vim default
                    Err(e) => {
                        app.status_message = Some(lazycsv::input::StatusMessage::from(format!(
                            "keys.toml: {}",
                            e
                        )));
                    }
                }
            }
            app
        }
        Err(e) => {
            // If cancelled, exit cleanly
            if e.downcast_ref::<lazycsv::cancel::CancelledError>()
                .is_some()
            {
                restore_terminal(supports_enhancement);
                return Ok(());
            }
            restore_terminal(supports_enhancement);
            return Err(e);
        }
    };

    // Load saved view settings for the active file
    {
        let store = views::load_views();
        let active_path = app
            .session
            .files()
            .get(app.session.active_file_index())
            .cloned();
        if let Some(ref path) = active_path {
            let key = views::canonical_key(path);
            if let Some(fv) = store.files.get(&key) {
                views::apply_file_view(path, fv, &mut app.session, &mut app.view_state);
            }
        }
    }

    // Run app (wrapped to ensure cleanup)
    let result = run(&mut terminal, &mut app);

    // Save view settings for all open files before exit
    views::save_current_views(&app);

    // Persist `:` command history before exit
    lazycsv::config::save_command_history(
        &app.command_history,
        app.config.defaults.command_history_limit,
    );
    // Persist file-menu shell history before exit
    lazycsv::config::save_shell_history(
        &app.shell_history,
        app.config.defaults.shell_history_limit,
    );

    // Always restore terminal
    restore_terminal(supports_enhancement);

    // Exit immediately to avoid slow destructor cleanup for large documents.
    // The OS reclaims all memory when the process exits.
    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => Err(e),
    }
}

/// Resolve a sheet specifier (name or 1-based index) against available sheet names.
fn resolve_sheet_spec(spec: &str, sheets: &[String]) -> Result<String> {
    // Try as 1-based index first
    if let Ok(idx) = spec.parse::<usize>() {
        if idx == 0 || idx > sheets.len() {
            anyhow::bail!(
                "Sheet index {} out of range (1-{}). Available sheets: {}",
                idx,
                sheets.len(),
                sheets.join(", ")
            );
        }
        return Ok(sheets[idx - 1].clone());
    }
    // Treat as sheet name
    if !sheets.iter().any(|s| s == spec) {
        anyhow::bail!(
            "Sheet '{}' not found. Available sheets: {}",
            spec,
            sheets.join(", ")
        );
    }
    Ok(spec.to_string())
}

fn restore_terminal(supports_enhancement: bool) {
    // Reset mouse pointer shape to default
    {
        use std::io::Write;
        let _ = write!(std::io::stdout(), "\x1b]22;default\x07");
        let _ = std::io::stdout().flush();
    }
    crossterm::execute!(std::io::stdout(), DisableMouseCapture).ok();
    if supports_enhancement {
        crossterm::execute!(std::io::stdout(), PopKeyboardEnhancementFlags).ok();
    }
    ratatui::restore();
    // Ensure cursor is visible after leaving the TUI
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Show).ok();
    // Print a newline so zsh doesn't show the "no newline" indicator (%)
    // Must use write + flush since std::process::exit() follows and won't flush buffers
    use std::io::Write;
    let _ = std::io::stdout().write_all(b"\n");
    let _ = std::io::stdout().flush();
}

fn run(terminal: &mut Term, app: &mut App) -> Result<()> {
    let mut needs_redraw = true;
    let mut last_mtime_check = Instant::now();

    loop {
        if needs_redraw {
            terminal
                .draw(|frame| ui::render(frame, app))
                .context("Failed to render UI")?;
            needs_redraw = false;
        }

        if event::poll(Duration::from_millis(100)).context("Failed to poll for events")? {
            match event::read().context("Failed to read event")? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let result = app.handle_key(key)?;
                    needs_redraw = true;
                    handle_input_result(terminal, app, result)?;
                }
                Event::Mouse(mouse) => {
                    // Skip Moved events — they fire on every pixel of mouse
                    // movement and cause expensive redraws with column
                    // width recalculation.
                    if matches!(mouse.kind, crossterm::event::MouseEventKind::Moved) {
                        continue;
                    }
                    let (result, mouse_redraw) = app.handle_mouse(mouse);
                    if mouse_redraw {
                        needs_redraw = true;
                    }
                    handle_input_result(terminal, app, result)?;
                }
                _ => {}
            }
        }

        if last_mtime_check.elapsed() >= Duration::from_secs(2) {
            last_mtime_check = Instant::now();
            if app.check_current_file_modification() {
                needs_redraw = true;
            }
            if app.check_config_reload() {
                needs_redraw = true;
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Dispatch input results to appropriate handlers
fn handle_input_result(terminal: &mut Term, app: &mut App, result: InputResult) -> Result<()> {
    match result {
        InputResult::ReloadFile => handle_reload_file(terminal, app)?,
        InputResult::Quit => app.should_quit = true,
        InputResult::SwitchToDocument(doc) => handle_switch_document(terminal, app, doc)?,
        InputResult::SortDocument {
            col_indices,
            ascending,
            description,
        } => handle_sort_document(terminal, app, col_indices, ascending, description)?,
        InputResult::ExecuteQuery { query } => handle_execute_query(terminal, app, query)?,
        InputResult::OpenFile(path) => handle_open_file(terminal, app, path)?,
        InputResult::RunShell { command, cwd } => handle_run_shell(terminal, app, command, cwd)?,
        InputResult::Continue => {}
    }
    Ok(())
}

/// Suspend the TUI, run a shell command via `$SHELL -c` in `cwd`, then
/// re-init. Stdout is discarded; stderr is captured (≤ 64 KiB). Outcome is
/// surfaced via `app.status_message` for the file-menu status bar.
fn handle_run_shell(
    terminal: &mut Term,
    app: &mut App,
    command: String,
    cwd: std::path::PathBuf,
) -> Result<()> {
    use lazycsv::input::StatusMessage;
    use std::process::{Command, Stdio};

    // Truncate captured stderr to keep memory bounded.
    const STDERR_CAP: usize = 64 * 1024;

    // Suspend the TUI so the command sees a normal terminal (and its own
    // output, if it produces any, doesn't corrupt the alternate screen).
    ratatui::restore();

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let output = Command::new(&shell)
        .arg("-c")
        .arg(&command)
        .current_dir(&cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    // Re-enter the alternate screen + raw mode so we can render again.
    *terminal = ratatui::init();
    // Aggressively clear: ratatui's frame buffer + the terminal's screen
    // buffer + reset cursor. Without this, some terminals leak the previous
    // frame's content (stale rows, ghost glyphs) until the next nudge.
    use crossterm::{cursor, execute, terminal::Clear, terminal::ClearType};
    let _ = execute!(
        std::io::stdout(),
        Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    );
    let _ = terminal.clear();
    let _ = terminal.autoresize();

    match output {
        Ok(out) => {
            let mut stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            if stderr.len() > STDERR_CAP {
                stderr.truncate(STDERR_CAP);
                stderr.push_str("\n…(truncated)");
            }
            let stderr_first_line = stderr
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .to_string();
            let stderr_is_multiline = stderr.lines().filter(|l| !l.trim().is_empty()).count() > 1;
            if out.status.success() {
                if !stderr_first_line.is_empty() {
                    app.status_message =
                        Some(StatusMessage::from(format!("Shell: {}", stderr_first_line)));
                    if stderr_is_multiline {
                        app.shell_error_popup = Some(lazycsv::app::ShellErrorPopup {
                            title: format!(" Shell output: {} ", truncate_for_title(&command, 40)),
                            body: stderr,
                            scroll: 0,
                        });
                    }
                }
                // Success — file listing auto-refreshes on next render via
                // scan_directory_filtered (no explicit invalidation needed).
            } else {
                let code = out
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".into());
                let detail = if stderr_first_line.is_empty() {
                    String::new()
                } else {
                    format!(": {}", stderr_first_line)
                };
                app.status_message = Some(StatusMessage::from(format!(
                    "Shell error (exit {}){}",
                    code, detail
                )));
                if stderr_is_multiline {
                    app.shell_error_popup = Some(lazycsv::app::ShellErrorPopup {
                        title: format!(
                            " Shell error (exit {}): {} ",
                            code,
                            truncate_for_title(&command, 40)
                        ),
                        body: stderr,
                        scroll: 0,
                    });
                }
            }
        }
        Err(e) => {
            app.status_message = Some(StatusMessage::from(format!(
                "Shell error: failed to spawn {}: {}",
                shell, e
            )));
        }
    }

    Ok(())
}

/// Trim a command string to a fixed display width with an ellipsis.
fn truncate_for_title(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let kept: String = chars.iter().take(max.saturating_sub(1)).collect();
        format!("{}…", kept)
    }
}

/// Handle file reload with cancellation support
fn handle_reload_file(terminal: &mut Term, app: &mut App) -> Result<()> {
    app.external_modification_pending = false;
    app.search_state = None;

    let filename = app
        .current_file()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    app.status_message = Some(lazycsv::input::StatusMessage::new_persistent(format!(
        "Loading {}... (Esc to cancel)",
        filename
    )));
    terminal
        .draw(|frame| ui::render(frame, app))
        .context("Failed to render UI")?;

    let cancelled = Arc::new(AtomicBool::new(false));
    let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
    let reload_result = app.reload_current_file_cancellable(&cancelled);
    watcher.stop();

    match reload_result {
        Ok(true) => {
            let current_path = app.current_file().clone();
            app.invalidate_duckdb_cache_for(&current_path);
            // Restore per-file history
            if let Some(history) = app.session.take_history(&current_path) {
                app.history = history;
            }
            app.status_message = None;
            terminal.clear().context("Failed to clear terminal")?;
        }
        Ok(false) => {
            app.status_message = Some(lazycsv::input::StatusMessage::from(
                "Load cancelled".to_string(),
            ));
        }
        Err(e) => return Err(e).context("Failed to reload CSV file"),
    }
    Ok(())
}

/// Handle opening a new file from the file browser with a loading screen.
fn handle_open_file(terminal: &mut Term, app: &mut App, path: std::path::PathBuf) -> Result<()> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    terminal.clear().context("Failed to clear terminal")?;
    terminal
        .draw(|frame| {
            ui::render_loading(frame, &format!("Loading {}... (Esc to cancel)", filename))
        })
        .context("Failed to render UI")?;

    let cancelled = Arc::new(AtomicBool::new(false));
    let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);

    let config = app.session.config();
    let load_result = lazycsv::csv::Document::from_file_cancellable(
        &path,
        config.delimiter,
        config.no_headers,
        config.encoding.clone(),
        &cancelled,
    );
    watcher.stop();

    let document = match load_result {
        Ok(doc) => doc,
        Err(err) => {
            if err
                .downcast_ref::<lazycsv::cancel::CancelledError>()
                .is_some()
            {
                app.status_message = Some(lazycsv::input::StatusMessage::from(
                    "Load cancelled".to_string(),
                ));
            } else {
                app.status_message = Some(lazycsv::input::StatusMessage::from(format!(
                    "Failed to load: {}",
                    err
                )));
            }
            return Ok(());
        }
    };

    // Save current file's history before switching
    {
        let current_path = app.current_file().clone();
        let history = std::mem::replace(
            &mut app.history,
            lazycsv::history::History::new(app.config.defaults.undo_limit),
        );
        app.session.cache_history(current_path, history);
    }

    let new_index = app.session.add_file(path.clone());
    app.session.set_active_file_index(new_index);
    app.session.record_file_mtime(&path);

    let old_storage = app.document.take_storage();
    std::thread::spawn(move || drop(old_storage));
    app.document = document;

    app.view_state = lazycsv::ui::ViewState::default();
    app.view_state.table_state.select(Some(0));

    {
        use lazycsv::config::views;
        let store = views::load_views();
        let key = views::canonical_key(&path);
        if let Some(fv) = store.files.get(&key) {
            views::apply_file_view(&path, fv, &mut app.session, &mut app.view_state);
        }
    }

    app.status_message = Some(lazycsv::input::StatusMessage::from(format!(
        "Loaded: {}",
        filename
    )));
    Ok(())
}

/// Handle switching to a different document (from query results or file switch)
fn handle_switch_document(
    terminal: &mut Term,
    app: &mut App,
    doc: lazycsv::csv::Document,
) -> Result<()> {
    terminal.clear().context("Failed to clear terminal")?;

    // Save current file's history before switching
    {
        let current_path = app.current_file().clone();
        let history = std::mem::replace(
            &mut app.history,
            lazycsv::history::History::new(app.config.defaults.undo_limit),
        );
        app.session.cache_history(current_path, history);
    }

    if app.document.is_dirty {
        let current_path = app.current_file().clone();
        app.session.mark_dirty(&current_path);
        // Only cache small dirty documents (edited files).
        // Large query results live in temp CSV files and don't need cloning,
        // which would materialize millions of mmap'd rows into RAM.
        if !app.document.is_lazy() {
            app.session
                .cache_document(current_path, app.document.clone());
        }
    }

    let doc_filename = doc.filename.clone();
    let existing_idx = app.session.files().iter().position(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s == doc_filename)
            .unwrap_or(false)
    });

    if let Some(idx) = existing_idx {
        app.session.set_active_file_index(idx);
    } else {
        let path = std::path::PathBuf::from(&doc_filename);
        let idx = app.session.add_file(path);
        app.session.set_active_file_index(idx);
    }

    let current_path = app.current_file().clone();
    app.session.mark_query_output(&current_path);

    let old_storage = app.document.take_storage();
    std::thread::spawn(move || drop(old_storage));
    app.document = doc;

    // Mark query result as unsaved so the tab shows (*)
    app.document.is_dirty = true;
    let result_path = app.current_file().clone();
    app.session.mark_dirty(&result_path);

    // Restore history for the new file (if any)
    let new_path = app.current_file().clone();
    if let Some(history) = app.session.take_history(&new_path) {
        app.history = history;
    }

    app.view_state = lazycsv::ui::ViewState::default();
    // Start at row 0 (displays as row 1)
    app.view_state.table_state.select(Some(0));
    Ok(())
}

/// Handle document sorting
fn handle_sort_document(
    terminal: &mut Term,
    app: &mut App,
    col_indices: Vec<usize>,
    ascending: bool,
    description: String,
) -> Result<()> {
    let direction = if ascending { "ascending" } else { "descending" };
    app.status_message = Some(lazycsv::input::StatusMessage::new_persistent(format!(
        "Sorting by {} {}... (Esc to cancel)",
        description, direction
    )));
    terminal
        .draw(|frame| ui::render(frame, app))
        .context("Failed to render UI")?;

    let cancelled = Arc::new(AtomicBool::new(false));
    let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
    let column_types = app.session.column_types().cloned().unwrap_or_default();
    let completed =
        app.document
            .sort_by_columns_typed(&col_indices, ascending, &cancelled, &column_types);
    watcher.stop();

    if completed {
        let current_file = app.current_file().clone();
        app.session.mark_dirty(&current_file);
        app.status_message = Some(lazycsv::input::StatusMessage::from(format!(
            "Sorted by {} {}",
            description, direction
        )));
    } else {
        app.status_message = Some(lazycsv::input::StatusMessage::from(
            "Sort cancelled".to_string(),
        ));
    }
    Ok(())
}

/// Handle SQL query execution with cancellation support
/// Handle a DML statement by modifying the current document in-place.
fn handle_execute_dml(terminal: &mut Term, app: &mut App, query: String) -> Result<()> {
    app.mode = lazycsv::app::Mode::Normal;
    terminal
        .draw(|frame| ui::render_loading(frame, "Executing DML... (Esc to cancel)"))
        .context("Failed to render UI")?;

    let cancelled = Arc::new(AtomicBool::new(false));
    let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
    let mut on_progress = |msg: &str| {
        let full_msg = format!("{} (Esc to cancel)", msg);
        let _ = terminal.draw(|frame| ui::render_loading(frame, &full_msg));
    };
    let (success, was_cancelled) =
        app.execute_sql_dml_cancellable(&query, &cancelled, &mut on_progress);
    watcher.stop();

    if was_cancelled {
        app.mode = lazycsv::app::Mode::SqlEditor;
        app.status_message = Some(lazycsv::input::StatusMessage::from(
            "DML cancelled".to_string(),
        ));
    } else if success {
        app.status_message = Some(lazycsv::input::StatusMessage::from(
            "DML executed successfully".to_string(),
        ));
    } else {
        // Error is already in app.sql_error
        app.mode = lazycsv::app::Mode::SqlEditor;
    }
    Ok(())
}

/// Check if a SQL query is a DML statement (INSERT, UPDATE, DELETE, ALTER).
fn is_dml_query(query: &str) -> bool {
    let trimmed = query.trim();
    // Skip leading comments and whitespace
    let first_word = trimmed
        .split_whitespace()
        .find(|w| !w.starts_with("--"))
        .unwrap_or("")
        .to_ascii_uppercase();
    matches!(
        first_word.as_str(),
        "INSERT" | "UPDATE" | "DELETE" | "ALTER" | "DROP" | "CREATE"
    )
}

fn handle_execute_query(terminal: &mut Term, app: &mut App, query: String) -> Result<()> {
    app.push_sql_history(query.clone());
    lazycsv::config::save_sql_history(&app.sql_history, app.config.sql.sql_history_limit);

    // DML statements modify the current document in-place
    if is_dml_query(&query) {
        return handle_execute_dml(terminal, app, query);
    }

    let output_name = app
        .session
        .find_query_output_file()
        .and_then(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| app.generate_output_filename());

    let existing_idx = app.session.files().iter().position(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s == output_name)
            .unwrap_or(false)
    });
    let newly_added = existing_idx.is_none();
    if newly_added {
        let path = std::path::PathBuf::from(&output_name);
        app.session.add_file(path);
    }

    app.mode = lazycsv::app::Mode::Normal;
    terminal
        .draw(|frame| ui::render_loading(frame, "Executing query... (Esc to cancel)"))
        .context("Failed to render UI")?;

    let cancelled = Arc::new(AtomicBool::new(false));
    let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
    let mut on_progress = |msg: &str| {
        let full_msg = format!("{} (Esc to cancel)", msg);
        let _ = terminal.draw(|frame| ui::render_loading(frame, &full_msg));
    };
    let (query_result, was_cancelled) =
        app.execute_sql_query_cancellable(&query, &output_name, &cancelled, &mut on_progress);
    watcher.stop();

    if was_cancelled {
        if newly_added {
            let path = std::path::PathBuf::from(&output_name);
            app.session.remove_file(&path);
        }
        app.mode = lazycsv::app::Mode::SqlEditor;
        app.status_message = Some(lazycsv::input::StatusMessage::from(
            "Query cancelled".to_string(),
        ));
    } else if let Some(doc) = query_result {
        handle_switch_document(terminal, app, doc)?;
    } else {
        if newly_added {
            let path = std::path::PathBuf::from(&output_name);
            app.session.remove_file(&path);
        }
        app.mode = lazycsv::app::Mode::SqlEditor;
        app.status_message = None;
    }
    Ok(())
}

/// Spawn a clipboard command and return the child process.
fn spawn_clipboard_command() -> Result<std::process::Child> {
    #[cfg(target_os = "macos")]
    {
        use std::process::{Command, Stdio};
        Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to run pbcopy. Is it available?")
    }

    #[cfg(target_os = "linux")]
    {
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
            .context("No clipboard tool found. Install xclip, xsel, or wl-copy.")
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("Clipboard not supported on this platform")
}

/// Read text from the system clipboard.
fn read_from_clipboard() -> Result<String> {
    #[cfg(target_os = "macos")]
    let output = {
        use std::process::{Command, Stdio};
        Command::new("pbpaste")
            .stdout(Stdio::piped())
            .output()
            .context("Failed to run pbpaste. Is it available?")?
    };

    #[cfg(target_os = "linux")]
    let output = {
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
            .context("No clipboard tool found. Install xclip, xsel, or wl-paste.")?
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("Clipboard not supported on this platform");

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        if !output.status.success() {
            anyhow::bail!("Clipboard read failed");
        }
        String::from_utf8(output.stdout).context("Clipboard contains invalid UTF-8")
    }
}

/// Copy text to the system clipboard.
fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::io::Write;

    let mut child = spawn_clipboard_command()?;
    child
        .stdin
        .as_mut()
        .context("Failed to open clipboard stdin")?
        .write_all(text.as_bytes())?;
    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("Clipboard command failed");
    }
    Ok(())
}

/// Stream a file directly to the clipboard without loading it all into memory.
/// For CSV files, streams raw bytes. For spreadsheets, converts sheet to CSV and streams.
fn stream_file_to_clipboard(path: &std::path::Path, cli_args: &cli::CliArgs) -> Result<usize> {
    use std::io::{BufReader, Write};

    let mut child = spawn_clipboard_command()?;
    let mut pipe = std::io::BufWriter::with_capacity(
        1024 * 1024,
        child
            .stdin
            .take()
            .context("Failed to open clipboard stdin")?,
    );

    let row_count;

    if lazycsv::csv::foreign_formats::is_foreign_format(path) {
        let rows = lazycsv::csv::foreign_formats::load_foreign_format(path, None)?;
        row_count = rows.len().saturating_sub(1);
        write_csv_rows(&mut pipe, &rows)?;
    } else if lazycsv::csv::xlsx::is_spreadsheet(path) {
        let sheets = lazycsv::csv::xlsx::get_sheet_names(path)?;
        if sheets.is_empty() {
            anyhow::bail!("Spreadsheet has no sheets");
        }
        let sheet_name = match cli_args.sheet_from_path() {
            Some(spec) => resolve_sheet_spec(spec, &sheets)?,
            None => sheets[0].clone(),
        };
        let (rows, _) = lazycsv::csv::xlsx::load_sheet(path, &sheet_name)?;
        row_count = rows.len().saturating_sub(1);
        write_csv_rows(&mut pipe, &rows)?;
    } else {
        // Stream CSV file directly — no intermediate String/Vec allocation.
        // Count newlines while streaming to avoid reading the file twice.
        let file =
            std::fs::File::open(path).context(format!("Failed to open: {}", path.display()))?;
        let mut reader = BufReader::with_capacity(1024 * 1024, file);
        let mut lines = 0usize;
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = std::io::Read::read(&mut reader, &mut buf)?;
            if n == 0 {
                break;
            }
            lines += memchr::memchr_iter(b'\n', &buf[..n]).count();
            pipe.write_all(&buf[..n])?;
        }
        // Subtract header row
        row_count = lines.saturating_sub(1);
    }

    pipe.flush()?;
    drop(pipe); // Close stdin so clipboard command finishes

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("Clipboard command failed");
    }

    Ok(row_count)
}

/// Generate a CSV file with synthetic data and write to stdout or file.
fn execute_generate(
    rows: usize,
    cols: usize,
    gen_type: &str,
    cli_args: &cli::CliArgs,
) -> Result<()> {
    let output_path = cli_args.output.as_deref().filter(|s| *s != "-");

    if let Some(path) = output_path {
        let out_path = std::path::PathBuf::from(path);
        if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::io::BufWriter::new(std::fs::File::create(&out_path)?);
        lazycsv::generate::generate_csv(&mut file, rows, cols, gen_type)?;
        eprintln!(
            "Generated {} rows x {} columns ({}) → {}",
            rows,
            cols,
            gen_type,
            out_path.display()
        );
    } else {
        let stdout = std::io::stdout();
        let mut writer = std::io::BufWriter::new(stdout.lock());
        lazycsv::generate::generate_csv(&mut writer, rows, cols, gen_type)?;
    }
    Ok(())
}

/// Split a CSV/XLSX/ODS file into multiple CSV files with N rows each.
fn execute_split(
    path: &std::path::Path,
    rows_per_file: usize,
    cli_args: &cli::CliArgs,
) -> Result<()> {
    if rows_per_file == 0 {
        anyhow::bail!("Split row count must be greater than 0");
    }

    // Determine output directory
    let out_dir = match &cli_args.output {
        Some(val) if val != "-" => PathBuf::from(val),
        _ => path.parent().map(|p| p.to_path_buf()).unwrap_or_default(),
    };
    std::fs::create_dir_all(&out_dir)
        .context(format!("Failed to create directory: {}", out_dir.display()))?;

    if lazycsv::csv::foreign_formats::is_foreign_format(path) {
        // Foreign formats: load into memory via DuckDB, then split
        let rows = lazycsv::csv::foreign_formats::load_foreign_format(path, None)?;
        split_in_memory_rows(&rows, rows_per_file, &out_dir)?;
        Ok(())
    } else if lazycsv::csv::xlsx::is_spreadsheet(path) {
        // Spreadsheets must be loaded into memory (no streaming)
        split_spreadsheet(path, rows_per_file, &out_dir, cli_args)
    } else {
        // CSV: stream line by line, never load entire file into memory
        split_csv_streaming(path, rows_per_file, &out_dir, cli_args)
    }
}

/// Split a spreadsheet file (must load into memory).
fn split_spreadsheet(
    path: &std::path::Path,
    rows_per_file: usize,
    out_dir: &std::path::Path,
    cli_args: &cli::CliArgs,
) -> Result<()> {
    let sheets = lazycsv::csv::xlsx::get_sheet_names(path)?;
    if sheets.is_empty() {
        anyhow::bail!("Spreadsheet has no sheets");
    }
    let sheet_name = match cli_args.sheet_from_path() {
        Some(spec) => resolve_sheet_spec(spec, &sheets)?,
        None => sheets[0].clone(),
    };
    let (rows, _) = lazycsv::csv::xlsx::load_sheet(path, &sheet_name)?;
    if rows.len() <= 1 {
        anyhow::bail!("File has no data rows");
    }

    let header = &rows[0];
    let data_rows = &rows[1..];
    let mut file_num: usize = 0;
    for chunk in data_rows.chunks(rows_per_file) {
        let file_path = out_dir.join(format!("{}.csv", file_num));
        let file = std::fs::File::create(&file_path)?;
        let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);
        write_csv_rows(&mut writer, std::slice::from_ref(header))?;
        write_csv_rows(&mut writer, chunk)?;
        eprintln!("  {}.csv ({} rows)", file_num, chunk.len());
        file_num += 1;
    }
    eprintln!(
        "Split {} data rows into {} files in {}/",
        data_rows.len(),
        file_num,
        out_dir.display()
    );
    Ok(())
}

/// Split in-memory rows (header + data) into multiple CSV files.
fn split_in_memory_rows(
    rows: &[Vec<String>],
    rows_per_file: usize,
    out_dir: &std::path::Path,
) -> Result<()> {
    if rows.len() <= 1 {
        anyhow::bail!("File has no data rows");
    }
    let header = &rows[0];
    let data_rows = &rows[1..];
    let mut file_num: usize = 0;
    for chunk in data_rows.chunks(rows_per_file) {
        let file_path = out_dir.join(format!("{}.csv", file_num));
        let file = std::fs::File::create(&file_path)?;
        let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);
        write_csv_rows(&mut writer, std::slice::from_ref(header))?;
        write_csv_rows(&mut writer, chunk)?;
        eprintln!("  {}.csv ({} rows)", file_num, chunk.len());
        file_num += 1;
    }
    eprintln!(
        "Split {} data rows into {} files in {}/",
        data_rows.len(),
        file_num,
        out_dir.display()
    );
    Ok(())
}

/// Split a CSV file by streaming — never loads the entire file into memory.
fn split_csv_streaming(
    path: &std::path::Path,
    rows_per_file: usize,
    out_dir: &std::path::Path,
    _cli_args: &cli::CliArgs,
) -> Result<()> {
    use std::io::{BufRead, Write};

    std::fs::create_dir_all(out_dir)
        .context(format!("Failed to create directory: {}", out_dir.display()))?;

    let file = std::fs::File::open(path).context(format!("Failed to open: {}", path.display()))?;
    let reader = std::io::BufReader::with_capacity(1024 * 1024, file);

    let mut lines = reader.lines();

    // Read header line
    let header_line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => return Err(e.into()),
        None => anyhow::bail!("File is empty"),
    };

    let mut file_num: usize = 0;
    let mut total_rows: usize = 0;
    let mut rows_in_current = 0usize;
    let mut writer: Option<std::io::BufWriter<std::fs::File>> = None;

    for line_result in lines {
        let line = line_result?;

        // Start a new output file if needed
        if rows_in_current == 0 || rows_in_current >= rows_per_file {
            // Flush and report previous file
            if let Some(mut w) = writer.take() {
                w.flush()?;
                eprintln!("  {}.csv ({} rows)", file_num, rows_in_current);
                file_num += 1;
            }
            // Open new file
            let file_path = out_dir.join(format!("{}.csv", file_num));
            let file = std::fs::File::create(&file_path)?;
            let mut w = std::io::BufWriter::with_capacity(1024 * 1024, file);
            // Write header
            writeln!(w, "{}", header_line)?;
            writer = Some(w);
            rows_in_current = 0;
        }

        if let Some(ref mut w) = writer {
            writeln!(w, "{}", line)?;
            rows_in_current += 1;
            total_rows += 1;
        }
    }

    // Flush last file
    if let Some(mut w) = writer.take() {
        w.flush()?;
        if rows_in_current > 0 {
            eprintln!("  {}.csv ({} rows)", file_num, rows_in_current);
            file_num += 1;
        }
    }

    eprintln!(
        "Split {} data rows into {} files in {}/",
        total_rows,
        file_num,
        out_dir.display()
    );
    Ok(())
}

/// Check if stdin has piped data (not a terminal).
fn stdin_is_piped() -> bool {
    use std::io::IsTerminal;
    !std::io::stdin().is_terminal()
}

/// Non-interactive xlsx-to-csv conversion.
/// Output modes:
///   (no -o): write to <excel_name>/ directory
///   -o (bare): write to stdout
///   -o dir/: write to specified directory
///   -o file.csv: write to specific file (single sheet only)
fn execute_xlsx_convert(xlsx_path: &std::path::Path, cli_args: &cli::CliArgs) -> Result<()> {
    use lazycsv::csv::xlsx;

    if !xlsx_path.is_file() {
        anyhow::bail!("File not found: {}", xlsx_path.display());
    }
    if !xlsx::is_spreadsheet(xlsx_path) {
        anyhow::bail!(
            "Not a spreadsheet file: {} (expected .xlsx or .xls)",
            xlsx_path.display()
        );
    }

    let all_sheets = xlsx::get_sheet_names(xlsx_path)?;
    if all_sheets.is_empty() {
        anyhow::bail!("Spreadsheet has no sheets");
    }

    // Determine which sheets to extract
    let single_sheet = cli_args.sheet_from_path().is_some();
    let sheets_to_extract: Vec<String> = match cli_args.sheet_from_path() {
        Some(spec) => vec![resolve_sheet_spec(spec, &all_sheets)?],
        None => all_sheets,
    };

    // Determine output mode
    enum OutputMode {
        Stdout,
        File(PathBuf),
        Directory(PathBuf),
    }

    let output_mode = match &cli_args.output {
        Some(val) if val == "-" => {
            // -o with no value (default_missing_value = "-") → stdout
            OutputMode::Stdout
        }
        Some(val) => {
            let p = PathBuf::from(val);
            // If it's an existing directory, or ends with separator, or has no extension → directory
            if p.is_dir()
                || val.ends_with('/')
                || val.ends_with(std::path::MAIN_SEPARATOR)
                || p.extension().is_none()
            {
                OutputMode::Directory(p)
            } else {
                // Looks like a file path
                if !single_sheet && sheets_to_extract.len() > 1 {
                    anyhow::bail!(
                        "Cannot output multiple sheets to a single file.\n\
                         Specify a sheet: lazycsv {} <sheet> -x -o {}\n\
                         Or use a directory: lazycsv {} -x -o {}/",
                        xlsx_path.display(),
                        val,
                        xlsx_path.display(),
                        val,
                    );
                }
                OutputMode::File(p)
            }
        }
        None => {
            // No -o flag → default directory named after the xlsx file
            let stem = xlsx_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            OutputMode::Directory(PathBuf::from(stem))
        }
    };

    match output_mode {
        OutputMode::Stdout => {
            let stdout = std::io::stdout();
            let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, stdout.lock());
            for sheet_name in &sheets_to_extract {
                let (rows, _) = xlsx::load_sheet(xlsx_path, sheet_name)?;
                write_csv_rows(&mut writer, &rows)?;
            }
        }
        OutputMode::File(path) => {
            let (rows, sheet) = xlsx::load_sheet(xlsx_path, &sheets_to_extract[0])?;
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            let file = std::fs::File::create(&path)
                .context(format!("Failed to create: {}", path.display()))?;
            let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);
            write_csv_rows(&mut writer, &rows)?;
            eprintln!(
                "  {} ({} rows) -> {}",
                sheet,
                rows.len().saturating_sub(1),
                path.display()
            );
        }
        OutputMode::Directory(dir) => {
            std::fs::create_dir_all(&dir)
                .context(format!("Failed to create directory: {}", dir.display()))?;
            for sheet_name in &sheets_to_extract {
                let (rows, sheet) = xlsx::load_sheet(xlsx_path, sheet_name)?;
                let output_path = dir.join(format!("{}.csv", sheet));
                let file = std::fs::File::create(&output_path)
                    .context(format!("Failed to create: {}", output_path.display()))?;
                let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);
                write_csv_rows(&mut writer, &rows)?;
                eprintln!(
                    "  {} ({} rows) -> {}",
                    sheet,
                    rows.len().saturating_sub(1),
                    output_path.display()
                );
            }
            eprintln!(
                "Extracted {} sheet(s) to {}/",
                sheets_to_extract.len(),
                dir.display()
            );
        }
    }

    Ok(())
}

/// Write rows as CSV to any writer.
fn write_csv_rows<W: std::io::Write>(writer: &mut W, rows: &[Vec<String>]) -> Result<()> {
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                write!(writer, ",")?;
            }
            if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                write!(writer, "\"{}\"", cell.replace('"', "\"\""))?;
            } else {
                write!(writer, "{}", cell)?;
            }
        }
        writeln!(writer)?;
    }
    Ok(())
}

/// Save piped stdin to a temporary file and return its path.
/// The caller is responsible for cleanup (or let it be cleaned up on process exit).
fn save_stdin_to_tempfile() -> Result<PathBuf> {
    use std::io::Read;
    let mut buf = Vec::new();
    std::io::stdin().lock().read_to_end(&mut buf)?;
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("stdin.csv");
    std::fs::write(&temp_path, &buf)?;
    Ok(temp_path)
}

/// Non-interactive query mode with stdin support.
fn execute_query_mode(query: &str, cli_args: &cli::CliArgs) -> Result<()> {
    let config = FileConfig::with_options(
        cli_args.delimiter,
        cli_args.no_headers,
        cli_args.encoding.clone(),
    );

    // Resolve the query path (file, xlsx temp dir, stdin temp, or cwd)
    let (query_path, temp_cleanup) = if let Some(path) = cli_args.file_path() {
        if lazycsv::csv::xlsx::is_spreadsheet(&path) {
            let temp_dir = extract_xlsx_to_temp(&path, cli_args)?;
            (temp_dir.clone(), Some(temp_dir))
        } else {
            (path, None)
        }
    } else if stdin_is_piped() {
        let temp_path = save_stdin_to_tempfile()?;
        (temp_path.clone(), Some(temp_path))
    } else {
        (PathBuf::from("."), None)
    };

    // Run the query — write to file if -o is given, otherwise stdout
    let output_file = cli_args.output.as_deref().filter(|o| *o != "-");
    let result = if let Some(out) = output_file {
        let out_path = std::path::PathBuf::from(out);

        // Detect non-CSV format from extension
        let format = lazycsv::export::ExportFormat::from_extension(&out_path);
        if matches!(format, Some(f) if f != lazycsv::export::ExportFormat::Csv) {
            // Run query to document, then export in requested format
            let doc = lazycsv::query::execute_query_to_doc_from_path(&query_path, query, &config)?;
            let (headers, rows) = lazycsv::export::collect_document_data(&doc);
            lazycsv::export::export_to_file(format.unwrap(), &out_path, &headers, &rows)
                .inspect(|_| eprintln!("Query results exported to {}", out_path.display()))
        } else {
            lazycsv::query::execute_query_to_file(
                &query_path,
                query,
                &config,
                &out_path,
                cli_args.no_headers,
            )
            .inspect(|_| eprintln!("Query results written to {}", out_path.display()))
        }
    } else {
        lazycsv::query::execute_query(&query_path, query, &config, cli_args.no_headers)
    };

    // If clipboard requested, re-run the query to capture output (query is fast from SQLite cache)
    if cli_args.clipboard && result.is_ok() {
        // Re-read what was just printed by running the query to a document
        // This avoids needing to refactor the query module's stdout writing
        if let Ok(doc) = lazycsv::query::execute_query_to_doc_from_path(&query_path, query, &config)
        {
            let mut buf = Vec::new();
            if lazycsv::csv::write_csv_content(&mut buf, &doc, ',').is_ok() {
                if let Ok(csv_str) = String::from_utf8(buf) {
                    if copy_to_clipboard(&csv_str).is_ok() {
                        let lines = csv_str.lines().count().saturating_sub(1);
                        eprintln!("Copied {} rows to clipboard", lines);
                    }
                }
            }
        }
    }

    // Clean up temp files
    if let Some(temp) = temp_cleanup {
        if temp.is_dir() {
            let _ = std::fs::remove_dir_all(&temp);
        } else {
            let _ = std::fs::remove_file(&temp);
        }
    }

    result
}

/// Extract xlsx sheets to a temp directory and return the path.
fn extract_xlsx_to_temp(xlsx_path: &std::path::Path, cli_args: &cli::CliArgs) -> Result<PathBuf> {
    use lazycsv::csv::xlsx;

    let all_sheets = xlsx::get_sheet_names(xlsx_path)?;
    let sheets_to_extract: Vec<String> = match cli_args.sheet_from_path() {
        Some(spec) => vec![resolve_sheet_spec(spec, &all_sheets)?],
        None => all_sheets,
    };

    let temp_dir = std::env::temp_dir().join(format!("lazycsv_xlsx_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    for sheet_name in &sheets_to_extract {
        let (rows, sheet) = xlsx::load_sheet(xlsx_path, sheet_name)?;
        let csv_path = temp_dir.join(format!("{}.csv", sheet));
        let file = std::fs::File::create(&csv_path)?;
        let mut writer = std::io::BufWriter::new(file);
        write_csv_rows(&mut writer, &rows)?;
    }

    Ok(temp_dir)
}

/// Non-interactive stats mode: compute per-column statistics using DuckDB.
fn execute_stats_mode(cli_args: &cli::CliArgs) -> Result<()> {
    use duckdb::Connection;

    let config = FileConfig::with_options(
        cli_args.delimiter,
        cli_args.no_headers,
        cli_args.encoding.clone(),
    );

    // Resolve the file path (file, xlsx temp dir, stdin temp, or cwd)
    let (query_path, _temp_cleanup) = if let Some(path) = cli_args.file_path() {
        if lazycsv::csv::xlsx::is_spreadsheet(&path) {
            let temp_dir = extract_xlsx_to_temp(&path, cli_args)?;
            (temp_dir.clone(), Some(temp_dir))
        } else {
            (path, None)
        }
    } else if stdin_is_piped() {
        let temp_path = save_stdin_to_tempfile()?;
        (temp_path.clone(), Some(temp_path))
    } else {
        anyhow::bail!("Usage: lazycsv <file> --stats");
    };

    // Resolve to a single file or directory of files
    let csv_files = if query_path.is_file() {
        vec![query_path]
    } else if query_path.is_dir() {
        let files = lazycsv::file_system::scan_directory(&query_path)?;
        if files.is_empty() {
            anyhow::bail!("No CSV files found in {}", query_path.display());
        }
        files
    } else {
        anyhow::bail!("Path does not exist: {}", query_path.display());
    };

    let conn = Connection::open_in_memory().context("Failed to open DuckDB")?;
    let mut writer = output_writer(cli_args)?;

    for (idx, file_path) in csv_files.iter().enumerate() {
        let table_name = lazycsv::query::table_name_from_path(file_path);
        lazycsv::query::load_csv_file_into_duckdb(&conn, file_path, &table_name, &config)?;

        if csv_files.len() > 1 {
            if idx > 0 {
                writeln!(writer)?;
            }
            let name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            writeln!(writer, "--- {} ---", name)?;
        }

        print_column_stats(&conn, &table_name, &mut writer)?;
    }

    report_output_file(cli_args, "Stats");
    Ok(())
}

/// Print per-column statistics for a table loaded in DuckDB.
fn print_column_stats<W: std::io::Write>(
    conn: &duckdb::Connection,
    table_name: &str,
    writer: &mut W,
) -> Result<()> {
    let escaped = table_name.replace('"', "\"\"");

    // Get column names and types from DuckDB
    let mut col_stmt = conn.prepare(&format!(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
        escaped.replace('\'', "''")
    ))?;
    let col_info: Vec<(String, String)> = col_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if col_info.is_empty() {
        writeln!(writer, "(no columns)")?;
        return Ok(());
    }

    let headers = [
        "col_name",
        "data_type",
        "min",
        "max",
        "min_len",
        "max_len",
        "mean",
        "stddev",
        "median",
        "mode",
        "cardinality",
    ];

    let mut all_rows: Vec<Vec<String>> = Vec::new();

    for (col_name, col_type) in &col_info {
        let c = col_name.replace('"', "\"\"");

        // Build per-column stats query
        let stats_sql = format!(
            "SELECT \
                CAST(MIN(\"{c}\") AS VARCHAR), \
                CAST(MAX(\"{c}\") AS VARCHAR), \
                MIN(LENGTH(CAST(\"{c}\" AS VARCHAR))), \
                MAX(LENGTH(CAST(\"{c}\" AS VARCHAR))), \
                ROUND(AVG(TRY_CAST(\"{c}\" AS DOUBLE)), 4), \
                ROUND(STDDEV_SAMP(TRY_CAST(\"{c}\" AS DOUBLE)), 4), \
                ROUND(MEDIAN(TRY_CAST(\"{c}\" AS DOUBLE)), 4), \
                COUNT(DISTINCT \"{c}\") \
            FROM \"{table}\"",
            c = c,
            table = escaped
        );

        let stats = conn.query_row(&stats_sql, [], |row| {
            Ok(vec![
                row.get::<_, String>(0).unwrap_or_else(|_| "NULL".into()),
                row.get::<_, String>(1).unwrap_or_else(|_| "NULL".into()),
                row.get::<_, i64>(2)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "NULL".into()),
                row.get::<_, i64>(3)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "NULL".into()),
                duckdb_get_stat_string(row, 4),
                duckdb_get_stat_string(row, 5),
                duckdb_get_stat_string(row, 6),
                row.get::<_, i64>(7)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "NULL".into()),
            ])
        })?;

        // mode: most frequent value
        let mode_sql = format!(
            "SELECT CAST(\"{c}\" AS VARCHAR) FROM \"{table}\" \
             GROUP BY \"{c}\" ORDER BY COUNT(*) DESC LIMIT 1",
            c = c,
            table = escaped
        );
        let mode_val = conn
            .query_row(&mode_sql, [], |row| row.get::<_, String>(0))
            .unwrap_or_else(|_| "NULL".to_string());

        // [column, type, min, max, min_len, max_len, mean, stddev, median, mode, cardinality]
        let mut row = vec![col_name.clone(), col_type.clone()];
        // min, max
        row.push(stats[0].clone());
        row.push(stats[1].clone());
        // min_len, max_len
        row.push(stats[2].clone());
        row.push(stats[3].clone());
        // mean, stddev, median
        row.push(stats[4].clone());
        row.push(stats[5].clone());
        row.push(stats[6].clone());
        // mode
        row.push(mode_val);
        // cardinality
        row.push(stats[7].clone());

        all_rows.push(row);
    }

    // Output as CSV for pipeable results
    let mut wtr = csv::Writer::from_writer(writer);
    wtr.write_record(headers)?;
    for row in &all_rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;

    Ok(())
}

/// Extract a stat value from a DuckDB row as a string, handling NULLs.
fn duckdb_get_stat_string(row: &duckdb::Row, idx: usize) -> String {
    if let Ok(v) = row.get::<_, f64>(idx) {
        if v.fract() == 0.0 && v.abs() < i64::MAX as f64 {
            return format!("{}", v as i64);
        }
        return format!("{}", v);
    }
    if let Ok(v) = row.get::<_, i64>(idx) {
        return v.to_string();
    }
    if let Ok(v) = row.get::<_, String>(idx) {
        return v;
    }
    "NULL".to_string()
}

/// Non-interactive row/column count mode with stdin support.
fn execute_count_mode(cli_args: &cli::CliArgs) -> Result<()> {
    let separator = if cli_args.format {
        detect_thousands_separator()
    } else {
        '\0'
    };

    let mut writer = output_writer(cli_args)?;

    if cli_args.file_path().is_none() && stdin_is_piped() {
        // Read from stdin
        let doc = {
            let stdin = std::io::stdin();
            let reader = std::io::BufReader::new(stdin.lock());
            lazycsv::csv::Document::from_reader(
                reader,
                cli_args.delimiter,
                cli_args.no_headers,
                "stdin".to_string(),
            )?
        };
        if cli_args.headers {
            let headers: Vec<String> = (0..doc.column_count())
                .map(|i| doc.header(lazycsv::ColIndex::new(i)))
                .collect();
            writeln!(writer, "{}", headers.join(", "))?;
            report_output_file(cli_args, "Headers");
            return Ok(());
        }
        let mut parts = Vec::new();
        if cli_args.is_rows_flag() {
            let count = if doc.row_count() > 0 {
                doc.row_count() - 1 // subtract row 0 (which typically contains column names)
            } else {
                0
            };
            parts.push(format!("{} rows", format_number(count, separator)));
        }
        if cli_args.is_columns_flag() {
            parts.push(format!(
                "{} columns",
                format_number(doc.column_count(), separator)
            ));
        }
        writeln!(writer, "stdin: {}", parts.join(", "))?;
        report_output_file(cli_args, "Output");
        return Ok(());
    }

    let path = cli_args.file_path().unwrap_or_else(|| PathBuf::from("."));
    let files = if path.is_file() {
        vec![path]
    } else if path.is_dir() {
        let csv_files = lazycsv::file_system::scan_directory(&path)?;
        if csv_files.is_empty() {
            anyhow::bail!("No CSV files found in {}", path.display());
        }
        csv_files
    } else {
        anyhow::bail!("Path does not exist: {}", path.display());
    };

    for file in &files {
        let name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if cli_args.headers {
            let headers = lazycsv::csv::Document::read_headers(
                file,
                cli_args.delimiter,
                cli_args.no_headers,
                cli_args.encoding.clone(),
            )?;
            if files.len() > 1 {
                writeln!(writer, "{}: {}", name, headers.join(", "))?;
            } else {
                writeln!(writer, "{}", headers.join(", "))?;
            }
            continue;
        }

        let mut parts = Vec::new();

        if cli_args.is_rows_flag() {
            let count = lazycsv::csv::Document::count_rows(
                file,
                cli_args.delimiter,
                cli_args.no_headers,
                cli_args.encoding.clone(),
            )?;
            parts.push(format!("{} rows", format_number(count, separator)));
        }

        if cli_args.is_columns_flag() {
            let count = lazycsv::csv::Document::count_columns(
                file,
                cli_args.delimiter,
                cli_args.encoding.clone(),
            )?;
            parts.push(format!("{} columns", format_number(count, separator)));
        }

        writeln!(writer, "{}: {}", name, parts.join(", "))?;
    }
    report_output_file(cli_args, "Output");
    Ok(())
}

/// Non-interactive dedup: remove duplicate rows and write CSV to stdout.
fn execute_dedup(dedup_spec: &str, cli_args: &cli::CliArgs) -> Result<()> {
    use duckdb::Connection;

    let config = FileConfig::with_options(
        cli_args.delimiter,
        cli_args.no_headers,
        cli_args.encoding.clone(),
    );

    // Resolve input file
    let (file_path, _temp_cleanup) = if let Some(path) = cli_args.file_path() {
        if lazycsv::csv::xlsx::is_spreadsheet(&path) {
            let temp_dir = extract_xlsx_to_temp(&path, cli_args)?;
            (temp_dir.clone(), Some(temp_dir))
        } else if path.is_file() {
            (path, None)
        } else if path.is_dir() {
            let csv_files = lazycsv::file_system::scan_directory(&path)?;
            let first = csv_files
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No CSV files found in {}", path.display()))?;
            (first, None)
        } else {
            anyhow::bail!("Path does not exist: {}", path.display());
        }
    } else if stdin_is_piped() {
        let temp_path = save_stdin_to_tempfile()?;
        (temp_path.clone(), Some(temp_path))
    } else {
        anyhow::bail!("No input file specified. Provide a file path or pipe data via stdin.");
    };

    let conn = Connection::open_in_memory().context("Failed to open DuckDB")?;
    let table_name = lazycsv::query::table_name_from_path(&file_path);
    lazycsv::query::load_csv_file_into_duckdb(&conn, &file_path, &table_name, &config)?;

    let escaped = table_name.replace('"', "\"\"");

    // Get all column names
    let mut col_stmt = conn.prepare(&format!(
        "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
        escaped.replace('\'', "''")
    ))?;
    let all_columns: Vec<String> = col_stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    if all_columns.is_empty() {
        anyhow::bail!("No columns found in file");
    }

    // Resolve PK columns
    let pk_columns = if dedup_spec.is_empty() {
        // No PK specified: use all columns
        all_columns.clone()
    } else {
        let specs: Vec<&str> = dedup_spec.split(',').map(|s| s.trim()).collect();
        let mut pk = Vec::new();
        for s in &specs {
            if s.is_empty() {
                continue;
            }
            if let Ok(num) = s.parse::<usize>() {
                if num == 0 || num > all_columns.len() {
                    anyhow::bail!("Column {} out of range (1-{})", num, all_columns.len());
                }
                pk.push(all_columns[num - 1].clone());
            } else {
                // Try case-insensitive header match
                let found = all_columns
                    .iter()
                    .find(|c| c.eq_ignore_ascii_case(s))
                    .cloned();
                if let Some(col) = found {
                    pk.push(col);
                } else {
                    anyhow::bail!("Column \"{}\" not found", s);
                }
            }
        }
        if pk.is_empty() {
            anyhow::bail!("No valid columns specified for dedup");
        }
        pk
    };

    // Check for all-NULL PK rows (only when explicit PK columns specified)
    let has_explicit_pk = !dedup_spec.is_empty();
    if has_explicit_pk && !cli_args.allow_nulls {
        let null_check_parts: Vec<String> = pk_columns
            .iter()
            .map(|c| format!("\"{}\" IS NULL", c.replace('"', "\"\"")))
            .collect();
        let null_check_sql = format!(
            "SELECT COUNT(*) FROM \"{}\" WHERE {}",
            escaped,
            null_check_parts.join(" AND ")
        );
        let null_count: i64 = conn.query_row(&null_check_sql, [], |row| row.get(0))?;
        if null_count > 0 {
            anyhow::bail!(
                "Found {} row(s) where all PK columns are NULL (ambiguous). \
                 Use --allow-nulls to include them.",
                null_count
            );
        }
    }

    // Build dedup query using ROW_NUMBER window function
    let partition_cols: Vec<String> = pk_columns
        .iter()
        .map(|c| {
            let escaped_col = format!("\"{}\"", c.replace('"', "\"\""));
            if cli_args.ignore_case {
                format!("LOWER(CAST({} AS VARCHAR))", escaped_col)
            } else {
                escaped_col
            }
        })
        .collect();
    let all_cols: Vec<String> = all_columns
        .iter()
        .map(|c| format!("\"{}\"", c.replace('"', "\"\"")))
        .collect();

    // "last wins" means we want the highest row position; "first wins" the lowest
    let order = if cli_args.keep_first { "ASC" } else { "DESC" };

    if cli_args.report_only {
        // Report mode: show all rows that have duplicates (count > 1), with row numbers
        let pk_col_names: Vec<String> = pk_columns
            .iter()
            .map(|c| format!("\"{}\"", c.replace('"', "\"\"")))
            .collect();
        let report_sql = format!(
            "SELECT _rownum, {all_cols}, _dup_count FROM (\
                SELECT *, COUNT(*) OVER (PARTITION BY {pk}) AS _dup_count, \
                    ROW_NUMBER() OVER () AS _rownum \
                FROM \"{table}\"\
            ) WHERE _dup_count > 1 ORDER BY {pk_orig}, _rownum",
            all_cols = all_cols.join(", "),
            pk = partition_cols.join(", "),
            pk_orig = pk_col_names.join(", "),
            table = escaped
        );

        let col_count = all_columns.len();
        let mut stmt = conn.prepare(&report_sql)?;
        let rows = stmt
            .query_map([], |row| {
                let rownum: i64 = row.get(0)?;
                let values: Vec<String> = (1..=col_count)
                    .map(|i| lazycsv::query::duckdb_get_string(row, i))
                    .collect();
                let dup_count: i64 = row.get(col_count + 1)?;
                Ok((rownum, values, dup_count))
            })
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Write CSV report to output
        let mut wtr = csv::Writer::from_writer(output_writer(cli_args)?);

        // Header: row_number, original columns, dup_count
        let mut header = vec!["row_number".to_string()];
        header.extend(all_columns.iter().cloned());
        header.push("dup_count".to_string());
        wtr.write_record(&header)?;

        for row_result in rows {
            let (rownum, values, dup_count) =
                row_result.context("Failed to read report result row")?;
            let mut record = vec![rownum.to_string()];
            record.extend(values);
            record.push(dup_count.to_string());
            wtr.write_record(&record)?;
        }
        wtr.flush()?;
        report_output_file(cli_args, "Duplicate report");
    } else {
        // Dedup mode: remove duplicates (or show only duplicates if --duplicates is set)
        let filter = if cli_args.duplicates {
            "_rn > 1"
        } else {
            "_rn = 1"
        };
        let dedup_sql = format!(
            "SELECT {cols} FROM (\
                SELECT *, ROW_NUMBER() OVER (PARTITION BY {pk} ORDER BY _rownum {order}) AS _rn \
                FROM (\
                    SELECT *, ROW_NUMBER() OVER () AS _rownum FROM \"{table}\"\
                )\
            ) WHERE {filter} ORDER BY _rownum",
            cols = all_cols.join(", "),
            pk = partition_cols.join(", "),
            order = order,
            table = escaped,
            filter = filter
        );

        let col_count = all_columns.len();
        let mut stmt = conn.prepare(&dedup_sql)?;
        let rows = stmt
            .query_map([], |row| {
                let values: Vec<String> = (0..col_count)
                    .map(|i| lazycsv::query::duckdb_get_string(row, i))
                    .collect();
                Ok(values)
            })
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let label = if cli_args.duplicates {
            "Duplicate rows"
        } else {
            "Deduplicated output"
        };

        if cli_args.is_rows_flag() {
            let count = rows.count();
            let separator = detect_thousands_separator();
            let mut writer = output_writer(cli_args)?;
            writeln!(writer, "{}", format_number(count, separator))?;
            return Ok(());
        }

        // Write output — detect format from -o extension
        if let Some(out_path) = cli_args.output.as_deref().filter(|o| *o != "-") {
            let path = std::path::Path::new(out_path);
            let format = lazycsv::export::ExportFormat::from_extension(path);
            if matches!(
                format,
                Some(
                    lazycsv::export::ExportFormat::Json
                        | lazycsv::export::ExportFormat::Tsv
                        | lazycsv::export::ExportFormat::Markdown
                        | lazycsv::export::ExportFormat::Xlsx
                )
            ) {
                let collected: Vec<Vec<String>> = rows
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .context("Failed to read dedup result")?;
                lazycsv::export::export_to_file(format.unwrap(), path, &all_columns, &collected)?;
                eprintln!("{} exported to {}", label, path.display());
            } else {
                let mut wtr = csv::Writer::from_writer(output_writer(cli_args)?);
                wtr.write_record(&all_columns)?;
                for row_result in rows {
                    let values = row_result.context("Failed to read dedup result row")?;
                    wtr.write_record(&values)?;
                }
                wtr.flush()?;
                report_output_file(cli_args, label);
            }
        } else {
            let mut wtr = csv::Writer::from_writer(output_writer(cli_args)?);
            wtr.write_record(&all_columns)?;
            for row_result in rows {
                let values = row_result.context("Failed to read dedup result row")?;
                wtr.write_record(&values)?;
            }
            wtr.flush()?;
        }
    }

    Ok(())
}

/// Non-interactive sort: load CSV, sort by columns, write sorted CSV to stdout.
fn execute_sort_and_output(sort_spec: &str, cli_args: &cli::CliArgs) -> Result<()> {
    // Load document from file path or stdin
    let mut doc = if let Some(path) = cli_args.file_path() {
        let file_path = if path.is_file() {
            path.clone()
        } else if path.is_dir() {
            let csv_files = lazycsv::file_system::scan_directory(&path)?;
            csv_files
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No CSV files found in {}", path.display()))?
        } else {
            anyhow::bail!("Path does not exist: {}", path.display());
        };
        lazycsv::csv::Document::from_file(
            &file_path,
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        )?
    } else if stdin_is_piped() {
        let stdin = std::io::stdin();
        let reader = std::io::BufReader::new(stdin.lock());
        lazycsv::csv::Document::from_reader(
            reader,
            cli_args.delimiter,
            cli_args.no_headers,
            "stdin".to_string(),
        )?
    } else {
        anyhow::bail!("No input file specified. Provide a file path or pipe data via stdin.");
    };

    // Parse sort spec: optional '!' prefix for descending
    let (spec, ascending) = if let Some(stripped) = sort_spec.strip_prefix('!') {
        (stripped, false)
    } else {
        (sort_spec, true)
    };

    let specs: Vec<&str> = spec.split(',').map(|s| s.trim()).collect();
    let mut col_indices = Vec::new();

    for s in &specs {
        if s.is_empty() {
            continue;
        }
        if let Ok(num) = s.parse::<usize>() {
            if num == 0 || num > doc.column_count() {
                anyhow::bail!("Column {} out of range (1-{})", num, doc.column_count());
            }
            col_indices.push(num - 1);
        } else {
            // Try header name (case-insensitive)
            let col_count = doc.column_count();
            let header_match = (0..col_count).find(|&i| {
                doc.header(lazycsv::ColIndex::new(i))
                    .eq_ignore_ascii_case(s)
            });
            if let Some(idx) = header_match {
                col_indices.push(idx);
            } else if s.chars().all(|c| c.is_ascii_alphabetic()) {
                match lazycsv::ui::utils::excel_letter_to_column(s) {
                    Ok(idx) if idx < doc.column_count() => {
                        col_indices.push(idx);
                    }
                    _ => {
                        anyhow::bail!("Column \"{}\" not found", s);
                    }
                }
            } else {
                anyhow::bail!("Column \"{}\" not found", s);
            }
        }
    }

    if col_indices.is_empty() {
        anyhow::bail!("No valid columns specified in sort: {}", sort_spec);
    }

    // Sort the document (non-interactive, no cancellation)
    let no_cancel = AtomicBool::new(false);
    doc.sort_by_columns(&col_indices, ascending, &no_cancel);

    // Write sorted output — detect format from -o extension
    if let Some(out_path) = cli_args.output.as_deref().filter(|o| *o != "-") {
        let path = std::path::Path::new(out_path);
        let format = lazycsv::export::ExportFormat::from_extension(path);
        if matches!(format, Some(f) if f != lazycsv::export::ExportFormat::Csv) {
            let (headers, rows) = lazycsv::export::collect_document_data(&doc);
            lazycsv::export::export_to_file(format.unwrap(), path, &headers, &rows)?;
            eprintln!("Sorted output exported to {}", path.display());
            return Ok(());
        }
    }
    let delimiter = doc.delimiter;
    let mut out = output_writer(cli_args)?;
    lazycsv::csv::write_csv_content(&mut out, &doc, delimiter)?;
    report_output_file(cli_args, "Sorted output");

    Ok(())
}

/// Non-interactive add-header mode: prepend a header row to a CSV file.
///
/// If `header_spec` is empty, generates C1, C2, ... headers based on column count.
/// If `header_spec` is provided, parses it as CSV and validates the column count.
/// With -o, writes to the output file; without -o, modifies the input file in place.
fn execute_add_header(header_spec: &str, cli_args: &cli::CliArgs) -> Result<()> {
    // Read file content (or stdin)
    let (content, input_path) = if let Some(path) = cli_args.file_path() {
        if !path.is_file() {
            anyhow::bail!("Path does not exist or is not a file: {}", path.display());
        }
        let text = if let Some(ref enc) = cli_args.encoding {
            let raw = std::fs::read(&path)?;
            let label = encoding_rs::Encoding::for_label(enc.as_bytes())
                .ok_or_else(|| anyhow::anyhow!("Unknown encoding: {}", enc))?;
            let (decoded, _, _) = label.decode(&raw);
            decoded.into_owned()
        } else {
            std::fs::read_to_string(&path).context(format!("Failed to read {}", path.display()))?
        };
        (text, Some(path))
    } else if stdin_is_piped() {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin().lock(), &mut buf)?;
        (buf, None)
    } else {
        anyhow::bail!("No input file specified. Provide a file path or pipe data via stdin.");
    };

    if content.trim().is_empty() {
        anyhow::bail!("Input file is empty");
    }

    // Detect delimiter
    let delimiter = cli_args
        .delimiter
        .unwrap_or_else(|| lazycsv::csv::detect_delimiter(&content));
    // Count columns from the first row
    let first_line = content.lines().next().unwrap_or("");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_reader(first_line.as_bytes());
    let col_count = rdr
        .records()
        .next()
        .and_then(|r| r.ok())
        .map(|r| r.len())
        .unwrap_or(0);

    if col_count == 0 {
        anyhow::bail!("Could not determine column count from input file");
    }

    // Build header row
    let header_values: Vec<String> = if header_spec.is_empty() {
        // Auto-generate C1, C2, ...
        (1..=col_count).map(|i| format!("C{}", i)).collect()
    } else {
        // Parse user-provided CSV header
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(header_spec.as_bytes());
        let values: Vec<String> = rdr
            .records()
            .next()
            .and_then(|r| r.ok())
            .map(|r| r.iter().map(String::from).collect())
            .unwrap_or_default();

        if values.len() != col_count {
            anyhow::bail!(
                "Header has {} values but the CSV file has {} columns",
                values.len(),
                col_count
            );
        }
        values
    };

    // Build the header line using the csv writer for proper escaping
    let mut hdr_buf = Vec::new();
    {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(&mut hdr_buf);
        wtr.write_record(&header_values)?;
        wtr.flush()?;
    }
    let header_line = String::from_utf8(hdr_buf)?;

    // Combine header + original content
    let output = format!("{}{}", header_line, content);

    // Write output
    let output_file = cli_args.output.as_deref().filter(|o| *o != "-");
    if let Some(out) = output_file {
        // Write to -o file
        let out_path = std::path::PathBuf::from(out);
        if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out_path, &output)?;
        eprintln!("Added header to {}", out_path.display());
    } else if let Some(ref path) = input_path {
        // Modify input file in place
        std::fs::write(path, &output)?;
        eprintln!("Added header to {}", path.display());
    } else {
        // stdin mode: write to stdout
        std::io::Write::write_all(&mut std::io::stdout().lock(), output.as_bytes())?;
    }

    Ok(())
}

/// Format a number with thousands separators. If `sep` is '\0', return plain number.
/// Get output writer based on -o flag: file, or stdout.
fn output_writer(cli_args: &cli::CliArgs) -> Result<Box<dyn std::io::Write>> {
    let output_file = cli_args.output.as_deref().filter(|o| *o != "-");
    if let Some(out) = output_file {
        Ok(Box::new(std::io::BufWriter::new(
            std::fs::File::create(out).context(format!("Failed to create output file: {}", out))?,
        )))
    } else {
        Ok(Box::new(std::io::stdout().lock()))
    }
}

/// Print message to stderr if -o file was specified.
fn report_output_file(cli_args: &cli::CliArgs, label: &str) {
    if let Some(out) = cli_args.output.as_deref().filter(|o| *o != "-") {
        eprintln!("{} written to {}", label, out);
    }
}

fn format_number(n: usize, sep: char) -> String {
    if sep == '\0' {
        return n.to_string();
    }
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(sep);
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

/// Detect the locale-appropriate thousands separator from environment variables.
/// Returns ',' for English-like locales, '.' for European locales (de, fr, es, it, pt, nl, etc.).
fn detect_thousands_separator() -> char {
    let locale = std::env::var("LC_NUMERIC")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_lowercase();

    // European locales that use '.' as thousands separator
    let dot_locales = [
        "de", "fr", "es", "it", "pt", "nl", "sv", "nb", "nn", "da", "fi", "pl", "cs", "sk", "hu",
        "ro", "bg", "hr", "sl", "sr", "tr", "el", "ru", "uk", "vi", "id",
    ];
    for prefix in &dot_locales {
        if locale.starts_with(prefix) {
            return '.';
        }
    }

    ','
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CliArgs;
    use clap::Parser;

    // ── resolve_sheet_spec ─────────────────────────────────────

    #[test]
    fn test_resolve_sheet_spec_by_name() {
        let sheets = vec!["Sales".to_string(), "Reports".to_string()];
        assert_eq!(resolve_sheet_spec("Sales", &sheets).unwrap(), "Sales");
        assert_eq!(resolve_sheet_spec("Reports", &sheets).unwrap(), "Reports");
    }

    #[test]
    fn test_resolve_sheet_spec_by_index() {
        let sheets = vec!["Sheet1".to_string(), "Sheet2".to_string()];
        assert_eq!(resolve_sheet_spec("1", &sheets).unwrap(), "Sheet1");
        assert_eq!(resolve_sheet_spec("2", &sheets).unwrap(), "Sheet2");
    }

    #[test]
    fn test_resolve_sheet_spec_index_out_of_range() {
        let sheets = vec!["Sheet1".to_string()];
        assert!(resolve_sheet_spec("0", &sheets).is_err());
        assert!(resolve_sheet_spec("2", &sheets).is_err());
    }

    #[test]
    fn test_resolve_sheet_spec_name_not_found() {
        let sheets = vec!["Sheet1".to_string()];
        assert!(resolve_sheet_spec("NoSuchSheet", &sheets).is_err());
    }

    // ── write_csv_rows ─────────────────────────────────────────

    #[test]
    fn test_write_csv_rows_simple() {
        let rows = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["1".to_string(), "2".to_string()],
        ];
        let mut buf = Vec::new();
        write_csv_rows(&mut buf, &rows).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "A,B\n1,2\n");
    }

    #[test]
    fn test_write_csv_rows_escapes_commas() {
        let rows = vec![vec!["hello, world".to_string()]];
        let mut buf = Vec::new();
        write_csv_rows(&mut buf, &rows).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\"hello, world\"\n");
    }

    #[test]
    fn test_write_csv_rows_escapes_quotes() {
        let rows = vec![vec!["say \"hi\"".to_string()]];
        let mut buf = Vec::new();
        write_csv_rows(&mut buf, &rows).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\"say \"\"hi\"\"\"\n");
    }

    #[test]
    fn test_write_csv_rows_escapes_newlines() {
        let rows = vec![vec!["line1\nline2".to_string()]];
        let mut buf = Vec::new();
        write_csv_rows(&mut buf, &rows).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\"line1\nline2\"\n");
    }

    #[test]
    fn test_write_csv_rows_empty() {
        let rows: Vec<Vec<String>> = vec![];
        let mut buf = Vec::new();
        write_csv_rows(&mut buf, &rows).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "");
    }

    // ── split_csv_streaming ────────────────────────────────────

    #[test]
    fn test_split_csv_streaming_basic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input = temp_dir.path().join("input.csv");
        std::fs::write(
            &input,
            "Name,Value\nAlice,1\nBob,2\nCharlie,3\nDave,4\nEve,5\n",
        )
        .unwrap();

        let out_dir = temp_dir.path().join("out");
        let cli = CliArgs::try_parse_from(["lazycsv"]).unwrap();
        split_csv_streaming(&input, 2, &out_dir, &cli).unwrap();

        // Should produce 3 files: 0.csv (2 rows), 1.csv (2 rows), 2.csv (1 row)
        let f0 = std::fs::read_to_string(out_dir.join("0.csv")).unwrap();
        let f1 = std::fs::read_to_string(out_dir.join("1.csv")).unwrap();
        let f2 = std::fs::read_to_string(out_dir.join("2.csv")).unwrap();

        // Each file should have the header
        assert!(f0.starts_with("Name,Value\n"));
        assert!(f1.starts_with("Name,Value\n"));
        assert!(f2.starts_with("Name,Value\n"));

        // Check row counts (header + data lines)
        assert_eq!(f0.lines().count(), 3); // header + 2 data
        assert_eq!(f1.lines().count(), 3); // header + 2 data
        assert_eq!(f2.lines().count(), 2); // header + 1 data
    }

    #[test]
    fn test_split_csv_streaming_single_chunk() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input = temp_dir.path().join("input.csv");
        std::fs::write(&input, "A\n1\n2\n").unwrap();

        let out_dir = temp_dir.path().join("out");
        let cli = CliArgs::try_parse_from(["lazycsv"]).unwrap();
        split_csv_streaming(&input, 100, &out_dir, &cli).unwrap();

        // All rows fit in one file
        let f0 = std::fs::read_to_string(out_dir.join("0.csv")).unwrap();
        assert_eq!(f0, "A\n1\n2\n");
        assert!(!out_dir.join("1.csv").exists());
    }

    #[test]
    fn test_split_csv_streaming_exact_boundary() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input = temp_dir.path().join("input.csv");
        std::fs::write(&input, "H\n1\n2\n3\n4\n").unwrap();

        let out_dir = temp_dir.path().join("out");
        let cli = CliArgs::try_parse_from(["lazycsv"]).unwrap();
        split_csv_streaming(&input, 2, &out_dir, &cli).unwrap();

        assert_eq!(
            std::fs::read_to_string(out_dir.join("0.csv"))
                .unwrap()
                .lines()
                .count(),
            3
        ); // header + 2
        assert_eq!(
            std::fs::read_to_string(out_dir.join("1.csv"))
                .unwrap()
                .lines()
                .count(),
            3
        ); // header + 2
        assert!(!out_dir.join("2.csv").exists());
    }

    #[test]
    fn test_split_csv_preserves_quoted_fields() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input = temp_dir.path().join("input.csv");
        std::fs::write(&input, "Name\n\"Last, First\"\n\"He said \"\"hi\"\"\"\n").unwrap();

        let out_dir = temp_dir.path().join("out");
        let cli = CliArgs::try_parse_from(["lazycsv"]).unwrap();
        split_csv_streaming(&input, 1, &out_dir, &cli).unwrap();

        let f0 = std::fs::read_to_string(out_dir.join("0.csv")).unwrap();
        let f1 = std::fs::read_to_string(out_dir.join("1.csv")).unwrap();

        assert!(f0.contains("\"Last, First\""));
        assert!(f1.contains("\"He said \"\"hi\"\"\""));
    }

    // ── execute_split (CSV path) ───────────────────────────────

    #[test]
    fn test_execute_split_csv() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input = temp_dir.path().join("data.csv");
        std::fs::write(&input, "A,B\n1,2\n3,4\n5,6\n").unwrap();

        let out_dir = temp_dir.path().join("split_out");
        let cli = CliArgs::try_parse_from([
            "lazycsv",
            input.to_str().unwrap(),
            "-S",
            "2",
            "-o",
            out_dir.to_str().unwrap(),
        ])
        .unwrap();
        execute_split(&input, 2, &cli).unwrap();

        let f0 = std::fs::read_to_string(out_dir.join("0.csv")).unwrap();
        let f1 = std::fs::read_to_string(out_dir.join("1.csv")).unwrap();

        assert!(f0.starts_with("A,B\n"));
        assert_eq!(f0.lines().count(), 3); // header + 2
        assert_eq!(f1.lines().count(), 2); // header + 1
        assert!(!out_dir.join("2.csv").exists());
    }

    #[test]
    fn test_execute_split_zero_rows_errors() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input = temp_dir.path().join("data.csv");
        std::fs::write(&input, "A\n1\n").unwrap();

        let cli = CliArgs::try_parse_from(["lazycsv"]).unwrap();
        let result = execute_split(&input, 0, &cli);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_split_default_output_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input = temp_dir.path().join("data.csv");
        std::fs::write(&input, "H\n1\n2\n").unwrap();

        // No -o flag, should output to input file's parent directory
        let cli = CliArgs::try_parse_from(["lazycsv"]).unwrap();
        execute_split(&input, 10, &cli).unwrap();

        // Output goes to same dir as input
        assert!(temp_dir.path().join("0.csv").exists());
    }

    // ── is_dml_query ───────────────────────────────────────────

    #[test]
    fn test_is_dml_query_insert() {
        assert!(is_dml_query("INSERT INTO t VALUES (1, 'a')"));
        assert!(is_dml_query("  insert into t values (1)"));
    }

    #[test]
    fn test_is_dml_query_update() {
        assert!(is_dml_query("UPDATE t SET a = 1"));
        assert!(is_dml_query("  UPDATE t SET a = 1 WHERE b = 2"));
    }

    #[test]
    fn test_is_dml_query_delete() {
        assert!(is_dml_query("DELETE FROM t WHERE a = 1"));
        assert!(is_dml_query("delete from t"));
    }

    #[test]
    fn test_is_dml_query_alter() {
        assert!(is_dml_query("ALTER TABLE t ADD COLUMN c TEXT"));
    }

    #[test]
    fn test_is_dml_query_create_drop() {
        assert!(is_dml_query("CREATE TABLE t (a TEXT)"));
        assert!(is_dml_query("DROP TABLE t"));
    }

    #[test]
    fn test_is_dml_query_select_is_not_dml() {
        assert!(!is_dml_query("SELECT * FROM t"));
        assert!(!is_dml_query("  select count(*) from t"));
    }

    #[test]
    fn test_is_dml_query_empty() {
        assert!(!is_dml_query(""));
        assert!(!is_dml_query("   "));
    }
}
