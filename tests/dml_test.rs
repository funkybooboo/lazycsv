use clap::Parser;
use lazycsv::cli::CliArgs;
use lazycsv::App;
use std::fs::write;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

fn create_app(csv_content: &str) -> (App, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("data.csv");
    write(&file_path, csv_content).unwrap();
    let args = CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap()]).unwrap();
    let app = App::from_cli(args).unwrap();
    (app, temp_dir)
}

#[test]
fn test_dml_update() {
    let (mut app, _dir) = create_app("Name,Value\nAlice,10\nBob,20\nCharlie,30\n");

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};
    let (success, was_cancelled) = app.execute_sql_dml_cancellable(
        "UPDATE data SET Value = '99' WHERE Name = 'Bob'",
        &cancelled,
        &mut progress,
    );

    assert!(success, "DML should succeed");
    assert!(!was_cancelled);
    assert!(app.document.is_dirty, "Document should be dirty after DML");

    // Find the row where Name = Bob and check Value is 99
    let mut found = false;
    for i in 1..app.document.row_count() {
        let row = app.document.get_rows_range(i, i + 1);
        if row[0].first().map(|s| s.as_str()) == Some("Bob") {
            assert_eq!(row[0][1], "99");
            found = true;
        }
    }
    assert!(found, "Should find Bob's updated row");
}

#[test]
fn test_dml_insert() {
    let (mut app, _dir) = create_app("Name,Value\nAlice,10\n");

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};
    let original_count = app.document.row_count();

    let (success, _) = app.execute_sql_dml_cancellable(
        "INSERT INTO data (Name, Value) VALUES ('New', '42')",
        &cancelled,
        &mut progress,
    );

    assert!(success);
    assert_eq!(app.document.row_count(), original_count + 1);
    assert!(app.document.is_dirty);
}

#[test]
fn test_dml_delete() {
    let (mut app, _dir) = create_app("Name,Value\nAlice,10\nBob,20\nCharlie,30\n");

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};
    let original_count = app.document.row_count();

    let (success, _) = app.execute_sql_dml_cancellable(
        "DELETE FROM data WHERE Name = 'Bob'",
        &cancelled,
        &mut progress,
    );

    assert!(success);
    assert_eq!(app.document.row_count(), original_count - 1);
    assert!(app.document.is_dirty);
}

#[test]
fn test_dml_invalid_table_errors() {
    let (mut app, _dir) = create_app("Name,Value\nAlice,10\n");

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};

    let (success, _) = app.execute_sql_dml_cancellable(
        "UPDATE nonexistent SET Value = '99'",
        &cancelled,
        &mut progress,
    );

    assert!(!success, "Should fail for nonexistent table");
}

#[test]
fn test_dml_invalid_sql_errors() {
    let (mut app, _dir) = create_app("Name,Value\nAlice,10\n");

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};

    let (success, _) =
        app.execute_sql_dml_cancellable("INVALID SQL GARBAGE", &cancelled, &mut progress);

    assert!(!success, "Should fail for invalid SQL");
}

#[test]
fn test_dml_preserves_column_structure() {
    let (mut app, _dir) = create_app("A,B,C\n1,2,3\n4,5,6\n");

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};

    let (success, _) = app.execute_sql_dml_cancellable(
        "UPDATE data SET B = '99' WHERE A = '1'",
        &cancelled,
        &mut progress,
    );

    assert!(success);
    // Column count should be preserved
    assert_eq!(app.document.column_count(), 3);
    // Header should be preserved
    let header = app.document.get_rows_range(0, 1);
    assert_eq!(header[0], vec!["A", "B", "C"]);
}
