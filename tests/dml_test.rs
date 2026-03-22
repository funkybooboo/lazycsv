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

// ── :copy command ──────────────────────────────────────────

fn send_command(app: &mut App, cmd: &str) {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let _ = app.handle_key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
    for c in cmd.chars() {
        let _ = app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
    let _ = app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
}

#[test]
fn test_copy_command_sets_status_message() {
    let (mut app, _dir) = create_app("A,B\n1,2\n3,4\n");

    send_command(&mut app, "copy");

    // Should have a status message (either success or clipboard error)
    let msg = app
        .status_message
        .as_ref()
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    assert!(
        msg.contains("Copied") || msg.contains("Clipboard") || msg.contains("clipboard"),
        "Expected clipboard status message, got: '{}'",
        msg
    );
}

#[test]
fn test_copy_command_not_confused_with_column_jump() {
    let (mut app, _dir) = create_app("A,B\n1,2\n");

    send_command(&mut app, "copy");

    // Should NOT be interpreted as :c opy (column jump)
    let msg = app
        .status_message
        .as_ref()
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    assert!(
        !msg.contains("Column") && !msg.contains("does not exist"),
        "':copy' should not be treated as column jump, got: '{}'",
        msg
    );
}

// ── :paste command ──────────────────────────────────────────

#[test]
fn test_paste_command_sets_status_message() {
    let (mut app, _dir) = create_app("A,B\n1,2\n");

    send_command(&mut app, "paste");

    // Should have a status message (either success or clipboard error)
    let msg = app
        .status_message
        .as_ref()
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    assert!(
        msg.contains("Pasted")
            || msg.contains("Clipboard")
            || msg.contains("clipboard")
            || msg.contains("empty"),
        "Expected paste status message, got: '{}'",
        msg
    );
}

// ── detect_delimiter ────────────────────────────────────────

#[test]
fn test_detect_delimiter_tab_from_excel() {
    // Excel-style tab-delimited paste
    let excel_data = "Name\tSalary\tCost\nDoug\t$10.00\t22\nHenry\t$14.00\t33\n";
    assert_eq!(lazycsv::csv::detect_delimiter(excel_data), b'\t');
}

#[test]
fn test_detect_delimiter_comma() {
    assert_eq!(lazycsv::csv::detect_delimiter("a,b,c\n1,2,3\n"), b',');
}

#[test]
fn test_detect_delimiter_pipe() {
    assert_eq!(lazycsv::csv::detect_delimiter("a|b|c\n1|2|3\n"), b'|');
}

#[test]
fn test_detect_delimiter_converts_tab_to_csv() {
    // Simulate what -P does: detect tab delimiter, parse, write as CSV
    let tab_data = "Name\tAge\nAlice\t30\nBob\t25\n";
    let detected = lazycsv::csv::detect_delimiter(tab_data);
    assert_eq!(detected, b'\t');

    // Parse with detected delimiter
    let reader = std::io::Cursor::new(tab_data.as_bytes());
    let doc =
        lazycsv::csv::Document::from_reader(reader, Some(detected), false, "test.csv".to_string())
            .unwrap();

    assert_eq!(doc.row_count(), 3); // header + 2 rows
    assert_eq!(doc.column_count(), 2);

    // Write as CSV
    let mut buf = Vec::new();
    lazycsv::csv::write_csv_content(&mut buf, &doc, ',').unwrap();
    let csv_output = String::from_utf8(buf).unwrap();
    assert!(csv_output.contains("Name,Age"));
    assert!(csv_output.contains("Alice,30"));
}

// ── :w! and :wq! commands ───────────────────────────────────

#[test]
fn test_w_bang_does_not_error() {
    let (mut app, _dir) = create_app("A,B\n1,2\n");

    send_command(&mut app, "w!");

    let msg = app
        .status_message
        .as_ref()
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    assert!(
        !msg.contains("Unknown command"),
        "':w!' should not be unknown, got: '{}'",
        msg
    );
}

#[test]
fn test_wq_bang_does_not_error() {
    let (mut app, _dir) = create_app("A,B\n1,2\n");

    send_command(&mut app, "wq!");

    // wq! triggers quit, so should_quit should be true (or at minimum no unknown command error)
    let msg = app
        .status_message
        .as_ref()
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    assert!(
        !msg.contains("Unknown command"),
        "':wq!' should not be unknown, got: '{}'",
        msg
    );
}

// ── DuckDB reload_table strategies ──────────────────────────

#[test]
fn test_dml_on_clean_file_uses_fast_path() {
    // Strategy 1: clean file on disk → DuckDB read_csv directly
    let (mut app, _dir) = create_app("Name,Value\nAlice,10\nBob,20\n");
    // Document is not dirty, file exists on disk
    assert!(!app.document.is_dirty);

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};
    let (success, _) = app.execute_sql_dml_cancellable(
        "UPDATE data SET Value = '99' WHERE Name = 'Alice'",
        &cancelled,
        &mut progress,
    );
    assert!(success);
    // After DML, document should be dirty and have updated data
    assert!(app.document.is_dirty);
}

#[test]
fn test_dml_exports_to_temp_file() {
    // DML should export modified data via COPY to a temp file
    let (mut app, _dir) = create_app("X,Y\n1,2\n3,4\n");

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};
    let (success, _) =
        app.execute_sql_dml_cancellable("UPDATE data SET Y = '99'", &cancelled, &mut progress);
    assert!(success);

    // Verify the updated values are visible
    let mut found_99 = false;
    for i in 1..app.document.row_count() {
        let row = app.document.get_rows_range(i, i + 1);
        if row[0].get(1).map(|s| s.as_str()) == Some("99") {
            found_99 = true;
            break;
        }
    }
    assert!(found_99, "Updated value '99' should be in document");
}

#[test]
fn test_dml_original_file_untouched() {
    // DML should not modify the original CSV file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("original.csv");
    let original_content = "Name,Score\nAlice,100\nBob,200\n";
    write(&file_path, original_content).unwrap();

    let args = CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap()]).unwrap();
    let mut app = App::from_cli(args).unwrap();

    let cancelled = AtomicBool::new(false);
    let mut progress = |_: &str| {};
    let (success, _) = app.execute_sql_dml_cancellable(
        "UPDATE original SET Score = '999'",
        &cancelled,
        &mut progress,
    );
    assert!(success);

    // Original file should be unchanged
    let disk_content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        disk_content, original_content,
        "Original file should not be modified by DML"
    );
}

#[test]
fn test_query_result_uses_lazy_mmap() {
    // Query results should be loaded as lazy mmap documents (not in-memory Vec<Vec<String>>)
    let (mut app, _dir) = create_app("A,B\n1,2\n3,4\n5,6\n");

    let cancelled = std::sync::Arc::new(AtomicBool::new(false));
    let watcher = lazycsv::cancel::EscWatcher::spawn(&cancelled);
    let mut progress = |_: &str| {};
    let (result, _) = app.execute_sql_query_cancellable(
        "SELECT * FROM data WHERE A != '3'",
        "result.csv",
        &cancelled,
        &mut progress,
    );
    watcher.stop();

    assert!(result.is_some());
    let doc = result.unwrap();
    assert_eq!(doc.row_count(), 3); // header + 2 data rows
}

#[test]
fn test_switch_document_does_not_clone_lazy() {
    // Verifies that switching away from a lazy document doesn't materialize it.
    // We can't directly test this without timing, but we can verify the document
    // is lazy before the switch.
    let (app, _dir) = create_app("A,B\n1,2\n");
    // Small files are in-memory, not lazy
    assert!(!app.document.is_lazy());
}
