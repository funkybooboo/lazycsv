mod app;
mod csv_data;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::event::{self, Event, KeyEventKind};
use csv_data::CsvData;
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<()> {
    // Parse CLI args
    let args: Vec<String> = std::env::args().collect();
    let file_path = parse_args(&args)?;

    // Scan directory for other CSV files
    let csv_files = scan_directory_for_csvs(&file_path)?;
    let current_file_index = csv_files
        .iter()
        .position(|p| p == &file_path)
        .unwrap_or(0);

    // Load CSV data
    let csv_data = CsvData::from_file(&file_path)
        .context(format!("Failed to load CSV file: {}", file_path.display()))?;

    // Initialize terminal
    let mut terminal = ratatui::init();

    // Run app (wrapped to ensure cleanup)
    let result = run(&mut terminal, csv_data, csv_files, current_file_index);

    // Always restore terminal
    ratatui::restore();

    result
}

fn parse_args(args: &[String]) -> Result<PathBuf> {
    if args.len() < 2 {
        anyhow::bail!(
            "Usage: lazycsv <file.csv>\n\n\
             Example: lazycsv data.csv\n\n\
             LazyCSV is a fast, ergonomic TUI for CSV files.\n\
             Press ? in the app for keyboard shortcuts."
        );
    }

    let path = PathBuf::from(&args[1]);
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    if !path.is_file() {
        anyhow::bail!("Path is not a file: {}", path.display());
    }

    Ok(path)
}

/// Scan directory for other CSV files
fn scan_directory_for_csvs(file_path: &PathBuf) -> Result<Vec<PathBuf>> {
    let dir = file_path
        .parent()
        .context("Failed to get parent directory")?;

    let mut csv_files = Vec::new();

    // Read directory entries
    for entry in std::fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Check if it's a CSV file
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_str() == Some("csv") {
                    csv_files.push(path);
                }
            }
        }
    }

    // Sort alphabetically
    csv_files.sort();

    // If no CSV files found (shouldn't happen), at least include the current file
    if csv_files.is_empty() {
        csv_files.push(file_path.clone());
    }

    Ok(csv_files)
}

fn run(
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    csv_data: CsvData,
    csv_files: Vec<PathBuf>,
    current_file_index: usize,
) -> Result<()> {
    let mut app = App::new(csv_data, csv_files, current_file_index);

    loop {
        // Draw UI
        terminal
            .draw(|frame| ui::render(frame, &mut app))
            .context("Failed to render UI")?;

        // Poll for events (100ms timeout)
        if event::poll(Duration::from_millis(100)).context("Failed to poll for events")? {
            if let Event::Key(key) = event::read().context("Failed to read event")? {
                // Only process KeyPress events (ignore KeyRelease)
                if key.kind == KeyEventKind::Press {
                    // Handle key press - returns true if we need to reload file
                    let should_reload = app.handle_key(key)?;

                    if should_reload {
                        // Reload CSV data from new file
                        app.reload_current_file()
                            .context("Failed to reload CSV file")?;
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
