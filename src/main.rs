use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use lazycsv::{cli, ui, App, InputResult};
use std::time::Duration;

fn main() -> Result<()> {
    // Parse CLI args and create App
    let app = App::from_cli(cli::parse_args())?;

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
                            // Reload CSV data from new file
                            app.reload_current_file()
                                .context("Failed to reload CSV file")?;
                        }
                        InputResult::Quit => {
                            app.should_quit = true;
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
