//! TOML deserialization and config application.

use ratatui::style::Color;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Config;

// ── TOML deserialization types ─────────────────────────────────

#[derive(Deserialize, Default)]
pub(super) struct TomlConfig {
    #[serde(default)]
    pub(super) defaults: TomlDefaults,
    #[serde(default)]
    pub(super) theme: TomlTheme,
    #[serde(default)]
    pub(super) sql: TomlSql,
}

#[derive(Deserialize, Default)]
pub(super) struct TomlDefaults {
    pub(super) delimiter: Option<String>,
    pub(super) encoding: Option<String>,
    pub(super) zebra_striping: Option<bool>,
    pub(super) max_column_width: Option<u16>,
    pub(super) undo_limit: Option<usize>,
    pub(super) show_footer: Option<bool>,
}

#[derive(Deserialize, Default)]
pub(super) struct TomlTheme {
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
pub(super) struct TomlSql {
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
pub(super) fn load_toml_file(path: &Path) -> Result<Option<TomlConfig>, String> {
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
pub(super) fn apply_toml(
    config: &mut Config,
    toml: &TomlConfig,
    path: &Path,
    warnings: &mut Vec<String>,
) {
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
    if let Some(f) = toml.defaults.show_footer {
        config.defaults.show_footer = f;
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
pub(super) fn validate_config(config: &Config, warnings: &mut Vec<String>) {
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

/// Parse a color string. Supports:
/// - Named colors: "black", "white", "red", "green", "blue", "yellow", "cyan", "magenta",
///   "gray", "darkgray", "lightred", "lightgreen", "lightyellow", "lightblue",
///   "lightmagenta", "lightcyan"
/// - Hex RGB: "#1e1e1e", "#ff0000"
pub(crate) fn parse_color(s: &str) -> Option<Color> {
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
