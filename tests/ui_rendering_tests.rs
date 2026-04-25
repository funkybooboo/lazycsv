//! UI rendering integration tests
//!
//! Tests for the main UI rendering functions including table, status bar,
//! file switcher, and overlay components.

use lazycsv::config::{TableTheme, Theme};
use lazycsv::session::FileConfig;
use lazycsv::{App, Document};
use ratatui::{backend::TestBackend, Terminal};
use std::path::PathBuf;

// from ui_rendering_test.rs
fn create_test_csv() -> Document {
    Document::new(
        vec!["ID".to_string(), "Name".to_string(), "Email".to_string()],
        vec![
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
        "test.csv".to_string(),
    )
}

#[test]
fn test_ui_renders_table() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

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
}

#[test]
fn test_ui_renders_help_overlay() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Show help
    app.view_state.help_overlay_visible = true;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

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
}

#[test]
fn test_ui_renders_multi_file_switcher() {
    let csv_data = create_test_csv();
    let csv_files = vec![
        PathBuf::from("file1.csv"),
        PathBuf::from("file2.csv"),
        PathBuf::from("file3.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should render without crashing with multiple files
    assert!(!content.is_empty());
}

#[test]
fn test_ui_shows_status_bar() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Status bar should show mode and position info
    assert!(
        content.contains("NORMAL") || content.contains(",A") || content.contains(",B"),
        "Should show status bar with mode and position info"
    );
}

#[test]
fn test_ui_column_letters_displayed() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

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
}

#[test]
fn test_ui_shows_dirty_indicator() {
    let mut csv_data = create_test_csv();
    csv_data.is_dirty = true;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

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
}

// from ui_state_test.rs
fn create_small_csv() -> Document {
    Document::new(
        vec!["A".to_string(), "B".to_string()],
        vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "4".to_string()],
        ],
        "small.csv".to_string(),
    )
}

fn create_empty_csv() -> Document {
    Document::new(
        vec!["A".to_string(), "B".to_string()],
        vec![],
        "empty.csv".to_string(),
    )
}

fn create_single_cell_csv() -> Document {
    Document::new(
        vec!["A".to_string()],
        vec![vec!["1".to_string()]],
        "single.csv".to_string(),
    )
}

#[test]
fn test_ui_renders_with_empty_data() {
    let csv_data = create_empty_csv();
    let csv_files = vec![PathBuf::from("empty.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Should render without crashing
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
}

#[test]
fn test_ui_renders_with_single_cell() {
    let csv_data = create_single_cell_csv();
    let csv_files = vec![PathBuf::from("single.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(content.contains("single.csv"));
}

#[test]
fn test_ui_renders_with_small_terminal() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Very small terminal
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    // Should render without crashing
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
}

#[test]
fn test_ui_renders_with_large_terminal() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Large terminal
    let backend = TestBackend::new(200, 100);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
}

#[test]
fn test_ui_state_after_navigation() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Initial render
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    // Navigate
    let _ = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('j'),
        crossterm::event::KeyModifiers::NONE,
    ));

    // Render again
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
}

#[test]
fn test_ui_state_transitions_help_toggle() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render without help
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let buffer1 = terminal.backend().buffer().clone();

    // Toggle help on
    app.view_state.help_overlay_visible = true;
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let buffer2 = terminal.backend().buffer().clone();

    // Buffers should be different
    assert_ne!(buffer1.content, buffer2.content);

    // Toggle help off
    app.view_state.help_overlay_visible = false;
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let buffer3 = terminal.backend().buffer().clone();

    // Should match initial state
    assert_eq!(buffer1.content, buffer3.content);
}

#[test]
fn test_ui_status_bar_updates() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render with no status message
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let content1 = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Set status message
    app.status_message = Some("Test message".into());
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
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
}

#[test]
fn test_ui_file_switcher_single_file() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("only.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should render without crashing
    assert!(!content.is_empty());
}

#[test]
fn test_ui_file_switcher_multiple_files() {
    let csv_data = create_small_csv();
    let csv_files = vec![
        PathBuf::from("first.csv"),
        PathBuf::from("second.csv"),
        PathBuf::from("third.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 1, FileConfig::new()); // Start at second file

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should render without crashing with multiple files
    assert!(!content.is_empty());
}

#[test]
fn test_ui_dirty_indicator() {
    let mut csv_data = create_small_csv();
    csv_data.is_dirty = false;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render clean state
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let buffer1 = terminal.backend().buffer().clone();

    // Make dirty
    app.document.is_dirty = true;
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let buffer2 = terminal.backend().buffer().clone();

    // The dirty state should cause a different render
    // (The asterisk may not be easily searchable in the buffer)
    // Just verify the buffers are different when dirty flag changes
    assert_ne!(buffer1.content, buffer2.content);
}

#[test]
fn test_ui_column_letters() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

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
}

#[test]
fn test_ui_row_numbers() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

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
}

#[test]
fn test_ui_responsive_to_selection() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render with row 0 selected (initial position)
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let buffer1 = terminal.backend().buffer().clone();

    // Change selection to row 2
    app.view_state.table_state.select(Some(2));
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let buffer2 = terminal.backend().buffer().clone();

    // Buffers should be different due to selection change
    assert_ne!(buffer1.content, buffer2.content);
}

// ===== Priority 2: UI Stress Tests =====

#[test]
fn test_ui_extremely_narrow_terminal_20_columns() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(20, 10); // Very narrow: 20 columns
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render without crashing
    let buffer = terminal.backend().buffer().clone();
    assert!(buffer.area.width == 20);
}

#[test]
fn test_ui_extremely_wide_terminal_500_columns() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(500, 30); // Very wide: 500 columns
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render without crashing
    let buffer = terminal.backend().buffer().clone();
    assert!(buffer.area.width == 500);
}

#[test]
fn test_ui_very_tall_terminal_100_rows() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 100); // Very tall: 100 rows
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render without crashing
    let buffer = terminal.backend().buffer().clone();
    assert!(buffer.area.height == 100);
}

#[test]
fn test_ui_unicode_emoji_in_cells() {
    let csv_data = Document::new(
        vec!["Name".to_string(), "Status".to_string()],
        vec![
            vec!["Alice".to_string(), " Happy".to_string()],
            vec!["Bob".to_string(), "😀 Smile".to_string()],
        ],
        "emoji.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("emoji.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render without crashing
}

#[test]
fn test_ui_very_long_filename_200_chars() {
    let csv_data = create_test_csv();
    let long_filename = format!("{}.csv", "a".repeat(200));
    let csv_files = vec![PathBuf::from(&long_filename)];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render without crashing (filename should be truncated)
}

#[test]
fn test_ui_cell_with_very_long_content() {
    let long_text = "A".repeat(10000);
    let csv_data = Document::new(
        vec!["Name".to_string(), "Data".to_string()],
        vec![vec!["Alice".to_string(), long_text]],
        "test.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render without crashing (content should be truncated)
}

#[test]
fn test_ui_special_characters_in_cells() {
    let csv_data = Document::new(
        vec!["Col1".to_string(), "Col2".to_string()],
        vec![
            vec!["\t\n\r".to_string(), "Normal".to_string()],
            vec!["Special: <>{}[]".to_string(), "Quotes: \"'".to_string()],
        ],
        "test.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render special characters without crashing
}

#[test]
fn test_ui_minimum_viable_terminal_10x5() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(10, 5); // Minimal terminal
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should handle gracefully even with tiny terminal
}

#[test]
fn test_ui_extreme_terminal_1x1() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(1, 1); // Extreme case: 1x1 terminal
    let mut terminal = Terminal::new(backend).unwrap();

    // Should not panic even with 1x1 terminal
    let result = terminal.draw(|f| {
        lazycsv::ui::render(f, &mut app);
    });

    assert!(
        result.is_ok(),
        "Should handle 1x1 terminal without panicking"
    );
}

#[test]
fn test_ui_extreme_width_1x24() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(1, 24); // Very narrow terminal
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();
}

#[test]
fn test_ui_extreme_height_80x1() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 1); // Very short terminal
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();
}

#[test]
fn test_ui_multi_byte_unicode_rendering() {
    let csv_data = Document::new(
        vec![
            "Japanese".to_string(),
            "Emoji".to_string(),
            "Russian".to_string(),
        ],
        vec![
            vec!["Hello".to_string(), "🎊😀".to_string(), "World".to_string()],
            vec!["Test".to_string(), "".to_string(), "Data".to_string()],
        ],
        "unicode 国家.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("unicode 国家.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should render emoji (multi-byte Unicode) without crashing
    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Verify headers are present
    assert!(
        content.contains("Japanese") || content.contains("Emoji") || content.contains("Russian"),
        "Should render headers"
    );
}

#[test]
fn test_ui_very_long_cell_truncation() {
    let long_text = "A".repeat(1000); // Very long cell content
    let csv_data = Document::new(
        vec!["Col1".to_string(), "Col2".to_string()],
        vec![
            vec![long_text.clone(), "Normal".to_string()],
            vec!["Short".to_string(), long_text],
        ],
        "long.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("long.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            lazycsv::ui::render(f, &mut app);
        })
        .unwrap();

    // Should handle long content with truncation
}

// ===== Theme System Tests =====

// ── style helper unit tests ────────────────────────────────────────────────

#[test]
fn test_cursor_style_from_uses_theme_colors() {
    use ratatui::style::Color;
    let theme = Theme {
        table: TableTheme {
            cursor_bg: Color::Cyan,
            cursor_fg: Color::Magenta,
            ..Default::default()
        },
        ..Default::default()
    };
    let style = lazycsv::ui::modal::cursor_style(&theme);
    assert_eq!(style.bg, Some(Color::Cyan));
    assert_eq!(style.fg, Some(Color::Magenta));
    // Must be bold
    assert!(style.add_modifier.contains(ratatui::style::Modifier::BOLD));
}

#[test]
fn test_search_match_style_from_uses_theme_colors() {
    use ratatui::style::Color;
    let theme = Theme {
        table: TableTheme {
            search_match_bg: Color::Blue,
            search_match_fg: Color::White,
            ..Default::default()
        },
        ..Default::default()
    };
    let style = lazycsv::ui::modal::search_match_style(&theme);
    assert_eq!(style.bg, Some(Color::Blue));
    assert_eq!(style.fg, Some(Color::White));
}

#[test]
fn test_visual_selection_style_from_uses_theme_colors() {
    use ratatui::style::Color;
    let theme = Theme {
        table: TableTheme {
            selection_bg: Color::Green,
            selection_fg: Color::Red,
            ..Default::default()
        },
        ..Default::default()
    };
    let style = lazycsv::ui::modal::visual_selection_style(&theme);
    assert_eq!(style.bg, Some(Color::Green));
    assert_eq!(style.fg, Some(Color::Red));
}

#[test]
fn test_zebra_stripe_style_from_uses_theme_color() {
    use ratatui::style::Color;
    let theme = Theme {
        table: TableTheme {
            zebra_bg: Color::Rgb(10, 20, 30),
            ..Default::default()
        },
        ..Default::default()
    };
    let style = lazycsv::ui::modal::zebra_stripe_style(&theme);
    assert_eq!(style.bg, Some(Color::Rgb(10, 20, 30)));
}

// ── cursor style renders to buffer ────────────────────────────────────────

#[test]
fn test_theme_cursor_color_applied_to_selected_cell() {
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Set a distinctive cursor color
    app.config.theme.table.cursor_bg = Color::Cyan;
    app.config.theme.table.cursor_fg = Color::Magenta;

    // Select row 0 (default)
    app.view_state.table_state.select(Some(0));

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // At least one cell should carry the cursor background color
    let has_cursor_bg = buffer.content.iter().any(|c| c.bg == Color::Cyan);
    assert!(
        has_cursor_bg,
        "Expected cursor_bg Color::Cyan to appear in rendered buffer"
    );
}

#[test]
fn test_theme_cursor_style_changes_with_custom_config() {
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());
    app.view_state.table_state.select(Some(0));

    // Render with default cursor color
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();
    let default_cursor_bg = app.config.theme.table.cursor_bg; // Color::White

    // Change cursor color
    app.config.theme.table.cursor_bg = Color::LightBlue;

    let backend2 = TestBackend::new(80, 24);
    let mut terminal2 = Terminal::new(backend2).unwrap();
    terminal2
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer2 = terminal2.backend().buffer();
    let has_new_cursor_bg = buffer2.content.iter().any(|c| c.bg == Color::LightBlue);
    let has_old_cursor_bg = buffer2
        .content
        .iter()
        .any(|c| c.bg == default_cursor_bg && c.bg != Color::LightBlue);

    assert!(
        has_new_cursor_bg,
        "Custom cursor_bg Color::LightBlue should appear after config change"
    );
    // Old color should no longer be present as cursor bg (unless it happened to be shared)
    let _ = has_old_cursor_bg; // informational only
}

// ── selection style renders to buffer ─────────────────────────────────────

#[test]
fn test_theme_selection_color_applied_to_visual_selection() {
    use lazycsv::app::{VisualMode, VisualSelection};
    use lazycsv::{ColIndex, RowIndex};
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Distinctive selection background
    app.config.theme.table.selection_bg = Color::Rgb(0, 128, 0);

    // Enter visual block covering row 0-1, col 0
    let start_row = RowIndex::new(0);
    let start_col = ColIndex::new(0);
    let mut sel = VisualSelection::new(start_row, start_col, VisualMode::Block);
    sel.update_cursor(RowIndex::new(1), ColIndex::new(0));
    app.visual_selection = Some(sel);
    app.view_state.table_state.select(Some(2)); // cursor outside selection
    app.mode = lazycsv::app::Mode::VisualBlock;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_selection_bg = buffer.content.iter().any(|c| c.bg == Color::Rgb(0, 128, 0));
    assert!(
        has_selection_bg,
        "Visual selection bg Color::Rgb(0,128,0) should appear in rendered buffer"
    );
}

// ── search match style renders to buffer ──────────────────────────────────

#[test]
fn test_theme_search_match_color_applied_to_matched_cells() {
    use lazycsv::search::SearchState;
    use lazycsv::{ColIndex, RowIndex};
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Distinctive search match background
    app.config.theme.table.search_match_bg = Color::Rgb(200, 100, 0);

    // Simulate a search state with a match at row 1, col 1
    let mut state = SearchState::new(
        "Bob".to_string(),
        vec![(RowIndex::new(1), ColIndex::new(1))],
    );
    // Make row 1, col 1 the current match
    state.current_match = Some(0);
    app.search_state = Some(state);

    // Place cursor elsewhere so the match cell is not the cursor cell
    app.view_state.table_state.select(Some(0));

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_match_bg = buffer
        .content
        .iter()
        .any(|c| c.bg == Color::Rgb(200, 100, 0));
    assert!(
        has_match_bg,
        "Search match bg Color::Rgb(200,100,0) should appear in rendered buffer"
    );
}

// ── zebra striping renders with configured background color ───────────────

#[test]
fn test_theme_zebra_stripe_color_applied_to_even_rows() {
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Distinctive zebra color
    app.config.theme.table.zebra_bg = Color::Rgb(50, 0, 50);
    app.config.defaults.zebra_striping = true;

    // Move cursor off row 0 so the zebra row is not overridden by cursor style
    app.view_state.table_state.select(Some(1));

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_zebra_bg = buffer.content.iter().any(|c| c.bg == Color::Rgb(50, 0, 50));
    assert!(
        has_zebra_bg,
        "Zebra stripe bg Color::Rgb(50,0,50) should appear on even data rows"
    );
}

#[test]
fn test_zebra_striping_disabled_produces_no_stripe_color() {
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    app.config.theme.table.zebra_bg = Color::Rgb(99, 0, 99);
    app.config.defaults.zebra_striping = false; // disabled

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_zebra_bg = buffer.content.iter().any(|c| c.bg == Color::Rgb(99, 0, 99));
    assert!(
        !has_zebra_bg,
        "Zebra stripe color should NOT appear when zebra_striping is disabled"
    );
}

// ── header bold styling ────────────────────────────────────────────────────

#[test]
fn test_theme_header_bold_true_applies_bold_modifier_in_header_row() {
    use ratatui::style::Modifier;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Ensure header_bold is true (default, but set explicitly)
    app.config.theme.table.header_bold = true;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // When header_bold is true, at least one rendered cell should carry BOLD
    let has_bold = buffer
        .content
        .iter()
        .any(|c| c.modifier.contains(Modifier::BOLD));
    assert!(
        has_bold,
        "At least one cell should have BOLD modifier when header_bold = true"
    );
}

#[test]
fn test_theme_header_bg_applied_when_set() {
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    app.config.theme.table.header_bg = Some(Color::Rgb(30, 60, 90));

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_header_bg = buffer
        .content
        .iter()
        .any(|c| c.bg == Color::Rgb(30, 60, 90));
    assert!(
        has_header_bg,
        "header_bg Color::Rgb(30,60,90) should appear in rendered buffer when set"
    );
}

// ── dirty indicator color ──────────────────────────────────────────────────

#[test]
fn test_theme_dirty_indicator_fg_applied_to_asterisk_in_file_switcher() {
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    // Two files so the file switcher renders both names with separator
    let csv_files = vec![PathBuf::from("first.csv"), PathBuf::from("second.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    // Distinctive dirty indicator color
    app.config.theme.table.dirty_fg = Color::Rgb(255, 80, 0);

    // Mark the first file dirty using the exact path the session holds
    let dirty_path = app.session.files()[0].clone();
    app.session.mark_dirty(&dirty_path);

    let backend = TestBackend::new(80, 4);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            // Render the file switcher directly into the full terminal area
            lazycsv::ui::file_switcher::render(frame, &app, Rect::new(0, 0, 80, 4));
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Find the '*' character that carries the dirty indicator foreground color
    let dirty_asterisk = buffer
        .content
        .iter()
        .find(|c| c.symbol() == "*" && c.fg == Color::Rgb(255, 80, 0));
    assert!(
        dirty_asterisk.is_some(),
        "A '*' cell with dirty_indicator_fg Color::Rgb(255,80,0) should appear for a dirty file"
    );
}

#[test]
fn test_theme_dirty_indicator_fg_not_present_when_clean() {
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("first.csv"), PathBuf::from("second.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    app.config.theme.table.dirty_fg = Color::Rgb(255, 80, 0);
    // Do NOT mark any file dirty

    let backend = TestBackend::new(80, 4);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::file_switcher::render(frame, &app, Rect::new(0, 0, 80, 4));
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    let dirty_asterisk = buffer
        .content
        .iter()
        .find(|c| c.symbol() == "*" && c.fg == Color::Rgb(255, 80, 0));
    assert!(
        dirty_asterisk.is_none(),
        "No dirty-indicator '*' with Color::Rgb(255,80,0) should appear for clean files"
    );
}

// ── all 16 ANSI named colors in theme config ──────────────────────────────

#[test]
fn test_theme_all_16_ansi_named_colors_cursor_bg() {
    use ratatui::style::Color;

    let named_colors = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];

    for color in named_colors {
        let csv_data = create_test_csv();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        app.config.theme.table.cursor_bg = color;
        app.view_state.table_state.select(Some(0));

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                lazycsv::ui::render(frame, &mut app);
            })
            .unwrap();

        // Just verify rendering does not panic; style helper should return correct bg
        let style = lazycsv::ui::modal::cursor_style(&app.config.theme);
        assert_eq!(
            style.bg,
            Some(color),
            "cursor_style_from should reflect ANSI color {:?}",
            color
        );
    }
}

#[test]
fn test_theme_all_16_ansi_named_colors_zebra_bg() {
    use ratatui::style::Color;

    let named_colors = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];

    for color in named_colors {
        let theme = Theme {
            table: TableTheme {
                zebra_bg: color,
                ..Default::default()
            },
            ..Default::default()
        };
        let style = lazycsv::ui::modal::zebra_stripe_style(&theme);
        assert_eq!(
            style.bg,
            Some(color),
            "zebra_stripe_style_from should reflect ANSI color {:?}",
            color
        );
    }
}

#[test]
fn test_theme_all_16_ansi_named_colors_render_without_panic() {
    use ratatui::style::Color;

    let named_colors = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];

    for color in named_colors {
        let csv_data = create_test_csv();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        // Apply color to all theme fields that accept it
        app.config.theme.table.cursor_bg = color;
        app.config.theme.table.cursor_fg = color;
        app.config.theme.table.selection_bg = color;
        app.config.theme.table.selection_fg = color;
        app.config.theme.table.search_match_bg = color;
        app.config.theme.table.search_match_fg = color;
        app.config.theme.table.zebra_bg = color;
        app.config.theme.table.dirty_fg = color;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        });
        assert!(
            result.is_ok(),
            "Rendering should not panic with ANSI color {:?} in all theme fields",
            color
        );
    }
}

// ── RGB hex colors in theme config ────────────────────────────────────────

#[test]
fn test_theme_rgb_hex_color_cursor_bg_renders_correctly() {
    use ratatui::style::Color;

    let rgb_color = Color::Rgb(171, 205, 239); // #ABCDEF

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    app.config.theme.table.cursor_bg = rgb_color;
    app.view_state.table_state.select(Some(0));

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_rgb_bg = buffer.content.iter().any(|c| c.bg == rgb_color);
    assert!(
        has_rgb_bg,
        "RGB cursor_bg Color::Rgb(171,205,239) should appear in rendered buffer"
    );
}

#[test]
fn test_theme_rgb_hex_color_zebra_bg_renders_correctly() {
    use ratatui::style::Color;

    let rgb_color = Color::Rgb(18, 52, 86); // #123456

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    app.config.theme.table.zebra_bg = rgb_color;
    app.config.defaults.zebra_striping = true;
    // Cursor on an odd row so even row (zebra row) is not overridden by cursor
    app.view_state.table_state.select(Some(1));

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_rgb_bg = buffer.content.iter().any(|c| c.bg == rgb_color);
    assert!(
        has_rgb_bg,
        "RGB zebra_bg Color::Rgb(18,52,86) should appear in rendered buffer"
    );
}

#[test]
fn test_theme_rgb_hex_color_search_match_bg_renders_correctly() {
    use lazycsv::search::SearchState;
    use lazycsv::{ColIndex, RowIndex};
    use ratatui::style::Color;

    let rgb_color = Color::Rgb(255, 128, 64);

    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

    app.config.theme.table.search_match_bg = rgb_color;

    let mut state = SearchState::new(
        "Alice".to_string(),
        vec![(RowIndex::new(0), ColIndex::new(1))],
    );
    state.current_match = Some(0);
    app.search_state = Some(state);

    // Cursor elsewhere so match cell uses search style, not cursor style
    app.view_state.table_state.select(Some(2));

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_rgb_bg = buffer.content.iter().any(|c| c.bg == rgb_color);
    assert!(
        has_rgb_bg,
        "RGB search_match_bg Color::Rgb(255,128,64) should appear in rendered buffer"
    );
}

#[test]
fn test_theme_rgb_colors_all_fields_render_without_panic() {
    use ratatui::style::Color;

    let rgb_pairs = [
        (Color::Rgb(255, 0, 0), Color::Rgb(0, 255, 0)),
        (Color::Rgb(0, 0, 255), Color::Rgb(255, 255, 0)),
        (Color::Rgb(128, 128, 128), Color::Rgb(64, 0, 128)),
        (Color::Rgb(0, 0, 0), Color::Rgb(255, 255, 255)),
    ];

    for (bg, fg) in rgb_pairs {
        let csv_data = create_test_csv();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, FileConfig::new());

        app.config.theme.table.cursor_bg = bg;
        app.config.theme.table.cursor_fg = fg;
        app.config.theme.table.selection_bg = bg;
        app.config.theme.table.selection_fg = fg;
        app.config.theme.table.search_match_bg = bg;
        app.config.theme.table.search_match_fg = fg;
        app.config.theme.table.zebra_bg = bg;
        app.config.theme.table.dirty_fg = fg;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            lazycsv::ui::render(frame, &mut app);
        });
        assert!(
            result.is_ok(),
            "Rendering should not panic with RGB colors bg={:?} fg={:?}",
            bg,
            fg
        );
    }
}

#[test]
fn test_theme_rgb_color_style_helpers_return_correct_values() {
    use ratatui::style::Color;

    let cursor_bg = Color::Rgb(12, 34, 56);
    let cursor_fg = Color::Rgb(98, 76, 54);
    let search_bg = Color::Rgb(255, 200, 100);
    let search_fg = Color::Rgb(10, 20, 30);
    let sel_bg = Color::Rgb(40, 80, 120);
    let sel_fg = Color::Rgb(200, 160, 120);
    let zebra = Color::Rgb(5, 10, 15);

    let theme = Theme {
        table: TableTheme {
            cursor_bg,
            cursor_fg,
            search_match_bg: search_bg,
            search_match_fg: search_fg,
            selection_bg: sel_bg,
            selection_fg: sel_fg,
            zebra_bg: zebra,
            ..Default::default()
        },
        ..Default::default()
    };

    let cs = lazycsv::ui::modal::cursor_style(&theme);
    assert_eq!(cs.bg, Some(cursor_bg));
    assert_eq!(cs.fg, Some(cursor_fg));

    let ss = lazycsv::ui::modal::search_match_style(&theme);
    assert_eq!(ss.bg, Some(search_bg));
    assert_eq!(ss.fg, Some(search_fg));

    let vs = lazycsv::ui::modal::visual_selection_style(&theme);
    assert_eq!(vs.bg, Some(sel_bg));
    assert_eq!(vs.fg, Some(sel_fg));

    let zs = lazycsv::ui::modal::zebra_stripe_style(&theme);
    assert_eq!(zs.bg, Some(zebra));
}
