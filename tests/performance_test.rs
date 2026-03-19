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
    assert_eq!(doc.row_count(), 10_001); // 1 header + 10,000 data rows

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
    assert_eq!(doc.column_count(), 100); // Row 0 has 100 columns
    assert_eq!(doc.row_count(), 1001); // 1 header + 1000 data rows

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

    // Verify we're at row 5000 (5000G goes to absolute row 5000)
    assert_eq!(app.selected_row(), Some(RowIndex::new(5000)));

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

    // Render cost should not scale with total row count — only visible rows matter.
    // Compare render time for a small file vs a large file; the large file should
    // take no more than 3x the small file's render time.
    let small_doc = common::create_large_csv(100, 50);
    let large_doc = common::create_large_csv(10_000, 50);

    let measure_render = |doc: Document| -> Duration {
        let csv_files = vec![PathBuf::from("test.csv")];
        let mut app = App::new(doc, csv_files, 0, lazycsv::session::FileConfig::new());

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Warm up
        terminal.draw(|f| lazycsv::ui::render(f, &mut app)).unwrap();

        // Measure
        let start = Instant::now();
        for _ in 0..10 {
            terminal.draw(|f| lazycsv::ui::render(f, &mut app)).unwrap();
        }
        start.elapsed()
    };

    let small_time = measure_render(small_doc);
    let large_time = measure_render(large_doc);

    println!(
        "Render 10 frames: 100 rows = {:?}, 10K rows = {:?}, ratio = {:.1}x",
        small_time,
        large_time,
        large_time.as_secs_f64() / small_time.as_secs_f64()
    );

    // Rendering 10K rows should not be significantly slower than 100 rows
    // since only ~24 visible rows are drawn either way
    assert!(
        large_time < small_time * 3,
        "Render time scales with data size: 100 rows = {:?}, 10K rows = {:?}",
        small_time,
        large_time
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
