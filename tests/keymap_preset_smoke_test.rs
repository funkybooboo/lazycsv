//! End-to-end smoke tests for the three shipped keymap presets.
//!
//! These tests exercise the full path that runs at startup: parse the
//! preset TOML → build a [`Keymap`] → install it on a real `App` → fire
//! representative keypresses → assert the right behaviour ran.
//!
//! Where unit tests in `src/config/keys.rs` only verify parsing, these
//! tests verify the *integration*: keymap → dispatcher → handler →
//! observable app state change.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::{App, Mode};
use lazycsv::config::keys::{Keymap, KeymapToml};
use lazycsv::csv::Document;
use lazycsv::session::FileConfig;
use std::path::PathBuf;

const VIM_TOML: &str = include_str!("../keymaps/vim.toml");
const EMACS_TOML: &str = include_str!("../keymaps/emacs.toml");
const EXCEL_TOML: &str = include_str!("../keymaps/excel.toml");

fn make_app_with_preset(preset_toml: &str) -> App {
    let document = Document::new(
        vec!["A".into(), "B".into(), "C".into()],
        vec![
            vec!["a1".into(), "b1".into(), "c1".into()],
            vec!["a2".into(), "b2".into(), "c2".into()],
            vec!["a3".into(), "b3".into(), "c3".into()],
        ],
        "test.csv".into(),
    );
    let mut app = App::new(
        document,
        vec![PathBuf::from("test.csv")],
        0,
        FileConfig::default(),
    );
    let toml: KeymapToml = toml::from_str(preset_toml).expect("preset must parse");
    let mut warnings = Vec::new();
    app.keymap = Keymap::from_toml(&toml, &mut warnings);
    assert!(
        warnings.is_empty(),
        "preset produced warnings: {:?}",
        warnings
    );
    app
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// ── vim preset ───────────────────────────────────────────────────────

#[test]
fn vim_j_moves_down() {
    let mut app = make_app_with_preset(VIM_TOML);
    let start = app.view_state.table_state.selected().unwrap_or(0);
    app.handle_key(key(KeyCode::Char('j'), KeyModifiers::NONE))
        .unwrap();
    let end = app.view_state.table_state.selected().unwrap_or(0);
    assert_eq!(end, start + 1, "vim's `j` should move cursor down");
}

#[test]
fn vim_gg_jumps_to_first_row() {
    let mut app = make_app_with_preset(VIM_TOML);
    // Move down a few rows first.
    app.handle_key(key(KeyCode::Char('j'), KeyModifiers::NONE))
        .unwrap();
    app.handle_key(key(KeyCode::Char('j'), KeyModifiers::NONE))
        .unwrap();
    assert!(app.view_state.table_state.selected().unwrap_or(0) > 0);

    // gg → row 0.
    app.handle_key(key(KeyCode::Char('g'), KeyModifiers::NONE))
        .unwrap();
    app.handle_key(key(KeyCode::Char('g'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.view_state.table_state.selected().unwrap_or(0), 0);
}

#[test]
fn vim_i_enters_insert_mode() {
    let mut app = make_app_with_preset(VIM_TOML);
    assert_eq!(app.mode, Mode::Normal);
    app.handle_key(key(KeyCode::Char('i'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.mode, Mode::Insert);
}

#[test]
fn vim_colon_enters_command_mode() {
    let mut app = make_app_with_preset(VIM_TOML);
    app.handle_key(key(KeyCode::Char(':'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.mode, Mode::Command);
}

// ── emacs preset ─────────────────────────────────────────────────────

#[test]
fn emacs_ctrl_n_moves_down() {
    let mut app = make_app_with_preset(EMACS_TOML);
    let start = app.view_state.table_state.selected().unwrap_or(0);
    app.handle_key(key(KeyCode::Char('n'), KeyModifiers::CONTROL))
        .unwrap();
    let end = app.view_state.table_state.selected().unwrap_or(0);
    assert_eq!(end, start + 1, "emacs's Ctrl-n should move cursor down");
}

#[test]
fn emacs_ctrl_p_moves_up() {
    let mut app = make_app_with_preset(EMACS_TOML);
    // Move down first.
    app.handle_key(key(KeyCode::Char('n'), KeyModifiers::CONTROL))
        .unwrap();
    app.handle_key(key(KeyCode::Char('n'), KeyModifiers::CONTROL))
        .unwrap();
    let mid = app.view_state.table_state.selected().unwrap_or(0);
    assert!(mid > 0);

    app.handle_key(key(KeyCode::Char('p'), KeyModifiers::CONTROL))
        .unwrap();
    let end = app.view_state.table_state.selected().unwrap_or(0);
    assert_eq!(end, mid - 1, "emacs's Ctrl-p should move cursor up");
}

#[test]
fn emacs_ctrl_a_jumps_to_first_column() {
    let mut app = make_app_with_preset(EMACS_TOML);
    // Move right first.
    app.handle_key(key(KeyCode::Char('f'), KeyModifiers::CONTROL))
        .unwrap();
    app.handle_key(key(KeyCode::Char('f'), KeyModifiers::CONTROL))
        .unwrap();
    assert!(app.view_state.selected_column.get() > 0);

    app.handle_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL))
        .unwrap();
    assert_eq!(app.view_state.selected_column.get(), 0);
}

#[test]
fn emacs_alt_x_enters_command_mode() {
    let mut app = make_app_with_preset(EMACS_TOML);
    app.handle_key(key(KeyCode::Char('x'), KeyModifiers::ALT))
        .unwrap();
    assert_eq!(
        app.mode,
        Mode::Command,
        "emacs's M-x should enter command mode"
    );
}

#[test]
fn emacs_inherited_vim_chord_still_works() {
    // emacs.toml inherits from vim.toml, so vim's `gg` should still fire.
    let mut app = make_app_with_preset(EMACS_TOML);
    app.handle_key(key(KeyCode::Char('n'), KeyModifiers::CONTROL))
        .unwrap();
    app.handle_key(key(KeyCode::Char('n'), KeyModifiers::CONTROL))
        .unwrap();
    assert!(app.view_state.table_state.selected().unwrap_or(0) > 0);

    app.handle_key(key(KeyCode::Char('g'), KeyModifiers::NONE))
        .unwrap();
    app.handle_key(key(KeyCode::Char('g'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.view_state.table_state.selected().unwrap_or(0), 0);
}

// ── excel preset ─────────────────────────────────────────────────────

#[test]
fn excel_arrow_down_moves_down() {
    let mut app = make_app_with_preset(EXCEL_TOML);
    let start = app.view_state.table_state.selected().unwrap_or(0);
    app.handle_key(key(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    let end = app.view_state.table_state.selected().unwrap_or(0);
    assert_eq!(end, start + 1, "excel's <down> should move cursor down");
}

#[test]
fn excel_arrow_right_advances_column() {
    let mut app = make_app_with_preset(EXCEL_TOML);
    let start = app.view_state.selected_column.get();
    app.handle_key(key(KeyCode::Right, KeyModifiers::NONE))
        .unwrap();
    let end = app.view_state.selected_column.get();
    assert_eq!(end, start + 1);
}

#[test]
fn excel_f2_enters_edit_mode() {
    let mut app = make_app_with_preset(EXCEL_TOML);
    assert_eq!(app.mode, Mode::Normal);
    app.handle_key(key(KeyCode::F(2), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(app.mode, Mode::Insert);
}

#[test]
fn excel_letter_i_does_not_enter_insert_mode() {
    // excel.toml unbinds `i` so the user can type the letter.
    let mut app = make_app_with_preset(EXCEL_TOML);
    app.handle_key(key(KeyCode::Char('i'), KeyModifiers::NONE))
        .unwrap();
    assert_eq!(
        app.mode,
        Mode::Normal,
        "excel preset should NOT enter insert mode on bare `i`"
    );
}

#[test]
fn excel_enter_dives_into_edit_mode() {
    let mut app = make_app_with_preset(EXCEL_TOML);
    app.handle_key(key(KeyCode::Enter, KeyModifiers::NONE))
        .unwrap();
    assert_eq!(
        app.mode,
        Mode::Insert,
        "excel preset's <enter> should enter edit mode"
    );
}

#[test]
fn excel_ctrl_home_jumps_to_first_row() {
    let mut app = make_app_with_preset(EXCEL_TOML);
    app.handle_key(key(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    app.handle_key(key(KeyCode::Down, KeyModifiers::NONE))
        .unwrap();
    assert!(app.view_state.table_state.selected().unwrap_or(0) > 0);

    app.handle_key(key(KeyCode::Home, KeyModifiers::CONTROL))
        .unwrap();
    assert_eq!(app.view_state.table_state.selected().unwrap_or(0), 0);
}

#[test]
fn excel_ctrl_end_jumps_to_last_row() {
    let mut app = make_app_with_preset(EXCEL_TOML);
    app.handle_key(key(KeyCode::End, KeyModifiers::CONTROL))
        .unwrap();
    let last = app.view_state.table_state.selected().unwrap_or(0);
    assert_eq!(
        last,
        app.document.row_count().saturating_sub(1),
        "Ctrl-End should jump to the last row"
    );
}
