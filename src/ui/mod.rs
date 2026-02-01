mod help;
mod status;
mod table;
pub mod utils;

pub const MAX_VISIBLE_COLS: usize = 10;
pub const MAX_CELL_WIDTH: usize = 20;

use crate::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Main UI rendering function
pub fn render(frame: &mut Frame, app: &mut App) {
    // Split terminal into main area + sheet switcher + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Table area
            Constraint::Length(3), // Sheet switcher
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    // Render table with row/column numbers
    table::render_table(frame, app, chunks[0]);

    // Render sheet switcher (always visible)
    status::render_sheet_switcher(frame, app, chunks[1]);

    // Render status bar
    status::render_status_bar(frame, app, chunks[2]);

    // Render help overlay if active
    if app.ui.show_cheatsheet {
        help::render_cheatsheet(frame);
    }
}

// Re-export public utilities
pub use utils::column_index_to_letter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{App, CsvData};
    use ratatui::{backend::TestBackend, Terminal};
    use std::io;
    use std::path::PathBuf;

    // from ui_rendering_test.rs
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
            render(frame, &mut app);
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
            render(frame, &mut app);
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
            render(frame, &mut app);
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
            render(frame, &mut app);
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
            render(frame, &mut app);
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
            render(frame, &mut app);
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

    // from ui_state_test.rs
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
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        // Should render without crashing
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;

        Ok(())
    }

    #[test]
    fn test_ui_renders_with_single_cell() -> io::Result<()> {
        let csv_data = create_single_cell_csv();
        let csv_files = vec![PathBuf::from("single.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        assert!(content.contains("single.csv"));

        Ok(())
    }

    #[test]
    fn test_ui_renders_with_small_terminal() -> io::Result<()> {
        let csv_data = create_small_csv();
        let csv_files = vec![PathBuf::from("small.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Very small terminal
        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend)?;

        // Should render without crashing
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;

        Ok(())
    }

    #[test]
    fn test_ui_renders_with_large_terminal() -> io::Result<()> {
        let csv_data = create_small_csv();
        let csv_files = vec![PathBuf::from("small.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        // Large terminal
        let backend = TestBackend::new(200, 100);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;

        Ok(())
    }

    #[test]
    fn test_ui_state_after_navigation() -> io::Result<()> {
        let csv_data = create_small_csv();
        let csv_files = vec![PathBuf::from("small.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        // Initial render
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;

        // Navigate
        let _ = app.handle_key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::NONE,
        ));

        // Render again
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;

        Ok(())
    }

    #[test]
    fn test_ui_state_transitions_help_toggle() -> io::Result<()> {
        let csv_data = create_small_csv();
        let csv_files = vec![PathBuf::from("small.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        // Render without help
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;
        let buffer1 = terminal.backend().buffer().clone();

        // Toggle help on
        app.ui.show_cheatsheet = true;
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;
        let buffer2 = terminal.backend().buffer().clone();

        // Buffers should be different
        assert_ne!(buffer1.content, buffer2.content);

        // Toggle help off
        app.ui.show_cheatsheet = false;
        terminal.draw(|frame| {
            render(frame, &mut app);
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
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        // Render with no status message
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;
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
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            render(frame, &mut app);
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
        let mut app = App::new(csv_data, csv_files, 1, None, false, None); // Start at second file

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            render(frame, &mut app);
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
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        // Render clean state
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;
        let buffer1 = terminal.backend().buffer().clone();

        // Make dirty
        app.csv_data.is_dirty = true;
        terminal.draw(|frame| {
            render(frame, &mut app);
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
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            render(frame, &mut app);
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
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            render(frame, &mut app);
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
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        // Render with row 0 selected
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;
        let buffer1 = terminal.backend().buffer().clone();

        // Change selection
        app.ui.table_state.select(Some(1));
        terminal.draw(|frame| {
            render(frame, &mut app);
        })?;
        let buffer2 = terminal.backend().buffer().clone();

        // Buffers should be different due to selection change
        assert_ne!(buffer1.content, buffer2.content);

        Ok(())
    }

    // ===== Priority 2: UI Stress Tests =====

    #[test]
    fn test_ui_extremely_narrow_terminal_20_columns() -> io::Result<()> {
        let csv_data = create_test_csv();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(20, 10); // Very narrow: 20 columns
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should render without crashing
        let buffer = terminal.backend().buffer().clone();
        assert!(buffer.area.width == 20);

        Ok(())
    }

    #[test]
    fn test_ui_extremely_wide_terminal_500_columns() -> io::Result<()> {
        let csv_data = create_test_csv();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(500, 30); // Very wide: 500 columns
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should render without crashing
        let buffer = terminal.backend().buffer().clone();
        assert!(buffer.area.width == 500);

        Ok(())
    }

    #[test]
    fn test_ui_very_tall_terminal_100_rows() -> io::Result<()> {
        let csv_data = create_test_csv();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 100); // Very tall: 100 rows
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should render without crashing
        let buffer = terminal.backend().buffer().clone();
        assert!(buffer.area.height == 100);

        Ok(())
    }

    #[test]
    fn test_ui_unicode_emoji_in_cells() -> io::Result<()> {
        let csv_data = CsvData {
            headers: vec!["Name".to_string(), "Status".to_string()],
            rows: vec![
                vec!["Alice".to_string(), "🎉 Happy".to_string()],
                vec!["Bob".to_string(), "😀 Smile".to_string()],
            ],
            filename: "emoji.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("emoji.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should render without crashing
        Ok(())
    }

    #[test]
    fn test_ui_very_long_filename_200_chars() -> io::Result<()> {
        let csv_data = create_test_csv();
        let long_filename = format!("{}.csv", "a".repeat(200));
        let csv_files = vec![PathBuf::from(&long_filename)];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should render without crashing (filename should be truncated)
        Ok(())
    }

    #[test]
    fn test_ui_cell_with_very_long_content() -> io::Result<()> {
        let long_text = "A".repeat(10000);
        let csv_data = CsvData {
            headers: vec!["Name".to_string(), "Data".to_string()],
            rows: vec![vec!["Alice".to_string(), long_text]],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should render without crashing (content should be truncated)
        Ok(())
    }

    #[test]
    fn test_ui_special_characters_in_cells() -> io::Result<()> {
        let csv_data = CsvData {
            headers: vec!["Col1".to_string(), "Col2".to_string()],
            rows: vec![
                vec!["\t\n\r".to_string(), "Normal".to_string()],
                vec!["Special: <>{}[]".to_string(), "Quotes: \"'".to_string()],
            ],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should render special characters without crashing
        Ok(())
    }

    #[test]
    fn test_ui_minimum_viable_terminal_10x5() -> io::Result<()> {
        let csv_data = create_test_csv();
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(csv_data, csv_files, 0, None, false, None);

        let backend = TestBackend::new(10, 5); // Minimal terminal
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            render(f, &mut app);
        })?;

        // Should handle gracefully even with tiny terminal
        Ok(())
    }
}
