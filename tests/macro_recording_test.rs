//! Integration tests for macro recording/playback (v0.22.0).

use std::io::Write;
use tempfile::NamedTempFile;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::session::FileConfig;
use lazycsv::{App, ColIndex, Document, RowIndex};

fn create_test_app() -> App {
    let csv = "name,value,category\nAlice,100,A\nBob,200,B\nCharlie,300,C\nDana,400,D\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv.as_bytes()).unwrap();
    let path = temp_file.path().to_path_buf();
    temp_file.keep().unwrap();

    let csv_data = Document::from_file(&path, None, false, None).unwrap();
    let file_config = FileConfig::with_options(None, false, None);
    App::new(csv_data, vec![path], 0, file_config)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_keys(app: &mut App, codes: &[KeyCode]) {
    for c in codes {
        app.handle_key(key(*c)).unwrap();
    }
}

fn selected_row(app: &App) -> usize {
    app.view_state.table_state.selected().unwrap()
}

// ─────────────────────────────────────────────────────────────────
// Recording
// ─────────────────────────────────────────────────────────────────

#[test]
fn qa_starts_recording_into_register_a() {
    let mut app = create_test_app();
    type_keys(&mut app, &[KeyCode::Char('q'), KeyCode::Char('a')]);
    assert!(app.macros.is_recording());
    assert_eq!(app.macros.recording_register(), Some('a'));
}

#[test]
fn q_alone_stops_recording() {
    let mut app = create_test_app();
    type_keys(&mut app, &[KeyCode::Char('q'), KeyCode::Char('a')]);
    type_keys(&mut app, &[KeyCode::Char('j'), KeyCode::Char('q')]);
    assert!(!app.macros.is_recording());
    assert!(app.macros.get('a').is_some());
}

#[test]
fn qa_q_keys_not_in_buffer() {
    let mut app = create_test_app();
    type_keys(&mut app, &[KeyCode::Char('q'), KeyCode::Char('a')]);
    type_keys(
        &mut app,
        &[KeyCode::Char('j'), KeyCode::Char('j'), KeyCode::Char('q')],
    );
    let macro_keys = app.macros.get('a').unwrap();
    // Two `j` presses recorded; the qa-prefix and stop-q must be excluded.
    assert_eq!(macro_keys.len(), 2);
    for k in macro_keys {
        assert_eq!(k.code, KeyCode::Char('j'));
    }
}

#[test]
fn qa_with_only_stop_records_empty_macro() {
    let mut app = create_test_app();
    type_keys(
        &mut app,
        &[KeyCode::Char('q'), KeyCode::Char('a'), KeyCode::Char('q')],
    );
    assert!(!app.macros.is_recording());
    assert_eq!(app.macros.get('a').map(|s| s.len()), Some(0));
}

#[test]
fn invalid_register_does_not_start_recording() {
    let mut app = create_test_app();
    type_keys(&mut app, &[KeyCode::Char('q'), KeyCode::Char('1')]);
    assert!(!app.macros.is_recording());
    // Pending state should also be cleared (multi-key fall-through).
    assert!(!app.input_state.has_pending_command());
}

// ─────────────────────────────────────────────────────────────────
// Replay
// ─────────────────────────────────────────────────────────────────

#[test]
fn at_a_replays_macro() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(0));
    // Record `j` (move down) into register a
    type_keys(
        &mut app,
        &[
            KeyCode::Char('q'),
            KeyCode::Char('a'),
            KeyCode::Char('j'),
            KeyCode::Char('q'),
        ],
    );
    let row_after_record = selected_row(&app);
    // Replay should move us down once more.
    type_keys(&mut app, &[KeyCode::Char('@'), KeyCode::Char('a')]);
    assert_eq!(selected_row(&app), row_after_record + 1);
}

#[test]
fn at_at_replays_last_macro() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(0));
    type_keys(
        &mut app,
        &[
            KeyCode::Char('q'),
            KeyCode::Char('a'),
            KeyCode::Char('j'),
            KeyCode::Char('q'),
        ],
    );
    type_keys(&mut app, &[KeyCode::Char('@'), KeyCode::Char('a')]);
    let after_first_replay = selected_row(&app);
    type_keys(&mut app, &[KeyCode::Char('@'), KeyCode::Char('@')]);
    assert_eq!(selected_row(&app), after_first_replay + 1);
}

#[test]
fn at_at_with_no_prior_replay_shows_message() {
    let mut app = create_test_app();
    type_keys(&mut app, &[KeyCode::Char('@'), KeyCode::Char('@')]);
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("No previous macro"));
}

#[test]
fn at_unknown_register_shows_empty_message() {
    let mut app = create_test_app();
    type_keys(&mut app, &[KeyCode::Char('@'), KeyCode::Char('z')]);
    let msg = app.status_message.as_ref().unwrap().as_str();
    assert!(msg.contains("empty"));
}

// ─────────────────────────────────────────────────────────────────
// Multiple registers
// ─────────────────────────────────────────────────────────────────

#[test]
fn multiple_registers_isolated() {
    let mut app = create_test_app();
    // Record `j` into a, `k` into b
    type_keys(
        &mut app,
        &[
            KeyCode::Char('q'),
            KeyCode::Char('a'),
            KeyCode::Char('j'),
            KeyCode::Char('q'),
        ],
    );
    type_keys(
        &mut app,
        &[
            KeyCode::Char('q'),
            KeyCode::Char('b'),
            KeyCode::Char('k'),
            KeyCode::Char('q'),
        ],
    );

    let macro_a = app.macros.get('a').unwrap();
    let macro_b = app.macros.get('b').unwrap();
    assert_eq!(macro_a.len(), 1);
    assert_eq!(macro_b.len(), 1);
    assert_eq!(macro_a[0].code, KeyCode::Char('j'));
    assert_eq!(macro_b[0].code, KeyCode::Char('k'));
}

#[test]
fn rerecording_register_replaces_old_macro() {
    let mut app = create_test_app();
    type_keys(
        &mut app,
        &[
            KeyCode::Char('q'),
            KeyCode::Char('a'),
            KeyCode::Char('j'),
            KeyCode::Char('j'),
            KeyCode::Char('j'),
            KeyCode::Char('q'),
        ],
    );
    assert_eq!(app.macros.get('a').map(|s| s.len()), Some(3));

    type_keys(
        &mut app,
        &[
            KeyCode::Char('q'),
            KeyCode::Char('a'),
            KeyCode::Char('k'),
            KeyCode::Char('q'),
        ],
    );
    assert_eq!(app.macros.get('a').map(|s| s.len()), Some(1));
}

// ─────────────────────────────────────────────────────────────────
// Replay performs real edits
// ─────────────────────────────────────────────────────────────────

#[test]
fn replay_repeats_cell_edit() {
    let mut app = create_test_app();
    app.view_state.table_state.select(Some(1));
    app.view_state.selected_column = ColIndex::new(0);

    // Record: edit current cell to "X" then move down.
    // We use commit_cell_value indirectly via `cw` cell-yank → no, simplest is
    // to actually edit through `i` insert mode. But insert mode keys aren't
    // captured in normal-mode recording flow. Use `x` (delete cell content)
    // which is one keystroke and observable.
    type_keys(
        &mut app,
        &[
            KeyCode::Char('q'),
            KeyCode::Char('a'),
            KeyCode::Delete,
            KeyCode::Char('j'),
            KeyCode::Char('q'),
        ],
    );

    // Cell at row 1 should be cleared after recording.
    assert_eq!(app.document.cell(RowIndex::new(1), ColIndex::new(0)), "");
    // We're now on row 2 (due to recorded `j`).
    assert_eq!(selected_row(&app), 2);

    // Replay: clear row 2 col 0, then move to row 3.
    type_keys(&mut app, &[KeyCode::Char('@'), KeyCode::Char('a')]);
    assert_eq!(app.document.cell(RowIndex::new(2), ColIndex::new(0)), "");
    assert_eq!(selected_row(&app), 3);
}

// ─────────────────────────────────────────────────────────────────
// Pending state cleanup
// ─────────────────────────────────────────────────────────────────

#[test]
fn esc_cancels_pending_q_and_at() {
    let mut app = create_test_app();
    type_keys(&mut app, &[KeyCode::Char('q')]);
    assert!(app.input_state.has_pending_command());
    type_keys(&mut app, &[KeyCode::Esc]);
    assert!(!app.input_state.has_pending_command());
    assert!(!app.macros.is_recording());

    type_keys(&mut app, &[KeyCode::Char('@')]);
    assert!(app.input_state.has_pending_command());
    type_keys(&mut app, &[KeyCode::Esc]);
    assert!(!app.input_state.has_pending_command());
}
