//! Benchmark SQL query performance
//!
//! Tests performance of SQL-related operations:
//! - CSV loading into DuckDB (single and multi-table)
//! - Query operations: SELECT, WHERE, ORDER BY, JOIN, GROUP BY
//! - Result conversion back to Document
//! - Dataset sizes: 1K, 10K, 100K rows
//!
//! Targets (from roadmap v0.8.1):
//! - Simple SELECT <50ms for 100K rows
//! - JOIN <200ms for 10K rows

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use duckdb::Connection;
use lazycsv::csv::Document;
use lazycsv::query::{
    execute_query_to_document_cancellable, load_csv_into_duckdb, table_name_from_path,
};
use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Create test document with specified row/column count
fn create_test_document(rows: usize, cols: usize, name: &str) -> Document {
    let headers: Vec<String> = (0..cols).map(|i| format!("col{}", i)).collect();
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|r| (0..cols).map(|c| format!("r{}c{}", r, c)).collect())
        .collect();

    Document::new(headers, data_rows, name.to_string())
}

/// Create a sales document for JOIN benchmarks
fn create_sales_document(rows: usize, name: &str) -> Document {
    let headers = vec![
        "order_id".to_string(),
        "customer_id".to_string(),
        "product_id".to_string(),
        "quantity".to_string(),
        "total".to_string(),
    ];
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|i| {
            vec![
                format!("ORD{:06}", i),
                format!("CUST{:04}", i % 1000),
                format!("PROD{:03}", i % 100),
                format!("{}", (i % 10) + 1),
                format!("{:.2}", ((i % 10) + 1) as f64 * 9.99),
            ]
        })
        .collect();

    Document::new(headers, data_rows, name.to_string())
}

/// Create a customers document for JOIN benchmarks
fn create_customers_document(rows: usize, name: &str) -> Document {
    let headers = vec![
        "customer_id".to_string(),
        "name".to_string(),
        "city".to_string(),
        "state".to_string(),
    ];
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|i| {
            vec![
                format!("CUST{:04}", i),
                format!("Customer {}", i),
                format!("City{}", i % 50),
                format!("State{}", i % 10),
            ]
        })
        .collect();

    Document::new(headers, data_rows, name.to_string())
}

/// Create a products document for JOIN benchmarks
fn create_products_document(rows: usize, name: &str) -> Document {
    let headers = vec![
        "product_id".to_string(),
        "name".to_string(),
        "category".to_string(),
        "price".to_string(),
    ];
    let data_rows: Vec<Vec<String>> = (0..rows)
        .map(|i| {
            vec![
                format!("PROD{:03}", i),
                format!("Product {}", i),
                format!("Category{}", i % 10),
                format!("{:.2}", (i % 100) as f64 * 9.99),
            ]
        })
        .collect();

    Document::new(headers, data_rows, name.to_string())
}

/// Benchmark loading CSV into SQLite (single table)
fn bench_csv_load_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/load_single_table");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10, "test.csv");

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let conn = Connection::open_in_memory().unwrap();
                load_csv_into_duckdb(black_box(&conn), black_box(&doc), black_box("test")).unwrap();
                black_box(conn);
            });
        });
    }

    group.finish();
}

/// Benchmark loading multiple CSVs into SQLite
fn bench_csv_load_multiple(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/load_multiple_tables");

    for &size in &[1_000, 10_000, 100_000] {
        let sales = create_sales_document(size, "sales.csv");
        let customers = create_customers_document(size / 10, "customers.csv");
        let products = create_products_document(100, "products.csv");

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let conn = Connection::open_in_memory().unwrap();
                load_csv_into_duckdb(&conn, black_box(&sales), "sales").unwrap();
                load_csv_into_duckdb(&conn, black_box(&customers), "customers").unwrap();
                load_csv_into_duckdb(&conn, black_box(&products), "products").unwrap();
                black_box(conn);
            });
        });
    }

    group.finish();
}

/// Benchmark simple SELECT query
fn bench_query_select(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/query_select");

    // Critical test: <50ms for 100K rows (roadmap requirement)
    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10, "test.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &doc, "test").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box("SELECT * FROM test LIMIT 100"),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark SELECT with WHERE clause
fn bench_query_where(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/query_where");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10, "test.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &doc, "test").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box("SELECT * FROM test WHERE col0 LIKE 'r50%'"),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark SELECT with ORDER BY
fn bench_query_order_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/query_order_by");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10, "test.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &doc, "test").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box("SELECT * FROM test ORDER BY col0 DESC LIMIT 100"),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark 2-way JOIN
fn bench_query_join_2way(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/query_join_2way");

    // Critical test: <200ms for 10K rows (roadmap requirement)
    for &size in &[1_000, 10_000, 100_000] {
        let sales = create_sales_document(size, "sales.csv");
        let customers = create_customers_document(size / 10, "customers.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &sales, "sales").unwrap();
        load_csv_into_duckdb(&conn, &customers, "customers").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box(
                        "SELECT s.order_id, c.name, s.total 
                         FROM sales s 
                         JOIN customers c ON s.customer_id = c.customer_id 
                         LIMIT 100",
                    ),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark 3-way JOIN
fn bench_query_join_3way(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/query_join_3way");

    for &size in &[1_000, 10_000, 100_000] {
        let sales = create_sales_document(size, "sales.csv");
        let customers = create_customers_document(size / 10, "customers.csv");
        let products = create_products_document(100, "products.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &sales, "sales").unwrap();
        load_csv_into_duckdb(&conn, &customers, "customers").unwrap();
        load_csv_into_duckdb(&conn, &products, "products").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box(
                        "SELECT s.order_id, c.name, p.name, s.quantity, s.total
                         FROM sales s
                         JOIN customers c ON s.customer_id = c.customer_id
                         JOIN products p ON s.product_id = p.product_id
                         LIMIT 100",
                    ),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark GROUP BY with aggregations
fn bench_query_group_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/query_group_by");

    for &size in &[1_000, 10_000, 100_000] {
        let sales = create_sales_document(size, "sales.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &sales, "sales").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box(
                        "SELECT customer_id, COUNT(*) as order_count, SUM(CAST(total AS REAL)) as total_spent
                         FROM sales 
                         GROUP BY customer_id 
                         ORDER BY total_spent DESC 
                         LIMIT 100",
                    ),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark result size impact: small results
fn bench_query_result_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/result_size_small");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10, "test.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &doc, "test").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box("SELECT * FROM test LIMIT 10"),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark result size impact: medium results
fn bench_query_result_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/result_size_medium");

    for &size in &[1_000, 10_000, 100_000] {
        let doc = create_test_document(size, 10, "test.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &doc, "test").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box("SELECT * FROM test LIMIT 1000"),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark result size impact: large results
fn bench_query_result_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/result_size_large");

    for &size in &[10_000, 100_000] {
        let doc = create_test_document(size, 10, "test.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &doc, "test").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box("SELECT * FROM test LIMIT 50000"),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark complex query with multiple operations
fn bench_query_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/query_complex");

    for &size in &[1_000, 10_000, 100_000] {
        let sales = create_sales_document(size, "sales.csv");
        let customers = create_customers_document(size / 10, "customers.csv");
        let products = create_products_document(100, "products.csv");
        let conn = Connection::open_in_memory().unwrap();
        load_csv_into_duckdb(&conn, &sales, "sales").unwrap();
        load_csv_into_duckdb(&conn, &customers, "customers").unwrap();
        load_csv_into_duckdb(&conn, &products, "products").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let cancelled = AtomicBool::new(false);
                let result = execute_query_to_document_cancellable(
                    black_box(&conn),
                    black_box(
                        "SELECT c.state, p.category, 
                                COUNT(*) as orders,
                                SUM(CAST(s.quantity AS INTEGER)) as total_quantity,
                                SUM(CAST(s.total AS REAL)) as revenue
                         FROM sales s
                         JOIN customers c ON s.customer_id = c.customer_id
                         JOIN products p ON s.product_id = p.product_id
                         WHERE CAST(s.total AS REAL) > 50.0
                         GROUP BY c.state, p.category
                         HAVING COUNT(*) > 5
                         ORDER BY revenue DESC
                         LIMIT 100",
                    ),
                    black_box("result.csv".to_string()),
                    black_box(&cancelled),
                )
                .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark table name derivation
fn bench_table_name_from_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql/table_name_from_path");

    group.bench_function("simple", |b| {
        b.iter(|| {
            let name = table_name_from_path(black_box(Path::new("test.csv")));
            black_box(name);
        });
    });

    group.bench_function("special_chars", |b| {
        b.iter(|| {
            let name = table_name_from_path(black_box(Path::new("my-data@2024.csv")));
            black_box(name);
        });
    });

    group.bench_function("long_path", |b| {
        b.iter(|| {
            let name = table_name_from_path(black_box(Path::new(
                "/very/long/path/to/some/deeply/nested/directory/data.csv",
            )));
            black_box(name);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_load_single,
    bench_csv_load_multiple,
    bench_query_select,
    bench_query_where,
    bench_query_order_by,
    bench_query_join_2way,
    bench_query_join_3way,
    bench_query_group_by,
    bench_query_result_small,
    bench_query_result_medium,
    bench_query_result_large,
    bench_query_complex,
    bench_table_name_from_path,
);

criterion_main!(benches);
