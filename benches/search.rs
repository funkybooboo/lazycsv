//! Benchmark search performance
//!
//! Tests performance of search-related operations:
//! - Pattern compilation and matching
//! - Document scanning at different dataset sizes
//! - Jump navigation (n/N) performance
//! - Regex vs literal search comparison
//!
//! Target: <100ms for 100K rows (from roadmap v0.7.1)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use lazycsv::csv::Document;
use lazycsv::search::{find_matches, SearchState};
use lazycsv::{ColIndex, RowIndex};
use std::hint::black_box;

/// Create test document with specified row/column count
fn create_test_document(rows: usize, cols: usize) -> Document {
    let headers: Vec<String> = (0..cols).map(|i| format!("Col{}", i)).collect();
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|r| (0..cols).map(|c| format!("Row{}Col{}", r, c)).collect())
        .collect();

    Document::new(headers, data_rows, "benchmark.csv".to_string())
}

/// Create document with sparse matches (useful for testing jump performance)
fn create_sparse_match_document(rows: usize, cols: usize, match_frequency: usize) -> Document {
    let headers: Vec<String> = (0..cols).map(|i| format!("Col{}", i)).collect();
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| {
                    if r % match_frequency == 0 {
                        "MATCH".to_string()
                    } else {
                        format!("Row{}Col{}", r, c)
                    }
                })
                .collect()
        })
        .collect();

    Document::new(headers, data_rows, "sparse.csv".to_string())
}

/// Create document with dense matches (worst case for search)
fn create_dense_match_document(rows: usize, cols: usize) -> Document {
    let headers: Vec<String> = (0..cols).map(|i| format!("Col{}", i)).collect();
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|_| (0..cols).map(|_| "MATCH".to_string()).collect())
        .collect();

    Document::new(headers, data_rows, "dense.csv".to_string())
}

/// Benchmark simple literal search at different dataset sizes
fn bench_simple_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/simple_literal");

    // Critical test: 100K rows must search <100ms (roadmap requirement)
    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let matches = find_matches(black_box(&doc), black_box("Row5000"));
                black_box(matches);
            });
        });
    }

    group.finish();
}

/// Benchmark regex search at different dataset sizes
fn bench_regex_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/regex_pattern");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                // Regex pattern: match "Row" followed by 4 digits
                let matches = find_matches(black_box(&doc), black_box(r"^Row\d{4}"));
                black_box(matches);
            });
        });
    }

    group.finish();
}

/// Benchmark case-insensitive search
fn bench_case_insensitive_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/case_insensitive");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                // Search lowercase, should match "Row5000" (case-insensitive)
                let matches = find_matches(black_box(&doc), black_box("row5000"));
                black_box(matches);
            });
        });
    }

    group.finish();
}

/// Benchmark regex vs literal search comparison
fn bench_regex_vs_literal(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/regex_vs_literal");
    let doc = create_test_document(10_000, 10);

    group.bench_function("literal_substring", |b| {
        b.iter(|| {
            let matches = find_matches(black_box(&doc), black_box("Row5000"));
            black_box(matches);
        });
    });

    group.bench_function("regex_pattern", |b| {
        b.iter(|| {
            let matches = find_matches(black_box(&doc), black_box(r"Row\d+"));
            black_box(matches);
        });
    });

    group.bench_function("complex_regex", |b| {
        b.iter(|| {
            let matches = find_matches(black_box(&doc), black_box(r"^Row\d{4}Col[0-9]$"));
            black_box(matches);
        });
    });

    group.finish();
}

/// Benchmark jump to next match navigation
fn bench_jump_to_next(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/jump_to_next");

    for &size in &[1_000, 10_000, 100_000] {
        // Sparse matches: every 100th row has a match
        let doc = create_sparse_match_document(size, 10, 100);
        let matches = find_matches(&doc, "MATCH");
        let mut state = SearchState::new("MATCH".to_string(), matches);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                // Jump from start of document
                let result =
                    state.jump_to_next(black_box(RowIndex::new(0)), black_box(ColIndex::new(0)));
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark jump to previous match navigation
fn bench_jump_to_prev(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/jump_to_prev");

    for &size in &[1_000, 10_000, 100_000] {
        // Sparse matches: every 100th row has a match
        let doc = create_sparse_match_document(size, 10, 100);
        let matches = find_matches(&doc, "MATCH");
        let mut state = SearchState::new("MATCH".to_string(), matches);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                // Jump from end of document
                let result =
                    state.jump_to_prev(black_box(RowIndex::new(size)), black_box(ColIndex::new(0)));
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark worst-case search: all cells match
fn bench_worst_case_all_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/worst_case_all_match");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_dense_match_document(size, 10);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let matches = find_matches(black_box(&doc), black_box("MATCH"));
                black_box(matches);
            });
        });
    }

    group.finish();
}

/// Benchmark empty search (no matches)
fn bench_no_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/no_matches");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let matches = find_matches(black_box(&doc), black_box("NOTFOUND_XYZ"));
                black_box(matches);
            });
        });
    }

    group.finish();
}

/// Benchmark invalid regex fallback to literal search
fn bench_invalid_regex_fallback(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/invalid_regex_fallback");
    let doc = create_test_document(10_000, 10);

    group.bench_function("invalid_regex_pattern", |b| {
        b.iter(|| {
            // Invalid regex: unclosed bracket should fallback to literal
            let matches = find_matches(black_box(&doc), black_box("[invalid"));
            black_box(matches);
        });
    });

    group.finish();
}

/// Benchmark search with unicode content
fn bench_unicode_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/unicode_content");

    // Create document with unicode content
    let headers: Vec<String> = vec![
        "Name".to_string(),
        "City".to_string(),
        "Description".to_string(),
    ];
    let data_rows: Vec<Vec<String>> = (0..10_000)
        .map(|i| {
            vec![
                format!("User{}", i),
                "東京".to_string(), // Tokyo in Japanese
                "Hello 世界! 🌍".to_string(),
            ]
        })
        .collect();
    let doc = Document::new(headers, data_rows, "unicode 国家.csv".to_string());

    group.bench_function("search_japanese", |b| {
        b.iter(|| {
            let matches = find_matches(black_box(&doc), black_box("東京"));
            black_box(matches);
        });
    });

    group.bench_function("search_emoji", |b| {
        b.iter(|| {
            let matches = find_matches(black_box(&doc), black_box("🌍"));
            black_box(matches);
        });
    });

    group.bench_function("search_mixed_unicode", |b| {
        b.iter(|| {
            let matches = find_matches(black_box(&doc), black_box("世界"));
            black_box(matches);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_search,
    bench_regex_search,
    bench_case_insensitive_search,
    bench_regex_vs_literal,
    bench_jump_to_next,
    bench_jump_to_prev,
    bench_worst_case_all_match,
    bench_no_matches,
    bench_invalid_regex_fallback,
    bench_unicode_search,
);

criterion_main!(benches);
