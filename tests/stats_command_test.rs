use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, Document, FileConfig};
use std::path::PathBuf;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Helper to send a command and press Enter
fn send_command(app: &mut App, cmd: &str) {
    let _ = app.handle_key(key_event(KeyCode::Char(':')));
    for c in cmd.chars() {
        let _ = app.handle_key(key_event(KeyCode::Char(c)));
    }
    let _ = app.handle_key(key_event(KeyCode::Enter));
}

/// Helper to get the status message text
fn status_message(app: &App) -> String {
    app.status_message
        .as_ref()
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}

/// Create a test app with numeric data: Name, Price, Quantity
fn create_numeric_app() -> App {
    let doc = Document::new(
        vec![
            "Name".to_string(),
            "Price".to_string(),
            "Quantity".to_string(),
        ],
        vec![
            vec!["Apple".to_string(), "1.50".to_string(), "10".to_string()],
            vec!["Banana".to_string(), "0.75".to_string(), "20".to_string()],
            vec!["Cherry".to_string(), "3.00".to_string(), "5".to_string()],
            vec!["Date".to_string(), "5.25".to_string(), "15".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    App::new(doc, files, 0, FileConfig::new())
}

/// Create a test app with mixed data including empties
fn create_mixed_app() -> App {
    let doc = Document::new(
        vec![
            "Name".to_string(),
            "Value".to_string(),
            "Status".to_string(),
        ],
        vec![
            vec!["A".to_string(), "100".to_string(), "active".to_string()],
            vec!["B".to_string(), "".to_string(), "inactive".to_string()],
            vec!["C".to_string(), "200".to_string(), "active".to_string()],
            vec!["D".to_string(), "hello".to_string(), "active".to_string()],
            vec!["E".to_string(), "300".to_string(), "".to_string()],
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    App::new(doc, files, 0, FileConfig::new())
}

// ===== :sum =====

#[test]
fn test_sum_by_header_name() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum Price");
    assert_eq!(status_message(&app), "Price: sum = 10.5");
}

#[test]
fn test_sum_by_column_number() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum 3");
    assert_eq!(status_message(&app), "Quantity: sum = 50");
}

#[test]
fn test_sum_by_excel_letter() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum B");
    assert_eq!(status_message(&app), "Price: sum = 10.5");
}

#[test]
fn test_sum_non_numeric_column() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum Name");
    assert_eq!(status_message(&app), "Name: no numeric values");
}

#[test]
fn test_sum_with_empty_values() {
    let mut app = create_mixed_app();
    send_command(&mut app, "sum Value");
    // Only 100, 200, 300 are numeric (empty and "hello" are skipped)
    assert_eq!(status_message(&app), "Value: sum = 600");
}

#[test]
fn test_sum_no_argument() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum");
    assert!(status_message(&app).starts_with("Usage: :sum"));
}

// ===== :avg =====

#[test]
fn test_avg_by_header_name() {
    let mut app = create_numeric_app();
    send_command(&mut app, "avg Price");
    assert_eq!(status_message(&app), "Price: avg = 2.625 (4 values)");
}

#[test]
fn test_avg_integer_column() {
    let mut app = create_numeric_app();
    send_command(&mut app, "avg Quantity");
    assert_eq!(status_message(&app), "Quantity: avg = 12.5 (4 values)");
}

#[test]
fn test_avg_with_mixed_values() {
    let mut app = create_mixed_app();
    send_command(&mut app, "avg Value");
    // 100, 200, 300 → avg = 200
    assert_eq!(status_message(&app), "Value: avg = 200 (3 values)");
}

#[test]
fn test_average_alias() {
    let mut app = create_numeric_app();
    send_command(&mut app, "average Price");
    assert_eq!(status_message(&app), "Price: avg = 2.625 (4 values)");
}

// ===== :count =====

#[test]
fn test_count_full_column() {
    let mut app = create_numeric_app();
    send_command(&mut app, "count Name");
    assert_eq!(status_message(&app), "Name: 4 non-empty / 4 total");
}

#[test]
fn test_count_with_empty_values() {
    let mut app = create_mixed_app();
    send_command(&mut app, "count Value");
    // Row B has empty Value, rest have values
    assert_eq!(status_message(&app), "Value: 4 non-empty / 5 total");
}

#[test]
fn test_count_status_with_empty() {
    let mut app = create_mixed_app();
    send_command(&mut app, "count Status");
    // Row E has empty Status
    assert_eq!(status_message(&app), "Status: 4 non-empty / 5 total");
}

// ===== :distinct =====

#[test]
fn test_distinct_unique_values() {
    let mut app = create_numeric_app();
    send_command(&mut app, "distinct Name");
    assert_eq!(status_message(&app), "Name: 4 distinct values");
}

#[test]
fn test_distinct_with_duplicates() {
    let mut app = create_mixed_app();
    send_command(&mut app, "distinct Status");
    // "active" (×3), "inactive" (×1), "" (×1, not counted)
    assert_eq!(status_message(&app), "Status: 2 distinct values");
}

// ===== :stats =====

#[test]
fn test_stats_numeric_column() {
    let mut app = create_numeric_app();
    send_command(&mut app, "stats Quantity");
    let msg = status_message(&app);
    assert!(msg.contains("sum=50"), "expected sum=50, got: {}", msg);
    assert!(msg.contains("avg=12.5"), "expected avg=12.5, got: {}", msg);
    assert!(msg.contains("min=5"), "expected min=5, got: {}", msg);
    assert!(msg.contains("max=20"), "expected max=20, got: {}", msg);
    assert!(msg.contains("count=4/4"), "expected count=4/4, got: {}", msg);
    assert!(
        msg.contains("distinct=4"),
        "expected distinct=4, got: {}",
        msg
    );
}

#[test]
fn test_stats_text_column() {
    let mut app = create_mixed_app();
    send_command(&mut app, "stats Status");
    let msg = status_message(&app);
    // Text column — no sum/avg/min/max, just count and distinct
    assert!(
        msg.contains("count=4/5"),
        "expected count=4/5, got: {}",
        msg
    );
    assert!(
        msg.contains("distinct=2"),
        "expected distinct=2, got: {}",
        msg
    );
    assert!(!msg.contains("sum="), "should not have sum for text column");
}

#[test]
fn test_stats_mixed_column() {
    let mut app = create_mixed_app();
    send_command(&mut app, "stats Value");
    let msg = status_message(&app);
    // 100, 200, 300 are numeric; empty and "hello" are not
    assert!(msg.contains("sum=600"), "expected sum=600, got: {}", msg);
    assert!(msg.contains("avg=200"), "expected avg=200, got: {}", msg);
    assert!(msg.contains("min=100"), "expected min=100, got: {}", msg);
    assert!(msg.contains("max=300"), "expected max=300, got: {}", msg);
}

// ===== Column resolution =====

#[test]
fn test_stats_case_insensitive_header() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum price");
    assert_eq!(status_message(&app), "Price: sum = 10.5");
}

#[test]
fn test_stats_invalid_column() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum NonExistent");
    assert_eq!(
        status_message(&app),
        "Column \"NonExistent\" not found"
    );
}

#[test]
fn test_stats_column_out_of_range() {
    let mut app = create_numeric_app();
    send_command(&mut app, "sum 99");
    assert!(status_message(&app).contains("out of range"));
}
