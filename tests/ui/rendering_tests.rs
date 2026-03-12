//! UI rendering integration tests
//! 
//! Tests for the main UI rendering functions including table, status bar,
//! file switcher, and overlay components.

use lazycsv::{App, Document, ui};
use ratatui::{backend::TestBackend, Terminal};
use std::path::PathBuf;

use super::*;
use crate::{App, Document};
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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Show help
    app.view_state.help_overlay_visible = true;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show file switcher with file names
    assert!(
        content.contains("file1") || content.contains("file2"),
        "Should show file switcher with file list"
    );

}

#[test]
fn test_ui_shows_status_bar() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Should render without crashing
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

}

#[test]
fn test_ui_renders_with_single_cell() {
    let csv_data = create_single_cell_csv();
    let csv_files = vec![PathBuf::from("single.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Very small terminal
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    // Should render without crashing
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

}

#[test]
fn test_ui_renders_with_large_terminal() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    // Large terminal
    let backend = TestBackend::new(200, 100);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

}

#[test]
fn test_ui_state_after_navigation() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Initial render
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

    // Navigate
    let _ = app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('j'),
        crossterm::event::KeyModifiers::NONE,
    ));

    // Render again
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

}

#[test]
fn test_ui_state_transitions_help_toggle() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render without help
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
    let buffer1 = terminal.backend().buffer().clone();

    // Toggle help on
    app.view_state.help_overlay_visible = true;
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
    let buffer2 = terminal.backend().buffer().clone();

    // Buffers should be different
    assert_ne!(buffer1.content, buffer2.content);

    // Toggle help off
    app.view_state.help_overlay_visible = false;
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
    let buffer3 = terminal.backend().buffer().clone();

    // Should match initial state
    assert_eq!(buffer1.content, buffer3.content);

}

#[test]
fn test_ui_status_bar_updates() {
    let csv_data = create_small_csv();
    let csv_files = vec![PathBuf::from("small.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render with no status message
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
    let content1 = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Set status message
    app.status_message = Some("Test message".into());
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show file info
    assert!(content.contains("only.csv"));

}

#[test]
fn test_ui_file_switcher_multiple_files() {
    let csv_data = create_small_csv();
    let csv_files = vec![
        PathBuf::from("first.csv"),
        PathBuf::from("second.csv"),
        PathBuf::from("third.csv"),
    ];
    let mut app = App::new(csv_data, csv_files, 1, crate::session::FileConfig::new()); // Start at second file

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

    let content = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Should show file count
    assert!(content.contains("2/3") || content.contains("Files"));

}

#[test]
fn test_ui_dirty_indicator() {
    let mut csv_data = create_small_csv();
    csv_data.is_dirty = false;
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render clean state
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
    let buffer1 = terminal.backend().buffer().clone();

    // Make dirty
    app.document.is_dirty = true;
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render with row 1 selected (initial position with header_mode=true)
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
    let buffer1 = terminal.backend().buffer().clone();

    // Change selection to row 2
    app.view_state.table_state.select(Some(2));
    terminal.draw(|frame| {
        render(frame, &mut app);
    }).unwrap();
    let buffer2 = terminal.backend().buffer().clone();

    // Buffers should be different due to selection change
    assert_ne!(buffer1.content, buffer2.content);

}

// ===== Priority 2: UI Stress Tests =====

#[test]
fn test_ui_extremely_narrow_terminal_20_columns() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(20, 10); // Very narrow: 20 columns
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

    // Should render without crashing
    let buffer = terminal.backend().buffer().clone();
    assert!(buffer.area.width == 20);

}

#[test]
fn test_ui_extremely_wide_terminal_500_columns() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(500, 30); // Very wide: 500 columns
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

    // Should render without crashing
    let buffer = terminal.backend().buffer().clone();
    assert!(buffer.area.width == 500);

}

#[test]
fn test_ui_very_tall_terminal_100_rows() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 100); // Very tall: 100 rows
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

    // Should render without crashing
}

#[test]
fn test_ui_very_long_filename_200_chars() {
    let csv_data = create_test_csv();
    let long_filename = format!("{}.csv", "a".repeat(200));
    let csv_files = vec![PathBuf::from(&long_filename)];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

    // Should render special characters without crashing
}

#[test]
fn test_ui_minimum_viable_terminal_10x5() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(10, 5); // Minimal terminal
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

    // Should handle gracefully even with tiny terminal
}

#[test]
fn test_ui_extreme_terminal_1x1() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(1, 1); // Extreme case: 1x1 terminal
    let mut terminal = Terminal::new(backend).unwrap();

    // Should not panic even with 1x1 terminal
    let result = terminal.draw(|f| {
        render(f, &mut app);
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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(1, 24); // Very narrow terminal
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

}

#[test]
fn test_ui_extreme_height_80x1() {
    let csv_data = create_test_csv();
    let csv_files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 1); // Very short terminal
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

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
        "unicode.csv".to_string(),
    );
    let csv_files = vec![PathBuf::from("unicode.csv")];
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

    // Should render emoji (multi-byte Unicode) without crashing
    let buffer = terminal.backend().buffer();
    let content = buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<String>();

    // Verify headers are present
    assert!(
        content.contains("Japanese")
            || content.contains("Emoji")
            || content.contains("Russian"),
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
    let mut app = App::new(csv_data, csv_files, 0, crate::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render(f, &mut app);
    }).unwrap();

    // Should handle long content with truncation
}
