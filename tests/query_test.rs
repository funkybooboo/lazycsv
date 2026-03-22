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

#[test]
fn test_query_select_all() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "name,age\nAlice,30\nBob,25\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM data")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + 2 data rows
    assert_eq!(lines[0], "name,age");
    assert_eq!(lines[1], "Alice,30");
    assert_eq!(lines[2], "Bob,25");
}

#[test]
fn test_query_where_clause() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "people.csv",
        "name,age\nAlice,30\nBob,25\nCharlie,35\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT name FROM people WHERE CAST(age AS INTEGER) > 28")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + Alice(30) + Charlie(35)
    assert_eq!(lines[0], "name");
    assert!(lines.contains(&"Alice"));
    assert!(lines.contains(&"Charlie"));
}

#[test]
fn test_query_join_two_files() {
    let dir = TempDir::new().unwrap();
    write_csv(
        dir.path(),
        "orders.csv",
        "id,customer_id,product\n1,101,Widget\n2,102,Gadget\n",
    );
    write_csv(
        dir.path(),
        "customers.csv",
        "customer_id,name\n101,Alice\n102,Bob\n",
    );

    // Use directory path to load all CSVs
    let output = lazycsv_bin()
        .arg(dir.path().to_str().unwrap())
        .arg("-q")
        .arg("SELECT o.product, c.name FROM orders o JOIN customers c ON o.customer_id = c.customer_id ORDER BY o.id")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "product,name");
    assert_eq!(lines[1], "Widget,Alice");
    assert_eq!(lines[2], "Gadget,Bob");
}

#[test]
fn test_query_directory_loads_all_csvs() {
    let dir = TempDir::new().unwrap();
    write_csv(dir.path(), "a.csv", "x\n1\n");
    write_csv(dir.path(), "b.csv", "y\n2\n");

    // Query from table 'a' should work when pointing at the directory
    let output = lazycsv_bin()
        .arg(dir.path().to_str().unwrap())
        .arg("-q")
        .arg("SELECT x FROM a")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1"));
}

#[test]
fn test_query_invalid_sql_fails() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("INVALID SQL STATEMENT")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
}

#[test]
fn test_query_nonexistent_table_fails() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "a\n1\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT * FROM nonexistent")
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_query_with_delimiter() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "name;age\nAlice;30\nBob;25\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-d")
        .arg(";")
        .arg("-q")
        .arg("SELECT name FROM data ORDER BY name")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "name");
    assert_eq!(lines[1], "Alice");
    assert_eq!(lines[2], "Bob");
}

#[test]
fn test_query_no_headers() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(dir.path(), "data.csv", "Alice,30\nBob,25\n");

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("--no-headers")
        .arg("-q")
        .arg("SELECT * FROM data ORDER BY column0")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + 2 data rows
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Bob"));
}

#[test]
fn test_query_aggregation_count() {
    let dir = TempDir::new().unwrap();
    let file = write_csv(
        dir.path(),
        "data.csv",
        "city,name\nNY,Alice\nLA,Bob\nNY,Charlie\n",
    );

    let output = lazycsv_bin()
        .arg(file.to_str().unwrap())
        .arg("-q")
        .arg("SELECT city, COUNT(*) as cnt FROM data GROUP BY city ORDER BY city")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 3); // header + LA + NY
    assert_eq!(lines[0], "city,cnt");
    assert_eq!(lines[1], "LA,1");
    assert_eq!(lines[2], "NY,2");
}

#[test]
fn test_query_file_path_also_loads_siblings() {
    let dir = TempDir::new().unwrap();
    let file_a = write_csv(dir.path(), "alpha.csv", "id,val\n1,a\n");
    write_csv(dir.path(), "beta.csv", "id,val\n1,b\n");

    // Point at alpha.csv but query beta table (sibling loaded automatically)
    let output = lazycsv_bin()
        .arg(file_a.to_str().unwrap())
        .arg("-q")
        .arg("SELECT val FROM beta")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("b"));
}

#[test]
fn test_query_nonexistent_path_fails() {
    let output = lazycsv_bin()
        .arg("/nonexistent/path.csv")
        .arg("-q")
        .arg("SELECT 1")
        .output()
        .unwrap();

    assert!(!output.status.success());
}
