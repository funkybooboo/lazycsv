//! Benchmark magnifier (vim editor) performance
//!
//! Tests performance of magnifier mode operations:
//! - Vim motions (hjkl, w/b/e, 0/$, gg/G)
//! - Operators (x, dd, p, P, J, u)
//! - Search operations (/, n, N, *)
//! - Undo/redo with large undo history
//! - Rendering large documents
//!
//! Performance targets:
//! - Motions: <1ms per operation
//! - Operators: <10ms for single line operations
//! - Search: <50ms for 10K line documents
//! - Undo/redo: <5ms per operation
//! - Rendering: <16.67ms per frame (60 FPS)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lazycsv::domain::position::{ColIndex, RowIndex};
use lazycsv::magnifier::MagnifierState;

/// Create test magnifier with specified line count
fn create_test_magnifier(lines: usize, line_length: usize) -> MagnifierState {
    let content: Vec<String> = (0..lines)
        .map(|i| format!("Line {} {}", i, "content ".repeat(line_length / 8)))
        .collect();
    let content_str = content.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    MagnifierState::new(content_str, position)
}

/// Benchmark basic vim motions (hjkl)
fn bench_vim_motions_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/motions/basic");

    for &size in &[100, 1_000, 10_000] {
        let mut mag = create_test_magnifier(size, 80);

        group.bench_with_input(BenchmarkId::new("move_down", size), &size, |b, _| {
            b.iter(|| {
                mag.move_down();
                black_box(&mag);
            });
        });

        group.bench_with_input(BenchmarkId::new("move_up", size), &size, |b, _| {
            b.iter(|| {
                mag.move_up();
                black_box(&mag);
            });
        });

        group.bench_with_input(BenchmarkId::new("move_right", size), &size, |b, _| {
            b.iter(|| {
                mag.move_right();
                black_box(&mag);
            });
        });

        group.bench_with_input(BenchmarkId::new("move_left", size), &size, |b, _| {
            b.iter(|| {
                mag.move_left();
                black_box(&mag);
            });
        });
    }

    group.finish();
}

/// Benchmark word motions (w, b, e)
fn bench_vim_motions_word(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/motions/word");

    for &line_length in &[40, 100, 500] {
        let mut mag = create_test_magnifier(1000, line_length);

        group.bench_with_input(
            BenchmarkId::new("move_next_word", line_length),
            &line_length,
            |b, _| {
                b.iter(|| {
                    mag.move_next_word();
                    black_box(&mag);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("move_prev_word", line_length),
            &line_length,
            |b, _| {
                b.iter(|| {
                    mag.move_prev_word();
                    black_box(&mag);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("move_end_word", line_length),
            &line_length,
            |b, _| {
                b.iter(|| {
                    mag.move_end_word();
                    black_box(&mag);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark document navigation (gg, G, {, })
fn bench_vim_motions_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/motions/document");

    for &size in &[100, 1_000, 10_000] {
        let mut mag = create_test_magnifier(size, 80);

        group.bench_with_input(BenchmarkId::new("goto_first_line", size), &size, |b, _| {
            b.iter(|| {
                mag.move_to_first_line();
                black_box(&mag);
            });
        });

        group.bench_with_input(BenchmarkId::new("goto_last_line", size), &size, |b, _| {
            b.iter(|| {
                mag.move_to_last_line();
                black_box(&mag);
            });
        });
    }

    group.finish();
}

/// Benchmark line operators (x, dd, J)
fn bench_vim_operators(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/operators");

    for &size in &[100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("delete_char", size), &size, |b, _| {
            b.iter_batched(
                || create_test_magnifier(size, 80),
                |mut mag| {
                    mag.push_undo();
                    mag.delete_char();
                    black_box(mag);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("delete_line", size), &size, |b, _| {
            b.iter_batched(
                || create_test_magnifier(size, 80),
                |mut mag| {
                    mag.push_undo();
                    mag.delete_line();
                    black_box(mag);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("join_lines", size), &size, |b, _| {
            b.iter_batched(
                || create_test_magnifier(size, 80),
                |mut mag| {
                    mag.push_undo();
                    mag.join_lines();
                    black_box(mag);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark paste operations
fn bench_vim_paste(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/paste");

    for &size in &[100, 1_000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("yank_and_paste_below", size),
            &size,
            |b, _| {
                b.iter_batched(
                    || {
                        let mut mag = create_test_magnifier(size, 80);
                        mag.yank_line(); // Populate the register
                        mag
                    },
                    |mut mag| {
                        mag.push_undo();
                        mag.paste_below();
                        black_box(mag);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("yank_and_paste_above", size),
            &size,
            |b, _| {
                b.iter_batched(
                    || {
                        let mut mag = create_test_magnifier(size, 80);
                        mag.yank_line(); // Populate the register
                        mag
                    },
                    |mut mag| {
                        mag.push_undo();
                        mag.paste_above();
                        black_box(mag);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark undo/redo operations
fn bench_vim_undo_redo(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/undo_redo");

    for &undo_depth in &[10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("undo", undo_depth),
            &undo_depth,
            |b, &depth| {
                b.iter_batched(
                    || {
                        let mut mag = create_test_magnifier(1000, 80);
                        // Build undo history
                        for _ in 0..depth {
                            mag.push_undo();
                            mag.delete_char();
                        }
                        mag
                    },
                    |mut mag| {
                        mag.undo();
                        black_box(mag);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("redo", undo_depth),
            &undo_depth,
            |b, &depth| {
                b.iter_batched(
                    || {
                        let mut mag = create_test_magnifier(1000, 80);
                        // Build undo history and undo once
                        for _ in 0..depth {
                            mag.push_undo();
                            mag.delete_char();
                        }
                        mag.undo();
                        mag
                    },
                    |mut mag| {
                        mag.redo();
                        black_box(mag);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark search operations
fn bench_vim_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/search");

    for &size in &[100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("search_forward", size), &size, |b, _| {
            b.iter_batched(
                || create_test_magnifier(size, 80),
                |mut mag| {
                    mag.search_forward("content".to_string());
                    black_box(mag);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("jump_to_next_match", size),
            &size,
            |b, _| {
                b.iter_batched(
                    || {
                        let mut mag = create_test_magnifier(size, 80);
                        mag.search_forward("content".to_string());
                        mag
                    },
                    |mut mag| {
                        mag.jump_to_next_match();
                        black_box(mag);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark text insertion
fn bench_vim_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/insert");

    for &size in &[100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("insert_char", size), &size, |b, _| {
            b.iter_batched(
                || create_test_magnifier(size, 80),
                |mut mag| {
                    mag.insert_char('x');
                    black_box(mag);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("newline", size), &size, |b, _| {
            b.iter_batched(
                || create_test_magnifier(size, 80),
                |mut mag| {
                    mag.newline();
                    black_box(mag);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark visual selection operations
fn bench_vim_visual(c: &mut Criterion) {
    let mut group = c.benchmark_group("magnifier/visual");

    for &size in &[100, 1_000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("get_visual_selection", size),
            &size,
            |b, _| {
                b.iter_batched(
                    || {
                        let mut mag = create_test_magnifier(size, 80);
                        mag.enter_visual_mode();
                        for _ in 0..10 {
                            mag.move_down();
                        }
                        mag
                    },
                    |mag| {
                        let sel = mag.visual_selection();
                        black_box(sel);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(BenchmarkId::new("delete_selection", size), &size, |b, _| {
            b.iter_batched(
                || {
                    let mut mag = create_test_magnifier(size, 80);
                    mag.enter_visual_mode();
                    for _ in 0..10 {
                        mag.move_down();
                    }
                    mag
                },
                |mut mag| {
                    mag.delete_selection();
                    black_box(mag);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_vim_motions_basic,
    bench_vim_motions_word,
    bench_vim_motions_document,
    bench_vim_operators,
    bench_vim_paste,
    bench_vim_undo_redo,
    bench_vim_search,
    bench_vim_insert,
    bench_vim_visual
);
criterion_main!(benches);
