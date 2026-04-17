//! Integration tests for `:` command history (v0.22.0).

use std::io::Write;
use tempfile::NamedTempFile;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::session::FileConfig;
use lazycsv::{App, Document};

fn create_test_app() -> App {
    let csv = "a,b,c\n1,2,3\n4,5,6\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv.as_bytes()).unwrap();
    let path = temp_file.path().to_path_buf();
    temp_file.keep().unwrap();
    let csv_data = Document::from_file(&path, None, false, None).unwrap();
    App::new(
        csv_data,
        vec![path],
        0,
        FileConfig::with_options(None, false, None),
    )
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_string(app: &mut App, s: &str) {
    for c in s.chars() {
        app.handle_key(key(KeyCode::Char(c))).unwrap();
    }
}

/// Run a `:`-command end to end: enter command mode, type the command (without
/// the leading `:`), press Enter.
fn run_command(app: &mut App, cmd: &str) {
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    type_string(app, cmd);
    app.handle_key(key(KeyCode::Enter)).unwrap();
}

// ─────────────────────────────────────────────────────────────────
// push_command_history
// ─────────────────────────────────────────────────────────────────

#[test]
fn successful_command_appended_to_history() {
    let mut app = create_test_app();
    run_command(&mut app, "noh");
    assert_eq!(app.command_history.len(), 1);
    assert_eq!(app.command_history[0], "noh");
}

#[test]
fn unknown_command_still_appended_to_history() {
    // vim records all entered commands regardless of success — useful for retry.
    let mut app = create_test_app();
    run_command(&mut app, "xyzzy");
    assert_eq!(app.command_history[0], "xyzzy");
}

#[test]
fn empty_command_not_recorded() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    app.handle_key(key(KeyCode::Enter)).unwrap();
    assert!(app.command_history.is_empty());
}

#[test]
fn duplicate_command_moved_to_front() {
    let mut app = create_test_app();
    run_command(&mut app, "noh");
    run_command(&mut app, "footer");
    run_command(&mut app, "noh"); // duplicate
    assert_eq!(app.command_history.len(), 2);
    assert_eq!(app.command_history[0], "noh");
    assert_eq!(app.command_history[1], "footer");
}

#[test]
fn most_recent_command_is_index_0() {
    let mut app = create_test_app();
    run_command(&mut app, "first");
    run_command(&mut app, "second");
    run_command(&mut app, "third");
    assert_eq!(app.command_history[0], "third");
    assert_eq!(app.command_history[2], "first");
}

#[test]
fn history_capped_to_limit() {
    let mut app = create_test_app();
    app.config.defaults.command_history_limit = 3;
    for cmd in ["a1", "a2", "a3", "a4", "a5"] {
        run_command(&mut app, cmd);
    }
    assert_eq!(app.command_history.len(), 3);
    assert_eq!(app.command_history[0], "a5");
    assert_eq!(app.command_history[2], "a3");
}

#[test]
fn limit_zero_disables_history() {
    let mut app = create_test_app();
    app.config.defaults.command_history_limit = 0;
    run_command(&mut app, "noh");
    assert!(app.command_history.is_empty());
}

// ─────────────────────────────────────────────────────────────────
// Up/Down navigation
// ─────────────────────────────────────────────────────────────────

#[test]
fn up_arrow_recalls_previous_command() {
    let mut app = create_test_app();
    run_command(&mut app, "noh");
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "noh");
}

#[test]
fn up_arrow_walks_back_through_history() {
    let mut app = create_test_app();
    run_command(&mut app, "first");
    run_command(&mut app, "second");
    run_command(&mut app, "third");
    app.handle_key(key(KeyCode::Char(':'))).unwrap();

    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "third");
    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "second");
    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "first");
    // At oldest — stays put
    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "first");
}

#[test]
fn down_arrow_walks_forward_and_restores_pending() {
    let mut app = create_test_app();
    run_command(&mut app, "alpha");
    run_command(&mut app, "beta");

    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    type_string(&mut app, "wip");
    app.handle_key(key(KeyCode::Up)).unwrap(); // beta
    app.handle_key(key(KeyCode::Up)).unwrap(); // alpha
    app.handle_key(key(KeyCode::Down)).unwrap(); // beta
    assert_eq!(app.input_state.command_buffer, "beta");
    app.handle_key(key(KeyCode::Down)).unwrap(); // back to "wip"
    assert_eq!(app.input_state.command_buffer, "wip");
}

#[test]
fn down_with_no_navigation_in_progress_is_noop() {
    let mut app = create_test_app();
    run_command(&mut app, "alpha");
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    type_string(&mut app, "fresh");
    app.handle_key(key(KeyCode::Down)).unwrap();
    assert_eq!(app.input_state.command_buffer, "fresh");
}

#[test]
fn typing_after_recall_invalidates_navigation() {
    let mut app = create_test_app();
    run_command(&mut app, "alpha");
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "alpha");
    type_string(&mut app, "x");
    assert_eq!(app.input_state.command_buffer, "alphax");
    assert!(app.command_history_index.is_none());
}

#[test]
fn history_index_reset_on_re_entry() {
    let mut app = create_test_app();
    run_command(&mut app, "alpha");
    run_command(&mut app, "beta");
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    app.handle_key(key(KeyCode::Up)).unwrap();
    app.handle_key(key(KeyCode::Esc)).unwrap();
    // Re-enter — index should reset, first Up goes to most recent again.
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "beta");
}

#[test]
fn up_with_empty_history_is_noop() {
    let mut app = create_test_app();
    app.handle_key(key(KeyCode::Char(':'))).unwrap();
    app.handle_key(key(KeyCode::Up)).unwrap();
    assert_eq!(app.input_state.command_buffer, "");
}

// ─────────────────────────────────────────────────────────────────
// :history command
// ─────────────────────────────────────────────────────────────────

#[test]
fn history_command_lists_recent_entries() {
    let mut app = create_test_app();
    run_command(&mut app, "first");
    run_command(&mut app, "second");
    run_command(&mut app, "history");
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("History"));
    assert!(msg.contains("first"));
    assert!(msg.contains("second"));
}

#[test]
fn history_command_with_empty_history_shows_message() {
    let mut app = create_test_app();
    run_command(&mut app, "history");
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("empty"));
}

// ─────────────────────────────────────────────────────────────────
// Persistence (load/save roundtrip)
// ─────────────────────────────────────────────────────────────────

#[test]
fn save_and_load_roundtrip() {
    use lazycsv::config::{load_command_history, save_command_history};

    // Stash anything currently on disk so the test doesn't clobber real history.
    let path = lazycsv::config::command_history_path().unwrap();
    let backup = std::fs::read(&path).ok();

    let history = vec![
        "noh".to_string(),
        "footer".to_string(),
        "sort A".to_string(),
    ];
    save_command_history(&history, 50);

    let loaded = load_command_history();
    assert_eq!(loaded, history);

    // Restore previous content (or remove if there was none).
    match backup {
        Some(bytes) => {
            std::fs::write(&path, bytes).unwrap();
        }
        None => {
            let _ = std::fs::remove_file(&path);
        }
    }
}
