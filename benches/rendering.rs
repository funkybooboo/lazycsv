//! Benchmark rendering pipeline performance
//!
//! Tests performance of rendering-related operations:
//! - Full frame rendering at different dataset sizes
//! - Viewport calculations
//! - Cell styling and selection checks
//!
//! Target: <16.67ms per frame (60 FPS) for 100K rows

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lazycsv::csv::Document;
use lazycsv::session::FileConfig;
use lazycsv::ui;
use lazycsv::App;
use ratatui::{backend::TestBackend, Terminal};
use std::path::PathBuf;

/// Create test app with specified row/column count
fn create_test_app(rows: usize, cols: usize) -> App {
    let headers: Vec<String> = (0..cols).map(|i| format!("Col{}", i)).collect();
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|r| (0..cols).map(|c| format!("R{}C{}", r, c)).collect())
        .collect();

    let document = Document::new(headers, data_rows, "benchmark.csv".to_string());
    let csv_files = vec![PathBuf::from("benchmark.csv")];
    App::new(document, csv_files, 0, FileConfig::new())
}

/// Benchmark full frame rendering at different dataset sizes
fn bench_full_frame_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering/full_frame");

    // Critical test: 100K rows must render <16.67ms (60 FPS)
    for &size in &[1_000, 10_000, 100_000] {
        let mut app = create_test_app(size, 10);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let backend = TestBackend::new(120, 40);
                let mut terminal = Terminal::new(backend).unwrap();
                terminal
                    .draw(|f| {
                        ui::render(f, black_box(&mut app));
                    })
                    .unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark viewport scrolling performance
fn bench_viewport_scrolling(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering/viewport_scroll");

    for &size in &[1_000, 10_000, 100_000] {
        let mut app = create_test_app(size, 10);
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        group.bench_with_input(
            BenchmarkId::new("scroll_and_render", size),
            &size,
            |b, _| {
                b.iter(|| {
                    // Simulate scrolling through the document
                    app.view_state.table_state.select(Some(black_box(size / 2)));
                    terminal
                        .draw(|f| {
                            ui::render(f, black_box(&mut app));
                        })
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark get_cell performance (critical for rendering)
fn bench_get_cell(c: &mut Criterion) {
    use lazycsv::domain::position::{ColIndex, RowIndex};

    let mut group = c.benchmark_group("rendering/get_cell");

    for &size in &[1_000, 10_000, 100_000] {
        let app = create_test_app(size, 20);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                // Access cells at various positions
                let row = black_box(size / 2);
                let col = black_box(5);
                let cell = app
                    .document
                    .get_cell(RowIndex::new(row), ColIndex::new(col));
                black_box(cell);
            });
        });
    }

    group.finish();
}

/// Benchmark column width calculation
fn bench_column_width_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering/column_widths");

    for &cols in &[10, 50, 100] {
        let app = create_test_app(1000, cols);

        group.bench_with_input(BenchmarkId::from_parameter(cols), &cols, |b, _| {
            b.iter(|| {
                // Simulate calculating column widths for visible columns
                for i in 0..black_box(10.min(cols)) {
                    let header = app
                        .document
                        .get_header(lazycsv::domain::position::ColIndex::new(i));
                    black_box(header.len());
                }
            });
        });
    }

    group.finish();
}

/// Benchmark visual selection checking (affects cell styling)
fn bench_visual_selection_check(c: &mut Criterion) {
    use lazycsv::app::{VisualMode, VisualSelection};
    use lazycsv::domain::position::{ColIndex, RowIndex};

    let mut group = c.benchmark_group("rendering/visual_selection");

    let mut app = create_test_app(10_000, 20);

    // Set up a visual block selection
    app.visual_selection = Some(VisualSelection::new(
        RowIndex::new(1000),
        ColIndex::new(5),
        VisualMode::Block,
    ));

    group.bench_function("is_in_selection_block", |b| {
        b.iter(|| {
            // Check multiple cells for selection status
            for row in 900..1100 {
                for col in 0..10 {
                    if let Some(sel) = &app.visual_selection {
                        let (start_row, end_row, start_col, end_col) = sel.bounds();
                        let in_selection = RowIndex::new(row) >= start_row
                            && RowIndex::new(row) <= end_row
                            && ColIndex::new(col) >= start_col
                            && ColIndex::new(col) <= end_col;
                        black_box(in_selection);
                    }
                }
            }
        });
    });

    group.finish();
}

/// Benchmark document cell access patterns
fn bench_cell_access_patterns(c: &mut Criterion) {
    use lazycsv::domain::position::{ColIndex, RowIndex};

    let mut group = c.benchmark_group("rendering/cell_access");

    for &size in &[1_000, 10_000, 100_000] {
        let app = create_test_app(size, 20);

        group.bench_with_input(
            BenchmarkId::new("sequential_access", size),
            &size,
            |b, _| {
                b.iter(|| {
                    // Simulate accessing cells during rendering
                    // (typical: 40 visible rows × 10 visible columns)
                    let start_row = black_box(size / 2);
                    for row_offset in 0..40 {
                        for col in 0..10 {
                            let cell = app.document.get_cell(
                                RowIndex::new((start_row + row_offset).min(size - 1)),
                                ColIndex::new(col),
                            );
                            black_box(cell);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_full_frame_render,
    bench_viewport_scrolling,
    bench_get_cell,
    bench_column_width_calculation,
    bench_visual_selection_check,
    bench_cell_access_patterns
);
criterion_main!(benches);
