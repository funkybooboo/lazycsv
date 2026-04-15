//! Configuration system for LazyCSV.
//!
//! Loads settings from:
//! 1. `~/.config/lazycsv/config.toml` (global)
//! 2. `.lazycsv.toml` in the current directory (per-project, overrides global)
//!
//! All settings are optional — missing values use sensible defaults.
//! Invalid values produce warnings but never panic.

// Items in this module are used by the binary crate (main.rs) but not
// referenced from within the library crate, so the compiler reports them
// as unused. Suppressing here rather than marking each individually.
#![allow(dead_code)]

mod sql_history;
mod toml_parsing;
pub mod views;
mod watcher;

pub use sql_history::{load_sql_history, save_sql_history, sql_history_path};
#[cfg(test)]
use std::path::Path;
pub(crate) use toml_parsing::parse_color;
#[cfg(test)]
use toml_parsing::TomlConfig;
pub use watcher::ConfigWatcher;

use ratatui::style::Color;
use std::path::PathBuf;

/// Top-level configuration.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub defaults: Defaults,
    pub theme: Theme,
    pub sql: SqlConfig,
}

/// Default behaviors.
#[derive(Debug, Clone)]
pub struct Defaults {
    pub delimiter: Option<char>,
    pub encoding: Option<String>,
    pub zebra_striping: bool,
    pub max_column_width: u16,
    pub undo_limit: usize,
    pub show_footer: bool,
}

/// Theme/color configuration.
#[derive(Debug, Clone)]
pub struct Theme {
    pub zebra_bg: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub search_match_bg: Color,
    pub search_match_fg: Color,
    pub header_bold: bool,
    pub header_bg: Option<Color>,
    pub dirty_indicator_fg: Color,
    // File menu colors
    pub file_menu_dir_fg: Color,
    pub file_menu_highlight_bg: Color,
    pub file_menu_highlight_fg: Color,
    pub file_menu_separator_fg: Color,
    pub file_menu_status_bg: Color,
    pub file_menu_status_mode_bg: Color,
    pub file_menu_status_accent_bg: Color,
    pub file_menu_active_indicator_fg: Color,
    pub file_menu_preview_col_1: Color,
    pub file_menu_preview_col_2: Color,
    pub file_menu_preview_col_3: Color,
    pub file_menu_preview_col_4: Color,
    pub file_menu_preview_col_5: Color,
    pub file_menu_preview_col_6: Color,
    pub file_menu_preview_col_7: Color,
    pub file_menu_preview_col_8: Color,
}

/// SQL editor configuration.
#[derive(Debug, Clone)]
pub struct SqlConfig {
    pub format_uppercase: bool,
    /// Maximum number of SQL queries kept in history (0 = disabled).
    pub sql_history_limit: usize,
}

impl Default for Defaults {
    fn default() -> Self {
        Defaults {
            delimiter: None,
            encoding: None,
            zebra_striping: true,
            max_column_width: 100,
            undo_limit: 1000,
            show_footer: false,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            zebra_bg: Color::Rgb(30, 30, 30),
            cursor_bg: Color::White,
            cursor_fg: Color::Black,
            selection_bg: Color::DarkGray,
            selection_fg: Color::Yellow,
            search_match_bg: Color::Yellow,
            search_match_fg: Color::Black,
            header_bold: true,
            header_bg: None,
            dirty_indicator_fg: Color::Red,
            // File menu defaults
            file_menu_dir_fg: Color::Blue,
            file_menu_highlight_bg: Color::White,
            file_menu_highlight_fg: Color::Black,
            file_menu_separator_fg: Color::Gray,
            file_menu_status_bg: Color::DarkGray,
            file_menu_status_mode_bg: Color::Blue,
            file_menu_status_accent_bg: Color::Magenta,
            file_menu_active_indicator_fg: Color::Green,
            file_menu_preview_col_1: Color::Blue,
            file_menu_preview_col_2: Color::Green,
            file_menu_preview_col_3: Color::Yellow,
            file_menu_preview_col_4: Color::Cyan,
            file_menu_preview_col_5: Color::Magenta,
            file_menu_preview_col_6: Color::Red,
            file_menu_preview_col_7: Color::LightBlue,
            file_menu_preview_col_8: Color::LightGreen,
        }
    }
}

impl Default for SqlConfig {
    fn default() -> Self {
        SqlConfig {
            format_uppercase: true,
            sql_history_limit: 15,
        }
    }
}

pub struct ConfigResult {
    pub config: Config,
    pub warnings: Vec<String>,
}

pub fn load_config() -> Config {
    load_config_with_warnings().config
}

/// Load configuration and return any warnings about invalid values.
pub fn load_config_with_warnings() -> ConfigResult {
    let mut config = Config::default();
    let mut warnings = Vec::new();

    // 1. Load global config
    if let Some(global_path) = global_config_path() {
        match toml_parsing::load_toml_file(&global_path) {
            Ok(Some(toml)) => {
                toml_parsing::apply_toml(&mut config, &toml, &global_path, &mut warnings)
            }
            Ok(None) => {} // File doesn't exist
            Err(e) => warnings.push(format!("{}: {}", global_path.display(), e)),
        }
    }

    // 2. Load per-directory config (overrides global)
    let local_path = PathBuf::from(".lazycsv.toml");
    match toml_parsing::load_toml_file(&local_path) {
        Ok(Some(toml)) => toml_parsing::apply_toml(&mut config, &toml, &local_path, &mut warnings),
        Ok(None) => {}
        Err(e) => warnings.push(format!("{}: {}", local_path.display(), e)),
    }

    // 3. Validate final config
    toml_parsing::validate_config(&config, &mut warnings);

    ConfigResult { config, warnings }
}

/// Get the global config file path (~/.config/lazycsv/config.toml).
fn global_config_path() -> Option<PathBuf> {
    dirs_path().map(|p| p.join("config.toml"))
}

/// Get the config directory (~/.config/lazycsv/).
pub fn dirs_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config").join("lazycsv"))
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
            .map(|p| p.join("lazycsv"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join("lazycsv"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.defaults.zebra_striping);
        assert_eq!(config.defaults.max_column_width, 100);
        assert_eq!(config.defaults.undo_limit, 1000);
        assert_eq!(config.theme.cursor_bg, Color::White);
        assert_eq!(config.theme.cursor_fg, Color::Black);
        assert_eq!(config.theme.dirty_indicator_fg, Color::Red);
        assert!(config.theme.header_bg.is_none());
        assert!(config.sql.format_uppercase);
    }

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#1e1e1e"), Some(Color::Rgb(30, 30, 30)));
        assert_eq!(parse_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00ff00"), Some(Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("white"), Some(Color::White));
        assert_eq!(parse_color("Black"), Some(Color::Black));
        assert_eq!(parse_color("YELLOW"), Some(Color::Yellow));
        assert_eq!(parse_color("darkgray"), Some(Color::DarkGray));
    }

    #[test]
    fn test_parse_color_invalid() {
        assert_eq!(parse_color("notacolor"), None);
        assert_eq!(parse_color("#xyz"), None);
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn test_apply_toml_partial() {
        let mut config = Config::default();
        let toml: TomlConfig = toml::from_str(
            r##"
            [defaults]
            zebra_striping = false
            max_column_width = 50

            [theme]
            cursor_bg = "#ff0000"
            "##,
        )
        .unwrap();

        toml_parsing::apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
        assert!(!config.defaults.zebra_striping);
        assert_eq!(config.defaults.max_column_width, 50);
        assert_eq!(config.theme.cursor_bg, Color::Rgb(255, 0, 0));
        // Unspecified values remain default
        assert_eq!(config.theme.cursor_fg, Color::Black);
    }

    #[test]
    fn test_apply_toml_delimiter() {
        let mut config = Config::default();
        let toml: TomlConfig = toml::from_str(
            r#"
            [defaults]
            delimiter = ";"
            "#,
        )
        .unwrap();

        toml_parsing::apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
        assert_eq!(config.defaults.delimiter, Some(';'));
    }

    #[test]
    fn test_load_config_no_files() {
        // Should return defaults when no config files exist
        let config = Config::default();
        assert!(config.defaults.delimiter.is_none());
        assert!(config.defaults.zebra_striping);
    }

    #[test]
    fn test_apply_toml_empty() {
        let mut config = Config::default();
        let toml: TomlConfig = toml::from_str("").unwrap();
        toml_parsing::apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
        // Everything should remain default
        assert!(config.defaults.zebra_striping);
        assert_eq!(config.theme.cursor_bg, Color::White);
    }

    #[test]
    fn test_apply_toml_undo_limit() {
        let mut config = Config::default();
        let toml: TomlConfig = toml::from_str(
            r#"
            [defaults]
            undo_limit = 500
            "#,
        )
        .unwrap();

        toml_parsing::apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
        assert_eq!(config.defaults.undo_limit, 500);
    }

    #[test]
    fn test_apply_toml_header_bg_and_dirty_indicator() {
        let mut config = Config::default();
        let toml: TomlConfig = toml::from_str(
            r##"
            [theme]
            header_bg = "#333333"
            dirty_indicator_fg = "yellow"
            "##,
        )
        .unwrap();

        toml_parsing::apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
        assert_eq!(config.theme.header_bg, Some(Color::Rgb(51, 51, 51)));
        assert_eq!(config.theme.dirty_indicator_fg, Color::Yellow);
    }

    // ── Validation & Warning Tests ────────────────────────────────

    fn apply_with_warnings(toml_str: &str) -> (Config, Vec<String>) {
        let mut config = Config::default();
        let mut warnings = Vec::new();
        let toml: TomlConfig = toml::from_str(toml_str).unwrap();
        toml_parsing::apply_toml(&mut config, &toml, Path::new("test.toml"), &mut warnings);
        (config, warnings)
    }

    #[test]
    fn test_warning_multi_char_delimiter() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            delimiter = "ab"
            "#,
        );
        assert!(config.defaults.delimiter.is_none()); // not applied
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("single character"));
    }

    #[test]
    fn test_warning_empty_delimiter() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            delimiter = ""
            "#,
        );
        assert!(config.defaults.delimiter.is_none());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("single character"));
    }

    #[test]
    fn test_warning_max_column_width_too_small() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            max_column_width = 2
            "#,
        );
        // Should not apply the invalid value
        assert_eq!(config.defaults.max_column_width, 100);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("max_column_width"));
    }

    #[test]
    fn test_warning_undo_limit_zero() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            undo_limit = 0
            "#,
        );
        // Should not apply zero
        assert_eq!(config.defaults.undo_limit, 1000);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("undo_limit"));
    }

    #[test]
    fn test_warning_invalid_color() {
        let (config, warnings) = apply_with_warnings(
            r##"
            [theme]
            cursor_bg = "notacolor"
            "##,
        );
        // Should keep default
        assert_eq!(config.theme.cursor_bg, Color::White);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("invalid color"));
        assert!(warnings[0].contains("cursor_bg"));
    }

    #[test]
    fn test_warning_invalid_header_bg() {
        let (_, warnings) = apply_with_warnings(
            r##"
            [theme]
            header_bg = "xyz"
            "##,
        );
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("header_bg"));
    }

    #[test]
    fn test_warning_multiple_invalid_colors() {
        let (_, warnings) = apply_with_warnings(
            r##"
            [theme]
            cursor_bg = "bad1"
            cursor_fg = "bad2"
            selection_bg = "bad3"
            "##,
        );
        assert_eq!(warnings.len(), 3);
    }

    #[test]
    fn test_no_warnings_for_valid_config() {
        let (_, warnings) = apply_with_warnings(
            r##"
            [defaults]
            delimiter = ","
            zebra_striping = true
            max_column_width = 50
            undo_limit = 500

            [theme]
            cursor_bg = "#ff0000"
            cursor_fg = "white"
            header_bg = "darkgray"
            dirty_indicator_fg = "yellow"

            [sql]
            format_uppercase = false
            "##,
        );
        assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
    }

    // ── parse_color edge cases ────────────────────────────────────

    #[test]
    fn test_parse_color_hex_uppercase() {
        assert_eq!(parse_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#AbCdEf"), Some(Color::Rgb(171, 205, 239)));
    }

    #[test]
    fn test_parse_color_hex_too_short() {
        assert_eq!(parse_color("#fff"), None);
        assert_eq!(parse_color("#12345"), None);
    }

    #[test]
    fn test_parse_color_hex_too_long() {
        assert_eq!(parse_color("#ff00ff00"), None);
    }

    #[test]
    fn test_parse_color_hex_no_hash() {
        assert_eq!(parse_color("ff0000"), None);
    }

    #[test]
    fn test_parse_color_whitespace() {
        assert_eq!(parse_color("  white  "), Some(Color::White));
        assert_eq!(parse_color(" #ff0000 "), Some(Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn test_parse_color_all_named() {
        let named = [
            ("black", Color::Black),
            ("white", Color::White),
            ("red", Color::Red),
            ("green", Color::Green),
            ("blue", Color::Blue),
            ("yellow", Color::Yellow),
            ("cyan", Color::Cyan),
            ("magenta", Color::Magenta),
            ("gray", Color::Gray),
            ("grey", Color::Gray),
            ("darkgray", Color::DarkGray),
            ("darkgrey", Color::DarkGray),
            ("lightred", Color::LightRed),
            ("lightgreen", Color::LightGreen),
            ("lightyellow", Color::LightYellow),
            ("lightblue", Color::LightBlue),
            ("lightmagenta", Color::LightMagenta),
            ("lightcyan", Color::LightCyan),
        ];
        for (name, expected) in &named {
            assert_eq!(parse_color(name), Some(*expected), "failed for {}", name);
        }
    }

    #[test]
    fn test_parse_color_hex_boundary_values() {
        assert_eq!(parse_color("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(parse_color("#ffffff"), Some(Color::Rgb(255, 255, 255)));
    }

    // ── TOML parsing edge cases ───────────────────────────────────

    #[test]
    fn test_malformed_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "[defaults\nbroken").unwrap();
        let result = toml_parsing::load_toml_file(&path);
        match result {
            Err(msg) => assert!(msg.contains("invalid TOML"), "unexpected error: {}", msg),
            Ok(_) => panic!("expected error for malformed TOML"),
        }
    }

    #[test]
    fn test_missing_file_returns_none() {
        let result = toml_parsing::load_toml_file(Path::new("/nonexistent/path/config.toml"));
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn test_valid_toml_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [defaults]
            zebra_striping = false
            "#,
        )
        .unwrap();
        let result = toml_parsing::load_toml_file(&path);
        let toml = result.unwrap().unwrap();
        assert_eq!(toml.defaults.zebra_striping, Some(false));
    }

    #[test]
    fn test_unknown_keys_ignored() {
        // TOML with unknown keys should parse without error (serde ignores them)
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [defaults]
            some_future_key = "value"
            zebra_striping = true
            "#,
        )
        .unwrap();
        let result = toml_parsing::load_toml_file(&path);
        // serde default behavior rejects unknown keys unless deny_unknown_fields
        // Our structs don't use deny_unknown_fields, so this should work
        assert!(result.is_ok());
    }

    #[test]
    fn test_wrong_type_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [defaults]
            zebra_striping = "not_a_bool"
            "#,
        )
        .unwrap();
        let result = toml_parsing::load_toml_file(&path);
        assert!(result.is_err());
    }

    // ── Config merge tests ────────────────────────────────────────

    #[test]
    fn test_second_toml_overrides_first() {
        let mut config = Config::default();
        let mut warnings = Vec::new();
        let path = Path::new("test");

        let toml1: TomlConfig = toml::from_str(
            r#"
            [defaults]
            zebra_striping = false
            max_column_width = 50
            "#,
        )
        .unwrap();
        toml_parsing::apply_toml(&mut config, &toml1, path, &mut warnings);

        let toml2: TomlConfig = toml::from_str(
            r#"
            [defaults]
            max_column_width = 80
            "#,
        )
        .unwrap();
        toml_parsing::apply_toml(&mut config, &toml2, path, &mut warnings);

        // Second TOML overrides max_column_width
        assert_eq!(config.defaults.max_column_width, 80);
        // First TOML's zebra_striping preserved (not in second)
        assert!(!config.defaults.zebra_striping);
    }

    #[test]
    fn test_merge_theme_partial_override() {
        let mut config = Config::default();
        let mut warnings = Vec::new();
        let path = Path::new("test");

        let global: TomlConfig = toml::from_str(
            r##"
            [theme]
            cursor_bg = "#ff0000"
            cursor_fg = "#00ff00"
            "##,
        )
        .unwrap();
        toml_parsing::apply_toml(&mut config, &global, path, &mut warnings);

        let local: TomlConfig = toml::from_str(
            r##"
            [theme]
            cursor_fg = "#0000ff"
            "##,
        )
        .unwrap();
        toml_parsing::apply_toml(&mut config, &local, path, &mut warnings);

        assert_eq!(config.theme.cursor_bg, Color::Rgb(255, 0, 0)); // from global
        assert_eq!(config.theme.cursor_fg, Color::Rgb(0, 0, 255)); // overridden by local
    }

    // ── Full config with all fields ───────────────────────────────

    #[test]
    fn test_full_config_all_fields() {
        let (config, warnings) = apply_with_warnings(
            r##"
            [defaults]
            delimiter = "\t"
            encoding = "latin1"
            zebra_striping = false
            max_column_width = 200
            undo_limit = 5000

            [theme]
            zebra_bg = "#222222"
            cursor_bg = "#ffffff"
            cursor_fg = "#000000"
            selection_bg = "blue"
            selection_fg = "white"
            search_match_bg = "red"
            search_match_fg = "white"
            header_bold = false
            header_bg = "#444444"
            dirty_indicator_fg = "green"

            [sql]
            format_uppercase = false
            "##,
        );

        assert!(warnings.is_empty(), "warnings: {:?}", warnings);
        assert_eq!(config.defaults.delimiter, Some('\t'));
        assert_eq!(config.defaults.encoding, Some("latin1".to_string()));
        assert!(!config.defaults.zebra_striping);
        assert_eq!(config.defaults.max_column_width, 200);
        assert_eq!(config.defaults.undo_limit, 5000);
        assert_eq!(config.theme.zebra_bg, Color::Rgb(34, 34, 34));
        assert_eq!(config.theme.cursor_bg, Color::Rgb(255, 255, 255));
        assert_eq!(config.theme.cursor_fg, Color::Rgb(0, 0, 0));
        assert_eq!(config.theme.selection_bg, Color::Blue);
        assert_eq!(config.theme.selection_fg, Color::White);
        assert_eq!(config.theme.search_match_bg, Color::Red);
        assert_eq!(config.theme.search_match_fg, Color::White);
        assert!(!config.theme.header_bold);
        assert_eq!(config.theme.header_bg, Some(Color::Rgb(68, 68, 68)));
        assert_eq!(config.theme.dirty_indicator_fg, Color::Green);
        assert!(!config.sql.format_uppercase);
    }

    // ── validate_config tests ─────────────────────────────────────

    #[test]
    fn test_validate_config_defaults_ok() {
        let config = Config::default();
        let mut warnings = Vec::new();
        toml_parsing::validate_config(&config, &mut warnings);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_load_config_with_warnings_no_files() {
        // Should return defaults with no warnings when no config files exist
        let result = load_config_with_warnings();
        assert!(result.config.defaults.zebra_striping);
        // May have warnings if a real config file exists in ~/.config, but won't panic
    }

    // ── Delimiter edge cases ──────────────────────────────────────

    #[test]
    fn test_delimiter_tab() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            delimiter = "\t"
            "#,
        );
        assert!(warnings.is_empty());
        assert_eq!(config.defaults.delimiter, Some('\t'));
    }

    #[test]
    fn test_delimiter_pipe() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            delimiter = "|"
            "#,
        );
        assert!(warnings.is_empty());
        assert_eq!(config.defaults.delimiter, Some('|'));
    }

    #[test]
    fn test_delimiter_semicolon() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            delimiter = ";"
            "#,
        );
        assert!(warnings.is_empty());
        assert_eq!(config.defaults.delimiter, Some(';'));
    }

    // ── max_column_width boundary ─────────────────────────────────

    #[test]
    fn test_max_column_width_minimum_valid() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            max_column_width = 4
            "#,
        );
        assert!(warnings.is_empty());
        assert_eq!(config.defaults.max_column_width, 4);
    }

    #[test]
    fn test_max_column_width_large() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            max_column_width = 10000
            "#,
        );
        assert!(warnings.is_empty());
        assert_eq!(config.defaults.max_column_width, 10000);
    }

    // ── undo_limit boundary ───────────────────────────────────────

    #[test]
    fn test_undo_limit_one() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            undo_limit = 1
            "#,
        );
        assert!(warnings.is_empty());
        assert_eq!(config.defaults.undo_limit, 1);
    }

    #[test]
    fn test_undo_limit_very_large() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [defaults]
            undo_limit = 100000
            "#,
        );
        assert!(warnings.is_empty());
        assert_eq!(config.defaults.undo_limit, 100000);
    }

    // ── Empty sections ────────────────────────────────────────────

    #[test]
    fn test_empty_defaults_section() {
        let (config, warnings) = apply_with_warnings("[defaults]\n");
        assert!(warnings.is_empty());
        assert!(config.defaults.zebra_striping); // unchanged default
    }

    #[test]
    fn test_empty_theme_section() {
        let (config, warnings) = apply_with_warnings("[theme]\n");
        assert!(warnings.is_empty());
        assert_eq!(config.theme.cursor_bg, Color::White);
    }

    #[test]
    fn test_empty_sql_section() {
        let (config, warnings) = apply_with_warnings("[sql]\n");
        assert!(warnings.is_empty());
        assert!(config.sql.format_uppercase);
    }

    #[test]
    fn test_only_sql_section() {
        let (config, warnings) = apply_with_warnings(
            r#"
            [sql]
            format_uppercase = false
            "#,
        );
        assert!(warnings.is_empty());
        assert!(!config.sql.format_uppercase);
        // Everything else is default
        assert!(config.defaults.zebra_striping);
        assert_eq!(config.theme.cursor_bg, Color::White);
    }

    // ── ConfigWatcher tests ───────────────────────────────────────

    #[test]
    fn test_watcher_no_change() {
        let mut watcher = ConfigWatcher::new();
        // Second call with no file changes should return false
        assert!(!watcher.has_changed());
    }

    #[test]
    fn test_watcher_detects_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".lazycsv.toml");

        // Create watcher before file exists
        let mut watcher = ConfigWatcher {
            global_path: None,
            local_path: path.clone(),
            global_mtime: None,
            local_mtime: None,
        };

        assert!(!watcher.has_changed()); // no file yet, no change

        // Create the file
        std::fs::write(&path, "[defaults]\nzebra_striping = false\n").unwrap();

        assert!(watcher.has_changed()); // file appeared
        assert!(!watcher.has_changed()); // no change since last check
    }

    #[test]
    fn test_watcher_detects_modification() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[defaults]\n").unwrap();

        let mut watcher = ConfigWatcher {
            global_path: Some(path.clone()),
            local_path: PathBuf::from("/nonexistent/.lazycsv.toml"),
            global_mtime: std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok()),
            local_mtime: None,
        };

        assert!(!watcher.has_changed()); // no change yet

        // Wait a moment and modify the file (mtime granularity)
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&path, "[defaults]\nzebra_striping = false\n").unwrap();

        assert!(watcher.has_changed()); // modification detected
        assert!(!watcher.has_changed()); // stable again
    }

    #[test]
    fn test_watcher_detects_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[defaults]\n").unwrap();

        let mut watcher = ConfigWatcher {
            global_path: Some(path.clone()),
            local_path: PathBuf::from("/nonexistent/.lazycsv.toml"),
            global_mtime: std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok()),
            local_mtime: None,
        };

        assert!(!watcher.has_changed());

        // Delete the file
        std::fs::remove_file(&path).unwrap();

        assert!(watcher.has_changed()); // deletion detected
        assert!(!watcher.has_changed()); // stable (file still gone)
    }
}
