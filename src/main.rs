use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use lazycsv::{cli, file_scanner, ui, App, CsvData};
use std::time::Duration;

fn main() -> Result<()> {
    // Parse CLI args (returns file or directory path)
    let args: Vec<String> = std::env::args().collect();
    let path = cli::parse_args(&args)?;

    // Determine the CSV file to load and scan directory for others
    let (file_path, csv_files, current_file_index) = if path.is_file() {
        // User provided a specific CSV file
        let csv_files = file_scanner::scan_directory_for_csvs(&path)?;
        let current_file_index = csv_files.iter().position(|p| p == &path).unwrap_or(0);
        (path, csv_files, current_file_index)
    } else if path.is_dir() {
        // User provided a directory - scan for CSV files
        let csv_files = file_scanner::scan_directory(&path)?;
        if csv_files.is_empty() {
            anyhow::bail!("No CSV files found in directory: {}", path.display());
        }
        // Load first CSV file alphabetically
        let file_path = csv_files[0].clone();
        (file_path, csv_files, 0)
    } else {
        anyhow::bail!("Invalid path: {}", path.display());
    };

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

fn run(
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    csv_data: CsvData,
    csv_files: Vec<std::path::PathBuf>,
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
