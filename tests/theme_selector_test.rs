//! Integration tests for the theme selector feature.
//!
//! Tests cover: keyboard navigation (j/k/g/G/PgUp/PgDn), Enter to apply,
//! Escape to cancel, scroll_selection, and theme display name formatting.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::app::Mode;
use lazycsv::config::{apply_theme_from_file, config_to_toml_string, Config};
use lazycsv::{App, Document, FileConfig};
use std::fs;
use tempfile::TempDir;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn key_event_shift_char(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::SHIFT)
}

fn create_test_app() -> (TempDir, App) {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    fs::write(&csv_path, "A,B\n1,2\n").unwrap();
    let doc = Document::from_file(&csv_path, None, false, None).unwrap();
    let files = vec![csv_path.clone()];
    let app = App::new(doc, files, 0, FileConfig::new());
    (temp_dir, app)
}

fn enter_theme_selector(app: &mut App) {
    let _ = app.handle_key(key_event(KeyCode::Char(' ')));
    let _ = app.handle_key(key_event(KeyCode::Char('t')));
}

/// Enter theme selector, then replace scan_themes() results with an empty list
/// so navigation tests have deterministic behaviour regardless of whether the
/// project's themes/ directory is on the search path.
fn enter_theme_selector_empty_list(app: &mut App) {
    enter_theme_selector(app);
    app.theme_list.clear();
}

/// Enter theme selector and inject a controlled set of fake theme entries.
fn enter_theme_selector_with_n_themes(app: &mut App, n: usize) {
    enter_theme_selector(app);
    app.theme_list = (0..n)
        .map(|i| {
            (
                format!("Theme {}", i),
                std::path::PathBuf::from(format!("/tmp/t{}.toml", i)),
            )
        })
        .collect();
    app.theme_selector_index = 0;
}

// ── Theme selector mode transition tests ──────────────────────────

#[test]
fn test_enter_theme_selector_mode() {
    let (_, mut app) = create_test_app();
    assert_eq!(app.mode, Mode::Normal);
    enter_theme_selector(&mut app);
    assert_eq!(app.mode, Mode::ThemeSelector);
}

#[test]
fn test_escape_exits_theme_selector() {
    let (_, mut app) = create_test_app();
    enter_theme_selector(&mut app);
    assert_eq!(app.mode, Mode::ThemeSelector);
    let _ = app.handle_key(key_event(KeyCode::Esc));
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn test_enter_applies_theme_and_exits() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    assert_eq!(app.mode, Mode::ThemeSelector);
    let _ = app.handle_key(key_event(KeyCode::Enter));
    assert_eq!(app.mode, Mode::Normal);
}

// ── Navigation with empty list ───────────────────────────────────────

#[test]
fn test_j_does_not_move_past_end_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_k_does_not_move_past_start_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_page_down_does_not_move_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event(KeyCode::PageDown));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_page_up_does_not_move_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event(KeyCode::PageUp));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_shift_g_stays_at_zero_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event_shift_char('G'));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_end_stays_at_zero_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event(KeyCode::End));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_down_arrow_does_not_move_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event(KeyCode::Down));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_up_arrow_does_not_move_with_empty_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_empty_list(&mut app);
    let _ = app.handle_key(key_event(KeyCode::Up));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_g_goes_to_top_regardless_of_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 5);
    app.theme_selector_index = 3;
    let _ = app.handle_key(key_event(KeyCode::Char('g')));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_home_goes_to_top_regardless_of_list() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 5);
    app.theme_selector_index = 3;
    let _ = app.handle_key(key_event(KeyCode::Home));
    assert_eq!(app.theme_selector_index, 0);
}

// ── Navigation with themes in the list ─────────────────────────────

#[test]
fn test_j_k_navigation_with_themes() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 3);

    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.theme_selector_index, 1);

    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.theme_selector_index, 2);

    // Can't go past the end
    let _ = app.handle_key(key_event(KeyCode::Char('j')));
    assert_eq!(app.theme_selector_index, 2);

    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.theme_selector_index, 1);

    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.theme_selector_index, 0);

    // Can't go before the start
    let _ = app.handle_key(key_event(KeyCode::Char('k')));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_shift_g_goes_to_last_item() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 3);

    let _ = app.handle_key(key_event_shift_char('G'));
    assert_eq!(app.theme_selector_index, 2);
}

#[test]
fn test_end_goes_to_last_item() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 3);

    let _ = app.handle_key(key_event(KeyCode::End));
    assert_eq!(app.theme_selector_index, 2);
}

#[test]
fn test_page_down_moves_10() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 20);

    let _ = app.handle_key(key_event(KeyCode::PageDown));
    assert_eq!(app.theme_selector_index, 10);

    let _ = app.handle_key(key_event(KeyCode::PageDown));
    assert_eq!(app.theme_selector_index, 19); // clamped to max

    let _ = app.handle_key(key_event(KeyCode::PageUp));
    assert_eq!(app.theme_selector_index, 9);
}

#[test]
fn test_page_up_from_start_stays_at_zero() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 5);

    let _ = app.handle_key(key_event(KeyCode::PageUp));
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_home_end_keys_with_themes() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 2);
    app.theme_selector_index = 1;

    let _ = app.handle_key(key_event(KeyCode::Home));
    assert_eq!(app.theme_selector_index, 0);

    let _ = app.handle_key(key_event(KeyCode::End));
    assert_eq!(app.theme_selector_index, 1);
}

// ── Applying a real theme file ────────────────────────────────────

#[test]
fn test_apply_theme_from_file_changes_config() {
    let dir = tempfile::tempdir().unwrap();
    let theme_path = dir.path().join("test_theme.toml");
    fs::write(
        &theme_path,
        r##"
[ui]
fg = "#ebdbb2"
bg = "#282828"

[table]
cursor_bg = "#d79921"
cursor_fg = "#282828"
header_fg = "#fabd2f"
"##,
    )
    .unwrap();

    let mut config = Config::default();
    let mut warnings = Vec::new();
    let result = apply_theme_from_file(&mut config, &theme_path, &mut warnings);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
    assert!(warnings.is_empty());
    assert_eq!(
        config.theme.ui.fg,
        ratatui::style::Color::Rgb(235, 219, 178)
    );
    assert_eq!(config.theme.ui.bg, ratatui::style::Color::Rgb(40, 40, 40));
    assert_eq!(
        config.theme.table.cursor_bg,
        ratatui::style::Color::Rgb(215, 153, 33)
    );
}

#[test]
fn test_apply_nonexistent_theme_returns_none() {
    let mut config = Config::default();
    let mut warnings = Vec::new();
    let result = apply_theme_from_file(
        &mut config,
        std::path::Path::new("/nonexistent/theme.toml"),
        &mut warnings,
    );
    assert!(matches!(result, Ok(None)));
}

#[test]
fn test_apply_theme_preserves_unspecified_fields() {
    let dir = tempfile::tempdir().unwrap();
    let theme_path = dir.path().join("partial.toml");
    fs::write(
        &theme_path,
        r##"
[ui]
fg = "red"
"##,
    )
    .unwrap();

    let mut config = Config::default();
    let mut warnings = Vec::new();
    let result = apply_theme_from_file(&mut config, &theme_path, &mut warnings);
    assert!(result.is_ok());
    assert_eq!(config.theme.ui.fg, ratatui::style::Color::Red);
    assert_eq!(config.theme.ui.bg, ratatui::style::Color::Reset);
    assert_eq!(config.theme.table.cursor_bg, ratatui::style::Color::White);
}

// ── Config serialization ────────────────────────────────────────────

#[test]
fn test_config_to_toml_string_contains_all_sections() {
    let config = Config::default();
    let toml_str = config_to_toml_string(&config).unwrap();

    assert!(toml_str.contains("[defaults]"));
    assert!(toml_str.contains("[ui]"));
    assert!(toml_str.contains("[table]"));
    assert!(toml_str.contains("[popup]"));
    assert!(toml_str.contains("[status]"));
    assert!(toml_str.contains("[file_menu]"));
    assert!(toml_str.contains("[sql]"));
}

#[test]
fn test_config_to_toml_string_roundtrip_via_apply() {
    let mut config = Config::default();
    config.theme.ui.fg = ratatui::style::Color::Rgb(255, 0, 0);
    config.theme.ui.bg = ratatui::style::Color::Rgb(0, 255, 0);
    config.theme.table.cursor_bg = ratatui::style::Color::Rgb(215, 153, 33);

    let toml_str = config_to_toml_string(&config).unwrap();

    // Write to a temp file and apply it to a fresh config via apply_theme_from_file
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("roundtrip.toml");
    fs::write(&path, &toml_str).unwrap();

    let mut restored = Config::default();
    let mut warnings = Vec::new();
    let result = apply_theme_from_file(&mut restored, &path, &mut warnings);
    assert!(result.is_ok());
    assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);

    assert_eq!(restored.theme.ui.fg, ratatui::style::Color::Rgb(255, 0, 0));
    assert_eq!(restored.theme.ui.bg, ratatui::style::Color::Rgb(0, 255, 0));
    assert_eq!(
        restored.theme.table.cursor_bg,
        ratatui::style::Color::Rgb(215, 153, 33)
    );
}

// ── Theme display name ─────────────────────────────────────────────

#[test]
fn test_theme_display_name_formatting() {
    use lazycsv::ui::theme_selector::theme_display_name;
    assert_eq!(theme_display_name("gruvbox-dark"), "Gruvbox Dark");
    assert_eq!(theme_display_name("catppuccin-mocha"), "Catppuccin Mocha");
    assert_eq!(theme_display_name("solarized-light"), "Solarized Light");
    assert_eq!(theme_display_name("nord"), "Nord");
    assert_eq!(theme_display_name("dracula"), "Dracula");
    assert_eq!(theme_display_name("tokyo-night"), "Tokyo Night");
}

// ── Theme selector mode state ──────────────────────────────────────

#[test]
fn test_theme_selector_index_resets_on_entry() {
    let (_, mut app) = create_test_app();
    enter_theme_selector(&mut app);
    assert_eq!(app.theme_selector_index, 0);
}

#[test]
fn test_unknown_keys_ignored_in_theme_selector() {
    let (_, mut app) = create_test_app();
    enter_theme_selector_with_n_themes(&mut app, 2);
    app.theme_selector_index = 1;

    let _ = app.handle_key(key_event(KeyCode::Char('x')));
    assert_eq!(app.theme_selector_index, 1);
    assert_eq!(app.mode, Mode::ThemeSelector);
}
