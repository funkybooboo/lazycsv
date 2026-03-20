use anyhow::{Context, Result};
use crossterm::event::{
    self, Event, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
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
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_main() -> Result<()> {
    let cli_args = cli::parse_args();

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
        std::fs::write(&out_path, &content)?;
        let lines = content.lines().count().saturating_sub(1);
        eprintln!("Pasted {} rows to {}", lines, out_path.display());
        return Ok(());
    }

    // Non-interactive split mode
    if let Some(rows_per_file) = cli_args.split {
        let path = cli_args
            .file_path()
            .context("Usage: lazycsv <file> -S <rows>")?;
        return execute_split(&path, rows_per_file, &cli_args);
    }

    // Non-interactive sort mode: load, sort, output CSV to stdout, and exit
    if let Some(ref sort_spec) = cli_args.sort {
        return execute_sort_and_output(sort_spec, &cli_args);
    }

    // Non-interactive row/column count mode: print counts and exit
    if cli_args.rows || cli_args.columns {
        return execute_count_mode(&cli_args);
    }

    // Piped stdin can't be used with the interactive TUI (stdin is needed for keyboard input)
    if cli_args.file_path().is_none() && stdin_is_piped() {
        let _ = crossterm::terminal::disable_raw_mode();
        eprintln!("Piped stdin is not supported in interactive TUI mode.");
        eprintln!("Use a non-interactive flag: -q <query>, --sort <col>, --rows, or --columns.");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  cat data.csv | lazycsv -q \"SELECT * FROM stdin\"");
        eprintln!("  cat data.csv | lazycsv --sort Salary");
        eprintln!("  cat data.csv | lazycsv --rows");
        std::process::exit(1);
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

    // For xlsx files, resolve sheet from CLI arg or prompt user
    let sheet_name = if lazycsv::csv::xlsx::is_spreadsheet(&file_path) {
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
        Ok(app) => app,
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

    // Run app (wrapped to ensure cleanup)
    let result = run(&mut terminal, &mut app);

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
    if supports_enhancement {
        crossterm::execute!(std::io::stdout(), PopKeyboardEnhancementFlags).ok();
    }
    ratatui::restore();
    // Ensure cursor is visible after leaving the TUI
    crossterm::execute!(std::io::stdout(), crossterm::cursor::Show).ok();
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
            if let Event::Key(key) = event::read().context("Failed to read event")? {
                if key.kind == KeyEventKind::Press {
                    let result = app.handle_key(key)?;
                    needs_redraw = true;
                    handle_input_result(terminal, app, result)?;
                }
            }
        }

        if last_mtime_check.elapsed() >= Duration::from_secs(2) {
            last_mtime_check = Instant::now();
            if app.check_current_file_modification() {
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
        InputResult::Continue => {}
    }
    Ok(())
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
            app.invalidate_sqlite_cache_for(&current_path);
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

/// Handle switching to a different document (from query results or file switch)
fn handle_switch_document(
    terminal: &mut Term,
    app: &mut App,
    doc: lazycsv::csv::Document,
) -> Result<()> {
    terminal.clear().context("Failed to clear terminal")?;

    if app.document.is_dirty {
        let current_path = app.current_file().clone();
        app.session.mark_dirty(&current_path);
        app.session
            .cache_document(current_path, app.document.clone());
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
    let completed = app
        .document
        .sort_by_columns(&col_indices, ascending, &cancelled);
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
fn handle_execute_query(terminal: &mut Term, app: &mut App, query: String) -> Result<()> {
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
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    {
        Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to run pbcopy. Is it available?")
    }

    #[cfg(target_os = "linux")]
    {
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
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    let output = Command::new("pbpaste")
        .stdout(Stdio::piped())
        .output()
        .context("Failed to run pbpaste. Is it available?")?;

    #[cfg(target_os = "linux")]
    let output = Command::new("xclip")
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
        .context("No clipboard tool found. Install xclip, xsel, or wl-paste.")?;

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("Clipboard not supported on this platform");

    if !output.status.success() {
        anyhow::bail!("Clipboard read failed");
    }

    String::from_utf8(output.stdout).context("Clipboard contains invalid UTF-8")
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

    if lazycsv::csv::xlsx::is_spreadsheet(path) {
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

    if lazycsv::csv::xlsx::is_spreadsheet(path) {
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

    // Run the query — always print to stdout, optionally also copy to clipboard
    let result = lazycsv::query::execute_query(&query_path, query, &config);

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

/// Non-interactive row/column count mode with stdin support.
fn execute_count_mode(cli_args: &cli::CliArgs) -> Result<()> {
    let separator = if cli_args.format {
        detect_thousands_separator()
    } else {
        '\0'
    };

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
        let mut parts = Vec::new();
        if cli_args.rows {
            let count = if doc.row_count() > 0 {
                doc.row_count() - 1 // subtract row 0 (which typically contains column names)
            } else {
                0
            };
            parts.push(format!("{} rows", format_number(count, separator)));
        }
        if cli_args.columns {
            parts.push(format!(
                "{} columns",
                format_number(doc.column_count(), separator)
            ));
        }
        println!("stdin: {}", parts.join(", "));
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

        let mut parts = Vec::new();

        if cli_args.rows {
            let count = lazycsv::csv::Document::count_rows(
                file,
                cli_args.delimiter,
                cli_args.no_headers,
                cli_args.encoding.clone(),
            )?;
            parts.push(format!("{} rows", format_number(count, separator)));
        }

        if cli_args.columns {
            let count = lazycsv::csv::Document::count_columns(
                file,
                cli_args.delimiter,
                cli_args.encoding.clone(),
            )?;
            parts.push(format!("{} columns", format_number(count, separator)));
        }

        println!("{}: {}", name, parts.join(", "));
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

    // Write sorted CSV to stdout
    let delimiter = doc.delimiter;
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    lazycsv::csv::write_csv_content(&mut out, &doc, delimiter)?;

    Ok(())
}

/// Format a number with thousands separators. If `sep` is '\0', return plain number.
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
}
