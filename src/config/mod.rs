//! Configuration system for LazyCSV.
//!
//! Loads settings from:
//! 1. `~/.config/lazycsv/config.toml` (global)
//! 2. `.lazycsv.toml` in the current directory (per-project, overrides global)
//!
//! All settings are optional — missing values use sensible defaults.
//! Invalid values produce warnings but never panic.

pub mod views;

use ratatui::style::Color;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

// ── TOML deserialization types ─────────────────────────────────

#[derive(Deserialize, Default)]
struct TomlConfig {
    #[serde(default)]
    defaults: TomlDefaults,
    #[serde(default)]
    theme: TomlTheme,
    #[serde(default)]
    sql: TomlSql,
}

#[derive(Deserialize, Default)]
struct TomlDefaults {
    delimiter: Option<String>,
    encoding: Option<String>,
    zebra_striping: Option<bool>,
    max_column_width: Option<u16>,
    undo_limit: Option<usize>,
}

#[derive(Deserialize, Default)]
struct TomlTheme {
    zebra_bg: Option<String>,
    cursor_bg: Option<String>,
    cursor_fg: Option<String>,
    selection_bg: Option<String>,
    selection_fg: Option<String>,
    search_match_bg: Option<String>,
    search_match_fg: Option<String>,
    header_bold: Option<bool>,
    header_bg: Option<String>,
    dirty_indicator_fg: Option<String>,
    // File menu colors
    file_menu_dir_fg: Option<String>,
    file_menu_highlight_bg: Option<String>,
    file_menu_highlight_fg: Option<String>,
    file_menu_separator_fg: Option<String>,
    file_menu_status_bg: Option<String>,
    file_menu_status_mode_bg: Option<String>,
    file_menu_status_accent_bg: Option<String>,
    file_menu_active_indicator_fg: Option<String>,
    file_menu_preview_col_1: Option<String>,
    file_menu_preview_col_2: Option<String>,
    file_menu_preview_col_3: Option<String>,
    file_menu_preview_col_4: Option<String>,
    file_menu_preview_col_5: Option<String>,
    file_menu_preview_col_6: Option<String>,
    file_menu_preview_col_7: Option<String>,
    file_menu_preview_col_8: Option<String>,
}

#[derive(Deserialize, Default)]
struct TomlSql {
    format_uppercase: Option<bool>,
    sql_history_limit: Option<usize>,
}

// ── Loading ────────────────────────────────────────────────────

/// Result of loading configuration, including any warnings.
pub struct ConfigResult {
    pub config: Config,
    pub warnings: Vec<String>,
}

/// Watches config files for changes by tracking their modification times.
#[derive(Debug, Clone)]
pub struct ConfigWatcher {
    global_path: Option<PathBuf>,
    local_path: PathBuf,
    global_mtime: Option<SystemTime>,
    local_mtime: Option<SystemTime>,
}

impl Default for ConfigWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWatcher {
    /// Create a new watcher, recording the current mtimes of config files.
    pub fn new() -> Self {
        let global_path = global_config_path();
        let local_path = PathBuf::from(".lazycsv.toml");

        let global_mtime = global_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        let local_mtime = std::fs::metadata(&local_path)
            .ok()
            .and_then(|m| m.modified().ok());

        Self {
            global_path,
            local_path,
            global_mtime,
            local_mtime,
        }
    }

    /// Check if any config file has been modified since last check.
    /// Returns true if config should be reloaded.
    pub fn has_changed(&mut self) -> bool {
        let new_global = self
            .global_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        let new_local = std::fs::metadata(&self.local_path)
            .ok()
            .and_then(|m| m.modified().ok());

        let changed = new_global != self.global_mtime || new_local != self.local_mtime;

        if changed {
            self.global_mtime = new_global;
            self.local_mtime = new_local;
        }

        changed
    }
}

/// Load configuration, merging global + per-directory configs.
/// Returns default config if no config files exist.
/// Collects warnings for invalid values (never panics).
pub fn load_config() -> Config {
    load_config_with_warnings().config
}

/// Load configuration and return any warnings about invalid values.
pub fn load_config_with_warnings() -> ConfigResult {
    let mut config = Config::default();
    let mut warnings = Vec::new();

    // 1. Load global config
    if let Some(global_path) = global_config_path() {
        match load_toml_file(&global_path) {
            Ok(Some(toml)) => apply_toml(&mut config, &toml, &global_path, &mut warnings),
            Ok(None) => {} // File doesn't exist
            Err(e) => warnings.push(format!("{}: {}", global_path.display(), e)),
        }
    }

    // 2. Load per-directory config (overrides global)
    let local_path = PathBuf::from(".lazycsv.toml");
    match load_toml_file(&local_path) {
        Ok(Some(toml)) => apply_toml(&mut config, &toml, &local_path, &mut warnings),
        Ok(None) => {}
        Err(e) => warnings.push(format!("{}: {}", local_path.display(), e)),
    }

    // 3. Validate final config
    validate_config(&config, &mut warnings);

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

/// Load and parse a TOML file.
/// Returns Ok(None) if file doesn't exist, Ok(Some) on success, Err on parse failure.
fn load_toml_file(path: &Path) -> Result<Option<TomlConfig>, String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("could not read file: {}", e)),
    };
    match toml::from_str(&content) {
        Ok(toml) => Ok(Some(toml)),
        Err(e) => Err(format!("invalid TOML: {}", e)),
    }
}

/// Apply TOML values to a Config, overriding only specified fields.
/// Collects warnings for invalid values (e.g. bad color strings, multi-char delimiters).
fn apply_toml(config: &mut Config, toml: &TomlConfig, path: &Path, warnings: &mut Vec<String>) {
    let file = path.display();

    // Defaults
    if let Some(ref d) = toml.defaults.delimiter {
        if d.chars().count() == 1 {
            config.defaults.delimiter = Some(d.chars().next().unwrap());
        } else {
            warnings.push(format!(
                "{}: delimiter must be a single character, got {:?}",
                file, d
            ));
        }
    }
    if toml.defaults.encoding.is_some() {
        config.defaults.encoding = toml.defaults.encoding.clone();
    }
    if let Some(z) = toml.defaults.zebra_striping {
        config.defaults.zebra_striping = z;
    }
    if let Some(w) = toml.defaults.max_column_width {
        if w >= 4 {
            config.defaults.max_column_width = w;
        } else {
            warnings.push(format!(
                "{}: max_column_width must be >= 4, got {}",
                file, w
            ));
        }
    }
    if let Some(u) = toml.defaults.undo_limit {
        if u >= 1 {
            config.defaults.undo_limit = u;
        } else {
            warnings.push(format!("{}: undo_limit must be >= 1, got {}", file, u));
        }
    }

    // Theme — helper to apply a color field with warning on bad value
    macro_rules! apply_color {
        ($field:expr, $toml_field:expr, $name:expr) => {
            if let Some(ref c) = $toml_field {
                if let Some(color) = parse_color(c) {
                    $field = color;
                } else {
                    warnings.push(format!("{}: invalid color for {}: {:?}", file, $name, c));
                }
            }
        };
    }

    apply_color!(config.theme.zebra_bg, toml.theme.zebra_bg, "zebra_bg");
    apply_color!(config.theme.cursor_bg, toml.theme.cursor_bg, "cursor_bg");
    apply_color!(config.theme.cursor_fg, toml.theme.cursor_fg, "cursor_fg");
    apply_color!(
        config.theme.selection_bg,
        toml.theme.selection_bg,
        "selection_bg"
    );
    apply_color!(
        config.theme.selection_fg,
        toml.theme.selection_fg,
        "selection_fg"
    );
    apply_color!(
        config.theme.search_match_bg,
        toml.theme.search_match_bg,
        "search_match_bg"
    );
    apply_color!(
        config.theme.search_match_fg,
        toml.theme.search_match_fg,
        "search_match_fg"
    );
    apply_color!(
        config.theme.dirty_indicator_fg,
        toml.theme.dirty_indicator_fg,
        "dirty_indicator_fg"
    );

    // File menu colors
    apply_color!(
        config.theme.file_menu_dir_fg,
        toml.theme.file_menu_dir_fg,
        "file_menu_dir_fg"
    );
    apply_color!(
        config.theme.file_menu_highlight_bg,
        toml.theme.file_menu_highlight_bg,
        "file_menu_highlight_bg"
    );
    apply_color!(
        config.theme.file_menu_highlight_fg,
        toml.theme.file_menu_highlight_fg,
        "file_menu_highlight_fg"
    );
    apply_color!(
        config.theme.file_menu_separator_fg,
        toml.theme.file_menu_separator_fg,
        "file_menu_separator_fg"
    );
    apply_color!(
        config.theme.file_menu_status_bg,
        toml.theme.file_menu_status_bg,
        "file_menu_status_bg"
    );
    apply_color!(
        config.theme.file_menu_status_mode_bg,
        toml.theme.file_menu_status_mode_bg,
        "file_menu_status_mode_bg"
    );
    apply_color!(
        config.theme.file_menu_status_accent_bg,
        toml.theme.file_menu_status_accent_bg,
        "file_menu_status_accent_bg"
    );
    apply_color!(
        config.theme.file_menu_active_indicator_fg,
        toml.theme.file_menu_active_indicator_fg,
        "file_menu_active_indicator_fg"
    );
    apply_color!(
        config.theme.file_menu_preview_col_1,
        toml.theme.file_menu_preview_col_1,
        "file_menu_preview_col_1"
    );
    apply_color!(
        config.theme.file_menu_preview_col_2,
        toml.theme.file_menu_preview_col_2,
        "file_menu_preview_col_2"
    );
    apply_color!(
        config.theme.file_menu_preview_col_3,
        toml.theme.file_menu_preview_col_3,
        "file_menu_preview_col_3"
    );
    apply_color!(
        config.theme.file_menu_preview_col_4,
        toml.theme.file_menu_preview_col_4,
        "file_menu_preview_col_4"
    );
    apply_color!(
        config.theme.file_menu_preview_col_5,
        toml.theme.file_menu_preview_col_5,
        "file_menu_preview_col_5"
    );
    apply_color!(
        config.theme.file_menu_preview_col_6,
        toml.theme.file_menu_preview_col_6,
        "file_menu_preview_col_6"
    );
    apply_color!(
        config.theme.file_menu_preview_col_7,
        toml.theme.file_menu_preview_col_7,
        "file_menu_preview_col_7"
    );
    apply_color!(
        config.theme.file_menu_preview_col_8,
        toml.theme.file_menu_preview_col_8,
        "file_menu_preview_col_8"
    );

    if let Some(b) = toml.theme.header_bold {
        config.theme.header_bold = b;
    }
    if let Some(ref c) = toml.theme.header_bg {
        if let Some(color) = parse_color(c) {
            config.theme.header_bg = Some(color);
        } else {
            warnings.push(format!("{}: invalid color for header_bg: {:?}", file, c));
        }
    }

    // SQL
    if let Some(u) = toml.sql.format_uppercase {
        config.sql.format_uppercase = u;
    }
    if let Some(limit) = toml.sql.sql_history_limit {
        config.sql.sql_history_limit = limit;
    }
}

/// Validate final merged config for logical consistency.
fn validate_config(config: &Config, warnings: &mut Vec<String>) {
    if config.defaults.max_column_width < 4 {
        warnings.push(format!(
            "max_column_width ({}) is too small, using minimum of 4",
            config.defaults.max_column_width
        ));
    }
    if config.defaults.undo_limit == 0 {
        warnings.push("undo_limit is 0, undo will be disabled".to_string());
    }
}

/// Path to the SQL history file (~/.config/lazycsv/sql_history).
pub fn sql_history_path() -> Option<PathBuf> {
    dirs_path().map(|p| p.join("sql_history"))
}

/// Load SQL history from disk. Returns an empty vec if the file doesn't exist or can't be read.
///
/// Format: one query per line; embedded newlines are stored as the two-character sequence `\n`.
pub fn load_sql_history() -> Vec<String> {
    let path = match sql_history_path() {
        Some(p) => p,
        None => return Vec::new(),
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.replace("\\n", "\n"))
        .collect()
}

/// Save SQL history to disk, capped to `limit` entries.
///
/// Silently ignores write errors (non-critical).
pub fn save_sql_history(history: &[String], limit: usize) {
    let path = match sql_history_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content: String = history
        .iter()
        .take(limit)
        .map(|q| q.replace('\n', "\\n"))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write(&path, content);
}

/// Parse a color string. Supports:
/// - Named colors: "black", "white", "red", "green", "blue", "yellow", "cyan", "magenta",
///   "gray", "darkgray", "lightred", "lightgreen", "lightyellow", "lightblue",
///   "lightmagenta", "lightcyan"
/// - Hex RGB: "#1e1e1e", "#ff0000"
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();

    // Hex color
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).ok()?;
        let g = u8::from_str_radix(&s[3..5], 16).ok()?;
        let b = u8::from_str_radix(&s[5..7], 16).ok()?;
        return Some(Color::Rgb(r, g, b));
    }

    // Named colors (terminal + extended CSS-style)
    match s.as_str() {
        // Terminal colors
        "black" => Some(Color::Black),
        "white" => Some(Color::White),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "yellow" => Some(Color::Yellow),
        "cyan" | "aqua" => Some(Color::Cyan),
        "magenta" | "fuchsia" => Some(Color::Magenta),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        // Grays
        "silver" => Some(Color::Rgb(192, 192, 192)),
        "dimgray" | "dimgrey" => Some(Color::Rgb(105, 105, 105)),
        // Reds/Pinks
        "crimson" => Some(Color::Rgb(220, 20, 60)),
        "pink" => Some(Color::Rgb(255, 192, 203)),
        "hotpink" => Some(Color::Rgb(255, 105, 180)),
        "firebrick" => Some(Color::Rgb(178, 34, 34)),
        // Blues
        "darkblue" => Some(Color::Rgb(0, 0, 139)),
        "teal" => Some(Color::Rgb(0, 128, 128)),
        // Greens
        "lime" => Some(Color::Rgb(0, 255, 0)),
        "forestgreen" => Some(Color::Rgb(34, 139, 34)),
        "seagreen" => Some(Color::Rgb(46, 139, 87)),
        "olive" => Some(Color::Rgb(128, 128, 0)),
        // Yellows/Oranges
        "gold" => Some(Color::Rgb(255, 215, 0)),
        "orange" => Some(Color::Rgb(255, 165, 0)),
        "darkorange" => Some(Color::Rgb(255, 140, 0)),
        "lemonchiffon" => Some(Color::Rgb(255, 250, 205)),
        // Purples
        "purple" => Some(Color::Rgb(128, 0, 128)),
        "rebeccapurple" => Some(Color::Rgb(102, 51, 153)),
        "indigo" => Some(Color::Rgb(75, 0, 130)),
        // Browns/Beiges
        "brown" => Some(Color::Rgb(165, 42, 42)),
        "maroon" => Some(Color::Rgb(128, 0, 0)),
        "sandybrown" => Some(Color::Rgb(244, 164, 96)),
        "beige" => Some(Color::Rgb(245, 245, 220)),
        "antiquewhite" => Some(Color::Rgb(250, 235, 215)),
        _ => None,
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

        apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
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

        apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
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
        apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
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

        apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
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

        apply_toml(&mut config, &toml, Path::new("test"), &mut Vec::new());
        assert_eq!(config.theme.header_bg, Some(Color::Rgb(51, 51, 51)));
        assert_eq!(config.theme.dirty_indicator_fg, Color::Yellow);
    }

    // ── Validation & Warning Tests ────────────────────────────────

    fn apply_with_warnings(toml_str: &str) -> (Config, Vec<String>) {
        let mut config = Config::default();
        let mut warnings = Vec::new();
        let toml: TomlConfig = toml::from_str(toml_str).unwrap();
        apply_toml(&mut config, &toml, Path::new("test.toml"), &mut warnings);
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
        let result = load_toml_file(&path);
        match result {
            Err(msg) => assert!(msg.contains("invalid TOML"), "unexpected error: {}", msg),
            Ok(_) => panic!("expected error for malformed TOML"),
        }
    }

    #[test]
    fn test_missing_file_returns_none() {
        let result = load_toml_file(Path::new("/nonexistent/path/config.toml"));
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
        let result = load_toml_file(&path);
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
        let result = load_toml_file(&path);
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
        let result = load_toml_file(&path);
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
        apply_toml(&mut config, &toml1, path, &mut warnings);

        let toml2: TomlConfig = toml::from_str(
            r#"
            [defaults]
            max_column_width = 80
            "#,
        )
        .unwrap();
        apply_toml(&mut config, &toml2, path, &mut warnings);

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
        apply_toml(&mut config, &global, path, &mut warnings);

        let local: TomlConfig = toml::from_str(
            r##"
            [theme]
            cursor_fg = "#0000ff"
            "##,
        )
        .unwrap();
        apply_toml(&mut config, &local, path, &mut warnings);

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
        validate_config(&config, &mut warnings);
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
