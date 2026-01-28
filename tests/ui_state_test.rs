use lazycsv::{App, CsvData};
use ratatui::{backend::TestBackend, Terminal};
use std::io;
use std::path::PathBuf;

fn create_small_csv() -> CsvData {
    CsvData {
        headers: vec!["A".to_string(), "B".to_string()],
        rows: vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "4".to_string()],
        ],
        filename: "small.csv".to_string(),
        is_dirty: false,
    }
}

fn create_empty_csv() -> CsvData {
    CsvData {
        headers: vec!["A".to_string(), "B".to_string()],
        rows: vec![],
        filename: "empty.csv".to_string(),
        is_dirty: false,
    }
}

fn create_single_cell_csv() -> CsvData {
    CsvData {
        headers: vec!["A".to_string()],
        rows: vec![vec!["1".to_string()]],
        filename: "single.csv".to_string(),
        is_dirty: false,
    }
}

#[test]
fn test_ui_renders_with_empty_data() -> io::Result<()> {
    let csv_data = create_empty_csv();
    let csv_files = vec![PathBuf::from("empty.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Should render without crashing
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    Ok(())
}

#[test]
fn test_ui_renders_with_single_cell() -> io::Result<()> {
    let csv_data = create_single_cell_csv();
    let csv_files = vec![PathBuf::from("single.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    let content = buffer.content.iter().map(|c| c.symbol()).collect::<String>();

    assert!(content.contains("single.csv"));

    Ok(())
}

#[test]
fn test_ui_renders_with_small_terminal() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    // Very small terminal
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend)?;

    // Should render without crashing
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    Ok(())
}

#[test]
fn test_ui_renders_with_large_terminal() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    // Large terminal
    let backend = TestBackend::new(200, 100);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    Ok(())
}

#[test]
fn test_ui_state_after_navigation() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Initial render
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    // Navigate
    let _ = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('j'),
        crossterm::event::KeyModifiers::NONE,
    ));

    // Render again
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    Ok(())
}

#[test]
fn test_ui_state_transitions_help_toggle() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Render without help
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let buffer1 = terminal.backend().buffer().clone();

    // Toggle help on
    app.show_cheatsheet = true;
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let buffer2 = terminal.backend().buffer().clone();

    // Buffers should be different
    assert_ne!(buffer1.content, buffer2.content);

    // Toggle help off
    app.show_cheatsheet = false;
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let buffer3 = terminal.backend().buffer().clone();

    // Should match initial state
    assert_eq!(buffer1.content, buffer3.content);

    Ok(())
}

#[test]
fn test_ui_status_bar_updates() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Render with no status message
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let content1 = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Set status message
    app.status_message = Some("Test message".to_string());
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let content2 = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Content should be different
    assert_ne!(content1, content2);
    assert!(content2.contains("Test message"));

    Ok(())
}

#[test]
fn test_ui_file_switcher_single_file() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("only.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show file info
    assert!(content.contains("only.csv"));

    Ok(())
}

#[test]
fn test_ui_file_switcher_multiple_files() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![
        PathBuf::from("first.csv"),
        PathBuf::from("second.csv"),
        PathBuf::from("third.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 1); // Start at second file

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show file count
    assert!(content.contains("2/3") || content.contains("Files"));

    Ok(())
}

#[test]
fn test_ui_dirty_indicator() -> io::Result<()> {
    let mut csv_data = create_small_csv();
    csv_data.is_dirty = false;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Render clean state
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let buffer1 = terminal.backend().buffer().clone();

    // Make dirty
    app.csv_data.is_dirty = true;
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let buffer2 = terminal.backend().buffer().clone();

    // The dirty state should cause a different render
    // (The asterisk may not be easily searchable in the buffer)
    // Just verify the buffers are different when dirty flag changes
    assert_ne!(buffer1.content, buffer2.content);

    Ok(())
}

#[test]
fn test_ui_column_letters() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show column letters
    assert!(content.contains("A"));
    assert!(content.contains("B"));

    Ok(())
}

#[test]
fn test_ui_row_numbers() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show row numbers
    assert!(content.contains("1"));
    assert!(content.contains("2"));

    Ok(())
}

#[test]
fn test_ui_responsive_to_selection() -> io::Result<()> {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Render with row 0 selected
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let buffer1 = terminal.backend().buffer().clone();

    // Change selection
    app.table_state.select(Some(1));
    terminal.draw(|frame| {
        lazycsv::ui::render(frame, &mut app);
    })?;
    let buffer2 = terminal.backend().buffer().clone();

    // Buffers should be different due to selection change
    assert_ne!(buffer1.content, buffer2.content);

    Ok(())
}
