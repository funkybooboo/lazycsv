use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use lazycsv::{cli, ui, App, FileConfig, InputResult};
use std::path::PathBuf;
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

    // Interactive TUI mode
    let app = App::from_cli(cli_args)?;

    // Initialize terminal
    let mut terminal = ratatui::init();

    // Run app (wrapped to ensure cleanup)
    let result = run(&mut terminal, app);

    // Always restore terminal
    ratatui::restore();

    result
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
                            // Clear screen before loading new file to prevent stray characters
                            terminal.clear().context("Failed to clear terminal")?;
                            // Reload CSV data from new file
                            app.reload_current_file()
                                .context("Failed to reload CSV file")?;
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
                            let existing_idx = app
                                .session
                                .files()
                                .iter()
                                .position(|p| {
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

    Ok(())
}
