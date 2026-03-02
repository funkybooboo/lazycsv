use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use lazycsv::{cli, ui, App, FileConfig, InputResult};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

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
    let app_result = App::load_file_cancellable(
        &file_path,
        csv_files,
        index,
        config,
        &cli_args,
        &cancelled,
    );
    watcher.stop();

    let app = match app_result {
        Ok(app) => app,
        Err(e) => {
            // If cancelled, exit cleanly
            if e.downcast_ref::<lazycsv::cancel::CancelledError>().is_some() {
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
    // Event-driven rendering: only redraw when state changes
    let mut needs_redraw = true;

    loop {
        // Only render if state has changed
        if needs_redraw {
            terminal
                .draw(|frame| ui::render(frame, &mut app))
                .context("Failed to render UI")?;
            needs_redraw = false;
        }

        // Poll for events (100ms timeout)
        if event::poll(Duration::from_millis(100)).context("Failed to poll for events")? {
            if let Event::Key(key) = event::read().context("Failed to read event")? {
                // Only process KeyPress events (ignore KeyRelease)
                if key.kind == KeyEventKind::Press {
                    // Handle key press
                    let result = app.handle_key(key)?;

                    // State changed, need to redraw
                    needs_redraw = true;

                    match result {
                        InputResult::ReloadFile => {
                            // Show loading feedback before blocking file load
                            let filename = app
                                .get_current_file()
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("file")
                                .to_string();
                            app.status_message =
                                Some(lazycsv::input::StatusMessage::new_persistent(format!(
                                    "Loading {}... (Esc to cancel)",
                                    filename
                                )));
                            terminal
                                .draw(|frame| ui::render(frame, &mut app))
                                .context("Failed to render UI")?;

                            // Reload with cancellation — background thread watches for Esc
                            let cancelled = Arc::new(AtomicBool::new(false));
                            let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
                            let reload_result =
                                app.reload_current_file_cancellable(&cancelled);
                            watcher.stop();

                            match reload_result {
                                Ok(true) => {
                                    // Successfully loaded — invalidate SQLite cache for this file
                                    let current_path = app.get_current_file().clone();
                                    app.invalidate_sqlite_cache_for(&current_path);
                                    app.status_message = None;
                                    terminal
                                        .clear()
                                        .context("Failed to clear terminal")?;
                                }
                                Ok(false) => {
                                    // Cancelled — keep existing document
                                    app.status_message =
                                        Some(lazycsv::input::StatusMessage::from(
                                            "Load cancelled".to_string(),
                                        ));
                                }
                                Err(e) => {
                                    return Err(e)
                                        .context("Failed to reload CSV file");
                                }
                            }
                        }
                        InputResult::Quit => {
                            app.should_quit = true;
                        }
                        InputResult::SwitchToDocument(doc) => {
                            terminal.clear().context("Failed to clear terminal")?;

                            // Cache current document if dirty
                            if app.document.is_dirty {
                                let current_path = app.get_current_file().clone();
                                app.session.mark_dirty(&current_path);
                                app.session
                                    .cache_document(current_path, app.document.clone());
                            }

                            // Check if doc.filename matches an existing session file
                            let doc_filename = doc.filename.clone();
                            let existing_idx = app.session.files().iter().position(|p| {
                                p.file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|s| s == doc_filename)
                                    .unwrap_or(false)
                            });

                            if let Some(idx) = existing_idx {
                                // Replace at that index
                                app.session.set_active_file_index(idx);
                            } else {
                                // Add as new file
                                let path = std::path::PathBuf::from(&doc_filename);
                                let idx = app.session.add_file(path);
                                app.session.set_active_file_index(idx);
                            }

                            // Mark as query output so re-running SQL replaces this sheet
                            let current_path = app.get_current_file().clone();
                            app.session.mark_query_output(&current_path);

                            // Drop old document rows on a background thread
                            let old_rows = std::mem::take(&mut app.document.rows);
                            std::thread::spawn(move || drop(old_rows));
                            app.document = doc;

                            // Reset view state
                            app.view_state = lazycsv::ui::ViewState::default();
                            let initial_row =
                                if app.document.header_mode && app.document.row_count() > 1 {
                                    1
                                } else {
                                    0
                                };
                            app.view_state.table_state.select(Some(initial_row));
                        }
                        InputResult::SortDocument {
                            col_indices,
                            ascending,
                            description,
                        } => {
                            let direction = if ascending { "ascending" } else { "descending" };
                            app.status_message =
                                Some(lazycsv::input::StatusMessage::new_persistent(format!(
                                    "Sorting by {} {}...",
                                    description, direction
                                )));
                            terminal
                                .draw(|frame| ui::render(frame, &mut app))
                                .context("Failed to render UI")?;
                            app.document.sort_by_columns(&col_indices, ascending);
                            let current_file = app.get_current_file().clone();
                            app.session.mark_dirty(&current_file);
                            app.status_message = Some(lazycsv::input::StatusMessage::from(
                                format!("Sorted by {} {}", description, direction),
                            ));
                        }
                        InputResult::ExecuteQuery { query } => {
                            // Determine the output filename upfront
                            let output_name = app
                                .session
                                .find_query_output_file()
                                .and_then(|p| {
                                    p.file_name()
                                        .and_then(|n| n.to_str())
                                        .map(|s| s.to_string())
                                })
                                .unwrap_or_else(|| app.generate_output_filename());

                            // Create the output sheet tab so it's visible during query
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

                            // Dismiss SQL overlay and show centered "Executing query..."
                            app.mode = lazycsv::app::Mode::Normal;
                            terminal
                                .draw(|frame| {
                                    ui::render_loading(
                                        frame,
                                        "Executing query... (Esc to cancel)",
                                    );
                                })
                                .context("Failed to render UI")?;

                            // Execute with cancellation — background thread watches for Esc
                            let cancelled = Arc::new(AtomicBool::new(false));
                            let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
                            let (query_result, was_cancelled) =
                                app.execute_sql_query_cancellable(&query, &cancelled);
                            watcher.stop();

                            if was_cancelled {
                                // Remove newly-added output tab if we created one
                                if newly_added {
                                    let path = std::path::PathBuf::from(&output_name);
                                    app.session.remove_file(&path);
                                }
                                // Restore SQL editor so user can try again
                                app.mode = lazycsv::app::Mode::SqlEditor;
                                app.status_message =
                                    Some(lazycsv::input::StatusMessage::from(
                                        "Query cancelled".to_string(),
                                    ));
                            } else if let Some(doc) = query_result {
                                // Switch to query result document
                                terminal.clear().context("Failed to clear terminal")?;

                                if app.document.is_dirty {
                                    let current_path = app.get_current_file().clone();
                                    app.session.mark_dirty(&current_path);
                                    app.session
                                        .cache_document(current_path, app.document.clone());
                                }

                                let doc_filename = doc.filename.clone();
                                let target_idx = app
                                    .session
                                    .files()
                                    .iter()
                                    .position(|p| {
                                        p.file_name()
                                            .and_then(|n| n.to_str())
                                            .map(|s| s == doc_filename)
                                            .unwrap_or(false)
                                    })
                                    .unwrap_or_else(|| {
                                        let path = std::path::PathBuf::from(&doc_filename);
                                        app.session.add_file(path)
                                    });
                                app.session.set_active_file_index(target_idx);

                                let current_path = app.get_current_file().clone();
                                app.session.mark_query_output(&current_path);

                                // Drop old document rows on a background thread to avoid
                                // blocking the UI for large documents.
                                let old_rows = std::mem::take(&mut app.document.rows);
                                std::thread::spawn(move || drop(old_rows));
                                app.document = doc;

                                app.view_state = lazycsv::ui::ViewState::default();
                                let initial_row =
                                    if app.document.header_mode && app.document.row_count() > 1 {
                                        1
                                    } else {
                                        0
                                    };
                                app.view_state.table_state.select(Some(initial_row));
                            } else {
                                // Query failed — restore SqlEditor so user can fix the query
                                // Remove newly-added output tab if we created one
                                if newly_added {
                                    let path = std::path::PathBuf::from(&output_name);
                                    app.session.remove_file(&path);
                                }
                                app.mode = lazycsv::app::Mode::SqlEditor;
                                app.status_message = None;
                            }
                        }
                        InputResult::Continue => {
                            // Normal operation, continue
                        }
                    }
                }
            }
        }

        // Check exit condition
        if app.should_quit {
            break;
        }
    }

    // Skip slow per-element destructors for large documents.
    // The OS reclaims all memory on process exit.
    std::mem::forget(app);

    Ok(())
}
