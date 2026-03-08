use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use lazycsv::{cli, ui, App, FileConfig, InputResult};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    let cli_args = cli::parse_args();

    // Non-interactive query mode: execute SQL and exit
    if let Some(ref query) = cli_args.query {
        let path = cli_args.path.clone().unwrap_or_else(|| PathBuf::from("."));
        let config = FileConfig::with_options(
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        );
        return lazycsv::query::execute_query(&path, query, &config);
    }

    // Non-interactive row/column count mode: print counts and exit
    if cli_args.rows || cli_args.columns {
        let path = cli_args.path.clone().unwrap_or_else(|| PathBuf::from("."));
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

        let separator = if cli_args.format {
            detect_thousands_separator()
        } else {
            '\0' // sentinel: no formatting
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
        return Ok(());
    }

    // Interactive TUI mode: resolve files first, then show loading screen
    let (file_path, csv_files, index, config) = App::resolve_files(&cli_args)?;

    // Initialize terminal before loading so we can show feedback
    let mut terminal = ratatui::init();

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
    let app_result =
        App::load_file_cancellable(&file_path, csv_files, index, config, &cli_args, &cancelled);
    watcher.stop();

    let app = match app_result {
        Ok(app) => app,
        Err(e) => {
            // If cancelled, exit cleanly
            if e.downcast_ref::<lazycsv::cancel::CancelledError>()
                .is_some()
            {
                ratatui::restore();
                return Ok(());
            }
            ratatui::restore();
            return Err(e);
        }
    };

    // Run app (wrapped to ensure cleanup)
    let result = run(&mut terminal, app);

    // Always restore terminal
    ratatui::restore();

    // Exit immediately to avoid slow destructor cleanup for large documents.
    // The OS reclaims all memory when the process exits.
    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => Err(e),
    }
}

fn run(
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    mut app: App,
) -> Result<()> {
    let mut needs_redraw = true;
    let mut last_mtime_check = Instant::now();

    loop {
        if needs_redraw {
            terminal
                .draw(|frame| ui::render(frame, &mut app))
                .context("Failed to render UI")?;
            needs_redraw = false;
        }

        if event::poll(Duration::from_millis(100)).context("Failed to poll for events")? {
            if let Event::Key(key) = event::read().context("Failed to read event")? {
                if key.kind == KeyEventKind::Press {
                    let result = app.handle_key(key)?;
                    needs_redraw = true;
                    handle_input_result(terminal, &mut app, result)?;
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

    std::mem::forget(app);
    Ok(())
}

/// Dispatch input results to appropriate handlers
fn handle_input_result(
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    app: &mut App,
    result: InputResult,
) -> Result<()> {
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
fn handle_reload_file(
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    app: &mut App,
) -> Result<()> {
    app.external_modification_pending = false;
    app.search_state = None;

    let filename = app
        .get_current_file()
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
            let current_path = app.get_current_file().clone();
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
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    app: &mut App,
    doc: lazycsv::csv::Document,
) -> Result<()> {
    terminal.clear().context("Failed to clear terminal")?;

    if app.document.is_dirty {
        let current_path = app.get_current_file().clone();
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

    let current_path = app.get_current_file().clone();
    app.session.mark_query_output(&current_path);

    let old_rows = std::mem::take(&mut app.document.rows);
    std::thread::spawn(move || drop(old_rows));
    app.document = doc;

    app.view_state = lazycsv::ui::ViewState::default();
    let initial_row = if app.document.header_mode && app.document.row_count() > 1 {
        1
    } else {
        0
    };
    app.view_state.table_state.select(Some(initial_row));
    Ok(())
}

/// Handle document sorting
fn handle_sort_document(
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    app: &mut App,
    col_indices: Vec<usize>,
    ascending: bool,
    description: String,
) -> Result<()> {
    let direction = if ascending { "ascending" } else { "descending" };
    app.status_message = Some(lazycsv::input::StatusMessage::new_persistent(format!(
        "Sorting by {} {}...",
        description, direction
    )));
    terminal
        .draw(|frame| ui::render(frame, app))
        .context("Failed to render UI")?;

    app.document.sort_by_columns(&col_indices, ascending);
    let current_file = app.get_current_file().clone();
    app.session.mark_dirty(&current_file);
    app.status_message = Some(lazycsv::input::StatusMessage::from(format!(
        "Sorted by {} {}",
        description, direction
    )));
    Ok(())
}

/// Handle SQL query execution with cancellation support
fn handle_execute_query(
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    app: &mut App,
    query: String,
) -> Result<()> {
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
    let (query_result, was_cancelled) =
        app.execute_sql_query_cancellable(&query, &output_name, &cancelled);
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
