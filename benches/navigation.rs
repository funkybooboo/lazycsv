//! Benchmark navigation commands with count prefixes
//!
//! Tests performance of common navigation patterns:
//! - Directional movement (hjkl) with count prefixes
//! - Jump commands (gg, G, goto_line)
//! - Page navigation (PageUp, PageDown)
//! - Word motion (w, b, e)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lazycsv::csv::Document;
use lazycsv::navigation::commands::*;
use lazycsv::session::FileConfig;
use lazycsv::App;
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

/// Benchmark directional movement with count prefix (5j, 100k, etc.)
fn bench_directional_movement(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/directional");

    // Test with different dataset sizes
    for &size in &[1_000, 10_000, 100_000] {
        let mut app = create_test_app(size, 10);

        group.bench_with_input(BenchmarkId::new("move_down_by", size), &size, |b, _| {
            b.iter(|| {
                app.view_state.table_state.select(Some(0));
                move_down_by(black_box(&mut app), black_box(100));
            });
        });

        group.bench_with_input(BenchmarkId::new("move_up_by", size), &size, |b, _| {
            b.iter(|| {
                app.view_state.table_state.select(Some(1000));
                move_up_by(black_box(&mut app), black_box(100));
            });
        });

        group.bench_with_input(BenchmarkId::new("move_right_by", size), &size, |b, _| {
            b.iter(|| {
                move_right_by(black_box(&mut app), black_box(5));
            });
        });

        group.bench_with_input(BenchmarkId::new("move_left_by", size), &size, |b, _| {
            b.iter(|| {
                move_left_by(black_box(&mut app), black_box(5));
            });
        });
    }

    group.finish();
}

/// Benchmark jump commands (gg, G, 5G)
fn bench_jump_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/jumps");

    for &size in &[1_000, 10_000, 100_000] {
        let mut app = create_test_app(size, 10);

        group.bench_with_input(BenchmarkId::new("goto_first_row", size), &size, |b, _| {
            b.iter(|| {
                goto_first_row(black_box(&mut app));
            });
        });

        group.bench_with_input(BenchmarkId::new("goto_last_row", size), &size, |b, _| {
            b.iter(|| {
                goto_last_row(black_box(&mut app));
            });
        });

        group.bench_with_input(BenchmarkId::new("goto_line", size), &size, |b, _| {
            b.iter(|| {
                goto_line(black_box(&mut app), black_box(size / 2));
            });
        });
    }

    group.finish();
}

/// Benchmark word motion (w, b, e)
fn bench_word_motion(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/word_motion");

    // Create dataset with sparse data (every 3rd cell has content)
    for &size in &[1_000, 10_000, 100_000] {
        let headers: Vec<String> = (0..30).map(|i| format!("Col{}", i)).collect();
        let data_rows: Vec<Vec<String>> = (0..size)
            .map(|r| {
                (0..30)
                    .map(|c| {
                        if c % 3 == 0 {
                            format!("R{}C{}", r, c)
                        } else {
                            String::new()
                        }
                    })
                    .collect()
            })
            .collect();

        let document = Document::new(headers, data_rows, "benchmark.csv".to_string());
        let csv_files = vec![PathBuf::from("benchmark.csv")];
        let mut app = App::new(document, csv_files, 0, FileConfig::new());

        group.bench_with_input(BenchmarkId::new("next_word", size), &size, |b, _| {
            b.iter(|| {
                next_word(black_box(&mut app));
            });
        });

        group.bench_with_input(BenchmarkId::new("prev_word", size), &size, |b, _| {
            b.iter(|| {
                prev_word(black_box(&mut app));
            });
        });

        group.bench_with_input(BenchmarkId::new("end_word", size), &size, |b, _| {
            b.iter(|| {
                end_word(black_box(&mut app));
            });
        });
    }

    group.finish();
}

/// Benchmark column navigation (goto_column, goto_column_by_number)
fn bench_column_navigation(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/columns");

    // Test with different column counts
    for &cols in &[10, 50, 200] {
        let mut app = create_test_app(1000, cols);

        group.bench_with_input(
            BenchmarkId::new("goto_column_letter", cols),
            &cols,
            |b, _| {
                b.iter(|| {
                    goto_column(black_box(&mut app), black_box("Z"));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("goto_column_number", cols),
            &cols,
            |b, _| {
                b.iter(|| {
                    goto_column_by_number(black_box(&mut app), black_box(25));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_directional_movement,
    bench_jump_commands,
    bench_word_motion,
    bench_column_navigation
);
criterion_main!(benches);
