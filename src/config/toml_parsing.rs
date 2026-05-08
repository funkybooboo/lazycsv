//! TOML deserialization and config application.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Config;

// ── TOML deserialization types ─────────────────────────────────

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlConfig {
    #[serde(default)]
    pub(super) defaults: TomlDefaults,
    #[serde(default)]
    pub(super) ui: TomlUi,
    #[serde(default)]
    pub(super) table: TomlTable,
    #[serde(default)]
    pub(super) popup: TomlPopup,
    #[serde(default)]
    pub(super) status: TomlStatus,
    #[serde(default)]
    pub(super) file_menu: TomlFileMenu,
    #[serde(default)]
    pub(super) sql: TomlSql,
}

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlDefaults {
    pub(super) delimiter: Option<String>,
    pub(super) encoding: Option<String>,
    pub(super) zebra_striping: Option<bool>,
    pub(super) max_column_width: Option<u16>,
    pub(super) undo_limit: Option<usize>,
    pub(super) show_footer: Option<bool>,
    pub(super) command_history_limit: Option<usize>,
    pub(super) shell_history_limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlUi {
    fg: Option<String>,
    bg: Option<String>,
    border_fg: Option<String>,
}

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlTable {
    header_fg: Option<String>,
    header_bg: Option<String>,
    header_bold: Option<bool>,
    zebra_bg: Option<String>,
    cursor_fg: Option<String>,
    cursor_bg: Option<String>,
    selection_fg: Option<String>,
    selection_bg: Option<String>,
    search_match_fg: Option<String>,
    search_match_bg: Option<String>,
    dirty_fg: Option<String>,
}

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlPopup {
    bg: Option<String>,
    fg: Option<String>,
    border_fg: Option<String>,
    title_fg: Option<String>,
    completion_sel_fg: Option<String>,
    completion_sel_bg: Option<String>,
}

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlStatus {
    fg: Option<String>,
    bg: Option<String>,
    mode_fg: Option<String>,
    mode_bg: Option<String>,
    error_fg: Option<String>,
    success_fg: Option<String>,
}

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlFileMenu {
    dir_fg: Option<String>,
    highlight_fg: Option<String>,
    highlight_bg: Option<String>,
    separator_fg: Option<String>,
    status_bg: Option<String>,
    status_mode_bg: Option<String>,
    status_accent_bg: Option<String>,
    active_indicator_fg: Option<String>,
    preview_cols: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Default)]
pub(super) struct TomlSql {
    format_uppercase: Option<bool>,
    sql_history_limit: Option<usize>,
    line_number_fg: Option<String>,
    diagnostic_error_fg: Option<String>,
    diagnostic_warning_fg: Option<String>,
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
pub(crate) fn load_toml_file(path: &Path) -> Result<Option<TomlConfig>, String> {
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
pub(crate) fn apply_toml(
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
    if let Some(limit) = toml.defaults.command_history_limit {
        config.defaults.command_history_limit = limit;
    }
    if let Some(limit) = toml.defaults.shell_history_limit {
        config.defaults.shell_history_limit = limit;
    }

    // Theme — helper to apply a color field with warning on bad value
    macro_rules! apply_color {
        ($field:expr, $toml_field:expr, $section:expr, $name:expr) => {
            if let Some(ref c) = $toml_field {
                if let Some(color) = parse_color(c) {
                    $field = color;
                } else {
                    warnings.push(format!(
                        "{}: invalid color for {}.{}: {:?}",
                        file, $section, $name, c
                    ));
                }
            }
        };
    }

    // [ui]
    apply_color!(config.theme.ui.fg, toml.ui.fg, "ui", "fg");
    apply_color!(config.theme.ui.bg, toml.ui.bg, "ui", "bg");
    apply_color!(
        config.theme.ui.border_fg,
        toml.ui.border_fg,
        "ui",
        "border_fg"
    );

    // [table]
    apply_color!(
        config.theme.table.header_fg,
        toml.table.header_fg,
        "table",
        "header_fg"
    );
    if let Some(ref c) = toml.table.header_bg {
        if let Some(color) = parse_color(c) {
            config.theme.table.header_bg = Some(color);
        } else {
            warnings.push(format!(
                "{}: invalid color for table.header_bg: {:?}",
                file, c
            ));
        }
    }
    if let Some(b) = toml.table.header_bold {
        config.theme.table.header_bold = b;
    }
    apply_color!(
        config.theme.table.zebra_bg,
        toml.table.zebra_bg,
        "table",
        "zebra_bg"
    );
    apply_color!(
        config.theme.table.cursor_fg,
        toml.table.cursor_fg,
        "table",
        "cursor_fg"
    );
    apply_color!(
        config.theme.table.cursor_bg,
        toml.table.cursor_bg,
        "table",
        "cursor_bg"
    );
    apply_color!(
        config.theme.table.selection_fg,
        toml.table.selection_fg,
        "table",
        "selection_fg"
    );
    apply_color!(
        config.theme.table.selection_bg,
        toml.table.selection_bg,
        "table",
        "selection_bg"
    );
    apply_color!(
        config.theme.table.search_match_fg,
        toml.table.search_match_fg,
        "table",
        "search_match_fg"
    );
    apply_color!(
        config.theme.table.search_match_bg,
        toml.table.search_match_bg,
        "table",
        "search_match_bg"
    );
    apply_color!(
        config.theme.table.dirty_fg,
        toml.table.dirty_fg,
        "table",
        "dirty_fg"
    );

    // [popup]
    apply_color!(config.theme.popup.bg, toml.popup.bg, "popup", "bg");
    apply_color!(config.theme.popup.fg, toml.popup.fg, "popup", "fg");
    apply_color!(
        config.theme.popup.border_fg,
        toml.popup.border_fg,
        "popup",
        "border_fg"
    );
    apply_color!(
        config.theme.popup.title_fg,
        toml.popup.title_fg,
        "popup",
        "title_fg"
    );
    apply_color!(
        config.theme.popup.completion_sel_fg,
        toml.popup.completion_sel_fg,
        "popup",
        "completion_sel_fg"
    );
    apply_color!(
        config.theme.popup.completion_sel_bg,
        toml.popup.completion_sel_bg,
        "popup",
        "completion_sel_bg"
    );

    // [status]
    apply_color!(config.theme.status.fg, toml.status.fg, "status", "fg");
    apply_color!(config.theme.status.bg, toml.status.bg, "status", "bg");
    apply_color!(
        config.theme.status.mode_fg,
        toml.status.mode_fg,
        "status",
        "mode_fg"
    );
    apply_color!(
        config.theme.status.mode_bg,
        toml.status.mode_bg,
        "status",
        "mode_bg"
    );
    apply_color!(
        config.theme.status.error_fg,
        toml.status.error_fg,
        "status",
        "error_fg"
    );
    apply_color!(
        config.theme.status.success_fg,
        toml.status.success_fg,
        "status",
        "success_fg"
    );

    // [file_menu]
    apply_color!(
        config.theme.file_menu.dir_fg,
        toml.file_menu.dir_fg,
        "file_menu",
        "dir_fg"
    );
    apply_color!(
        config.theme.file_menu.highlight_fg,
        toml.file_menu.highlight_fg,
        "file_menu",
        "highlight_fg"
    );
    apply_color!(
        config.theme.file_menu.highlight_bg,
        toml.file_menu.highlight_bg,
        "file_menu",
        "highlight_bg"
    );
    apply_color!(
        config.theme.file_menu.separator_fg,
        toml.file_menu.separator_fg,
        "file_menu",
        "separator_fg"
    );
    apply_color!(
        config.theme.file_menu.status_bg,
        toml.file_menu.status_bg,
        "file_menu",
        "status_bg"
    );
    apply_color!(
        config.theme.file_menu.status_mode_bg,
        toml.file_menu.status_mode_bg,
        "file_menu",
        "status_mode_bg"
    );
    apply_color!(
        config.theme.file_menu.status_accent_bg,
        toml.file_menu.status_accent_bg,
        "file_menu",
        "status_accent_bg"
    );
    apply_color!(
        config.theme.file_menu.active_indicator_fg,
        toml.file_menu.active_indicator_fg,
        "file_menu",
        "active_indicator_fg"
    );
    if let Some(ref cols) = toml.file_menu.preview_cols {
        if cols.len() != 8 {
            warnings.push(format!(
                "{}: file_menu.preview_cols must have exactly 8 entries, got {}",
                file,
                cols.len()
            ));
        } else {
            for (i, c) in cols.iter().enumerate() {
                if let Some(color) = parse_color(c) {
                    config.theme.file_menu.preview_cols[i] = color;
                } else {
                    warnings.push(format!(
                        "{}: invalid color for file_menu.preview_cols[{}]: {:?}",
                        file, i, c
                    ));
                }
            }
        }
    }

    // [sql]
    if let Some(u) = toml.sql.format_uppercase {
        config.sql.format_uppercase = u;
    }
    if let Some(limit) = toml.sql.sql_history_limit {
        config.sql.sql_history_limit = limit;
    }
    apply_color!(
        config.theme.sql.line_number_fg,
        toml.sql.line_number_fg,
        "sql",
        "line_number_fg"
    );
    apply_color!(
        config.theme.sql.diagnostic_error_fg,
        toml.sql.diagnostic_error_fg,
        "sql",
        "diagnostic_error_fg"
    );
    apply_color!(
        config.theme.sql.diagnostic_warning_fg,
        toml.sql.diagnostic_warning_fg,
        "sql",
        "diagnostic_warning_fg"
    );
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
        "reset" => Some(Color::Reset),
        _ => None,
    }
}

/// Convert a ratatui Color back to a config-file string.
///
/// Mirrors `parse_color` — every named color and hex RGB form round-trips.
pub fn color_to_config_string(color: &Color) -> String {
    match color {
        Color::Reset => "reset".to_string(),
        Color::Black => "black".to_string(),
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::Gray => "gray".to_string(),
        Color::DarkGray => "darkgray".to_string(),
        Color::LightRed => "lightred".to_string(),
        Color::LightGreen => "lightgreen".to_string(),
        Color::LightYellow => "lightyellow".to_string(),
        Color::LightBlue => "lightblue".to_string(),
        Color::LightMagenta => "lightmagenta".to_string(),
        Color::LightCyan => "lightcyan".to_string(),
        Color::White => "white".to_string(),
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
        Color::Indexed(i) => format!("indexed({})", i),
    }
}

/// Build a `TomlConfig` from a runtime `Config`, converting colors back to
/// their string representations. Used by the theme selector when persisting
/// the active theme to `config.toml`.
impl From<&Config> for TomlConfig {
    fn from(cfg: &Config) -> Self {
        let c = |color: &Color| color_to_config_string(color);
        let maybe_c = |opt: &Option<Color>| opt.as_ref().map(color_to_config_string);

        TomlConfig {
            defaults: TomlDefaults {
                delimiter: cfg.defaults.delimiter.map(|d| d.to_string()),
                encoding: cfg.defaults.encoding.clone(),
                zebra_striping: Some(cfg.defaults.zebra_striping),
                max_column_width: Some(cfg.defaults.max_column_width),
                undo_limit: Some(cfg.defaults.undo_limit),
                show_footer: Some(cfg.defaults.show_footer),
                command_history_limit: Some(cfg.defaults.command_history_limit),
                shell_history_limit: Some(cfg.defaults.shell_history_limit),
            },
            ui: TomlUi {
                fg: Some(c(&cfg.theme.ui.fg)),
                bg: Some(c(&cfg.theme.ui.bg)),
                border_fg: Some(c(&cfg.theme.ui.border_fg)),
            },
            table: TomlTable {
                header_fg: Some(c(&cfg.theme.table.header_fg)),
                header_bg: maybe_c(&cfg.theme.table.header_bg),
                header_bold: Some(cfg.theme.table.header_bold),
                zebra_bg: Some(c(&cfg.theme.table.zebra_bg)),
                cursor_fg: Some(c(&cfg.theme.table.cursor_fg)),
                cursor_bg: Some(c(&cfg.theme.table.cursor_bg)),
                selection_fg: Some(c(&cfg.theme.table.selection_fg)),
                selection_bg: Some(c(&cfg.theme.table.selection_bg)),
                search_match_fg: Some(c(&cfg.theme.table.search_match_fg)),
                search_match_bg: Some(c(&cfg.theme.table.search_match_bg)),
                dirty_fg: Some(c(&cfg.theme.table.dirty_fg)),
            },
            popup: TomlPopup {
                bg: Some(c(&cfg.theme.popup.bg)),
                fg: Some(c(&cfg.theme.popup.fg)),
                border_fg: Some(c(&cfg.theme.popup.border_fg)),
                title_fg: Some(c(&cfg.theme.popup.title_fg)),
                completion_sel_fg: Some(c(&cfg.theme.popup.completion_sel_fg)),
                completion_sel_bg: Some(c(&cfg.theme.popup.completion_sel_bg)),
            },
            status: TomlStatus {
                fg: Some(c(&cfg.theme.status.fg)),
                bg: Some(c(&cfg.theme.status.bg)),
                mode_fg: Some(c(&cfg.theme.status.mode_fg)),
                mode_bg: Some(c(&cfg.theme.status.mode_bg)),
                error_fg: Some(c(&cfg.theme.status.error_fg)),
                success_fg: Some(c(&cfg.theme.status.success_fg)),
            },
            file_menu: TomlFileMenu {
                dir_fg: Some(c(&cfg.theme.file_menu.dir_fg)),
                highlight_fg: Some(c(&cfg.theme.file_menu.highlight_fg)),
                highlight_bg: Some(c(&cfg.theme.file_menu.highlight_bg)),
                separator_fg: Some(c(&cfg.theme.file_menu.separator_fg)),
                status_bg: Some(c(&cfg.theme.file_menu.status_bg)),
                status_mode_bg: Some(c(&cfg.theme.file_menu.status_mode_bg)),
                status_accent_bg: Some(c(&cfg.theme.file_menu.status_accent_bg)),
                active_indicator_fg: Some(c(&cfg.theme.file_menu.active_indicator_fg)),
                preview_cols: Some(
                    cfg.theme
                        .file_menu
                        .preview_cols
                        .iter()
                        .map(color_to_config_string)
                        .collect(),
                ),
            },
            sql: TomlSql {
                format_uppercase: Some(cfg.sql.format_uppercase),
                sql_history_limit: Some(cfg.sql.sql_history_limit),
                line_number_fg: Some(c(&cfg.theme.sql.line_number_fg)),
                diagnostic_error_fg: Some(c(&cfg.theme.sql.diagnostic_error_fg)),
                diagnostic_warning_fg: Some(c(&cfg.theme.sql.diagnostic_warning_fg)),
            },
        }
    }
}

/// Load a theme TOML file and apply it to a Config, returning any warnings.
///
/// This is the narrow public API for the theme selector — callers should not
/// need to reach into `toml_parsing` internals.
pub fn apply_theme_from_file(
    config: &mut Config,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<Option<()>, String> {
    match load_toml_file(path)? {
        Some(toml) => {
            apply_toml(config, &toml, path, warnings);
            Ok(Some(()))
        }
        None => Ok(None),
    }
}

/// Serialize a `Config` to a TOML string.
///
/// Uses `TomlConfig::from` to convert the runtime config back to its
/// serializable form, then serializes with `toml::to_string_pretty`.
pub fn config_to_toml_string(config: &Config) -> Result<String, toml::ser::Error> {
    let toml_config = TomlConfig::from(config);
    toml::to_string_pretty(&toml_config)
}
