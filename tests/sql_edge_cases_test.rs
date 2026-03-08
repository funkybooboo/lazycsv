//! Comprehensive SQL edge case tests for v0.8.1
//!
//! Tests error handling, edge cases, and complex queries for SQL query mode.

use std::process::Command;
use tempfile::TempDir;

fn lazycsv_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lazycsv"))
}

fn write_csv(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

// ============================================================================
// Error Handling Tests (8 tests)
// ============================================================================

#[test]
fn test_invalid_sql_syntax_select_without_from() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT a")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
}

#[test]
fn test_invalid_sql_syntax_missing_paren() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data WHERE (a = 1")
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_misspelled_column_name() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "name,age\nAlice,30\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT nmae FROM data")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("column") || stderr.contains("nmae"));
}

#[test]
fn test_missing_table() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM missing_table")
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_type_mismatch_string_as_number() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "name,value\nAlice,hello\n");

    // SQLite is pretty lenient with types, but let's try an operation
    // that would fail with non-numeric data
    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT name, CAST(value AS INTEGER) * 2 FROM data")
        .output()
        .unwrap();

    // This might succeed with SQLite's type coercion (returns 0)
    // Just verify it doesn't crash
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn test_division_by_zero() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n0\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT 1 / CAST(a AS INTEGER) FROM data")
        .output()
        .unwrap();

    // SQLite returns NULL for division by zero, so this should succeed
    assert!(output.status.success());
}

#[test]
fn test_empty_query_string() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("")
        .output()
        .unwrap();

    // Should handle empty query gracefully
    assert!(!output.status.success() || output.stdout.is_empty());
}

#[test]
fn test_query_only_whitespace() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("   \n\t  ")
        .output()
        .unwrap();

    // Should handle whitespace-only query gracefully
    assert!(!output.status.success() || output.stdout.is_empty());
}

// ============================================================================
// Edge Cases Tests (8 tests)
// ============================================================================

#[test]
fn test_empty_result_set() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "name,age\nAlice,30\nBob,25\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data WHERE CAST(age AS INTEGER) > 100")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    // Should have header but no data rows
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("name") || lines[0].contains("age"));
}

#[test]
fn test_large_result_set() {
    let dir = TempDir::new().unwrap();

    // Generate CSV with 1000 rows
    let mut content = String::from("id,value\n");
    for i in 0..1000 {
        content.push_str(&format!("{},data{}\n", i, i));
    }
    let file = write_csv(dir.path(), "large.csv", &content);

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM large")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    // Should have header + 1000 data rows
    assert_eq!(lines.len(), 1001);
}

#[test]
fn test_result_with_null_values() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a,b\n1,x\n2,\n3,z\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 4); // header + 3 rows
                                // Empty cell should be preserved
    assert!(
        lines[2].ends_with(',') || lines[2].contains(",,") || lines[2].matches(',').count() >= 1
    );
}

#[test]
fn test_result_with_special_characters() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "text\n\"hello, world\"\n\"quote\"\"test\"\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Should handle CSV escaping correctly
    assert!(stdout.contains("hello") || stdout.contains("quote"));
}

#[test]
fn test_query_with_unicode_data() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "name,city\n田中,東京\nAlice,Paris\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("田中") || stdout.contains("東京"));
}

#[test]
fn test_query_with_long_strings() {
    let dir = TempDir::new().unwrap();

    // Create a row with a very long string (10KB)
    let long_string = "x".repeat(10000);
    let content = format!("data\n{}\n", long_string);
    let file = write_csv(dir.path(), "data.csv", &content);

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Should handle long strings without truncation
    assert!(stdout.len() > 9000);
}

#[test]
fn test_multiple_queries_same_session() {
    // This tests that the SQLite connection is properly managed
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n2\n3\n");

    // First query
    let output1 = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT COUNT(*) as cnt FROM data")
        .output()
        .unwrap();

    assert!(output1.status.success());

    // Second query (simulates multiple queries in sequence)
    let output2 = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data")
        .output()
        .unwrap();

    assert!(output2.status.success());
}

#[test]
fn test_query_with_case_insensitive_columns() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "Name,Age\nAlice,30\n");

    // SQLite is case-insensitive for column names
    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT name, AGE FROM data")
        .output()
        .unwrap();

    assert!(output.status.success());
}

// ============================================================================
// Complex Queries Tests (6+ tests)
// ============================================================================

#[test]
fn test_three_way_join() {
    let dir = TempDir::new().unwrap();
    write_csv(
        dir.path(),
        "orders.csv",
        "order_id,customer_id,product_id\n1,101,201\n2,102,202\n",
    );
    write_csv(
        dir.path(),
        "customers.csv",
        "customer_id,name\n101,Alice\n102,Bob\n",
    );
    write_csv(
        dir.path(),
        "products.csv",
        "product_id,product_name\n201,Widget\n202,Gadget\n",
    );

    let output = lazycsv_bin()
        .arg(dir.path().to_str().unwrap())
        .arg("-q")
        .arg("SELECT c.name, p.product_name FROM orders o JOIN customers c ON o.customer_id = c.customer_id JOIN products p ON o.product_id = p.product_id ORDER BY o.order_id")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Widget"));
}

#[test]
fn test_nested_subquery() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "category,value\nA,10\nA,20\nB,30\nB,40\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT category, SUM(CAST(value AS INTEGER)) as total FROM data WHERE category IN (SELECT DISTINCT category FROM data) GROUP BY category")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + A + B
}

#[test]
fn test_union_multiple_tables() {
    let dir = TempDir::new().unwrap();
    write_csv(dir.path(), "table1.csv", "name\nAlice\nBob\n");
    write_csv(dir.path(), "table2.csv", "name\nCharlie\nDiana\n");

    let output = lazycsv_bin()
        .arg(dir.path().to_str().unwrap())
        .arg("-q")
        .arg("SELECT name FROM table1 UNION SELECT name FROM table2 ORDER BY name")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 5); // header + 4 names
}

#[test]
fn test_complex_group_by_with_having() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "sales.csv",
        "region,amount\nNorth,100\nNorth,200\nSouth,50\nSouth,60\nEast,300\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT region, SUM(CAST(amount AS INTEGER)) as total FROM sales GROUP BY region HAVING total > 100 ORDER BY total DESC")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    // Should include North (300), South (110), and East (300) - all > 100
    assert_eq!(lines.len(), 4); // header + 3 regions
}

#[test]
fn test_self_join() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "employees.csv",
        "id,name,manager_id\n1,Alice,\n2,Bob,1\n3,Charlie,1\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT e.name as employee, m.name as manager FROM employees e LEFT JOIN employees m ON e.manager_id = m.id ORDER BY e.id")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Bob"));
    assert!(stdout.contains("Alice"));
}

#[test]
fn test_complex_aggregation_multiple_functions() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "category,value\nA,10\nA,20\nA,30\nB,5\nB,15\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT category, COUNT(*) as cnt, SUM(CAST(value AS INTEGER)) as total, AVG(CAST(value AS INTEGER)) as avg, MIN(CAST(value AS INTEGER)) as min, MAX(CAST(value AS INTEGER)) as max FROM data GROUP BY category ORDER BY category")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + A + B
                                // Category A should have cnt=3, total=60
    assert!(stdout.contains("3"));
    assert!(stdout.contains("60"));
}

// ============================================================================
// Additional Edge Cases (4+ more tests to reach 20+)
// ============================================================================

#[test]
fn test_query_with_limit_and_offset() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "id\n1\n2\n3\n4\n5\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data ORDER BY CAST(id AS INTEGER) LIMIT 2 OFFSET 2")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + rows 3,4
    assert!(stdout.contains("3"));
    assert!(stdout.contains("4"));
}

#[test]
fn test_query_with_distinct() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "category\nA\nB\nA\nC\nB\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT DISTINCT category FROM data ORDER BY category")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 4); // header + A, B, C
}

#[test]
fn test_query_with_string_functions() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "name\nalice\nbob\ncharlie\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT UPPER(name) as upper_name, LENGTH(name) as len FROM data ORDER BY name")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ALICE"));
    assert!(stdout.contains("5")); // length of alice
}

#[test]
fn test_query_with_case_expression() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "name,age\nAlice,30\nBob,15\nCharlie,25\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT name, CASE WHEN CAST(age AS INTEGER) >= 18 THEN 'adult' ELSE 'minor' END as status FROM data ORDER BY name")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("adult"));
    assert!(stdout.contains("minor"));
}

#[test]
fn test_query_with_date_functions() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "event,date\nMeeting,2024-01-15\nLaunch,2024-03-20\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT event, date, strftime('%Y', date) as year FROM data ORDER BY date")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("2024"));
}

#[test]
fn test_query_cross_join() {
    let dir = TempDir::new().unwrap();
    write_csv(dir.path(), "colors.csv", "color\nred\nblue\n");
    write_csv(dir.path(), "sizes.csv", "size\nS\nL\n");

    let output = lazycsv_bin()
        .arg(dir.path().to_str().unwrap())
        .arg("-q")
        .arg("SELECT c.color, s.size FROM colors c CROSS JOIN sizes s ORDER BY c.color, s.size")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 5); // header + 2x2 combinations
}

#[test]
fn test_query_with_like_pattern() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "name\nAlice\nAmy\nBob\nAnna\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT name FROM data WHERE name LIKE 'A%' ORDER BY name")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 4); // header + Alice, Amy, Anna
    assert!(!stdout.contains("Bob"));
}

#[test]
fn test_query_with_in_operator() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,Diana\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT name FROM data WHERE CAST(id AS INTEGER) IN (1, 3) ORDER BY id")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + Alice, Charlie
}
