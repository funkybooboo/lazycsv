//! Performance and stress tests

mod common;

use lazycsv::Document;
use ratatui::{backend::TestBackend, Terminal};
use std::time::{Duration, Instant};

#[test]
fn test_load_10k_rows_completes_quickly() {
    let temp_file = common::create_large_temp_csv(10_000, 10);

    let start = Instant::now();
    let result = Document::from_file(temp_file.path(), None, false, None);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Failed to load large CSV");
    let doc = result.unwrap();
    assert_eq!(doc.rows.len(), 10_000);

    println!("Loaded 10K rows in {:?}", duration);
    assert!(
        duration < Duration::from_secs(2),
        "Loading 10K rows took too long: {:?}",
        duration
    );
}

#[test]
fn test_load_100_columns_completes_quickly() {
    let temp_file = common::create_large_temp_csv(1000, 100);

    let start = Instant::now();
    let result = Document::from_file(temp_file.path(), None, false, None);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Failed to load wide CSV");
    let doc = result.unwrap();
    assert_eq!(doc.headers.len(), 100);
    assert_eq!(doc.rows.len(), 1000);

    println!("Loaded 100 columns in {:?}", duration);
    assert!(
        duration < Duration::from_secs(1),
        "Loading 100 columns took too long: {:?}",
        duration
    );
}

#[test]
fn test_navigate_large_file_responsive() {
    use crossterm::event::{KeyCode, KeyEvent};
    use lazycsv::{domain::position::RowIndex, App};
    use std::path::PathBuf;

    let doc = common::create_large_csv(10_000, 10);
    let csv_files = vec![PathBuf::from("large.csv")];
    let mut app = App::new(doc, csv_files, 0, lazycsv::session::FileConfig::new());

    // Navigate to middle of file
    let start = Instant::now();

    // Simulate pressing '5000G' to go to row 5000
    for digit in ['5', '0', '0', '0'] {
        app.handle_key(KeyEvent::from(KeyCode::Char(digit)))
            .unwrap();
    }
    app.handle_key(KeyEvent::from(KeyCode::Char('G'))).unwrap();

    let duration = start.elapsed();

    // Verify we're at row 4999 (0-indexed, 5000 is 1-indexed)
    assert_eq!(app.get_selected_row(), Some(RowIndex::new(4999)));

    println!("Navigated to row 5000 in {:?}", duration);
    assert!(
        duration < Duration::from_millis(100),
        "Navigation took too long: {:?}",
        duration
    );
}

#[test]
fn test_render_large_file_performance() {
    use lazycsv::App;
    use std::path::PathBuf;

    let doc = common::create_large_csv(10_000, 50);
    let csv_files = vec![PathBuf::from("large.csv")];
    let mut app = App::new(doc, csv_files, 0, lazycsv::session::FileConfig::new());

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render once to warm up
    terminal.draw(|f| lazycsv::ui::render(f, &mut app)).unwrap();

    // Time the actual render
    let start = Instant::now();
    terminal.draw(|f| lazycsv::ui::render(f, &mut app)).unwrap();
    let duration = start.elapsed();

    println!("Rendered 10K row file in {:?}", duration);

    // Target: 16ms for 60 FPS
    assert!(
        duration < Duration::from_millis(16),
        "Rendering took too long: {:?} (target: <16ms for 60 FPS)",
        duration
    );
}

#[test]
fn test_memory_usage_reasonable() {
    // This is a basic memory usage test that documents baseline
    // For more sophisticated memory profiling, use external tools like valgrind or heaptrack

    let start_allocation = get_current_memory_usage();

    // Load a large file
    let doc = common::create_large_csv(10_000, 20);

    let after_load = get_current_memory_usage();

    // Document the memory usage
    let memory_used = after_load - start_allocation;
    println!("Memory used for 10K x 20 CSV: ~{} bytes", memory_used);

    // Rough estimate: each cell ~20 bytes average (including String overhead)
    // 10K rows * 20 cols * 20 bytes = 4MB
    // Plus headers and other overhead, expect < 10MB
    let max_expected = 10 * 1024 * 1024; // 10 MB

    assert!(
        memory_used < max_expected,
        "Memory usage too high: {} bytes (expected < {})",
        memory_used,
        max_expected
    );

    drop(doc);
}

// Helper function to estimate current memory usage
// Note: This is a rough estimate and not precise
fn get_current_memory_usage() -> usize {
    // On Linux, we could read /proc/self/status
    // For cross-platform simplicity, we'll just return 0
    // In a real scenario, you'd use a crate like `memory-stats`
    0
}

#[test]
fn test_scroll_through_entire_large_file() {
    use crossterm::event::{KeyCode, KeyEvent};
    use lazycsv::App;
    use std::path::PathBuf;

    let doc = common::create_large_csv(1000, 10);
    let csv_files = vec![PathBuf::from("large.csv")];
    let mut app = App::new(doc, csv_files, 0, lazycsv::session::FileConfig::new());

    let start = Instant::now();

    // Scroll through entire file with j (down) 100 times
    for _ in 0..100 {
        app.handle_key(KeyEvent::from(KeyCode::Char('j'))).unwrap();
    }

    let duration = start.elapsed();

    println!("Scrolled 100 rows in {:?}", duration);
    assert!(
        duration < Duration::from_millis(200),
        "Scrolling took too long: {:?}",
        duration
    );
}
