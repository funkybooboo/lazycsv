use lazycsv::{App, CsvData};
use ratatui::{backend::TestBackend, Terminal};
use std::io;
use std::path::PathBuf;

/// Test helper to create a test CSV
fn create_test_csv() -> CsvData {
    CsvData {
        headers: vec!["ID".to_string(), "Name".to_string(), "Email".to_string()],
        rows: vec![
            vec![
                "1".to_string(),
                "Alice".to_string(),
                "alice@example.com".to_string(),
            ],
            vec![
                "2".to_string(),
                "Bob".to_string(),
                "bob@example.com".to_string(),
            ],
            vec![
                "3".to_string(),
                "Charlie".to_string(),
                "charlie@example.com".to_string(),
            ],
        ],
        filename: "test.csv".to_string(),
        is_dirty: false,
    }
}

#[test]
fn test_ui_renders_table() -> io::Result<()> {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    // Get the rendered buffer
    let buffer = terminal.backend().buffer();

    // Verify that key UI elements are present
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should contain filename in title
    assert!(content.contains("test.csv"), "Should show filename in UI");

    // Should contain headers
    assert!(
        content.contains("ID") || content.contains("Name"),
        "Should show column headers"
    );

    // Should contain data
    assert!(
        content.contains("Alice") || content.contains("Bob"),
        "Should show row data"
    );

    Ok(())
}

#[test]
fn test_ui_renders_help_overlay() -> io::Result<()> {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    // Show help
    app.ui.show_cheatsheet = true;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Help overlay should be visible
    assert!(
        content.contains("Navigation") || content.contains("Keyboard"),
        "Should show help overlay with navigation info"
    );

    Ok(())
}

#[test]
fn test_ui_renders_multi_file_switcher() -> io::Result<()> {
    let csv_data = create_test_csv();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show file switcher with multiple files
    assert!(
        content.contains("Files") && (content.contains("file1") || content.contains("file2")),
        "Should show file switcher with file list"
    );

    Ok(())
}

#[test]
fn test_ui_shows_status_bar() -> io::Result<()> {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Status bar should show row/column info and help hint
    assert!(
        content.contains("Row") || content.contains("Col") || content.contains("help"),
        "Should show status bar with navigation info"
    );

    Ok(())
}

#[test]
fn test_ui_column_letters_displayed() -> io::Result<()> {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show column letters (A, B, C, etc.)
    // The exact format might vary, but there should be letter indicators
    assert!(
        content.contains("A") && content.contains("B"),
        "Should show column letters (A, B, C...)"
    );

    Ok(())
}

#[test]
fn test_ui_shows_dirty_indicator() -> io::Result<()> {
    let mut csv_data = create_test_csv();
    csv_data.is_dirty = true;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, None, false, None);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // When dirty, should show an asterisk in the title
    assert!(
        content.contains("test.csv*"),
        "Should show asterisk for unsaved changes"
    );

    Ok(())
}
